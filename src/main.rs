// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::time::Instant;

use runelore::bitboard::Bitboard;

#[derive(Clone, Copy)]
struct BoardState {
    passed: bool,
}

fn isolate_lsb(x: u64) -> u64 {
    let (y, _) = x.overflowing_neg();
    x & y
}

fn print_bitboard(mut x: u64) {
    for i in 0..64 {
        if x & 1 == 1 {
            print!("#");
        } else {
            print!(".");
        }
        x >>= 1;
        if i % 8 == 7 {
            println!();
        }
    }
}

fn perft(depth: u8) -> u64 {
    fn perft_impl(board: Bitboard, state: BoardState, depth: u8) -> u64 {
        if depth <= 0 {
            return 1;
        }

        let mut moves = board.get_moves();

        
        // Logic for passing
        if moves == 0 {
            if state.passed {
                return 1
            }
            return perft_impl(board.pass(), BoardState { passed: true }, depth - 1);
        }

        let mut nodes = 0;
        while moves > 0 {
            let move_mask = isolate_lsb(moves);
            nodes += perft_impl(board.make_move(move_mask), BoardState { passed: false }, depth - 1);
            moves &= !move_mask;
        }

        nodes
    }

    perft_impl(Bitboard::init(), BoardState { passed: false }, depth)
}

fn main() {
    let now = Instant::now();
    println!("{}", perft(11));
    let elapsed = now.elapsed();
    println!("Took {} ms", elapsed.as_millis());
}
