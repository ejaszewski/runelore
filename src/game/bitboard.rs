// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt;
use std::ops;
use std::simd::{cmp::SimdPartialEq, num::SimdUint, u64x4};

/// Mask representing all squares that are not on the A file.
const NOT_A_FILE: u64 = 0xfefefefefefefefe;

/// Mask representing all squares that are not on the H file.
const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;

/// Mask representing a filled board.
const FILLED: u64 = u64::MAX;

/// A vectorized Kogge-Stone flood fill
///
/// A standard [Kogge-Stone fill] that computes the fill in 4 directions in the
/// lanes of a 4-wide u64 SIMD vector. This same approach is used in the [Coin]
/// and [reason] Othello bots.
///
/// [Kogge-Stone fill]: https://www.chessprogramming.org/Kogge-Stone_Algorithm
/// [Coin]: https://github.com/Tenebryo/coin
/// [reason]: https://github.com/maxwells-daemons/reason
fn vectorized_fill<const SHR: bool>(generator: u64x4, propagator: u64x4) -> u64x4 {
    const SHIFTS: u64x4 = u64x4::from_array([1, 7, 8, 9]);

    let shift = match SHR {
        true => <u64x4 as ops::Shr>::shr,
        false => <u64x4 as ops::Shl>::shl,
    };
    let masks = match SHR {
        true => u64x4::from_array([NOT_H_FILE, NOT_A_FILE, FILLED, NOT_H_FILE]),
        false => u64x4::from_array([NOT_A_FILE, NOT_H_FILE, FILLED, NOT_A_FILE]),
    };

    let mut gen = generator;
    let mut pro = propagator;

    pro &= masks;
    gen |= pro & shift(gen, SHIFTS);
    pro &= shift(pro, SHIFTS);
    gen |= pro & shift(gen, SHIFTS << 1);
    pro &= shift(pro, SHIFTS << 1);
    gen | (pro & shift(gen, SHIFTS << 2))
}

/// A vectorized directional shift
///
/// A standard [single shift] that computes the shift in 4 directions in the
/// lanes of a 4-wide u64 SIMD vector.
///
/// [single shift]:
///     https://www.chessprogramming.org/General_Setwise_Operations#OneStepOnly
fn vectorized_shift<const SHR: bool>(gen: u64x4) -> u64x4 {
    const SHIFTS: u64x4 = u64x4::from_array([1, 7, 8, 9]);

    let shift = match SHR {
        true => <u64x4 as ops::Shr>::shr,
        false => <u64x4 as ops::Shl>::shl,
    };
    let masks = match SHR {
        true => u64x4::from_array([NOT_H_FILE, NOT_A_FILE, FILLED, NOT_H_FILE]),
        false => u64x4::from_array([NOT_A_FILE, NOT_H_FILE, FILLED, NOT_A_FILE]),
    };

    shift(gen, SHIFTS) & masks
}

/// A low-level bitboard implemenation for Othello
///
/// Implements move generation and move making for an Othello board relative to
/// the side to move. The bitboard operations are implemented using u64 bit
/// manipulations and use explicit vectorization using `portable_simd` where
/// possible.
#[derive(Clone, Copy)]
pub struct Bitboard {
    me: u64,
    op: u64,
}

impl Bitboard {
    /// Returns a mask of empty disks
    pub fn empties(self) -> u64 {
        !(self.me | self.op)
    }

    /// Returns a mask containing all valid moves
    pub fn get_moves(self) -> u64 {
        // Copy the board data into vectors
        let me_vec = u64x4::splat(self.me);
        let op_vec = u64x4::splat(self.op);

        // Compute the fill from the board data
        let fill_shr = vectorized_fill::<true>(me_vec, op_vec);
        let fill_shl = vectorized_fill::<false>(me_vec, op_vec);

        // Compute the overlap between the fills and opponent disks, then shift
        // by one to find the possible moves.
        let shift_shr = vectorized_shift::<true>(fill_shr & op_vec).reduce_or();
        let shift_shl = vectorized_shift::<false>(fill_shl & op_vec).reduce_or();

        // Compute the available moves by combining the right and left shift
        // results and restricting them to empty squares.
        (shift_shr | shift_shl) & self.empties()
    }

    /// Returns a bitboard for the position after a pass is played
    /// 
    /// *This function does not verify that a pass is a valid move.*
    pub fn pass(self) -> Self {
        Self {
            me: self.op,
            op: self.me,
        }
    }

    /// Returns a bitboard for the position after the provided move is played
    /// 
    /// *This function does not verify that the provided move is valid.*
    pub fn make_move(self, move_mask: u64) -> Self {
        const ZERO: u64x4 = u64x4::from_array([0; 4]);

        // Copy the board data into vectors
        let me_vec = u64x4::splat(self.me);
        let op_vec = u64x4::splat(self.op);

        // Copy the input move into a vector
        let move_vec = u64x4::splat(move_mask);

        // Compute the fill from the board data
        let fill_shr = vectorized_fill::<true>(move_vec, op_vec);
        let fill_shl = vectorized_fill::<false>(move_vec, op_vec);

        // Shift the fills to find directions where there is no friendly disk
        let shift_shr = vectorized_shift::<true>(fill_shr) & me_vec;
        let mask_shr = shift_shr.simd_ne(ZERO);
        let shift_shl = vectorized_shift::<false>(fill_shl) & me_vec;
        let mask_shl = shift_shl.simd_ne(ZERO);

        // Zero out directions where we don't encounter with a friendly disk,
        // then combine the swap masks with a reduction.
        let swaps_shr = mask_shr.select(fill_shr, ZERO).reduce_or();
        let swaps_shl = mask_shl.select(fill_shl, ZERO).reduce_or();
        let swaps = swaps_shr | swaps_shl;

        Self {
            me: self.op & !swaps,
            op: self.me | swaps,
        }
    }

    pub fn score(self) -> i32 {
        self.me.count_ones().try_into().unwrap_or(0) - self.op.count_ones().try_into().unwrap_or(0)
    }

    pub fn get_me(self) -> u64 {
        self.me
    }

    pub fn get_op(self) -> u64 {
        self.op
    }
}

impl Default for Bitboard {
    /// Initializes the bitboard to the Othello starting position.
    fn default() -> Self {
        Self {
            me: 0x0000000810000000,
            op: 0x0000001008000000,
        }
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut me = self.me;
        let mut op = self.op;
        for idx in 0..64 {
            if idx % 8 == 0 {
                write!(f, "{}", (idx / 8) + 1)?
            }
            match (me & 1, op & 1) {
                (1, 1) => write!(f, "#")?,
                (1, 0) => write!(f, "X")?,
                (0, 1) => write!(f, "O")?,
                _ => write!(f, ".")?,
            }
            if idx % 8 == 7 {
                writeln!(f)?
            }
            me >>= 1;
            op >>= 1;
        }
        writeln!(f, " abcdefgh")
    }
}
