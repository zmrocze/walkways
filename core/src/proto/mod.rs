use std::convert::Infallible;
use std::net::SocketAddr;

use futures::future::BoxFuture;
use futures::FutureExt;
use http::{Request, Response};
use hyper::Body;
use tokio::net::TcpListener;
use tonic::body::BoxBody;
use tonic::transport::Server;
use tonic::transport::{Channel, NamedService, Uri};
use tower::Service;

pub mod platform {
  tonic::include_proto!("platform");
}

pub mod centre {
  tonic::include_proto!("centre");
}

#[derive(Debug, Clone, Copy)]
/// An argument to starting server with run_server
pub enum LocalAddress {
  UseOSAssignedPort,
  UsePort(SocketAddr),
}

/// Serves a given service.
/// The function returns SocketAddr on which the server runs and the running server as a future.
/// This future needs be executed or otherwise no server runs in the background.
pub async fn run_server<'a, S>(
  svc: S,
  our_address: LocalAddress,
) -> (
  SocketAddr,
  BoxFuture<'a, Result<(), tonic::transport::Error>>,
)
where
  // tldr: if proto service is "Platform" then S is something like PlatformServer::new(x) where x impl Platform
  S: Service<Request<Body>, Response = Response<BoxBody>, Error = Infallible>
    + NamedService
    + Clone
    + Send
    + 'static,
  S::Future: Send + 'static,
{
  let router = Server::builder().add_service(svc);
  match our_address {
    LocalAddress::UsePort(socket_addr) => (socket_addr, router.serve(socket_addr).boxed()),
    LocalAddress::UseOSAssignedPort => {
      let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
      let addr = listener.local_addr().unwrap();

      (
        addr,
        router
          .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
          .boxed(),
      )
    }
  }
}
