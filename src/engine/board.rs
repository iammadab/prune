use crate::engine::apply_move;
use crate::engine::fen::{parse_fen, validate_fen_semantics, STARTPOS_FEN};
use crate::engine::types::{move_from_uci, Color, Move, Piece, Square};
use crate::engine::zobrist;

pub struct Board {
    pub squares: [Option<Piece>; 128],
    pub side_to_move: Color,
    pub castling_rights: u8,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub hash: u64,
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self {
            squares: [None; 128],
            side_to_move: Color::White,
            castling_rights: 0,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            hash: 0,
        };
        board.hash = zobrist::compute_hash(&board);
        board
    }

    pub fn clear(&mut self) {
        self.squares = [None; 128];
        self.side_to_move = Color::White;
        self.castling_rights = 0;
        self.en_passant = None;
        self.halfmove_clock = 0;
        self.fullmove_number = 1;
        self.hash = zobrist::compute_hash(self);
    }

    pub fn set_startpos(&mut self) {
        self.set_fen(STARTPOS_FEN)
            .expect("startpos FEN should be valid");
    }

    pub fn set_fen(&mut self, fen: &str) -> Result<(), String> {
        let data = parse_fen(fen)?;
        validate_fen_semantics(&data)?;
        self.squares = data.squares;
        self.side_to_move = data.side_to_move;
        self.castling_rights = data.castling_rights;
        self.en_passant = data.en_passant;
        self.halfmove_clock = data.halfmove_clock;
        self.fullmove_number = data.fullmove_number;
        self.hash = zobrist::compute_hash(self);
        Ok(())
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn compute_hash(&self) -> u64 {
        zobrist::compute_hash(self)
    }

    pub fn apply_uci_move_list(&mut self, moves: &[String]) -> Result<(), String> {
        for mv in moves {
            let parsed = move_from_uci(mv).ok_or_else(|| format!("invalid UCI move: {mv}"))?;
            self.apply_move(parsed)?;
        }

        Ok(())
    }

    pub fn apply_move(&mut self, mv: Move) -> Result<(), String> {
        apply_move::apply_move(self, mv)
    }

    pub fn make_move(&mut self, mv: Move) -> Result<apply_move::MoveUndo, String> {
        apply_move::make_move(self, mv)
    }

    pub fn unmake_move(&mut self, mv: Move, undo: apply_move::MoveUndo) {
        apply_move::unmake_move(self, mv, undo)
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
    fn hash_matches_after_en_passant_sequence() {
        let mut board = Board::new();
        board.set_fen(STARTPOS_FEN).expect("startpos");

        let moves = ["e2e4", "a7a6", "e4e5", "d7d5", "e5d6"];
        for mv in moves {
            let parsed = move_from_uci(mv).expect("move");
            board.apply_move(parsed).expect("apply move");
            assert_eq!(board.hash(), board.compute_hash());
        }
    }

    #[test]
    fn hash_matches_after_castling_sequence() {
        let mut board = Board::new();
        board.set_fen(STARTPOS_FEN).expect("startpos");

        let moves = ["e2e4", "e7e5", "g1f3", "b8c6", "f1e2", "g8f6", "e1g1"];
        for mv in moves {
            let parsed = move_from_uci(mv).expect("move");
            board.apply_move(parsed).expect("apply move");
            assert_eq!(board.hash(), board.compute_hash());
        }
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
