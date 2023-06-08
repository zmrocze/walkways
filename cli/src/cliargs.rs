use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
  /// Optional name to operate on
  // name: Option<String>,

  /// Sets a custom config file
  #[arg(short, long, value_name = "FILE")]
  pub config: Option<PathBuf>,

  // /// Turn debugging information on
  // #[arg(short, long, action = clap::ArgAction::Count)]
  // debug: u8,
  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
  /// does testing things
  Test {
    /// lists test values
    #[arg(short, long)]
    list: bool,
  },
  /// does running things
  Run {
    /// lists run values?
    #[arg(short, long)]
    list: bool,
  },
  Config {},
  Error {},
}
