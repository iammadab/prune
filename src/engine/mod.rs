pub struct Engine {}

impl Engine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn set_position_startpos(&mut self) {
        let _ = self;
    }

    pub fn set_position_fen(&mut self, _fen: &str) {
        let _ = self;
    }

    pub fn apply_move_list(&mut self, _moves: &[String]) {
        let _ = self;
    }

    pub fn search_depth(&mut self, _depth: u32) -> String {
        let _ = self;
        "0000".to_string()
    }

    pub fn stop_search(&mut self) {
        let _ = self;
    }

    pub fn reset_state(&mut self) {
        let _ = self;
    }
}
