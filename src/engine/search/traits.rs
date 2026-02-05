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
