use crate::engine::board::Board;
use crate::engine::eval::Evaluator;
use crate::engine::movegen::{generate_legal, is_king_in_check};
use crate::engine::search::traits::{SearchAlgorithm, SearchResult};

const MATE_SCORE: i32 = 30_000;

pub struct MinimaxSearch;

impl SearchAlgorithm for MinimaxSearch {
    fn search(
        &mut self,
        board: &mut Board,
        evaluator: &impl Evaluator,
        depth: u32,
    ) -> SearchResult {
        let mut nodes = 0;
        let mut best_moves = Vec::new();
        let mut best_score = i32::MIN;

        let moves = generate_legal(board);
        if moves.is_empty() {
            return SearchResult {
                best_moves: Vec::new(),
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
                best_moves.clear();
                best_moves.push(mv);
            } else if score == best_score {
                best_moves.push(mv);
            }
        }

        SearchResult {
            best_moves,
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
    *nodes += 1;
    let moves = generate_legal(board);
    if moves.is_empty() {
        if is_king_in_check(board, board.side_to_move) {
            return -MATE_SCORE - depth as i32;
        }
        return 0;
    }

    if depth == 0 {
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
