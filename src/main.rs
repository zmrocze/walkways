#[cfg(not(debug_assertions))]
use human_panic::setup_panic;

#[cfg(debug_assertions)]
extern crate better_panic;

use utils::app_config::AppConfig;
use utils::error::Result;
use utils::logger::install_logger;

fn main() -> Result<()> {
  // Human Panic. Only enabled when *not* debugging.
  #[cfg(not(debug_assertions))]
  {
    setup_panic!();
  }

  // Better Panic. Only enabled *when* debugging.
  #[cfg(debug_assertions)]
  {
    better_panic::Settings::debug()
      .most_recent_first(false)
      .lineno_suffix(true)
      .verbosity(better_panic::Verbosity::Full)
      .install();
  }

  // Initialize Configuration
  let config_contents = include_str!("resources/default_config.toml");
  AppConfig::init(Some(config_contents))?;

  // Setup Logging
  install_logger()?;

  // Match Commands
  cli::cli_match()
}
