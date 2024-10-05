pub mod tictactoe;

use std::fmt::Debug;

pub trait Game: Debug {
    fn name(&self) -> &'static str;
    fn thumbnail(&self) -> &'static str;
    fn display(&self) -> String;
    fn display_size(&self) -> (u16, u16);
    fn move_history(&self) -> String;
    fn is_valid_move(&self, move_: &str) -> bool;
    fn play_move(&mut self, move_: &str);
}
