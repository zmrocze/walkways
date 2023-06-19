use tokio::net::unix::SocketAddr;
use tonic::Streaming;
use uom::marker::Add;
use tonic::{transport::Server, Request, Response, Status};

// use crate::platform::communication::platform::health_check_service_server::{SendAlertsService};
use crate::proto::centre::centre_server::{Centre, CentreServer};
use crate::proto::centre::centre_client::{CentreClient};
// use crate::proto::centre::centre_server::{AlertMsg, ReceivedAlerts};
use uom::si::f32::Time;

// pub struct ServerConfig {
//   health_check_interval: Time,
//   ping_interval: Time,
// } 

pub struct Addresses {
  server_addr : SocketAddr,
  platform_addrs : Vec<SocketAddr>,
}

pub fn server_main(addrs : Addresses) -> Result<(), Box<dyn std::error::Error>> {

  // let mut client = SendAlertsServiceClient::connect("http://[::1]:50051").await?;

  // let request = tonic::Request::new(SendAlerts {});

  // let response = client.check_health(request).await?;

  // println!("RESPONSE={:?}", response);

  Ok(())
}

#[derive(Default)]
pub struct SendAlertsImpl {}

// #[tonic::async_trait]
// impl SendAlerts for SendAlertsImpl {
//     async fn send_alerts(
//         &self,
//         request: Request<Streaming<Alert>>,
//     ) -> Result<Response<ReceivedAlerts>, Status> {
//         println!("Got a request from {:?}", request.remote_addr());

//         let reply = ReceivedAlerts {};
//         Ok(Response::new(reply))
//     }
// }

// #[tokio::main]
// async fn run_platform_server() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "[::1]:50051".parse().unwrap();
//     let server_ = ServerImpl::default();

//     println!(" listening on {}", addr);

//     Server::builder()
//         .add_service(SendAlertsServer::new(SendAlerts))
//         .serve(addr)
//         .await?;

//     Ok(())
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