pub mod new;

use anyhow::Result;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new Cargo workspace with optional members
    New(new::NewArgs),
}

impl Command {
    pub fn run(&self) -> Result<()> {
        match self {
            Command::New(args) => args.run(),
        }
    }
}
