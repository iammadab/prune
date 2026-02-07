use crate::engine::board::Board;
use crate::engine::eval::MaterialEvaluator;
use crate::engine::search::{AlphaBetaSearch, MinimaxSearch, SearchAlgorithm};
use crate::engine::types::uci_from_move;
use crate::engine::Engine;

fn tactical_capture_fen() -> &'static str {
    "3rk3/8/8/8/8/8/8/3QK3 w - - 0 1"
}

#[test]
fn minimax_returns_best_move() {
    let mut board = Board::new();
    board.set_fen(tactical_capture_fen()).expect("fen");

    let mut search = MinimaxSearch;
    let result = search.search(&mut board, &MaterialEvaluator, 1);
    let best: Vec<String> = result
        .best_moves
        .iter()
        .filter_map(|mv| uci_from_move(*mv))
        .collect();

    assert!(best.iter().any(|mv| mv == "d1d8"));
}

#[test]
fn alphabeta_matches_minimax_depth1() {
    let mut board = Board::new();
    board.set_fen(tactical_capture_fen()).expect("fen");

    let mut minimax = MinimaxSearch;
    let mut alphabeta = AlphaBetaSearch;

    let mut mini_best: Vec<String> = minimax
        .search(&mut board, &MaterialEvaluator, 1)
        .best_moves
        .iter()
        .filter_map(|mv| uci_from_move(*mv))
        .collect();

    let mut alpha_best: Vec<String> = alphabeta
        .search(&mut board, &MaterialEvaluator, 1)
        .best_moves
        .iter()
        .filter_map(|mv| uci_from_move(*mv))
        .collect();

    mini_best.sort();
    alpha_best.sort();

    assert_eq!(mini_best, alpha_best);
}

#[test]
fn seeded_search_depth_is_deterministic() {
    let mut engine_a = Engine::with_components(MaterialEvaluator, MinimaxSearch);
    engine_a.set_rng_seed(7);
    engine_a.set_position_startpos();

    let mut engine_b = Engine::with_components(MaterialEvaluator, MinimaxSearch);
    engine_b.set_rng_seed(7);
    engine_b.set_position_startpos();

    let move_a = engine_a.search_depth(1);
    let move_b = engine_b.search_depth(1);

    assert_eq!(move_a, move_b);
}
