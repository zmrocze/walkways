use std::fmt;
use std::any::type_name;

use crate::server::RunStatus;
// Parameters read from all the sensors.
// Logic for updating/measuring/fetching MeasuredParams - basically no matter how parameters are gotten,
// its job is to provide them to the controller.
pub struct Loader<'a, A> {
  /// morally loader: () -> A
  loader: Box<dyn (FnMut() -> A) + 'a>,
}

impl<'a,A> fmt::Debug for Loader<'a, A> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Loader<{:?}>", type_name::<A>())
  }
}

impl<'a, A: 'a> Loader<'a, A> {

  pub fn load(&mut self) -> A {
    (self.loader)()
  }

  pub fn new<F>(f: F) -> Loader<'a, A>
  where
    F: (FnMut() -> A) + 'a,
  {
    Loader {
      loader: Box::new(f),
    }
  }

  pub fn map<B, F>(mut self, f: F) -> Loader<'a, B>
  where
    F: Fn(A) -> B + 'a
  {
    let g = Box::new(move || f((self.loader)()));
    Loader { loader: g }
  }

  // /// Applicative <*>
  pub fn combine_load<B, F>(mut self, mut f: Loader<'a, F>) -> Loader<'a, B> 
  where
    F: Fn(A) -> B + 'a, B : 'a {
      Loader::new(move || {
        let a = self.load();
        let g = f.load();
        return g(a);
      })
  }
}

pub struct Setter<'a, A> {
  /// morally setter: A -> ()
  pub setter: Box<dyn (FnMut(A) -> ()) + 'a>,
}

impl<'a, A: 'a> Setter<'a, A> {
  pub fn new<F>(f: F) -> Setter<'a, A>
  where
    F: (FnMut(A) -> ()) + 'a,
  {
    Setter {
      setter: Box::new(f),
    }
  }

  fn premap<B, F>(mut self, f: F) -> Setter<'a, B>
  where
    F: Fn(B) -> A + 'a,
  {
    let g = Box::new(move |a| (self.setter)(f(a)));
    Setter { setter: g }
  }
}

// // Output of our platform controller
// pub trait ControlledParams {

// }

/// Just the pure math function calculating platforms desired parameters like acceleration
pub trait CalculateParams<I, O> {
  fn calculate(&self, input: I) -> O;
}

// A impl MeasuredParams
pub trait Monitor<I, A> {
  fn check(&self, runStatus : RunStatus, input: I) -> A;
  // should be more desriptive than bool
}

// Full logic of platform on pure data
pub struct Controller {
  // access MeasuredParams
  // access Server messages
  // control movemement
  // respond with alerts/confirmations
}

pub struct PlatformPure<I, O, A> {
  calculate_params: Box<dyn CalculateParams<I,O>>,
  monitor: Box<dyn Monitor<I, A>>,
}

pub struct PlatformImpure<'a, I, O, A> {
  loader: Loader<'a, I>,
  setter: Setter<'a, O>,
  client_connection: Box<dyn ClientConnection<A>>
}

/// Implementation of the platforms controller,
/// including pure controller logic 
/// and impure interfaces to hardware and client connection
pub struct Platform<'a, I,O,A> {
  platform_pure: PlatformPure<I,O,A>,
  platform_impure: PlatformImpure<'a ,I,O,A>,
}

/// Runs the platform controller.
pub fn run_platform_controller<'a, I, O, A>(platform: Platform<'a, I, O, A>) -> () {
  loop {
    let inp = platform.platform_impure.loader.load();
  } 
}

/// Initializes client connection and handshakes
// pub fn initialize_client_connection(params: ConnectionParams) -> ClientConnection

/// What the controller needs from the connection to the server.
pub trait ClientConnection<A> {
  fn send_alerts(&self, alerts: A);
  /// Get status: Start/Stop. Mustn't block.
  fn get_runstatus(&self) -> RunStatus;
}