use core::panic;

use uom::{
  si::{
    f64::{Acceleration, Length, Velocity},
    velocity::{centimeter_per_second, kilometer_per_hour},
  },
  ConstZero,
};

use crate::{
  common::{InputParams, RunStatus, TrackPosition},
  platform::Calculate,
};

struct TrackSection {
  /// section beginning, section lasts till next section
  beginning: TrackPosition,
  section_type: SectionType,
}

impl TrackSection {
  fn new(beginning: TrackPosition, section_type: SectionType) -> TrackSection {
    TrackSection {
      beginning: beginning,
      section_type: section_type,
    }
  }
}

pub enum SectionType {
  /// Aim to move at max speed at this section
  MaxSpeed,
  /// Aim to decelerate fast enough to reach exit_velocity when we'll exit the section.
  Decelerate { exit_velocity: Velocity },
}

pub struct Track {
  /// full length of the track (whole cycle)
  pub track_length: Length,
  pub max_vel: Velocity,
  pub max_acc: Acceleration,
  /// Maximal stopping deceleration, should be negative
  pub max_deceleration: Acceleration,
  // min_vel: Velocity,
  // min_acc: Acceleration,
  pub sections: Vec<TrackSection>,
}

impl Track {
  /// Standard track has a loop shape with minimal velocities at its ends.
  /// Otherwise platforms should reach maximal possible speed in the middle, accelerate after the breakpoints and decelerate before the breakpoints.
  /// We assume track is long enough that we actually can reach top speed (for ease). This typically would mean track is longer than 60m (overground).
  pub const fn standard_track(
    track_length: Length,
    max_vel: Velocity,
    max_acc: Acceleration,
    max_deceleration: Acceleration,
    min_vel: Velocity, // Velocity at the track entry/exit. Stationary velocity is always 0.
  ) -> Track {
    // (min_vel - max_vel) / max_deceleration = deceleration_time
    // Distance covered during deceleration is like the distance covered with average speed in the deceleration time:
    // (0.5 * (max_vel + min_vel)) * deceleration_time = deceleration_distance
    let deceleration_distance = 0.5 * (max_vel + min_vel) * (min_vel - max_vel) / max_deceleration;
    // For ease of calculations let's assume the deceleration distance is shorter than half the track (half is overground, half is underground)
    if deceleration_distance >= 0.5 * track_length {
      panic!("Unimplemented track calculation for short tracks");
    }
    let new_track_position = |pos: Length| TrackPosition::new(track_length, pos);
    // Here we are nearing till the half of the track (till the end of overground half), so we need to slow down.
    let first_slowingpoint = new_track_position(0.5 * track_length - deceleration_distance);
    let track_half = new_track_position(0.5 * track_length);
    // Analogous to first_slowingpoing but in the second half of the track (underground half)
    let second_slowingpoint = new_track_position(track_length - deceleration_distance);
    let sections = vec![
      TrackSection::new(new_track_position(Length::ZERO), SectionType::MaxSpeed),
      TrackSection::new(
        first_slowingpoint,
        SectionType::Decelerate {
          exit_velocity: min_vel,
        },
      ),
      TrackSection::new(track_half, SectionType::MaxSpeed),
      TrackSection::new(
        second_slowingpoint,
        SectionType::Decelerate {
          exit_velocity: min_vel,
        },
      ),
    ];

    Track {
      track_length: track_length,
      max_vel: max_vel,
      max_acc: max_acc,
      max_deceleration: max_deceleration,
      sections: sections,
    }
  }
}

pub struct SimpleCalculate {
  track: Track,
}

impl Calculate<InputParams, Acceleration> for SimpleCalculate {
  // TODO: incorporate below constraint on speed
  // # At any point in time, platform 1 (at the back) must be able to lower its speed to
  // # match the speed of platform 2 (in the front) at the exact loction platform 2 is at
  // # that point in time

  /// This implementation finds out its desired speed for the section and does one of the following:
  ///  1. doesn't accelerate if we are close to desired speed
  ///  2. accelerates with max acceleration
  ///  3. decelerates with max deceleration
  fn calculate(&self, run_status: &crate::common::RunStatus, input: &InputParams) -> Acceleration {
    let epsilon_vel = Velocity::new::<centimeter_per_second>(1.0);
    // Just check if we want to speed up or slow down and accelerate/decelerate fastest possible.
    let match_acc_to_desired_speed = |desired_speed: Velocity| {
      // slow down if faster than max_vel
      if self.track.max_vel < input.velocity {
        self.track.max_deceleration
      } else {
        let speed_diff = desired_speed - input.velocity;
        // we are almost at the desired speed, we don't accelerate for stability
        if speed_diff.abs() <= epsilon_vel {
          Acceleration::ZERO
        } else {
          if speed_diff.is_sign_positive() {
            self.track.max_acc // we need to speed up
          } else {
            self.track.max_deceleration // slow down
          }
        }
      }
    };
    match run_status {
      RunStatus::Stop => match_acc_to_desired_speed(Velocity::ZERO),
      RunStatus::Start => {
        // We need to find current section
        // We do a linear search, with an assumption that there's gonna be just a few sections
        // and binary search or some state tracking wouldn't be faster.
        // Alternatively we could hardcode if statements for the exact number of 4 sections (where 4 is the case now and maybe always)
        let n = self.track.sections.len();
        let section_ind = (0..n).find(|i|
          // we are in i-th section: from the section beginning to us is closer than to the section end
          self.track.sections[*i].beginning.distance_to(input.position) <= self.track.sections[*i].beginning.distance_to(self.track.sections[(*i+1) % n].beginning)
        ).unwrap(); // Will not be None for correct track sections with ascending beginnings
        match self.track.sections[section_ind].section_type {
          SectionType::MaxSpeed => match_acc_to_desired_speed(self.track.max_vel),
          SectionType::Decelerate { exit_velocity } => {
            let dist_to_section_exit = input
              .position
              .distance_to(self.track.sections[(section_ind + 1) % n].beginning);
            // At the section exit we should have speed exit_velocity, we are looking for current_desired_vel.
            // 0.5 * (exit_velocity + current_desired_vel) * (exit_velocity - current_desired_vel) / max_dec  = dist_to_section_exit
            // | ^ average speed till the section exit   |   | ^ time to section exit                       |
            // From the above:
            // current_desired_vel = sqrt(exit_velocity^2 - 2*max_dec*dist_to_section_exit)
            let current_desired_vel = (exit_velocity * exit_velocity
              - 2.0 * self.track.max_deceleration * dist_to_section_exit)
              .sqrt();
            match_acc_to_desired_speed(current_desired_vel)
          }
        }
      }
    }
  }
}
