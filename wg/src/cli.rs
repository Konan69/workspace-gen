use anyhow::Result;
use clap::Parser;

use crate::commands::Command;

#[derive(Debug, Parser)]
#[command(name = "wg", about = "Cargo workspace generator", version)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        self.command.run()
    }
}
