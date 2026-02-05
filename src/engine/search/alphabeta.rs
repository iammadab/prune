use crate::engine::board::Board;
use crate::engine::eval::Evaluator;
use crate::engine::movegen::generate_legal;
use crate::engine::search::traits::{SearchAlgorithm, SearchResult};

pub struct AlphaBetaSearch;

impl SearchAlgorithm for AlphaBetaSearch {
    fn search(
        &mut self,
        board: &mut Board,
        evaluator: &impl Evaluator,
        depth: u32,
    ) -> SearchResult {
        let mut nodes = 0;
        let mut best_move = None;
        let mut best_score = i32::MIN;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX;

        let moves = generate_legal(board);
        if moves.is_empty() {
            return SearchResult {
                best_move: None,
                score: evaluator.evaluate(board),
                nodes,
            };
        }

        for mv in moves {
            let undo = match board.make_move(mv) {
                Ok(undo) => undo,
                Err(_) => continue,
            };
            // Collapse opponent replies into a single score for this move.
            let score = -alphabeta(
                board,
                evaluator,
                depth.saturating_sub(1),
                -beta,
                -alpha,
                &mut nodes,
            );
            board.unmake_move(mv, undo);
            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
            if score > alpha {
                alpha = score;
            }
        }

        SearchResult {
            best_move,
            score: best_score,
            nodes,
        }
    }
}

fn alphabeta(
    board: &mut Board,
    evaluator: &impl Evaluator,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    nodes: &mut u64,
) -> i32 {
    if depth == 0 {
        *nodes += 1;
        return evaluator.evaluate(board);
    }

    let moves = generate_legal(board);
    if moves.is_empty() {
        *nodes += 1;
        return evaluator.evaluate(board);
    }

    let mut best = i32::MIN;
    for mv in moves {
        let undo = match board.make_move(mv) {
            Ok(undo) => undo,
            Err(_) => continue,
        };
        let score = -alphabeta(board, evaluator, depth - 1, -beta, -alpha, nodes);
        board.unmake_move(mv, undo);
        if score > best {
            best = score;
        }
        if score > alpha {
            alpha = score;
        }
        // If this move's collapsed score can't beat our current best, stop exploring replies.
        if alpha >= beta {
            break;
        }
    }

    best
}
