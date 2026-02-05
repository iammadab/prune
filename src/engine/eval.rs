use crate::engine::board::Board;
use crate::engine::types::{Color, PieceKind};

pub trait Evaluator {
    fn evaluate(&self, board: &Board) -> i32;
}

pub struct MaterialEvaluator;

impl Evaluator for MaterialEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        let mut score = 0;
        for square in board.squares.iter().flatten() {
            let value = match square.kind {
                PieceKind::Pawn => 100,
                PieceKind::Knight => 320,
                PieceKind::Bishop => 330,
                PieceKind::Rook => 500,
                PieceKind::Queen => 900,
                PieceKind::King => 0,
            };
            let sign = match (square.color, board.side_to_move) {
                (Color::White, Color::White) | (Color::Black, Color::Black) => 1,
                _ => -1,
            };
            score += sign * value;
        }
        score
    }
}
