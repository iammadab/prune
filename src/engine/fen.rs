use crate::engine::board::Board;
use crate::engine::castling::{
    has_kingside, has_queenside, CASTLE_BLACK_KING, CASTLE_BLACK_QUEEN, CASTLE_WHITE_KING,
    CASTLE_WHITE_QUEEN,
};
use crate::engine::movegen::is_square_attacked;
use crate::engine::types::{
    algebraic_from_square, is_valid_square, square_from_algebraic, Color, Piece, PieceKind, Square,
};

pub const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug)]
pub struct FenData {
    pub squares: [Option<Piece>; 128],
    pub side_to_move: Color,
    pub castling_rights: u8,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

pub fn parse_fen(fen: &str) -> Result<FenData, String> {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    if parts.len() != 6 {
        return Err("FEN must have 6 fields".to_string());
    }

    let squares = parse_piece_placement(parts[0])?;
    let side_to_move = match parts[1] {
        "w" => Color::White,
        "b" => Color::Black,
        _ => return Err("invalid side to move".to_string()),
    };
    let castling_rights = parse_castling_rights(parts[2])?;
    let en_passant = parse_en_passant(parts[3])?;
    let halfmove_clock = parts[4]
        .parse::<u32>()
        .map_err(|_| "invalid halfmove clock".to_string())?;
    let fullmove_number = parts[5]
        .parse::<u32>()
        .map_err(|_| "invalid fullmove number".to_string())?;

    Ok(FenData {
        squares,
        side_to_move,
        castling_rights,
        en_passant,
        halfmove_clock,
        fullmove_number,
    })
}

pub fn validate_fen_semantics(data: &FenData) -> Result<(), String> {
    let mut white_king = None;
    let mut black_king = None;

    for index in 0u8..128u8 {
        if !is_valid_square(index) {
            continue;
        }
        let piece = match data.squares[index as usize] {
            Some(piece) => piece,
            None => continue,
        };
        let rank = index >> 4;
        if piece.kind == PieceKind::Pawn && (rank == 0 || rank == 7) {
            return Err("invalid pawn on first or eighth rank".to_string());
        }
        if piece.kind == PieceKind::King {
            let square = Square(index);
            match piece.color {
                Color::White => {
                    if white_king.is_some() {
                        return Err("invalid king count".to_string());
                    }
                    white_king = Some(square);
                }
                Color::Black => {
                    if black_king.is_some() {
                        return Err("invalid king count".to_string());
                    }
                    black_king = Some(square);
                }
            }
        }
    }

    let white_king = white_king.ok_or_else(|| "missing white king".to_string())?;
    let black_king = black_king.ok_or_else(|| "missing black king".to_string())?;

    if has_kingside(data.castling_rights, Color::White) {
        if !is_piece_at(data, Square(4), Color::White, PieceKind::King)
            || !is_piece_at(data, Square(7), Color::White, PieceKind::Rook)
        {
            return Err("invalid white kingside castling rights".to_string());
        }
    }
    if has_queenside(data.castling_rights, Color::White) {
        if !is_piece_at(data, Square(4), Color::White, PieceKind::King)
            || !is_piece_at(data, Square(0), Color::White, PieceKind::Rook)
        {
            return Err("invalid white queenside castling rights".to_string());
        }
    }
    if has_kingside(data.castling_rights, Color::Black) {
        if !is_piece_at(data, Square(116), Color::Black, PieceKind::King)
            || !is_piece_at(data, Square(119), Color::Black, PieceKind::Rook)
        {
            return Err("invalid black kingside castling rights".to_string());
        }
    }
    if has_queenside(data.castling_rights, Color::Black) {
        if !is_piece_at(data, Square(116), Color::Black, PieceKind::King)
            || !is_piece_at(data, Square(112), Color::Black, PieceKind::Rook)
        {
            return Err("invalid black queenside castling rights".to_string());
        }
    }

    let board = Board {
        squares: data.squares,
        side_to_move: data.side_to_move,
        castling_rights: data.castling_rights,
        en_passant: data.en_passant,
        halfmove_clock: data.halfmove_clock,
        fullmove_number: data.fullmove_number,
    };
    let white_in_check = is_square_attacked(&board, white_king, Color::Black);
    let black_in_check = is_square_attacked(&board, black_king, Color::White);
    if white_in_check && black_in_check {
        return Err("both kings are in check".to_string());
    }

    if let Some(ep) = data.en_passant {
        validate_en_passant(data, ep)?;
    }

    Ok(())
}

fn is_piece_at(data: &FenData, square: Square, color: Color, kind: PieceKind) -> bool {
    matches!(
        data.squares[square.index() as usize],
        Some(Piece { color: c, kind: k }) if c == color && k == kind
    )
}

fn validate_en_passant(data: &FenData, ep: Square) -> Result<(), String> {
    let rank = ep.index() >> 4;
    let expected_rank = match data.side_to_move {
        Color::White => 5,
        Color::Black => 2,
    };
    if rank != expected_rank {
        return Err("invalid en passant rank".to_string());
    }
    if data.squares[ep.index() as usize].is_some() {
        return Err("en passant square is occupied".to_string());
    }

    let opponent = match data.side_to_move {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    let opponent_pawn_index = match opponent {
        Color::White => ep.index() as i16 + 16,
        Color::Black => ep.index() as i16 - 16,
    };
    if opponent_pawn_index < 0
        || opponent_pawn_index > 127
        || !is_valid_square(opponent_pawn_index as u8)
    {
        return Err("invalid en passant pawn position".to_string());
    }
    let opponent_pawn_square = Square(opponent_pawn_index as u8);
    if !is_piece_at(data, opponent_pawn_square, opponent, PieceKind::Pawn) {
        return Err("missing pawn for en passant".to_string());
    }

    let (left_offset, right_offset) = match data.side_to_move {
        Color::White => (-17, -15),
        Color::Black => (17, 15),
    };
    let mut can_capture = false;
    for offset in [left_offset, right_offset] {
        let candidate = ep.index() as i16 + offset;
        if candidate < 0 || candidate > 127 {
            continue;
        }
        if !is_valid_square(candidate as u8) {
            continue;
        }
        let square = Square(candidate as u8);
        if is_piece_at(data, square, data.side_to_move, PieceKind::Pawn) {
            can_capture = true;
            break;
        }
    }
    if !can_capture {
        return Err("no pawn can capture en passant".to_string());
    }

    Ok(())
}

fn parse_piece_placement(placement: &str) -> Result<[Option<Piece>; 128], String> {
    let mut squares = [None; 128];
    let mut rank_index = 7;
    let mut file_index = 0u8;

    for ch in placement.chars() {
        if ch == '/' {
            if file_index != 8 {
                return Err("invalid FEN rank length".to_string());
            }
            if rank_index == 0 {
                return Err("too many ranks in FEN".to_string());
            }
            rank_index -= 1;
            file_index = 0;
            continue;
        }

        if ch.is_ascii_digit() {
            let empty = ch.to_digit(10).ok_or("invalid digit in FEN")? as u8;
            if empty == 0 || file_index + empty > 8 {
                return Err("invalid empty count in FEN".to_string());
            }
            file_index += empty;
            continue;
        }

        let piece = piece_from_fen(ch).ok_or("invalid piece in FEN")?;
        if file_index > 7 {
            return Err("invalid FEN rank length".to_string());
        }

        let square = (rank_index * 16 + file_index) as u8;
        if !is_valid_square(square) {
            return Err("invalid square in FEN".to_string());
        }

        squares[square as usize] = Some(piece);
        file_index += 1;
    }

    if rank_index != 0 || file_index != 8 {
        return Err("invalid FEN rank count".to_string());
    }

    Ok(squares)
}

fn piece_from_fen(ch: char) -> Option<Piece> {
    let (color, kind) = match ch {
        'P' => (Color::White, PieceKind::Pawn),
        'N' => (Color::White, PieceKind::Knight),
        'B' => (Color::White, PieceKind::Bishop),
        'R' => (Color::White, PieceKind::Rook),
        'Q' => (Color::White, PieceKind::Queen),
        'K' => (Color::White, PieceKind::King),
        'p' => (Color::Black, PieceKind::Pawn),
        'n' => (Color::Black, PieceKind::Knight),
        'b' => (Color::Black, PieceKind::Bishop),
        'r' => (Color::Black, PieceKind::Rook),
        'q' => (Color::Black, PieceKind::Queen),
        'k' => (Color::Black, PieceKind::King),
        _ => return None,
    };

    Some(Piece { color, kind })
}

fn parse_castling_rights(text: &str) -> Result<u8, String> {
    if text == "-" {
        return Ok(0);
    }

    let mut rights = 0u8;
    for ch in text.chars() {
        match ch {
            'K' => rights |= CASTLE_WHITE_KING,
            'Q' => rights |= CASTLE_WHITE_QUEEN,
            'k' => rights |= CASTLE_BLACK_KING,
            'q' => rights |= CASTLE_BLACK_QUEEN,
            _ => return Err("invalid castling rights".to_string()),
        }
    }

    Ok(rights)
}

fn parse_en_passant(text: &str) -> Result<Option<Square>, String> {
    if text == "-" {
        return Ok(None);
    }

    let square = square_from_algebraic(text).ok_or("invalid en passant square")?;
    if algebraic_from_square(square).as_deref() != Some(text) {
        return Err("invalid en passant square".to_string());
    }

    Ok(Some(square))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board::Board;
    use crate::engine::types::{square_from_algebraic, PieceKind};

    #[test]
    fn parses_startpos() {
        let data = parse_fen(STARTPOS_FEN).expect("startpos parse");
        assert_eq!(data.side_to_move, Color::White);
        assert!(data.en_passant.is_none());

        let e1 = square_from_algebraic("e1").unwrap().index() as usize;
        let e8 = square_from_algebraic("e8").unwrap().index() as usize;
        let a2 = square_from_algebraic("a2").unwrap().index() as usize;

        let e1_piece = data.squares[e1].expect("e1 piece");
        let e8_piece = data.squares[e8].expect("e8 piece");
        let a2_piece = data.squares[a2].expect("a2 piece");

        assert_eq!(e1_piece.color, Color::White);
        assert_eq!(e1_piece.kind, PieceKind::King);
        assert_eq!(e8_piece.color, Color::Black);
        assert_eq!(e8_piece.kind, PieceKind::King);
        assert_eq!(a2_piece.color, Color::White);
        assert_eq!(a2_piece.kind, PieceKind::Pawn);
    }

    #[test]
    fn rejects_invalid_field_count() {
        let err = parse_fen("8/8/8/8/8/8/8/8 w - - 0").unwrap_err();
        assert!(err.contains("6 fields"));
    }

    #[test]
    fn rejects_invalid_side() {
        let err = parse_fen("8/8/8/8/8/8/8/8 x - - 0 1").unwrap_err();
        assert!(err.contains("side"));
    }

    #[test]
    fn rejects_missing_king() {
        let mut board = Board::new();
        let err = board.set_fen("8/8/8/8/8/8/8/4K3 w - - 0 1").unwrap_err();
        assert!(err.contains("king"));
    }

    #[test]
    fn rejects_pawn_on_last_rank() {
        let mut board = Board::new();
        let err = board.set_fen("7k/8/8/8/8/8/8/P3K3 w - - 0 1").unwrap_err();
        assert!(err.contains("pawn"));
    }

    #[test]
    fn rejects_both_kings_in_check() {
        let mut board = Board::new();
        let err = board.set_fen("8/8/8/8/8/8/4Kk2/8 w - - 0 1").unwrap_err();
        assert!(err.contains("check"));
    }

    #[test]
    fn rejects_invalid_castling_rights() {
        let mut board = Board::new();
        let err = board.set_fen("4k3/8/8/8/8/8/8/4K3 w K - 0 1").unwrap_err();
        assert!(err.contains("castling"));
    }

    #[test]
    fn rejects_invalid_en_passant() {
        let mut board = Board::new();
        let err = board
            .set_fen("8/8/8/4p3/8/8/8/4K2k w - e6 0 1")
            .unwrap_err();
        assert!(err.contains("en passant"));
    }
}
