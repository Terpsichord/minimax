#![allow(dead_code)]

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};

use app::App;
use cli::Cli;
use color_eyre::Result;

mod minimax;

mod action;
mod app;
mod cli;
mod components;
pub mod config;
pub mod errors;
mod games;
pub mod logging;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    errors::init()?;
    logging::init()?;

    let args = Cli::parse();

    let mut app = App::new()?;
    if let Some(game) = args.game {
        app.open_game_from_name(&game).unwrap_or_else(|_| {
            let mut cmd = Cli::command();
            cmd.error(
                ErrorKind::InvalidValue,
                format!("Can't find game with name \"{game}\""),
            )
            .exit();
        });
    }

    app.run().await?;

    Ok(())
}
