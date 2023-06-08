use clap::Parser;

use core::commands;
use utils::app_config::AppConfig;
use utils::error::Result;

mod cliargs;
use cliargs::{Cli, Commands::*};

/// Match commands
pub fn cli_match() -> Result<()> {
  // Get matches
  let Cli { config, command } = Cli::parse();

  // Merge clap config file if the value is set
  AppConfig::merge_config(config)?;

  // Matches Commands or display help
  return match command {
    Test { list } => Ok(println!("Tests!")),
    Run { list } => Ok(println!("Runs!")),
    Config {} => commands::config(),
    Error {} => commands::simulate_error(),
  };
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert()
}
