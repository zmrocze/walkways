pub mod communication;
pub mod mock_platform;
pub mod monitor;

use std::any::type_name;
use std::fmt::{self, Debug};
use std::net::SocketAddr;
use tonic::server;
use tracing::error;

use crate::common::RunStatus;

use crate::platform::communication::OpenConnection;

use self::communication::LocalAddress;

enum Error {
  VeryWeirdError,
}

// Parameters read from all the sensors.
// Logic for updating/measuring/fetching MeasuredParams - basically no matter how parameters are gotten,
// its job is to provide them to the controller.
pub struct Loader<'a, A> {
  /// morally loader: () -> A
  loader: Box<dyn (Fn() -> A) + 'a + Send>,
}

impl<'a, A> fmt::Debug for Loader<'a, A> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Loader<{:?}>", type_name::<A>())
  }
}

impl<'a, A: 'a> Loader<'a, A> {
  pub fn load(&self) -> A {
    (self.loader)()
  }

  pub fn new<F>(f: F) -> Loader<'a, A>
  where
    F: (Fn() -> A) + 'a + Send,
  {
    Loader {
      loader: Box::new(f),
    }
  }

  pub fn map<B, F>(self, f: F) -> Loader<'a, B>
  where
    F: Fn(A) -> B + 'a + Send,
  {
    let g = Box::new(move || f((self.loader)()));
    Loader { loader: g }
  }

  // /// Applicative <*>
  pub fn combine_load<B, F>(self, f: Loader<'a, F>) -> Loader<'a, B>
  where
    F: Fn(A) -> B + 'a,
    B: 'a,
  {
    Loader::new(move || {
      let a = self.load();
      let g = f.load();
      return g(a);
    })
  }
}

pub struct Setter<'a, A> {
  /// morally setter: A -> ()
  /// TODO: allow setter to return error
  pub setter: Box<dyn (Fn(A) -> ()) + 'a + Send>,
}

impl<'a, A: 'a> Setter<'a, A> {
  pub fn set(&self, a: A) {
    (self.setter)(a)
  }

  pub fn new<F>(f: F) -> Setter<'a, A>
  where
    F: (Fn(A) -> ()) + 'a + Send,
  {
    Setter {
      setter: Box::new(f),
    }
  }

  fn premap<B, F>(self, f: F) -> Setter<'a, B>
  where
    F: Fn(B) -> A + 'a + Send,
  {
    let g = Box::new(move |a| (self.setter)(f(a)));
    Setter { setter: g }
  }
}

/// Just the pure math function calculating platforms desired parameters like acceleration
pub trait Calculate<I, O> {
  fn calculate(&self, run_status: &RunStatus, input: &I) -> O;
}

/// Function monitoring input parameters for alerts
pub trait Monitor<I> {
  fn check(&self, run_status: &RunStatus, input: &I) -> Vec<crate::proto::centre::Alert>;
}

/// Implementation of the platforms controller,
/// including pure controller logic
/// and impure interfaces to hardware
pub struct Platform<'a, I, O, Calc: Calculate<I, O>, Mon: Monitor<I>> {
  loader: Loader<'a, I>,
  setter: Setter<'a, O>,
  // server_connection: Box<dyn CentreConnection>,
  calculate_params: Calc,
  monitor: Mon,
}

impl<'a, I, O, Calc: Calculate<I, O>, Mon: Monitor<I>> Debug for Platform<'a, I, O, Calc, Mon> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Platform<{:?}, {:?}, {:?}, {:?}>",
      type_name::<I>(),
      type_name::<O>(),
      type_name::<Calc>(),
      type_name::<Mon>()
    )
  }
}

/// What the controller needs from the connection to the centre.
#[tonic::async_trait]
pub trait CentreConnection {
  fn send_alerts(self, alerts: Vec<crate::proto::centre::Alert>);
  //  -> Result<(), tonic::Status>;
  /// Get status: Start/Stop. Mustn't block.
  fn get_runstatus(&self) -> RunStatus;
  // async fn server_platform_server
}

/// In a loop: loads params, calculates and sets params. Sends alerts.
pub fn run_platform_controller<
  'a,
  I,
  O,
  Calc: Calculate<I, O>,
  Mon: Monitor<I>,
  Conn: CentreConnection,
>(
  platform: Platform<'a, I, O, Calc, Mon>,
  centre_connection: Conn,
) -> () {
  loop {
    // TODO: Answer Q: where should the async be? here or inside loader?
    let inp = platform.loader.load();
    let run_status = centre_connection.get_runstatus();
    platform
      .setter
      .set(platform.calculate_params.calculate(&run_status, &inp));

    let alerts = platform.monitor.check(&run_status, &inp);
    centre_connection.send_alerts(alerts)
  }
}

#[tracing::instrument]
pub fn run_platform<'a, I, O, Calc: Calculate<I, O> + Send, Mon: Monitor<I> + Send>(
  platform: Platform<'a, I, O, Calc, Mon>,
  centre_addr: tonic::transport::Uri,
  our_address: LocalAddress,
) -> (SocketAddr, impl Fn() -> ())
// TODO: fix App-wide error type
{
  // tokio runtime for async centre communication handling
  let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap();

  let connection = runtime.block_on(OpenConnection::init(centre_addr))?;

  let running_controller = std::thread::spawn(|| {
    run_platform_controller(platform, connection);
  });

  let (our_socket, server_future) = runtime.block_on(connection.serve_platform_server(our_address));

  let running_server = std::thread::spawn(|| {
    runtime.block_on(server_future);
  });

  (our_socket, || {
    //
    match running_server.join() {
      Ok(_) => {
        error!("Server simply stopped working, shouldn't happen.");
        connection.send_alerts(vec![
          // TODO
        ]);
      }
      Err(err) => {
        error!("Server crashed with and error {:?}", err);
        connection.send_alerts(vec![
          // TODO
        ]);
        // TODO: recover?
      }
    }

    // This shouldn't error nor finish
    running_controller.join();
  })
}
