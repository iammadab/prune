pub mod apply_move;
pub mod board;
pub mod castling;
pub mod eval;
pub mod fen;
pub mod movegen;
pub mod search;
pub mod types;

use board::Board;
use eval::Evaluator;
use movegen::game_status;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use search::{SearchAlgorithm, SearchResult};
use types::GameStatus;

pub struct Engine<E: Evaluator, S: SearchAlgorithm> {
    evaluator: E,
    search: S,
    board: Board,
    rng: Option<SmallRng>,
}

impl<E: Evaluator, S: SearchAlgorithm> Engine<E, S> {
    pub fn with_components(evaluator: E, search: S) -> Self {
        Self {
            evaluator,
            search,
            board: Board::new(),
            rng: None,
        }
    }

    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng = Some(SmallRng::seed_from_u64(seed));
    }

    pub fn set_position_startpos(&mut self) {
        self.board.set_startpos();
    }

    pub fn set_position_fen(&mut self, fen: &str) -> Result<(), String> {
        self.board.set_fen(fen)
    }

    pub fn apply_move_list(&mut self, _moves: &[String]) {
        if let Err(err) = self.board.apply_uci_move_list(_moves) {
            eprintln!("invalid move list: {err}");
        }
    }

    pub fn search_depth(&mut self, _depth: u32) -> String {
        let (best_move, _) = self.search_depth_with_stats(_depth);
        best_move
    }

    pub fn search_depth_with_stats(&mut self, _depth: u32) -> (String, u64) {
        let (last_result, total_nodes) = self.search_iterative_depth(_depth);
        let SearchResult { best_moves, .. } = last_result;
        let mv = if best_moves.is_empty() {
            None
        } else if let Some(rng) = &mut self.rng {
            let index = rng.gen_range(0..best_moves.len());
            Some(best_moves[index])
        } else {
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..best_moves.len());
            Some(best_moves[index])
        };
        (
            mv.and_then(crate::engine::types::uci_from_move)
                .unwrap_or_else(|| "0000".to_string()),
            total_nodes,
        )
    }

    fn search_iterative_depth(&mut self, depth: u32) -> (SearchResult, u64) {
        let mut total_nodes = 0u64;
        let mut last_result = None;
        let mut preferred_root: Option<Vec<crate::engine::types::Move>> = None;

        if depth == 0 {
            let result = self.search.search_with_root_ordering(
                &mut self.board,
                &self.evaluator,
                0,
                preferred_root.as_deref(),
            );
            total_nodes = total_nodes.saturating_add(result.nodes);
            last_result = Some(result);
        } else {
            for current_depth in 1..=depth {
                let result = self.search.search_with_root_ordering(
                    &mut self.board,
                    &self.evaluator,
                    current_depth,
                    preferred_root.as_deref(),
                );
                total_nodes = total_nodes.saturating_add(result.nodes);
                preferred_root = Some(result.best_moves.clone());
                last_result = Some(result);
            }
        }

        (
            last_result.unwrap_or(SearchResult {
                best_moves: Vec::new(),
                score: 0,
                nodes: 0,
            }),
            total_nodes,
        )
    }

    pub fn game_status(&mut self) -> GameStatus {
        game_status(&mut self.board)
    }

    pub fn stop_search(&mut self) {
        let _ = self;
    }

    pub fn reset_state(&mut self) {
        self.board.clear();
    }
}
