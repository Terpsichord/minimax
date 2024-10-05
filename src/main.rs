use crate::games::tictactoe;
use crate::games::tictactoe::TicTacToeState;
use crate::minimax::State;
use clap::Parser;
use std::io;
use std::io::BufRead;

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
    app.run().await?;
    Ok(())
}

fn play() {
    let mut state = TicTacToeState::new();
    let mut input_moves = io::stdin().lock().lines().map(|line| {
        line.expect("failed to read input")
            .parse::<tictactoe::Move>()
            .expect("failed to parse move")
    });
    loop {
        // Play computer move
        let best_move = minimax::best_move(&state);
        dbg!(&best_move);
        state.place(best_move);
        println!("{:#?}\n", state);

        // Play user move
        print!("Enter move (in the format \"xy\"): ");
        let player_move = input_moves.next().expect("failed to read input");

        state.place(player_move);

        if state.is_terminal() {
            break;
        }
    }
}
