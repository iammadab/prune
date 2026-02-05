use crate::engine::board::Board;
use crate::engine::eval::Evaluator;
use crate::engine::movegen::generate_legal;
use crate::engine::search::traits::{SearchAlgorithm, SearchResult};

pub struct MinimaxSearch;

impl SearchAlgorithm for MinimaxSearch {
    fn search(
        &mut self,
        board: &mut Board,
        evaluator: &impl Evaluator,
        depth: u32,
    ) -> SearchResult {
        let mut nodes = 0;
        let mut best_move = None;
        let mut best_score = i32::MIN;

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
            let score = -negamax(board, evaluator, depth.saturating_sub(1), &mut nodes);
            board.unmake_move(mv, undo);
            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
        }

        SearchResult {
            best_move,
            score: best_score,
            nodes,
        }
    }
}

// Negamax explainer:
// Our evaluator always scores the position from the side-to-move’s perspective.
// When we make a move, the side to move flips, so a good score for them is a bad
// score for us. That’s why we negate the child score: it “re-centers” the value
// to the current player. This collapses max/min into a single loop.
fn negamax(board: &mut Board, evaluator: &impl Evaluator, depth: u32, nodes: &mut u64) -> i32 {
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
        let score = -negamax(board, evaluator, depth - 1, nodes);
        board.unmake_move(mv, undo);
        if score > best {
            best = score;
        }
    }

    best
}
