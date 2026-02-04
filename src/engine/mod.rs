pub mod types;
pub mod board;

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
        let _ = _fen;
    }

    pub fn apply_move_list(&mut self, _moves: &[String]) {
        let _ = _moves;
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
