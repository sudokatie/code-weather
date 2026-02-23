pub mod cli;
pub mod config;
pub mod error;
pub mod analysis;
pub mod git;
pub mod languages;
pub mod weather;
pub mod output;

use cli::Args;
use error::Result;

pub fn run(_args: Args) -> Result<()> {
    // TODO: Implement main logic
    println!("code-weather v0.1.0");
    Ok(())
}
