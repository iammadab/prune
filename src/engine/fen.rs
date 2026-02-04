use crate::engine::types::{
    algebraic_from_square, is_valid_square, square_from_algebraic, Color, Piece, PieceKind, Square,
};

const CASTLE_WHITE_KING: u8 = 1 << 0;
const CASTLE_WHITE_QUEEN: u8 = 1 << 1;
const CASTLE_BLACK_KING: u8 = 1 << 2;
const CASTLE_BLACK_QUEEN: u8 = 1 << 3;

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
}
