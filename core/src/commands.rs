
use utils::app_config::AppConfig;
use utils::error::Result;
use std::fs::File;
use tracing::{info};

/// Show the configuration file
pub fn config() -> Result<()> {
    let config = AppConfig::fetch()?;
    println!("{:#?}", config);

    Ok(())
}

/// Simulate an error
pub fn simulate_error() -> Result<()> {

    // Log this Error simulation
    info!("We are simulating an error");

    // Simulate an error
    simulate_error_aux()?;

    // We should never get here...
    Ok(())
}

/// Return, randomly, true or false
pub fn simulate_error_aux() -> Result<()> {
    // Trigger an error
    File::open("thisfiledoesnotexist")?;

    Ok(())
}
