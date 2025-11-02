mod cli;
mod config;
mod error;
mod fs;
mod git;

use anyhow::Result;
use tracing_subscriber;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse CLI arguments
    let args = cli::parse_args();

    // Execute the appropriate command
    cli::execute_command(args)?;

    Ok(())
}
