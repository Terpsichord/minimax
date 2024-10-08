use std::fmt;
use std::fmt::{Display, Formatter};
use itertools::Itertools;
use crate::games::{Game, WinState};
use shakmaty::{san::San, Position, Square, Piece, Color, Role, Outcome, Move, ByRole};
use crate::minimax;
use crate::minimax::Player;

#[derive(Debug, Default)]
pub struct Chess(shakmaty::Chess, Vec<San>);

impl Game for Chess {
    fn name(&self) -> &'static str {
        "Chess"
    }

    fn thumbnail(&self) -> &'static str {
        "   │ ♚ │ ♞
───┼───┼───
 ♟ │ ♕ │
───┼───┼───
 ♔ │   │  "
    }

    fn display(&self) -> String {
        self.to_string()
    }

    fn display_size(&self) -> (u16, u16) {
        (36, 18)
    }

    fn move_history(&self) -> Vec<(String, Option<String>)> {
        self.1
            .chunks(2)
            .map(|turn| (turn[0].to_string(), turn.get(1).map(|m| m.to_string())))
            .collect()
    }

    fn win_state(&self) -> Option<WinState> {
        self.0.outcome().map(|outcome| match outcome {
            Outcome::Decisive { .. } => WinState::Decisive,
            Outcome::Draw => WinState::Draw,
        })
    }

    fn is_valid_move(&self, move_: &str) -> bool {
        move_
            .parse::<San>()
            .is_ok_and(|s| s.to_move(&self.0).is_ok())
    }

    fn play_move(&mut self, move_: &str) {
        let move_ = move_.parse::<San>().expect("invalid SAN move");
        self.0 = self
            .0
            .clone()
            .play(&move_.to_move(&self.0).expect("invalid move"))
            .unwrap();
        self.1.push(move_);
    }

    fn computer_move(&self) -> String {
        San::from_move(&self.0, &minimax::best_move(self, 2)).to_string()
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Chess {
    fn get_piece_char(piece: Piece) -> char {
        match piece {
            Piece { color: Color::White, role: Role::King } => '♔',
            Piece { color: Color::White, role: Role::Queen } => '♕',
            Piece { color: Color::White, role: Role::Rook } => '♖',
            Piece { color: Color::White, role: Role::Bishop } => '♗',
            Piece { color: Color::White, role: Role::Knight } => '♘',
            Piece { color: Color::White, role: Role::Pawn } => '♙',

            Piece { color: Color::Black, role: Role::King } => '♚',
            Piece { color: Color::Black, role: Role::Queen } => '♛',
            Piece { color: Color::Black, role: Role::Rook } => '♜',
            Piece { color: Color::Black, role: Role::Bishop } => '♝',
            Piece { color: Color::Black, role: Role::Knight } => '♞',
            Piece { color: Color::Black, role: Role::Pawn } => '♟',
        }
    }

    fn get_piece_letter(piece: Piece) -> char {
        match piece {
            Piece { color: Color::White, role: Role::King } => 'K',
            Piece { color: Color::White, role: Role::Queen } => 'Q',
            Piece { color: Color::White, role: Role::Rook } => 'R',
            Piece { color: Color::White, role: Role::Bishop } => 'B',
            Piece { color: Color::White, role: Role::Knight } => 'N',
            Piece { color: Color::White, role: Role::Pawn } => 'P',

            Piece { color: Color::Black, role: Role::King } => 'k',
            Piece { color: Color::Black, role: Role::Queen } => 'q',
            Piece { color: Color::Black, role: Role::Rook } => 'r',
            Piece { color: Color::Black, role: Role::Bishop } => 'b',
            Piece { color: Color::Black, role: Role::Knight } => 'n',
            Piece { color: Color::Black, role: Role::Pawn } => 'p',
        }
    }
}

impl Display for Chess {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "  ┌───┬───┬───┬───┬───┬───┬───┬───┐")?;
        for rank in (0..8).rev() {
            write!(f, "{} │", rank + 1)?;
            for file in 0..8 {
                write!(f, " {} │", self.0.board().piece_at(Square::new(rank * 8 + file)).map(Self::get_piece_char).unwrap_or(' '))?;
            }
            if rank != 0 {
                writeln!(f, "\n  ├───┼───┼───┼───┼───┼───┼───┼───┤")?;
            }
        }
        write!(f, "\n  └───┴───┴───┴───┴───┴───┴───┴───┘\n    a   b   c   d   e   f   g   h")
    }
}


impl minimax::State<f32, Move> for Chess {
    fn is_terminal(&self) -> bool {
        self.0.outcome().is_some()
    }

    fn heuristic_value(&self) -> f32 {
        match self.0.outcome() {
            Some(Outcome::Decisive { winner: Color::White }) => f32::INFINITY,
            Some(Outcome::Decisive { winner: Color::Black }) => f32::NEG_INFINITY,
            Some(Outcome::Draw) => 0.0,
            None => {
                let material = self.0.board().material();
                let count = |role: ByRole<u8>| role.pawn * 1 + role.knight * 3 + role.bishop * 3 + role.rook * 5 + role.queen * 8;
                count(material.white) as f32 - count(material.black) as f32
            }
        }
    }

    fn current_player(&self) -> Player {
        match self.0.turn() {
            Color::White => Player::Max,
            Color::Black => Player::Min,
        }
    }

    fn actions(&self) -> Vec<Move> {
        self.0.legal_moves().into_iter().collect_vec()
    }

    fn result(&self, action: &Move) -> Self {
        let mut history = self.1.clone();
        history.push(San::from_move(&self.0, action));
        let position = self.0.clone().play(action).expect("expected valid move");

        Chess(position, history)
    }
}