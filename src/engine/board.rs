use crate::engine::fen::{parse_fen, STARTPOS_FEN};
use crate::engine::types::{move_from_uci, Color, Move, Piece, PieceKind, Square};

pub struct Board {
    pub squares: [Option<Piece>; 128],
    pub side_to_move: Color,
    pub castling_rights: u8,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::fen::STARTPOS_FEN;
    use crate::engine::types::{move_from_uci, square_from_algebraic, Color, PieceKind};

    #[test]
    fn apply_move_updates_side_and_piece() {
        let mut board = Board::new();
        board.set_fen(STARTPOS_FEN).expect("startpos");

        let mv = move_from_uci("e2e4").expect("move");
        board.apply_move(mv).expect("apply move");

        let e2 = square_from_algebraic("e2").unwrap().index() as usize;
        let e4 = square_from_algebraic("e4").unwrap().index() as usize;
        assert!(board.squares[e2].is_none());
        let piece = board.squares[e4].expect("piece on e4");
        assert_eq!(piece.kind, PieceKind::Pawn);
        assert_eq!(piece.color, Color::White);
        assert_eq!(board.side_to_move, Color::Black);
    }
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
        self.set_fen(STARTPOS_FEN)
            .expect("startpos FEN should be valid");
    }

    pub fn set_fen(&mut self, fen: &str) -> Result<(), String> {
        let data = parse_fen(fen)?;
        self.squares = data.squares;
        self.side_to_move = data.side_to_move;
        self.castling_rights = data.castling_rights;
        self.en_passant = data.en_passant;
        self.halfmove_clock = data.halfmove_clock;
        self.fullmove_number = data.fullmove_number;
        Ok(())
    }

    pub fn apply_uci_move_list(&mut self, moves: &[String]) -> Result<(), String> {
        for mv in moves {
            let parsed = move_from_uci(mv).ok_or_else(|| format!("invalid UCI move: {mv}"))?;
            self.apply_move(parsed)?;
        }

        Ok(())
    }

    pub fn apply_move(&mut self, mv: Move) -> Result<(), String> {
        let from_index = mv.from.index() as usize;
        let to_index = mv.to.index() as usize;
        let piece =
            self.squares[from_index].ok_or_else(|| "no piece on from square".to_string())?;
        let was_capture = self.squares[to_index].is_some();

        let moved_piece = match mv.promotion {
            Some(kind) => Piece {
                color: piece.color,
                kind,
            },
            None => piece,
        };

        self.squares[from_index] = None;
        self.squares[to_index] = Some(moved_piece);

        let is_pawn = piece.kind == PieceKind::Pawn;
        if is_pawn || was_capture {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock = self.halfmove_clock.saturating_add(1);
        }

        if self.side_to_move == Color::Black {
            self.fullmove_number = self.fullmove_number.saturating_add(1);
        }
        self.side_to_move = match self.side_to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        Ok(())
    }
}
