// use enum_display_derive;

use futures::sink::Sink;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};
use std::pin::Pin;
use tokio::time::Instant;
use uom::si::f64::Velocity;

use crate::common::InputParams;
use crate::platform::Error;
use crate::platform::Monitor;
use crate::track::Track;

pub enum Alert {
  // Unrecoverable critical situation
  Critical(CriticalAlert),
  // Undesired state within our assumed error margins
  Unsafe(UnsafeAlert),
  // Warning, information
  Warn(WarnAlert),
}

#[derive(enum_display_derive::Display, Serialize, Deserialize, Debug)]
pub enum CriticalAlert {}

#[derive(enum_display_derive::Display, Serialize, Deserialize, Debug)]
pub enum UnsafeAlert {
  MaxVelocityExceeded(DisplayVelocity),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DisplayVelocity(Velocity);

impl Display for DisplayVelocity {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Velocity({:?})", self.0)
  }
}

#[derive(enum_display_derive::Display, Serialize, Deserialize, Debug)]
enum WarnAlert {}

impl From<Alert> for crate::proto::centre::Alert {
  fn from(value: Alert) -> Self {
    fn encoded_alert_msg<T: ?Sized + Serialize>(alert: &T) -> Vec<u8> {
      let alert_msg = Vec::<u8>::new();
      ciborium::into_writer(alert, alert_msg).unwrap();
      alert_msg
    }
    match value {
      Alert::Critical(alert) => crate::proto::centre::Alert {
        alert_type: crate::proto::centre::alert::AlertType::Critical.into(),
        message: encoded_alert_msg(&alert),
      },
      Alert::Unsafe(alert) => crate::proto::centre::Alert {
        alert_type: crate::proto::centre::alert::AlertType::Unsafe.into(),
        message: encoded_alert_msg(&alert),
      },
      Alert::Warn(alert) => crate::proto::centre::Alert {
        alert_type: crate::proto::centre::alert::AlertType::Warn.into(),
        message: encoded_alert_msg(&alert),
      },
    }
  }
}

pub struct SimpleMonitor {
  track: Track,
}

impl Monitor<InputParams> for SimpleMonitor {
  fn check(
    &self,
    run_status: &crate::common::RunStatus,
    input: &InputParams,
  ) -> Vec<crate::proto::centre::Alert> {
    let mut alerts = Vec::new();
    if input.velocity > self.track.max_vel {
      alerts.push(Alert::Unsafe(UnsafeAlert::MaxVelocityExceeded(
        DisplayVelocity(input.velocity),
      )))
    }
    alerts.into_iter().map(|x| x.into()).collect()
  }
}

#[derive(Debug, Clone, Copy)]
pub struct PlatformInfo<I> {
  input_params: I,
  timestamp: std::time::Instant,
  platform_id: u32,
}

impl<I: Display> Display for PlatformInfo<I> {
  // This trait requires `fmt` with this exact signature.
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "[{:#?}] platform {}: {}",
      self.timestamp, self.platform_id, self.input_params
    )
  }
}

pub struct CollectingMonitor<'a, I, S: Sink<PlatformInfo<I>, Error = Error>> {
  monitor: Box<dyn Monitor<I>>,
  platform_id: u32,
  sink: Pin<&'a mut S>,
}

// impl<'a, I, S: Sink<PlatformInfo<InputParams>, Error = Error>>
impl<'a, I: Display, S: Sink<PlatformInfo<I>, Error = Error>> Monitor<I>
  for CollectingMonitor<'a, I, S>
{
  fn check(
    &self,
    run_status: &crate::common::RunStatus,
    input: &I,
  ) -> Vec<crate::proto::centre::Alert> {
    let log_event = PlatformInfo {
      input_params: input,
      timestamp: Instant::now().into(),
      platform_id: self.platform_id,
    };
    // TODO: figure out how to use Sink
    println!("{}", log_event);
    self.monitor.check(run_status, input)
  }
}
