use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Env;

mod clear;
mod cli;
mod dir;
mod get_tef;
mod list;
mod msg;
mod serve;
mod utils;

fn main() -> Result<()> {
    env_logger::init_from_env(Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));

    let cmd = cli::Command::try_parse().context("Parsing command line")?;
    match cmd {
        cli::Command::List(list) => list::run(list),
        cli::Command::Serve(serve) => serve::run(serve),
        cli::Command::GetTEF(g) => get_tef::run(g),
        cli::Command::Dir(d) => dir::run(d),
        cli::Command::Clear(cl) => clear::run(cl),
    }?;

    Ok(())
}
