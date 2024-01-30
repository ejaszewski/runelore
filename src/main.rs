// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::time::Instant;

use runelore::{game::{Bitboard, GameState, MoveType, Board}, search::negamax};

fn isolate_lsb(x: u64) -> u64 {
    let (y, _) = x.overflowing_neg();
    x & y
}

fn perft(depth: u8) -> u64 {
    fn perft_impl(board: Bitboard, state: GameState, depth: u8) -> u64 {
        if depth <= 0 {
            return 1;
        }

        let mut moves = board.get_moves();

        
        // Logic for passing
        if moves == 0 {
            if let MoveType::Pass = state.get_last() {
                return 1
            }
            return perft_impl(board.pass(), state.pass(), depth - 1);
        }

        let mut nodes = 0;
        while moves > 0 {
            let move_mask = isolate_lsb(moves);
            nodes += perft_impl(board.make_move(move_mask), state.play(), depth - 1);
            moves &= !move_mask;
        }

        nodes
    }

    perft_impl(Bitboard::default(), GameState::default(), depth)
}

fn main() {
    // println!("{}", perft(11));
    let mut b = Board::default();
    let now = Instant::now();
    let (m, s) = negamax(&b, 15).unwrap();
    let elapsed = now.elapsed();
    println!("Took {} ms", elapsed.as_millis());
    println!("{:?} scored {}", m, s);
}
