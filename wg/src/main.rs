mod cli;
mod commands;

use clap::Parser;

use crate::cli::Cli;

fn main() {
    if let Err(err) = Cli::parse().run() {
        eprintln!("wg error: {err:?}");
        std::process::exit(1);
    }
}
