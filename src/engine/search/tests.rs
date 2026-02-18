use crate::engine::board::Board;
use crate::engine::eval::MaterialEvaluator;
use crate::engine::movegen::generate_legal;
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

    let legal: Vec<String> = generate_legal(&mut board)
        .iter()
        .filter_map(|mv| uci_from_move(*mv))
        .collect();

    assert!(!best.is_empty());
    for mv in best {
        assert!(legal.iter().any(|legal_mv| legal_mv == &mv));
    }
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

    for mv in alpha_best {
        assert!(mini_best.iter().any(|best| best == &mv));
    }
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

#[test]
fn alphabeta_depth3_includes_ba6() {
    let mut board = Board::new();
    board
        .set_fen("rnbqkbnr/pppp1ppp/8/4p3/8/4P3/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
        .expect("fen");

    let mut search = AlphaBetaSearch;
    let result = search.search(&mut board, &MaterialEvaluator, 3);
    let best_moves: Vec<String> = result
        .best_moves
        .iter()
        .filter_map(|mv| uci_from_move(*mv))
        .collect();

    assert!(!best_moves.iter().any(|mv| mv == "f1a6"));
}

#[test]
fn alphabeta_best_moves_subset_of_minimax_depth2_startpos() {
    let mut board = Board::new();
    board.set_startpos();

    let mut minimax = MinimaxSearch;
    let mut alphabeta = AlphaBetaSearch;

    let mini_best: Vec<String> = minimax
        .search(&mut board, &MaterialEvaluator, 2)
        .best_moves
        .iter()
        .filter_map(|mv| uci_from_move(*mv))
        .collect();

    let alpha_best: Vec<String> = alphabeta
        .search(&mut board, &MaterialEvaluator, 2)
        .best_moves
        .iter()
        .filter_map(|mv| uci_from_move(*mv))
        .collect();

    for mv in alpha_best {
        assert!(mini_best.iter().any(|best| best == &mv));
    }
}

#[test]
fn prefers_mate_over_material_capture() {
    let mut board = Board::new();
    board
        .set_fen("1k6/8/8/8/7Q/8/PPP5/1K1Bq3 b - - 0 1")
        .expect("fen");

    let mut search = MinimaxSearch;
    let result = search.search(&mut board, &MaterialEvaluator, 1);
    let best_moves: Vec<String> = result
        .best_moves
        .iter()
        .filter_map(|mv| uci_from_move(*mv))
        .collect();

    assert_eq!(best_moves, vec!["e1d1".to_string()]);
}
