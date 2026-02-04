use crate::engine::castling::{revoke_all, revoke_kingside, revoke_queenside};
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
        if piece.color != self.side_to_move {
            return Err("piece does not match side to move".to_string());
        }
        let mut was_capture = self.squares[to_index].is_some();
        let prev_en_passant = self.en_passant;
        let is_en_passant_capture = piece.kind == PieceKind::Pawn
            && prev_en_passant.is_some()
            && prev_en_passant == Some(mv.to)
            && !was_capture;
        let from_file = mv.from.index() & 0x0f;
        let to_file = mv.to.index() & 0x0f;
        let from_rank = mv.from.index() >> 4;
        let to_rank = mv.to.index() >> 4;
        let is_castle = piece.kind == PieceKind::King
            && from_rank == to_rank
            && (from_file as i8 - to_file as i8).abs() == 2;

        let moved_piece = match mv.promotion {
            Some(kind) => Piece {
                color: piece.color,
                kind,
            },
            None => piece,
        };

        self.squares[from_index] = None;
        if is_en_passant_capture {
            let capture_index = match piece.color {
                Color::White => to_index - 16,
                Color::Black => to_index + 16,
            };
            self.squares[capture_index] = None;
            was_capture = true;
        }
        self.squares[to_index] = Some(moved_piece);

        if is_castle {
            let (rook_from_file, rook_to_file) = match to_file {
                6 => (7, 5),
                2 => (0, 3),
                _ => return Err("invalid castling target".to_string()),
            };
            let rook_rank = from_rank;
            let rook_from_index = (rook_rank * 16 + rook_from_file) as usize;
            let rook_to_index = (rook_rank * 16 + rook_to_file) as usize;
            let rook =
                self.squares[rook_from_index].ok_or_else(|| "no rook for castling".to_string())?;
            if rook.kind != PieceKind::Rook || rook.color != piece.color {
                return Err("invalid rook for castling".to_string());
            }
            self.squares[rook_from_index] = None;
            self.squares[rook_to_index] = Some(rook);
        }

        let mut new_en_passant = None;
        if piece.kind == PieceKind::Pawn {
            let from_rank = mv.from.index() >> 4;
            let to_rank = mv.to.index() >> 4;
            if piece.color == Color::White && from_rank == 1 && to_rank == 3 {
                new_en_passant = Some(Square(mv.from.index() + 16));
            } else if piece.color == Color::Black && from_rank == 6 && to_rank == 4 {
                new_en_passant = Some(Square(mv.from.index() - 16));
            }
        }
        self.en_passant = new_en_passant;

        let is_pawn = piece.kind == PieceKind::Pawn;
        if is_pawn || was_capture {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock = self.halfmove_clock.saturating_add(1);
        }

        update_castling_rights(
            &mut self.castling_rights,
            piece,
            from_file,
            from_rank,
            to_file,
            to_rank,
            was_capture,
        );

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

fn update_castling_rights(
    rights: &mut u8,
    piece: Piece,
    from_file: u8,
    from_rank: u8,
    to_file: u8,
    to_rank: u8,
    was_capture: bool,
) {
    if piece.kind == PieceKind::King {
        revoke_all(rights, piece.color);
    }

    if piece.kind == PieceKind::Rook {
        match (piece.color, from_file, from_rank) {
            (Color::White, 0, 0) => revoke_queenside(rights, Color::White),
            (Color::White, 7, 0) => revoke_kingside(rights, Color::White),
            (Color::Black, 0, 7) => revoke_queenside(rights, Color::Black),
            (Color::Black, 7, 7) => revoke_kingside(rights, Color::Black),
            _ => {}
        }
    }

    if was_capture {
        match (to_file, to_rank) {
            (0, 0) => revoke_queenside(rights, Color::White),
            (7, 0) => revoke_kingside(rights, Color::White),
            (0, 7) => revoke_queenside(rights, Color::Black),
            (7, 7) => revoke_kingside(rights, Color::Black),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::castling::{has_kingside, has_queenside};
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

    #[test]
    fn apply_move_rejects_wrong_side() {
        let mut board = Board::new();
        board.set_fen(STARTPOS_FEN).expect("startpos");

        let mv = move_from_uci("e7e5").expect("move");
        let err = board.apply_move(mv).unwrap_err();
        assert!(err.contains("side to move"));
    }

    #[test]
    fn apply_move_sets_en_passant_on_double_push() {
        let mut board = Board::new();
        board.set_fen(STARTPOS_FEN).expect("startpos");

        let mv = move_from_uci("e2e4").expect("move");
        board.apply_move(mv).expect("apply move");

        let ep = board.en_passant.expect("en passant square");
        assert_eq!(square_from_algebraic("e3").unwrap(), ep);
    }

    #[test]
    fn apply_move_clears_en_passant_on_single_push() {
        let mut board = Board::new();
        board.set_fen(STARTPOS_FEN).expect("startpos");

        let mv = move_from_uci("e2e3").expect("move");
        board.apply_move(mv).expect("apply move");

        assert!(board.en_passant.is_none());
    }

    #[test]
    fn apply_move_sets_en_passant_for_black_double_push() {
        let mut board = Board::new();
        board.set_fen(STARTPOS_FEN).expect("startpos");

        board
            .apply_move(move_from_uci("e2e4").unwrap())
            .expect("apply move");
        board
            .apply_move(move_from_uci("a7a5").unwrap())
            .expect("apply move");

        let ep = board.en_passant.expect("en passant square");
        assert_eq!(square_from_algebraic("a6").unwrap(), ep);
    }

    #[test]
    fn apply_move_handles_white_en_passant_capture() {
        let mut board = Board::new();
        board.set_fen(STARTPOS_FEN).expect("startpos");

        board
            .apply_move(move_from_uci("e2e4").unwrap())
            .expect("apply move");
        board
            .apply_move(move_from_uci("a7a6").unwrap())
            .expect("apply move");
        board
            .apply_move(move_from_uci("e4e5").unwrap())
            .expect("apply move");
        board
            .apply_move(move_from_uci("d7d5").unwrap())
            .expect("apply move");
        board
            .apply_move(move_from_uci("e5d6").unwrap())
            .expect("apply move");

        let d5 = square_from_algebraic("d5").unwrap().index() as usize;
        let d6 = square_from_algebraic("d6").unwrap().index() as usize;
        assert!(board.squares[d5].is_none());
        let pawn = board.squares[d6].expect("pawn on d6");
        assert_eq!(pawn.kind, PieceKind::Pawn);
        assert_eq!(pawn.color, Color::White);
    }

    #[test]
    fn apply_move_handles_black_en_passant_capture() {
        let mut board = Board::new();
        board.set_fen(STARTPOS_FEN).expect("startpos");

        board
            .apply_move(move_from_uci("a2a3").unwrap())
            .expect("apply move");
        board
            .apply_move(move_from_uci("d7d5").unwrap())
            .expect("apply move");
        board
            .apply_move(move_from_uci("a3a4").unwrap())
            .expect("apply move");
        board
            .apply_move(move_from_uci("d5d4").unwrap())
            .expect("apply move");
        board
            .apply_move(move_from_uci("e2e4").unwrap())
            .expect("apply move");
        board
            .apply_move(move_from_uci("d4e3").unwrap())
            .expect("apply move");

        let e4 = square_from_algebraic("e4").unwrap().index() as usize;
        let e3 = square_from_algebraic("e3").unwrap().index() as usize;
        assert!(board.squares[e4].is_none());
        let pawn = board.squares[e3].expect("pawn on e3");
        assert_eq!(pawn.kind, PieceKind::Pawn);
        assert_eq!(pawn.color, Color::Black);
    }

    #[test]
    fn apply_move_handles_white_castle_kingside() {
        let mut board = Board::new();
        board
            .set_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
            .expect("fen");

        board
            .apply_move(move_from_uci("e1g1").unwrap())
            .expect("castle");

        let g1 = square_from_algebraic("g1").unwrap().index() as usize;
        let f1 = square_from_algebraic("f1").unwrap().index() as usize;
        assert_eq!(board.squares[g1].unwrap().kind, PieceKind::King);
        assert_eq!(board.squares[f1].unwrap().kind, PieceKind::Rook);
        assert_eq!(board.squares[f1].unwrap().color, Color::White);
    }

    #[test]
    fn apply_move_handles_black_castle_queenside() {
        let mut board = Board::new();
        board
            .set_fen("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1")
            .expect("fen");

        board
            .apply_move(move_from_uci("e8c8").unwrap())
            .expect("castle");

        let c8 = square_from_algebraic("c8").unwrap().index() as usize;
        let d8 = square_from_algebraic("d8").unwrap().index() as usize;
        assert_eq!(board.squares[c8].unwrap().kind, PieceKind::King);
        assert_eq!(board.squares[d8].unwrap().kind, PieceKind::Rook);
        assert_eq!(board.squares[d8].unwrap().color, Color::Black);
    }

    #[test]
    fn apply_move_revokes_castling_on_king_move() {
        let mut board = Board::new();
        board
            .set_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
            .expect("fen");

        board
            .apply_move(move_from_uci("e1f1").unwrap())
            .expect("move");

        assert!(!has_kingside(board.castling_rights, Color::White));
        assert!(!has_queenside(board.castling_rights, Color::White));
        assert!(has_kingside(board.castling_rights, Color::Black));
        assert!(has_queenside(board.castling_rights, Color::Black));
    }

    #[test]
    fn apply_move_revokes_castling_on_rook_move() {
        let mut board = Board::new();
        board
            .set_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
            .expect("fen");

        board
            .apply_move(move_from_uci("h1h2").unwrap())
            .expect("move");

        assert!(!has_kingside(board.castling_rights, Color::White));
        assert!(has_queenside(board.castling_rights, Color::White));
    }

    #[test]
    fn apply_move_revokes_castling_on_rook_capture() {
        let mut board = Board::new();
        board
            .set_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
            .expect("fen");

        board
            .apply_move(move_from_uci("a1a8").unwrap())
            .expect("capture");

        assert!(!has_queenside(board.castling_rights, Color::Black));
        assert!(has_kingside(board.castling_rights, Color::Black));
    }
}
