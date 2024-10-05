use crate::app::GameId;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Back,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    OpenGame(GameId),
    CloseGame,
}
