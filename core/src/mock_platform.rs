//! Concrete implementations for the platform simulation
use std::{convert::identity, sync::Mutex};

use uom::si::f64::*;
use uom::si::time::second;

use crate::platform::{Loader, Setter};

/// Parameters that are the input to platforms control system.
struct InputParams {
  /// position ∈ [0,1), where 0 is the start of the track
  position: Length,
  velocity: Velocity,
  distance_at_front: Length,
  distance_at_back: Length,
  velocity_at_front: Velocity,
  velocity_at_back: Velocity,
}

#[derive(Debug, Copy, Clone)]
struct MotionVector {
  position: Length,
  velocity: Velocity,
  acceleration: Acceleration,
}

impl MotionVector {
  /// Assuming acceleration stayed constant from t0 to t1
  /// and self describes the vector at time t0
  /// return the vector at time t1
  fn update(&self, time_diff: Time) -> Self {
    // let time_diff = t1 - t0;
    MotionVector {
      position: self.position + (self.velocity + 0.5 * self.acceleration * time_diff) * time_diff,
      velocity: self.velocity + self.acceleration * time_diff,
      acceleration: self.acceleration,
    }
  }
}

///
// struct PlatformState {
//   platform_state: MotionVector,
// }

/// Component of the simulation.
/// Tracks platforms state and mimics to platform the hardware interface
struct StateManager<S> {
  /// Used to calculate the change to position and velocity.
  last_state_and_access_time: Mutex<(S, std::time::Instant)>,
}

type PlatformStateManager = StateManager<MotionVector>;

// TODO: extract time related functions into trait

/// Reads system time info and updates the vector to the present moment in time.
fn update_to_current_systemtime(
  vec: &MotionVector,
  last_time: &std::time::Instant,
) -> (MotionVector, std::time::Instant) {
  let now = std::time::Instant::now();
  let time_diff = Time::new::<second>(now.duration_since(*last_time).as_secs_f64());
  (vec.update(time_diff), now)
}

impl StateManager<MotionVector> {
  pub fn get_state(&self) -> MotionVector {
    self.update_then_modify(identity)
  }

  pub fn set_state(&self, acc: Acceleration) {
    self.update_then_modify(|vec| MotionVector {
      acceleration: acc,
      ..vec
    });
  }

  fn update_then_modify<F>(&self, modify: F) -> MotionVector
  where
    F: Fn(MotionVector) -> MotionVector,
  {
    {
      let mut last_state = self.last_state_and_access_time.lock().unwrap();
      let (vec, now) = update_to_current_systemtime(&last_state.0, &last_state.1);
      let new_vec = modify(vec);
      *last_state = (new_vec, now);
      return new_vec;
    }
  }
}

fn calculate_input_params(
  back_platform: MotionVector,
  this_platform: MotionVector,
  front_platform: MotionVector,
) -> InputParams {
  InputParams {
    position: this_platform.position,
    velocity: this_platform.velocity,
    distance_at_front: front_platform.position - this_platform.position,
    distance_at_back: this_platform.position - back_platform.position,
    velocity_at_front: front_platform.velocity,
    velocity_at_back: back_platform.velocity,
  }
}

struct TrackStateManager<S> {
  /// Platform states from the first to the last in order
  platform_states: Vec<StateManager<S>>,
}

impl<S: Copy> TrackStateManager<S> {
  fn init(ss: Vec<S>) -> TrackStateManager<S> {
    let now = std::time::Instant::now();
    TrackStateManager {
      platform_states: ss
        .iter()
        .map(|s| {
          let a = Mutex::new((*s, now));
          StateManager {
            last_state_and_access_time: a,
          }
        })
        .collect(),
    }
  }
}

/// What control system needs
/// Platforms impure parts 
pub struct ThinkOFName<'a, A, B> {
  loader: Loader<'a, A>,
  setter: Setter<'a, B>,
  // + server connection
}

impl<'a> TrackStateManager<MotionVector> {
  fn makeLoaders(&'a self) -> Vec<Loader<'a, InputParams>> {
    // A loader for one platform accesses mutexes of itself but also
    // of the two neighbouring platforms. This means that without any consideration,
    // we would run into the "Dining philosophers problem" quite literaly.
    //
    // We solve it by always accessing mutexes in the order of platforms:
    // all but the first and the last platform read state in order: at back, then their own, then at front.
    // But the last platform reads first the state at front, then at back, then their own
    // and the first platform reads first its own state, then at front, then at back.
    //
    // See https://en.wikipedia.org/wiki/Dining_philosophers_problem

    let states: &'a Vec<StateManager<MotionVector>> = &self.platform_states;
    let n = states.len();
    let loader_first_platform = Loader::new(move || {
      let this = states[0].get_state(); // this, first
      let front = states[1].get_state();
      let back = states[n - 1].get_state(); // last
      calculate_input_params(back, this, front)
    });
    let loader_last_platform = Loader::new(move || {
      let front = states[0].get_state(); // first
      let back = states[n - 2].get_state(); // this, last
      let this = states[n - 1].get_state(); // second to last
      calculate_input_params(back, this, front)
    });
    let mut loaders_rest: Vec<Loader<'a, InputParams>> = (1..states.len() - 1)
      .map(move |i| {
        Loader::new(move || {
          let back = states[i - 1].get_state();
          let this = states[i].get_state();
          let front = states[i + 1].get_state();
          calculate_input_params(back, this, front)
        })
      })
      .collect();
    loaders_rest.insert(0, loader_first_platform);
    loaders_rest.push(loader_last_platform);
    return loaders_rest;
  }

  fn makeSetters(&'a self) -> Vec<Setter<'a, Acceleration>> {
    (&self.platform_states)
      .into_iter()
      .map(|st| Setter::new(move |acc| st.set_state(acc)))
      .collect()
  }
}
