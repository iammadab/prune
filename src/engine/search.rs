use crate::engine::board::Board;
use crate::engine::eval::Evaluator;
use crate::engine::types::Move;

pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub nodes: u64,
}

pub trait SearchAlgorithm {
    fn search(&mut self, board: &mut Board, evaluator: &impl Evaluator, depth: u32)
        -> SearchResult;
}

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

        let moves = crate::engine::movegen::generate_legal(board);
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
            let score = -minimax(board, evaluator, depth.saturating_sub(1), &mut nodes);
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

fn minimax(board: &mut Board, evaluator: &impl Evaluator, depth: u32, nodes: &mut u64) -> i32 {
    if depth == 0 {
        *nodes += 1;
        return evaluator.evaluate(board);
    }

    let moves = crate::engine::movegen::generate_legal(board);
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
        let score = -minimax(board, evaluator, depth - 1, nodes);
        board.unmake_move(mv, undo);
        if score > best {
            best = score;
        }
    }

    best
}
