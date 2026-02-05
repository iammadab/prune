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
            let collapsed_score =
                collapsed_score_for_move(board, evaluator, depth.saturating_sub(1), mv, &mut nodes);
            if collapsed_score > best_score {
                best_score = collapsed_score;
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

fn collapsed_score_for_move(
    board: &mut Board,
    evaluator: &impl Evaluator,
    depth: u32,
    mv: crate::engine::types::Move,
    nodes: &mut u64,
) -> i32 {
    let undo = match board.make_move(mv) {
        Ok(undo) => undo,
        Err(_) => return i32::MIN,
    };
    let collapsed_score = -collapse_opponent_replies(board, evaluator, depth, nodes);
    board.unmake_move(mv, undo);
    collapsed_score
}

// Opponent replies explainer:
// Our evaluator always scores the position from the side-to-move's perspective.
// When we make a move, the side to move flips, so a good score for them is a bad
// score for us. We negate the reply score to re-center it for the current player.
// This collapses all opponent replies into a single worst-case score for the move.
fn collapse_opponent_replies(
    board: &mut Board,
    evaluator: &impl Evaluator,
    depth: u32,
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
        let score = -collapse_opponent_replies(board, evaluator, depth - 1, nodes);
        board.unmake_move(mv, undo);
        if score > best {
            best = score;
        }
    }

    best
}
