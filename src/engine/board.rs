use crate::engine::types::{Color, Piece, Square};

pub struct Board {
    pub squares: [Option<Piece>; 128],
    pub side_to_move: Color,
    pub castling_rights: u8,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Board {
    pub fn new() -> Self {
        Self {
            squares: [None; 128],
            side_to_move: Color::White,
            castling_rights: 0,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn clear(&mut self) {
        self.squares = [None; 128];
        self.side_to_move = Color::White;
        self.castling_rights = 0;
        self.en_passant = None;
        self.halfmove_clock = 0;
        self.fullmove_number = 1;
    }

    pub fn set_startpos(&mut self) {
        self.clear();
    }
}
