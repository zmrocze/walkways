use std::future::Future;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, MutexGuard};

use tonic::codegen::BoxFuture;
use tonic::transport::{Channel, Uri};
use tonic::{transport::Server, Request, Response, Status};

use crate::common::RunStatus;
use crate::proto::centre::centre_server::Centre;
use crate::proto::centre::{ImReadyMsg, ReadyStatus, Alert, ReceivedAlerts};
use crate::proto::platform::{PingMsg, PongAnswer, StartMsg, StartAnswer, StopMsg, StopAnswer};
// use crate::platform::communication::platform::health_check_service_server::{HealthCheckService};
use crate::proto::platform::platform_server::{Platform, PlatformServer};
use crate::proto::platform::platform_client::{PlatformClient};
use crate::proto::centre::centre_client::{CentreClient};

use super::CentreConnection;
// use crate::proto::centre::centre_server::{Alert, ReceivedAlerts};

struct PlatformServerState {
    // centre_client : CentreClient<Channel>,
    run_status: AtomicBool,
}

impl PlatformServerState { 

  fn new() -> PlatformServerState {
    PlatformServerState{
      run_status: AtomicBool::new(false)
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
      }
        , std::sync::atomic::Ordering::Relaxed)
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
  async fn ping(
      &self,
      request: Request<PingMsg>,
  ) -> Result<Response<PongAnswer>, Status> {
    println!("Got a ping request from {:?}", request.remote_addr());
    Ok(Response::new(PongAnswer {}))
  }

  async fn start_signal(&self, _: Request<StartMsg>) 
    -> Result<Response<StartAnswer>, Status> {
      self.write(RunStatus::Start);
      Ok(Response::new(StartAnswer{}))
    }
  
  async fn stop_signal(&self, _: Request<StopMsg>) 
    -> Result<Response<StopAnswer>, Status> {
      self.write(RunStatus::Stop);
      Ok(Response::new(StopAnswer{}))
    }
}

pub struct OpenConnection {
  centre_client: CentreClient<Channel>,
  // running_server: BoxFuture<(), tonic::transport::Error>,
  platform_runstatus: PlatformServerState
  // run_status: Mutex<RunStatus> to implement CentreConnection
}

// #[tonic::async_trait]
impl OpenConnection {

  pub async fn init(centre_addr: Uri) -> 
    Result<OpenConnection, Box<dyn std::error::Error>> {
    // TODO: when we know it started correctly? It's Future<Result<,>> instead of Result<Future>
    // answer: let the server check it with ping, because even if we start correctly it doesn't mean server knows the right address
    let platform_state = PlatformServerState::new();
    // let running_server = 
    //   Server::builder()
    //   .add_service(PlatformServer::new(platform_state))
    //   .serve(our_address);
    let mut client = CentreClient::connect(centre_addr).await?;
    let request = tonic::Request::new(
      ImReadyMsg {ready_status: ReadyStatus::ImReadyOk.into()});
    client.im_ready(request).await?;
    Ok( OpenConnection { 
      centre_client: client,  
      platform_runstatus: platform_state
    })
  }

  pub async fn serve_platform_server(self, our_address: SocketAddr)
    -> Result<(), tonic::transport::Error> {
    // let runtime = tokio::runtime::Builder::new_current_thread()
    //   .enable_all()
    //   .build()
    //   .unwrap();

    // runtime.block_on(
      Server::builder()
    .add_service(PlatformServer::new(self.platform_runstatus))
    .serve(our_address).await
  // )
  }
}

// enum Alert {
//   Error(String),
//   Warn(String)
// }

// impl From<Alert> for Alert {

// }

#[tonic::async_trait]
impl CentreConnection for OpenConnection
{  
  fn send_alerts(mut self, alerts: Vec<Alert>) 
  // -> Result<(), Status> 
  {
    let str = tokio_stream::iter(alerts);
      self.centre_client.send_alerts(
        Request::new(str)
      ).await.map(|_| ())
  }

  fn get_runstatus(&self) -> RunStatus {
    self.platform_runstatus.read()
  } 
}
