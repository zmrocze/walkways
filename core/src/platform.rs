use std::fmt;
use std::any::type_name;
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

// Just the pure math function modelling platform
// A impl MeasuredParams
// B impl ControlledParams
pub struct CalculateParams<A, B> {
  calculate: fn(A) -> B,
}

// A impl MeasuredParams
pub struct Monitor<A> {
  check: fn(A) -> bool,
  // should be more desriptive than bool
}

// Full logic of platform on pure data
pub struct Controller {
  // access MeasuredParams
  // access Server messages
  // control movemement
  // respond with alerts/confirmations
}
