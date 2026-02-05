use crate::engine::board::Board;

pub trait Evaluator {
    fn evaluate(&self, board: &Board) -> i32;
}
