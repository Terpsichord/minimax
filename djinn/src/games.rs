pub mod chess;
pub mod tictactoe;

use std::fmt::Debug;

#[derive(Copy, Clone, Debug)]
pub enum WinState {
    Decisive,
    Draw,
}

pub trait Game: Send + Sync {
    fn name(&self) -> String;
    fn thumbnail(&self) -> String;
    fn display(&self) -> String;
    fn display_size(&self) -> (u16, u16);
    fn move_history(&self) -> Vec<String>;
    fn win_state(&self) -> Option<WinState>;
    fn is_valid_move(&self, move_: &str) -> bool;
    fn play_move(&mut self, move_: &str);
    fn computer_move(&self) -> String;
    fn reset(&mut self);
}
