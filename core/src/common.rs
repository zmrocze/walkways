
use uom::si::f64::{Length, Velocity};
use uom::ConstZero;
use uom::{si::f64::*};


#[derive(Debug, Copy, Clone)]
pub enum RunStatus {
  Start, 
  Stop
}

#[derive(Debug, Copy, Clone)]
pub struct MotionVector {
  pub position: TrackPosition,
  pub velocity: Velocity,
  pub acceleration: Acceleration,
}

impl MotionVector {

  pub const fn zero(track_length : Length) -> MotionVector { 
    MotionVector {
      position: TrackPosition::new(track_length, Length::ZERO),
      velocity: Velocity::ZERO,
      acceleration: Acceleration::ZERO,
    }
  }

  /// Assuming acceleration stayed constant from t0 to t1
  /// and self describes the vector at time t0
  /// return the vector at time t1
  pub fn update(&self, time_diff: Time) -> Self {
    // let time_diff = t1 - t0;
    MotionVector {
      position: self.position.add((self.velocity + 0.5 * self.acceleration * time_diff) * time_diff),
      velocity: self.velocity + self.acceleration * time_diff,
      acceleration: self.acceleration,
    }
  }
}

#[derive(Debug, Copy, Clone)]
/// Parameters that are the input to platforms control system.
pub struct InputParams {
  /// position, where 0 is the start of the track
  pub position: TrackPosition,
  pub velocity: Velocity,
  pub distance_at_front: Length,
  pub distance_at_back: Length,
  pub velocity_at_front: Velocity,
  pub velocity_at_back: Velocity,
}

#[derive(Debug, Copy, Clone)]
/// On track our positions cycles from 0 to the track length.
/// This type wraps Length but enforces cycling through smart constructor
pub struct TrackPosition { 
  track_length: Length,
  position: Length,
}

impl TrackPosition {

  pub fn new(track_length: Length, position: Length) -> TrackPosition {
    TrackPosition { track_length: track_length, 
    position : position % track_length}
  }

  pub fn add(self, addend: Length) -> TrackPosition {
    TrackPosition{ position: (self.position + addend) % self.track_length ,
      track_length: self.track_length}
  }

  /// Positive distance from self to other in the direction of the track
  pub fn distance_to(self, other: TrackPosition) -> Length {
    debug_assert!(self.track_length == other.track_length);
    let d = other.position - self.position;
    if d.is_sign_negative() {
      d+self.track_length // track's starting point is between self and other 
    } else {
      d
    }
  }
}