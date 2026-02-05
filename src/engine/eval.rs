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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_eval_scores_side_to_move() {
        let mut board = Board::new();
        board
            .set_fen("4k3/8/8/8/8/8/P7/4K3 w - - 0 1")
            .expect("fen");
        let eval = MaterialEvaluator.evaluate(&board);
        assert_eq!(eval, 100);

        board
            .set_fen("4k3/8/8/8/8/8/P7/4K3 b - - 0 1")
            .expect("fen");
        let eval = MaterialEvaluator.evaluate(&board);
        assert_eq!(eval, -100);
    }

    #[test]
    fn material_eval_balances_both_sides() {
        let mut board = Board::new();
        board
            .set_fen("4k3/8/8/8/8/8/Pp6/4K3 w - - 0 1")
            .expect("fen");
        let eval = MaterialEvaluator.evaluate(&board);
        assert_eq!(eval, 0);
    }
}
