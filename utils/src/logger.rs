use super::error::{Error, Result};

// use tracing_subscriber::{fmt};
// use tracing_subscriber::prelude::*;
use tracing_subscriber;

// In code use:
//
// use std::{error::Error, io};
// use tracing::{trace, debug, info, warn, error, Level};

// // the `#[tracing::instrument]` attribute creates and enters a span
// // every time the instrumented function is called. The span is named after
// // the function or method. Parameters passed to the function are recorded as fields.
// #[tracing::myfn]
// pub fn myfn
//

pub fn install_logger() -> Result<()> {
  let subscriber = tracing_subscriber::fmt().finish();

  return tracing::subscriber::set_global_default(subscriber).map_err(Error::from);
}

// pub

// let fmt_layer = fmt::layer()
//     .with_target(false);
// let filter_layer = EnvFilter::try_from_default_env()
//     .or_else(|_| EnvFilter::try_new("info"))
//     .unwrap();
//  return tracing_subscriber::registry()
//     .with(filter_layer)
//     .with(fmt_layer)
//     .init();
