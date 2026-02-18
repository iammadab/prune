use crate::engine::board::Board;
use crate::engine::eval::Evaluator;
use crate::engine::movegen::{generate_legal, is_noisy_move};
use crate::engine::types::Move;

pub(crate) fn quiesce_ab(
    board: &mut Board,
    evaluator: &impl Evaluator,
    alpha: i32,
    beta: i32,
    nodes: &mut u64,
    q_depth: u32,
) -> i32 {
    quiesce_core(board, evaluator, alpha, beta, nodes, q_depth)
}

pub(crate) fn quiesce_mm(
    board: &mut Board,
    evaluator: &impl Evaluator,
    nodes: &mut u64,
    q_depth: u32,
) -> i32 {
    // Use wide bounds that still allow safe negation.
    quiesce_core(board, evaluator, i32::MIN / 2, i32::MAX / 2, nodes, q_depth)
}

pub(crate) fn quiesce_core(
    board: &mut Board,
    evaluator: &impl Evaluator,
    mut alpha: i32,
    beta: i32,
    nodes: &mut u64,
    q_depth: u32,
) -> i32 {
    *nodes += 1;

    let stand_pat = evaluator.evaluate(board);
    if stand_pat >= beta {
        return stand_pat;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    if q_depth == 0 {
        return stand_pat;
    }

    let noisy_moves = noisy_moves(board);
    if noisy_moves.is_empty() {
        return stand_pat;
    }

    for mv in noisy_moves {
        let undo = match board.make_move(mv) {
            Ok(undo) => undo,
            Err(_) => continue,
        };
        let score = -quiesce_core(board, evaluator, -beta, -alpha, nodes, q_depth - 1);
        board.unmake_move(mv, undo);

        if score >= beta {
            return score;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

fn noisy_moves(board: &mut Board) -> Vec<Move> {
    let moves = generate_legal(board);
    let mut noisy = Vec::new();
    for mv in moves {
        if is_noisy_move(board, mv) {
            noisy.push(mv);
        }
    }
    noisy
}
