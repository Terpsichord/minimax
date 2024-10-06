use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::games::{Game, WinState};
use crate::minimax::{self, Player, State};

#[derive(Debug, Default)]
pub struct TicTacToe(TicTacToeState);

impl Game for TicTacToe {
    fn name(&self) -> &'static str {
        "Tic Tac Toe"
    }

    fn thumbnail(&self) -> &'static str {
        " X │ O │
───┼───┼───
   │ X │
───┼───┼───
 O │   │ X "
    }

    fn display(&self) -> String {
        self.0.board.to_string()
    }

    fn display_size(&self) -> (u16, u16) {
        (16, 8)
    }

    fn move_history(&self) -> Vec<(String, Option<String>)> {
        self.0
            .move_history
            .chunks(2)
            .map(|turn| (turn[0].to_string(), turn.get(1).map(|m| m.to_string())))
            .collect()
    }

    fn win_state(&self) -> Option<WinState> {
        if self.0.is_terminal() {
            if self.0.winner.is_some() {
                Some(WinState::Decisive)
            } else {
                Some(WinState::Draw)
            }
        } else {
            None
        }
    }

    fn is_valid_move(&self, move_: &str) -> bool {
        if let Ok(move_) = Move::from_str(move_) {
            self.0.board.0[move_.y][move_.x] == Tile::Empty
        } else {
            false
        }
    }

    fn play_move(&mut self, move_: &str) {
        self.0
            .place(Move::from_str(move_).expect("expected valid move"))
    }

    fn computer_move(&self) -> String {
        minimax::best_move(&self.0, u32::MAX).to_string()
    }

    fn reset(&mut self) {
        self.0 = TicTacToeState::default();
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
enum Tile {
    #[default]
    Empty,
    Cross,
    Nought,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Tile::Empty => " ",
                Tile::Nought => "O",
                Tile::Cross => "X",
            }
        )
    }
}

impl From<Player> for Tile {
    fn from(value: Player) -> Self {
        match value {
            Player::Max => Tile::Cross,
            Player::Min => Tile::Nought,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Move {
    x: usize,
    y: usize,
    tile: Tile,
}

impl FromStr for Move {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut coords = s.chars();
        let x = match coords.next().ok_or("expected x-coordinate")? {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            _ => Err("invalid x-coordinate")?,
        };

        let y = match coords.next().ok_or("expected y-coordinate")? {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            _ => Err("invalid y-coordinate")?,
        };

        if coords.next().is_some() {
            Err("too many coordinates")?;
        }

        Ok(Move {
            x,
            y,
            tile: Tile::Empty,
        })
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let x = match self.x {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            _ => unreachable!("invalid x-coordinate"),
        };

        let y = match self.y {
            0 => '1',
            1 => '2',
            2 => '3',
            _ => unreachable!("invalid y-coordinate"),
        };

        write!(f, "{}{}", x, y)
    }
}

#[derive(Clone, Default)]
pub struct Board([[Tile; 3]; 3]);

impl Board {
    fn new() -> Self {
        Self::default()
    }

    fn check_win(&self, action: &Move) -> bool {
        let Move { x, y, tile } = *action;
        let board = self.0;

        let n = 3;

        // Check row
        let row = (0..n).all(|i| board[i][x] == tile);

        // Check column
        let column = (0..n).all(|i| board[y][i] == tile);

        // Check main diagonal
        let main_diag = x == y && (0..n).all(|i| board[i][i] == tile);

        // Check anti-diagonal
        let anti_diag = x + y == n - 1 && (0..n).all(|i| board[i][n - 1 - i] == tile);

        row || column || main_diag || anti_diag
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if f.alternate() {
            let board = self.0;
            write!(
                f,
                "Board(
     {} | {} | {}
    ---+---+---
     {} | {} | {}
    ---+---+---
     {} | {} | {}
)",
                board[0][0],
                board[0][1],
                board[0][2],
                board[1][0],
                board[1][1],
                board[1][2],
                board[2][0],
                board[2][1],
                board[2][2],
            )
        } else {
            f.debug_tuple("Board").field(&self.0).finish()
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let board = self.0;
        write!(
            f,
            "  ┌───┬───┬───┐
1 │ {} │ {} │ {} │
  ├───┼───┼───┤
2 │ {} │ {} │ {} │
  ├───┼───┼───┤
3 │ {} │ {} │ {} │
  └───┴───┴───┘
    a   b   c",
            board[0][0],
            board[0][1],
            board[0][2],
            board[1][0],
            board[1][1],
            board[1][2],
            board[2][0],
            board[2][1],
            board[2][2]
        )
    }
}

#[derive(Default, Debug)]
pub struct TicTacToeState {
    board: Board,
    player: Player,
    winner: Option<Player>,
    draw: bool,
    move_history: Vec<Move>,
}

impl TicTacToeState {
    pub fn new() -> Self {
        Self::default()
    }

    // Create a new Tic-Tac-Toe state with a given board where Crosses are to move
    fn with_board(board: Board) -> Self {
        Self {
            board,
            ..Default::default()
        }
    }

    pub fn place(&mut self, position: impl Into<Move>) {
        let mut action = position.into();
        action.tile = self.player.into();
        self.play_move(action);
    }

    fn play_move(&mut self, action: Move) {
        *self = self.result(&action);
    }
}

impl minimax::State<f32, Move> for TicTacToeState {
    fn is_terminal(&self) -> bool {
        self.winner.is_some() || self.draw
    }

    fn heuristic_value(&self) -> f32 {
        match self.winner {
            Some(Player::Min) => f32::NEG_INFINITY,
            Some(Player::Max) => f32::INFINITY,
            None => 0.0,
        }
    }

    fn current_player(&self) -> Player {
        self.player
    }

    fn actions(&self) -> Vec<Move> {
        let tile = self.player.into();

        let mut actions = vec![];

        for x in 0..3 {
            for y in 0..3 {
                if self.board.0[y][x] == Tile::Empty {
                    actions.push(Move { x, y, tile })
                }
            }
        }

        actions
    }

    // TODO: don't produce a whole new State for each minimax node
    fn result(&self, action: &Move) -> Self {
        assert_eq!(self.board.0[action.y][action.x], Tile::Empty);

        let mut board = self.board.clone();
        board.0[action.y][action.x] = action.tile;

        let win = board.check_win(action);
        let full = !board.0.as_flattened().iter().any(|tile| matches!(tile, Tile::Empty));
        let draw = full && !win;

        // TODO: remove the clones here
        let mut move_history = self.move_history.clone();
        move_history.push(action.clone());

        Self {
            board,
            player: self.player.opposite(),
            winner: if win { Some(self.player) } else { None },
            draw,
            move_history,
        }
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::minimax::State;

    #[test]
    fn test_win_diagonal() {
        let initial_state = TicTacToeState::with_board(Board([
            [Tile::Empty, Tile::Nought, Tile::Cross],
            [Tile::Nought, Tile::Cross, Tile::Empty],
            [Tile::Empty, Tile::Empty, Tile::Empty],
        ]));

        let winning_move = Move {
            x: 0,
            y: 2,
            tile: Tile::Cross,
        };

        let state = initial_state.result(&winning_move);

        assert_eq!(state.winner, Some(Player::Max));
    }

    #[test]
    fn win_horizontal() {
        let initial_state = TicTacToeState::with_board(Board([
            [Tile::Empty, Tile::Nought, Tile::Nought],
            [Tile::Nought, Tile::Cross, Tile::Nought],
            [Tile::Cross, Tile::Cross, Tile::Cross],
        ]));
    }

    #[test]
    fn ensure_draw() {
        let state = TicTacToeState::with_board(Board([
            [Tile::Cross, Tile::Empty, Tile::Empty],
            [Tile::Empty, Tile::Empty, Tile::Empty],
            [Tile::Empty, Tile::Empty, Tile::Empty],
        ]));
        let Move { x, y, .. } = minimax::best_move(&state);
        assert_eq!((x, y), (1, 1));
    }
}
