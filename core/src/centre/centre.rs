use futures::future::join_all;
use http::Uri;
// use tokio::net::unix::SocketAddr;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::net::SocketAddr;
use std::panic::UnwindSafe;
use std::sync::mpsc::{self, sync_channel, Sender, SyncSender};
use std::thread;
use tonic::Streaming;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, warn};
use uom::marker::Add;

use crate::common::{ClientError, PlatformID, RunStatus};
use crate::platform;
use crate::proto::centre::{Alert, ImReadyAnswer, ImReadyMsg, ReadyStatus, ReceivedAlerts};
use crate::proto::platform::platform_client::PlatformClient;
use crate::proto::platform::{StartMsg, StopMsg};
use crate::proto::{run_server, LocalAddress};
// use crate::platform::communication::platform::health_check_service_server::{SendAlertsService};
use crate::proto::centre::centre_client::CentreClient;
use crate::proto::centre::centre_server::{Centre, CentreServer};
// use crate::proto::centre::centre_server::{AlertMsg, ReceivedAlerts};
// use uom::si::f32::Time;

// pub struct ServerConfig {
//   health_check_interval: Time,
//   ping_interval: Time,
// }

pub struct Addresses {
  server_addr: LocalAddress,
  platform_addrs: HashMap<PlatformID, Uri>,
}

#[derive(Debug)]
/// Implements centre server request handlers
struct CentreServerImpl {
  ready_platforms_transmitter: SyncSender<PlatformID>,
}

#[tonic::async_trait]
impl Centre for CentreServerImpl {
  async fn send_alerts(
    &self,
    request: Request<Streaming<Alert>>,
  ) -> Result<Response<ReceivedAlerts>, Status> {
    // do nothing
    Ok(Response::new(ReceivedAlerts {}))
  }

  #[tracing::instrument]
  async fn im_ready(
    &self,
    request: Request<ImReadyMsg>,
  ) -> Result<Response<ImReadyAnswer>, Status> {
    // request.remote_addr()
    let ImReadyMsg {
      ready_status,
      platform_id,
    } = request.into_inner();
    let x = ReadyStatus::from_i32(ready_status).unwrap();
    match ReadyStatus::from_i32(ready_status) {
      Some(ReadyStatus::ImNotReadyError) => {
        info!("Platform {} not ready!", platform_id);
      }
      Some(ReadyStatus::ImReadyOk) => {
        self.ready_platforms_transmitter.send(platform_id.into()); // ignoring if channel closed
      }
      None => {
        warn!("Received unknown enumeration value for ReadyStatus")
      }
    }
    Ok(Response::new(ImReadyAnswer {}))
  }
}

/// Implements full centre functionality
pub struct CentreState {}

impl CentreState {
  /// Start the server, initialize platform connections
  /// and send Start signal once every platform is ready.
  pub async fn run(self, addrs: Addresses) {
    // We create a channel to communicate from server which platforms are ready
    // and then inside ready_check handle bookkeeping of which platforms are ready
    let (tx, rx) = sync_channel(15);
    // server handling incoming platform connections
    let server = run_server(
      CentreServer::new(CentreServerImpl {
        ready_platforms_transmitter: tx,
      }),
      addrs.server_addr,
    );
    // async thread that:
    //  - receives messages with platforms which are ready from the server
    //  - pings those platforms to verify
    //  - once every platform ready sends the Start signal and the track starts
    let ready_check = async {
      let mut not_ready_platforms: HashSet<PlatformID> = addrs.platform_addrs.into_keys().collect();
      let ready = rx.into_iter().try_for_each(|platform| {
        // TODO: PING PLATFORM
        // client_ping;
        not_ready_platforms.remove(&platform);
        if not_ready_platforms.is_empty() {
          Err(())
        } else {
          Ok(())
        }
      });
      match ready {
        // not ready, channel got close without all platforms ready
        Ok(_) => {}
        // All platforms reported being ready and responded to ping
        // We can send Start signal
        Err(_) => {
          broadcast_runstatus_signal(addrs.platform_addrs, RunStatus::Start);
        }
      };
    };
    futures::future::join(server, ready_check).await;
    ()
  }
}

/// Send to every platform the either the Start or Stop signal
pub async fn broadcast_runstatus_signal(
  addrs: &HashMap<PlatformID, Uri>,
  run_status: RunStatus,
) -> Result<(), ClientError> {
  join_all(addrs.into_values().map(|uri| {
    tokio::spawn(async move {
      let mut client = PlatformClient::connect(uri.clone()).await?;
      match run_status {
        RunStatus::Start => client
          .start_signal(Request::new(StartMsg {}))
          .await
          .map(|_| ())?,
        RunStatus::Stop => client
          .stop_signal(Request::new(StopMsg {}))
          .await
          .map(|_| ())?,
      }
      Result::<(), ClientError>::Ok(())
    })
  }))
  .await
  .into_iter()
  .map(|x| x?)
  .collect::<Result<Vec<()>, ClientError>>()
  .map(|vec| ())
}

// pub fn server_main(addrs : Addresses) -> Result<(), Box<dyn std::error::Error>> {

//   // let mut client = SendAlertsServiceClient::connect("http://[::1]:50051").await?;

//   // let request = tonic::Request::new(SendAlerts {});

//   // let response = client.check_health(request).await?;

//   // println!("RESPONSE={:?}", response);

//   Ok(())
// }

// Client code, used by platforms
// #[tokio::main]
// async fn send_alerts_to_server(
//   client: &mut SendAlertsClient<tonic::transport::Channel>,
//   alerts: impl tonic::IntoStreamingRequest<Message = Alert>)
//   -> Result<(), Box<dyn std::error::Error>> {
//     // let mut client = SendAlertsClient::connect("http://[::1]:50051").await?;

//     // let request = tonic::Request::new(SendAlerts {});

//     let response = client.send_alerts(alerts).await?;

//     println!("RESPONSE={:?}", response);

//     Ok(())
// }
