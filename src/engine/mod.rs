pub mod apply_move;
pub mod board;
pub mod castling;
pub mod eval;
pub mod fen;
pub mod movegen;
pub mod types;

use board::Board;
use eval::{Evaluator, MaterialEvaluator};
use movegen::game_status;
use types::GameStatus;

pub struct Engine<E: Evaluator = MaterialEvaluator> {
    evaluator: E,
    board: Board,
}

impl Engine<MaterialEvaluator> {
    pub fn new() -> Self {
        Self::with_evaluator(MaterialEvaluator)
    }
}

impl<E: Evaluator> Engine<E> {
    pub fn with_evaluator(evaluator: E) -> Self {
        Self {
            evaluator,
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
        let _ = _depth;
        "0000".to_string()
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
