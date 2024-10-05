pub mod chess;
pub mod tictactoe;

use std::fmt::Debug;

pub enum WinState {
    Decisive,
    Draw,
}

pub trait Game: Debug {
    fn name(&self) -> &'static str;
    fn thumbnail(&self) -> &'static str;
    fn display(&self) -> String;
    fn display_size(&self) -> (u16, u16);
    fn move_history(&self) -> Vec<(String, Option<String>)>;
    fn win_state(&self) -> Option<WinState>;
    fn is_valid_move(&self, move_: &str) -> bool;
    fn play_move(&mut self, move_: &str);
    fn computer_move(&self) -> String;
    fn reset(&mut self);
}
