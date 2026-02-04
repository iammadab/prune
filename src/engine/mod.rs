pub mod apply_move;
pub mod board;
pub mod castling;
pub mod fen;
pub mod movegen;
pub mod types;

use board::Board;

pub struct Engine {
    board: Board,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
        }
    }

    pub fn set_position_startpos(&mut self) {
        self.board.set_startpos();
    }

    pub fn set_position_fen(&mut self, _fen: &str) {
        if let Err(err) = self.board.set_fen(_fen) {
            eprintln!("invalid FEN: {err}");
        }
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

    pub fn stop_search(&mut self) {
        let _ = self;
    }

    pub fn reset_state(&mut self) {
        self.board.clear();
    }
}
