use crate::engine::board::Board;
use crate::engine::eval::Evaluator;
use crate::engine::types::Move;

#[derive(Clone)]
pub struct SearchResult {
    pub best_moves: Vec<Move>,
    pub score: i32,
    pub nodes: u64,
}

pub trait SearchAlgorithm {
    fn search(&mut self, board: &mut Board, evaluator: &impl Evaluator, depth: u32)
        -> SearchResult;

    fn search_with_root_ordering(
        &mut self,
        board: &mut Board,
        evaluator: &impl Evaluator,
        depth: u32,
        preferred_root: Option<&[Move]>,
    ) -> SearchResult {
        let _ = preferred_root;
        self.search(board, evaluator, depth)
    }
}
