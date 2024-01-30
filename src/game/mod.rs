mod bitboard;
mod state;

use std::fmt;

pub use bitboard::Bitboard;
pub use state::{GameState, MoveType, Side};
use thiserror::Error;

pub fn extract_lsb(x: u64) -> u64 {
    let (y, _) = x.overflowing_neg();
    x & y
}

/// An enum representing an Othello move.
/// 
/// `Play(index)` represents a disk placed at the index on the board
/// `Pass` represents a pass
#[derive(Clone, Copy, Debug)]
pub enum Move {
    Play(u8),
    Pass,
}

#[derive(Debug, Error)]
#[error("Invalid move played.")]
pub struct InvalidMoveError;


/// A high-level Othello board representation
/// 
/// Provides a higher level API for interacting with an Othello board. 
pub struct Board {
    bitboard: Bitboard,
    game_state: GameState,
}

fn isolate_lsb(x: u64) -> u64 {
    let (y, _) = x.overflowing_neg();
    x & y
}

impl Board {
    pub fn get_moves(&self) -> Vec<Move> {
        let mut valid_moves = self.bitboard.get_moves();
        // Create a vec with enough space for all valid moves, or at least one space for a pass
        let mut moves = Vec::with_capacity(valid_moves.count_ones().try_into().unwrap_or(0).max(1));
        // Add all valid moves to the vec
        while valid_moves > 0 {
            let move_mask = isolate_lsb(valid_moves);
            valid_moves &= !move_mask;
            moves.push(Move::Play(move_mask.trailing_zeros().try_into().unwrap_or(0)));
        }
        // If the vec is empty, then there were no valid moves, so add a pass.
        if moves.len() == 0 {
            moves.push(Move::Pass);
        }
        moves
    }

    pub fn play(&mut self, m: Move) -> Result<(), InvalidMoveError> {
        let valid_moves = self.bitboard.get_moves();
        if let Move::Play(idx) = m {
            let move_mask = 1 << idx;
            if valid_moves & move_mask == 0 {
                // Move is not valid, exit early with an error.
                return Err(InvalidMoveError);
            }
            self.bitboard = self.bitboard.make_move(move_mask);
            self.game_state = self.game_state.play();
        } else {
            if valid_moves == 0 {
                // There were non-pass moves, exit early with an error.
                return Err(InvalidMoveError);
            }
            self.bitboard = self.bitboard.pass();
            self.game_state = self.game_state.pass();
        }
        Ok(())
    }

    pub fn get_bitboard(&self) -> Bitboard {
        self.bitboard
    }

    pub fn get_game_state(&self) -> GameState {
        self.game_state
    }
}

impl Default for Board {
    /// Initializes the board to the Othello starting position.
    fn default() -> Self {
        Self { bitboard: Default::default(), game_state: Default::default() }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (mut white, mut black) = match self.game_state.get_side() {
            Side::Black => (self.bitboard.get_op(), self.bitboard.get_me()),
            Side::White => (self.bitboard.get_me(), self.bitboard.get_op()),
        };
        let num_black = black.count_ones();
        let num_white = white.count_ones();
        for idx in 0..64 {
            if idx % 8 == 0 {
                write!(f, "{} ", (idx / 8) + 1)?
            }
            match (black & 1, white & 1) {
                (1, 0) => write!(f, "X ")?,
                (0, 1) => write!(f, "O ")?,
                _ => write!(f, ". ")?,
            }
            if idx % 8 == 7 {
                if idx / 8 == 3 {
                    write!(f, "B: {:2}", num_black)?;
                    match self.game_state.get_side() {
                        Side::Black => write!(f, " <-")?,
                        Side::White => write!(f, "   ")?,
                    };
                }
                if idx / 8 == 4 {
                    write!(f, "W: {:2}", num_white)?;
                    match self.game_state.get_side() {
                        Side::Black => write!(f, "   ")?,
                        Side::White => write!(f, " <-")?,
                    };
                }
                writeln!(f)?;
            }
            black >>= 1;
            white >>= 1;
        }
        writeln!(f, "  a b c d e f g h")
    }
}