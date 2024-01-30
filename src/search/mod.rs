use crate::game::{extract_lsb, Bitboard, Board, GameState, MoveType, Move};

pub fn negamax(board: &Board, depth: u8) -> Option<(Move, i32)> {
    if depth == 0 {
        return None;
    }

    fn negamax_impl(
        bitboard: Bitboard,
        game_state: GameState,
        mut alpha: i32,
        beta: i32,
        depth: u8,
    ) -> i32 {
        if depth == 0 {
            return bitboard.score();
        }

        let mut valid_moves = bitboard.get_moves();

        if valid_moves == 0 {
            if let MoveType::Pass = game_state.get_last() {
                return bitboard.score();
            }
            return -negamax_impl(bitboard.pass(), game_state.pass(), -beta, -alpha, depth - 1);
        }

        while valid_moves > 0 {
            let move_mask = extract_lsb(valid_moves);
            valid_moves &= !move_mask;
            let score = -negamax_impl(
                bitboard.make_move(move_mask),
                game_state.play(),
                -beta,
                -alpha,
                depth - 1,
            );
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    let moves = board.get_moves();
    let bitboard = board.get_bitboard();
    let game_state = board.get_game_state();
    
    let mut best_score = i32::MIN;
    let mut best_move = *moves.get(0)?;

    for mv in moves {
        let score = match mv {
            Move::Play(idx) => {
                let move_mask = 1 << idx;
                -negamax_impl(bitboard.make_move(move_mask), game_state.play(), i32::MIN + 1, i32::MAX, depth - 1)
            },
            Move::Pass => {
                if let MoveType::Pass = game_state.get_last() {
                    return None;
                }
                -negamax_impl(bitboard.pass(), game_state.pass(), i32::MIN + 1, i32::MAX, depth - 1)
            }
        };
        if score > best_score {
            best_score = score;
            best_move = mv;
        }
    }

    Some((best_move, best_score))
}
