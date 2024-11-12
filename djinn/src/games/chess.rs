use crate::games::{Game, WinState};
use crate::minimax;
use crate::minimax::Player;
use itertools::Itertools;
use shakmaty::{san::San, ByColor, ByRole, Color, Move, Outcome, Piece, Position, Role, Square};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default)]
pub struct Chess(shakmaty::Chess, Vec<San>);

impl Game for Chess {
    fn name(&self) -> String {
        "Chess".to_string()
    }

    fn thumbnail(&self) -> String {
        "   │ ♚ │ ♞
───┼───┼───
 ♟ │ ♕ │
───┼───┼───
 ♔ │   │  "
            .to_string()
    }

    fn display(&self) -> String {
        self.to_string()
    }

    fn display_size(&self) -> (u16, u16) {
        (36, 18)
    }

    fn move_history(&self) -> Vec<String> {
        self.1.iter().map(San::to_string).collect()
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
        San::from_move(&self.0, &minimax::best_move(self, 3)).to_string()
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Chess {
    fn get_piece_char(piece: Piece) -> char {
        match piece {
            Piece {
                color: Color::White,
                role: Role::King,
            } => '♔',
            Piece {
                color: Color::White,
                role: Role::Queen,
            } => '♕',
            Piece {
                color: Color::White,
                role: Role::Rook,
            } => '♖',
            Piece {
                color: Color::White,
                role: Role::Bishop,
            } => '♗',
            Piece {
                color: Color::White,
                role: Role::Knight,
            } => '♘',
            Piece {
                color: Color::White,
                role: Role::Pawn,
            } => '♙',

            Piece {
                color: Color::Black,
                role: Role::King,
            } => '♚',
            Piece {
                color: Color::Black,
                role: Role::Queen,
            } => '♛',
            Piece {
                color: Color::Black,
                role: Role::Rook,
            } => '♜',
            Piece {
                color: Color::Black,
                role: Role::Bishop,
            } => '♝',
            Piece {
                color: Color::Black,
                role: Role::Knight,
            } => '♞',
            Piece {
                color: Color::Black,
                role: Role::Pawn,
            } => '♟',
        }
    }

    #[allow(dead_code)]
    fn get_piece_letter(piece: Piece) -> char {
        match piece {
            Piece {
                color: Color::White,
                role: Role::King,
            } => 'K',
            Piece {
                color: Color::White,
                role: Role::Queen,
            } => 'Q',
            Piece {
                color: Color::White,
                role: Role::Rook,
            } => 'R',
            Piece {
                color: Color::White,
                role: Role::Bishop,
            } => 'B',
            Piece {
                color: Color::White,
                role: Role::Knight,
            } => 'N',
            Piece {
                color: Color::White,
                role: Role::Pawn,
            } => 'P',

            Piece {
                color: Color::Black,
                role: Role::King,
            } => 'k',
            Piece {
                color: Color::Black,
                role: Role::Queen,
            } => 'q',
            Piece {
                color: Color::Black,
                role: Role::Rook,
            } => 'r',
            Piece {
                color: Color::Black,
                role: Role::Bishop,
            } => 'b',
            Piece {
                color: Color::Black,
                role: Role::Knight,
            } => 'n',
            Piece {
                color: Color::Black,
                role: Role::Pawn,
            } => 'p',
        }
    }

    const FLIP: [usize; 64] = [
        56, 57, 58, 59, 60, 61, 62, 63, 48, 49, 50, 51, 52, 53, 54, 55, 40, 41, 42, 43, 44, 45, 46,
        47, 32, 33, 34, 35, 36, 37, 38, 39, 24, 25, 26, 27, 28, 29, 30, 31, 16, 17, 18, 19, 20, 21,
        22, 23, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7,
    ];

    // Evaluate a heuristic value for non-terminal positions
    fn heuristic_value(position: &shakmaty::Chess) -> f32 {
        let color_diff = |color: ByColor<f32>| color.white - color.black;

        let material_count = position.board().material();
        let piece_values = [100, 320, 330, 550, 900];
        let count = |material: ByRole<u8>| {
            material
                .into_iter()
                .map(u16::from)
                .zip(piece_values)
                .map(|(count, score)| count * score)
                .sum::<u16>() as f32
        };

        let material = color_diff(material_count.map(count));

        let tables = Self::piece_square_tables();

        let (role_bitboards, color_bitboards) = position.board().clone().into_bitboards();

        let pst = color_diff(ByColor::new_with(|color| {
            let bitboards = role_bitboards.map(|board| board & *color_bitboards.get(color));
            tables
                .zip(bitboards)
                .map(|(table, bitboard)| {
                    bitboard
                        .into_iter()
                        .map(|square| {
                            f32::from(
                                table[match color {
                                    Color::White => Self::FLIP[square as usize],
                                    Color::Black => square as usize,
                                }],
                            )
                        })
                        .sum::<f32>()
                })
                .into_iter()
                .sum()
        }));

        material + pst
    }

    const fn piece_square_tables() -> ByRole<[i8; 64]> {
        let pawn = [
            0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10,
            5, 5, 10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5,
            10, 10, -20, -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let knight = [
            -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15,
            15, 10, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5,
            10, 15, 15, 10, 5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30,
            -40, -50,
        ];
        let bishop = [
            -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10,
            5, 0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10,
            10, 10, 10, 10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10,
            -20,
        ];
        let rook = [
            0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0,
            0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
            0, 0, -5, 0, 0, 0, 5, 5, 0, 0, 0,
        ];
        let queen = [
            -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5,
            0, -10, -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10,
            -10, 0, 5, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
        ];
        let king = [
            -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30,
            -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30,
            -30, -40, -40, -30, -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0,
            0, 20, 20, 20, 30, 10, 0, 0, 10, 30, 20,
        ];
        ByRole {
            pawn,
            knight,
            bishop,
            rook,
            queen,
            king,
        }
    }
}

impl Display for Chess {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "  ┌───┬───┬───┬───┬───┬───┬───┬───┐")?;
        for rank in (0..8).rev() {
            write!(f, "{} │", rank + 1)?;
            for file in 0..8 {
                write!(
                    f,
                    " {} │",
                    self.0
                        .board()
                        .piece_at(Square::new(rank * 8 + file))
                        .map(Self::get_piece_char)
                        .unwrap_or(' ')
                )?;
            }
            if rank != 0 {
                writeln!(f, "\n  ├───┼───┼───┼───┼───┼───┼───┼───┤")?;
            }
        }
        write!(
            f,
            "\n  └───┴───┴───┴───┴───┴───┴───┴───┘\n    a   b   c   d   e   f   g   h"
        )
    }
}

impl minimax::State<f32, Move> for Chess {
    fn is_terminal(&self) -> bool {
        self.0.outcome().is_some()
    }

    fn evaluation(&self) -> f32 {
        match self.0.outcome() {
            Some(Outcome::Decisive {
                winner: Color::White,
            }) => f32::INFINITY,
            Some(Outcome::Decisive {
                winner: Color::Black,
            }) => f32::NEG_INFINITY,
            Some(Outcome::Draw) => 0.0,
            None => Self::heuristic_value(&self.0),
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
