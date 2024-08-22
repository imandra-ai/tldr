use anyhow::{Context, Result};
use clap::Parser;

mod cli;
mod get_tef;
mod list;
mod msg;
mod serve;
mod utils;

fn main() -> Result<()> {
    env_logger::init();

    let cmd = cli::Command::try_parse().context("Parsing command line")?;
    match cmd {
        cli::Command::List(list) => list::run(list),
        cli::Command::Serve(serve) => serve::run(serve),
        cli::Command::GetTEF(g) => get_tef::run(g),
    }?;

    Ok(())
}
