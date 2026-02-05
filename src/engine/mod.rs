pub mod apply_move;
pub mod board;
pub mod castling;
pub mod eval;
pub mod fen;
pub mod movegen;
pub mod search;
pub mod types;

use board::Board;
use eval::{Evaluator, MaterialEvaluator};
use movegen::game_status;
use search::{MinimaxSearch, SearchAlgorithm, SearchResult};
use types::GameStatus;

pub struct Engine<E: Evaluator = MaterialEvaluator, S: SearchAlgorithm = MinimaxSearch> {
    evaluator: E,
    search: S,
    board: Board,
}

impl Engine<MaterialEvaluator, MinimaxSearch> {
    pub fn new() -> Self {
        Self::with_components(MaterialEvaluator, MinimaxSearch)
    }
}

impl<E: Evaluator, S: SearchAlgorithm> Engine<E, S> {
    pub fn with_components(evaluator: E, search: S) -> Self {
        Self {
            evaluator,
            search,
            board: Board::new(),
        }
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
        let SearchResult { best_move, .. } =
            self.search.search(&mut self.board, &self.evaluator, _depth);
        match best_move.and_then(crate::engine::types::uci_from_move) {
            Some(uci) => uci,
            None => "0000".to_string(),
        }
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
