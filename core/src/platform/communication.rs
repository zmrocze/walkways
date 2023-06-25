use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, MutexGuard};

use futures::future::BoxFuture;
use futures::FutureExt;
use tokio::net::TcpListener;
use tonic::transport::server::Router;
use tonic::transport::{Channel, Uri};
use tonic::{transport::Server, Request, Response, Status};

use crate::common::{PlatformID, RunStatus};
use crate::proto::centre::centre_server::Centre;
use crate::proto::centre::{Alert, ImReadyMsg, ReadyStatus, ReceivedAlerts};
use crate::proto::platform::{PingMsg, PongAnswer, StartAnswer, StartMsg, StopAnswer, StopMsg};
// use crate::platform::communication::platform::health_check_service_server::{HealthCheckService};
use crate::proto::centre::centre_client::CentreClient;
use crate::proto::platform::platform_client::PlatformClient;
use crate::proto::platform::platform_server::{Platform, PlatformServer};
use crate::proto::{run_server, LocalAddress};

use super::CentreConnection;
// use crate::proto::centre::centre_server::{Alert, ReceivedAlerts};

struct PlatformServerState {
  // centre_client : CentreClient<Channel>,
  run_status: AtomicBool,
}

impl PlatformServerState {
  fn new() -> PlatformServerState {
    PlatformServerState {
      run_status: AtomicBool::new(false),
    }
  }

  fn read(&self) -> RunStatus {
    // "Memory orderings specify the way atomic operations synchronize memory.
    // In its weakest Ordering::Relaxed, only the memory directly touched by the operation is synchronized."
    if self.run_status.load(std::sync::atomic::Ordering::Relaxed) {
      RunStatus::Start
    } else {
      RunStatus::Stop
    }
  }

  fn write(&self, value: RunStatus) {
    // "Memory orderings specify the way atomic operations synchronize memory.
    // In its weakest Ordering::Relaxed, only the memory directly touched by the operation is synchronized."
    self.run_status.store(
      match value {
        RunStatus::Start => true,
        RunStatus::Stop => false,
      },
      std::sync::atomic::Ordering::Relaxed,
    )
  }

  // fn startstop_signal_handler<R>(&self, action: impl FnOnce(MutexGuard<'_, RunStatus>) -> R)
  //   -> Result<Response<R>, Status> {
  //     match self.run_status.lock() {
  //   Err(err) => {
  //     // TODO: alert server
  //     let errmsg = format!("Can't read RunStatus {}", err);
  //     println!("{}", errmsg);
  //     Err(Status::internal(errmsg))
  //   }
  //   Ok(status) => {
  //     Ok(Response::new(action(status)))
  //   }
  // }
}

#[tonic::async_trait]
impl Platform for PlatformServerState {
  async fn ping(&self, request: Request<PingMsg>) -> Result<Response<PongAnswer>, Status> {
    println!("Got a ping request from {:?}", request.remote_addr());
    Ok(Response::new(PongAnswer {}))
  }

  async fn start_signal(&self, _: Request<StartMsg>) -> Result<Response<StartAnswer>, Status> {
    self.write(RunStatus::Start);
    Ok(Response::new(StartAnswer {}))
  }

  async fn stop_signal(&self, _: Request<StopMsg>) -> Result<Response<StopAnswer>, Status> {
    self.write(RunStatus::Stop);
    Ok(Response::new(StopAnswer {}))
  }
}

pub struct OpenConnection {
  centre_client: CentreClient<Channel>,
  // running_server: BoxFuture<(), tonic::transport::Error>,
  platform_runstatus: PlatformServerState, // run_status: Mutex<RunStatus> to implement CentreConnection
}

// #[tonic::async_trait]
impl OpenConnection {
  pub async fn init(
    platform_id: PlatformID,
    centre_addr: Uri,
  ) -> Result<OpenConnection, Box<dyn std::error::Error>> {
    // TODO: when we know it started correctly? It's Future<Result<,>> instead of Result<Future>
    // answer: let the server check it with ping, because even if we start correctly it doesn't mean server knows the right address
    let platform_state = PlatformServerState::new();
    let mut client = CentreClient::connect(centre_addr).await?;
    let request = tonic::Request::new(ImReadyMsg {
      ready_status: ReadyStatus::ImReadyOk.into(),
      platform_id: platform_id.into(),
    });
    client.im_ready(request).await?;
    Ok(OpenConnection {
      centre_client: client,
      platform_runstatus: platform_state,
    })
  }

  pub async fn serve_platform_server<'a>(
    self,
    our_address: LocalAddress,
  ) -> (
    SocketAddr,
    BoxFuture<'a, Result<(), tonic::transport::Error>>,
  ) {
    run_server(PlatformServer::new(self.platform_runstatus), our_address).await
  }
}

#[tonic::async_trait]
impl CentreConnection for OpenConnection {
  fn send_alerts(mut self, alerts: Vec<Alert>)
  // -> Result<(), Status>
  {
    let str = tokio_stream::iter(alerts);
    self
      .centre_client
      .send_alerts(Request::new(str))
      .await
      .map(|_| ())
  }

  fn get_runstatus(&self) -> RunStatus {
    self.platform_runstatus.read()
  }
}
