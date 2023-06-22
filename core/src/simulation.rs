use std::{error::Error, iter::repeat};

use uom::si::{f64::{Length, Acceleration, Velocity}, length::meter, acceleration::meter_per_second_squared, velocity::kilometer_per_hour};

use crate::track::Track;
use crate::common::MotionVector;
use crate::platform::mock_platform::TrackStateManager;


pub fn run_simulation() -> Result<(), Box<dyn Error>> {
  let num_platforms = 15;
  let track = Track::standard_track(
    Length::new::<meter>(300.0),
    Velocity::new::<kilometer_per_hour>(25.0),
    Acceleration::new::<meter_per_second_squared>(0.43),
    Acceleration::new::<meter_per_second_squared>(-0.43),
    Velocity::new::<kilometer_per_hour>(2.0)
    );
  let track_manager = TrackStateManager::init(
    repeat(MotionVector::zero(track.track_length)).take(num_platforms).collect());
  let loaders_and_setters = track_manager.make_loaders_and_setters();
  let platforms = loaders_and_setters.into_iter()
    .map(|(load, set)| {
      
    });
  _
}