use std::{error::Error, iter::repeat};

use tonic::transport::Uri;
use uom::si::{
  acceleration::meter_per_second_squared,
  f64::{Acceleration, Length, Velocity},
  length::meter,
  velocity::kilometer_per_hour,
};

use crate::common::MotionVector;
use crate::platform::mock_platform::TrackStateManager;
use crate::{
  platform::{
    communication::LocalAddress, monitor::SimpleMonitor, run_platform, run_platform_controller,
    Monitor, Platform,
  },
  track::{SimpleCalculate, Track},
};

pub fn run_simulation() -> Result<(), Box<dyn Error>> {
  let num_platforms = 15;
  let centre_addr = "https://www.rust-lang.org/install.html"
    .parse::<Uri>()
    .unwrap();

  let track = Track::standard_track(
    Length::new::<meter>(300.0),
    Velocity::new::<kilometer_per_hour>(25.0),
    Acceleration::new::<meter_per_second_squared>(0.43),
    Acceleration::new::<meter_per_second_squared>(-0.43),
    Velocity::new::<kilometer_per_hour>(2.0),
  );
  let track_manager = TrackStateManager::init(
    repeat(MotionVector::zero(track.track_length))
      .take(num_platforms)
      .collect(),
  );
  let loaders_and_setters = track_manager.make_loaders_and_setters();
  let platforms = loaders_and_setters.into_iter().map(|(load, set)| Platform {
    loader: load,
    setter: set,
    calculate_params: SimpleCalculate { track: track },
    monitor: SimpleMonitor { track: track },
  });
  let running_platforms = platforms
    .map(|platform| {
      std::thread::spawn(|| run_platform(platform, centre_addr, LocalAddress::UseOSAssignedPort))
    })
    .collect();
  let running_centre = _;
  // TODO: when do we stop the simulation?
  // maybe stop on shutdown signal
  // Parameterize simulation runner with a `Sink` that subscribes to events
  // Idea: don't subscribe to platforms, but make platforms send their InputParams to the server with altered monitor (Warn).
  // Connect the Sink/Stream to server. Problem: can server handle that many requests from platforms?
  _
  // run_platform(platform, centre_addr, our_address)
}
