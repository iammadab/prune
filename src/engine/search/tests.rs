use crate::engine::board::Board;
use crate::engine::eval::MaterialEvaluator;
use crate::engine::search::{AlphaBetaSearch, MinimaxSearch, SearchAlgorithm};
use crate::engine::types::uci_from_move;

fn tactical_capture_fen() -> &'static str {
    "3rk3/8/8/8/8/8/8/3QK3 w - - 0 1"
}

#[test]
fn minimax_returns_best_move() {
    let mut board = Board::new();
    board.set_fen(tactical_capture_fen()).expect("fen");

    let mut search = MinimaxSearch;
    let result = search.search(&mut board, &MaterialEvaluator, 1);
    let best = result.best_move.and_then(uci_from_move);

    assert_eq!(best.as_deref(), Some("d1d8"));
}

#[test]
fn alphabeta_matches_minimax_depth1() {
    let mut board = Board::new();
    board.set_fen(tactical_capture_fen()).expect("fen");

    let mut minimax = MinimaxSearch;
    let mut alphabeta = AlphaBetaSearch;

    let mini_best = minimax.search(&mut board, &MaterialEvaluator, 1).best_move;
    let mini_best = mini_best.and_then(uci_from_move);

    let alpha_best = alphabeta
        .search(&mut board, &MaterialEvaluator, 1)
        .best_move;
    let alpha_best = alpha_best.and_then(uci_from_move);

    assert_eq!(mini_best, alpha_best);
}
