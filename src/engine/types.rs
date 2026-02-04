#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square(pub u8);

impl Square {
    pub fn index(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
}

pub fn move_from_uci(text: &str) -> Option<Move> {
    let mut chars = text.chars();
    let from_file = chars.next()?;
    let from_rank = chars.next()?;
    let to_file = chars.next()?;
    let to_rank = chars.next()?;
    let promo = chars.next();

    if chars.next().is_some() {
        return None;
    }

    let from = square_from_algebraic(&format!("{from_file}{from_rank}"))?;
    let to = square_from_algebraic(&format!("{to_file}{to_rank}"))?;

    let promotion = match promo {
        None => None,
        Some('q') | Some('Q') => Some(PieceKind::Queen),
        Some('r') | Some('R') => Some(PieceKind::Rook),
        Some('b') | Some('B') => Some(PieceKind::Bishop),
        Some('n') | Some('N') => Some(PieceKind::Knight),
        _ => return None,
    };

    Some(Move {
        from,
        to,
        promotion,
    })
}

pub fn uci_from_move(mv: Move) -> Option<String> {
    let from = algebraic_from_square(mv.from)?;
    let to = algebraic_from_square(mv.to)?;
    let promo = match mv.promotion {
        None => String::new(),
        Some(PieceKind::Queen) => "q".to_string(),
        Some(PieceKind::Rook) => "r".to_string(),
        Some(PieceKind::Bishop) => "b".to_string(),
        Some(PieceKind::Knight) => "n".to_string(),
        Some(PieceKind::Pawn) | Some(PieceKind::King) => return None,
    };

    Some(format!("{from}{to}{promo}"))
}

pub fn is_valid_square(square: u8) -> bool {
    (square & 0x88) == 0
}

pub fn square_from_coords(file: u8, rank: u8) -> Option<Square> {
    if file > 7 || rank > 7 {
        return None;
    }

    let square = rank * 16 + file;
    if is_valid_square(square) {
        Some(Square(square))
    } else {
        None
    }
}

pub fn square_from_algebraic(text: &str) -> Option<Square> {
    let mut chars = text.chars();
    let file_char = chars.next()?;
    let rank_char = chars.next()?;

    if chars.next().is_some() {
        return None;
    }

    let file = match file_char {
        'a'..='h' => file_char as u8 - b'a',
        'A'..='H' => file_char as u8 - b'A',
        _ => return None,
    };

    let rank = match rank_char {
        '1'..='8' => rank_char as u8 - b'1',
        _ => return None,
    };

    square_from_coords(file, rank)
}

pub fn algebraic_from_square(square: Square) -> Option<String> {
    let index = square.index();
    if !is_valid_square(index) {
        return None;
    }

    let file = index & 0x0f;
    let rank = index >> 4;

    if file > 7 || rank > 7 {
        return None;
    }

    let file_char = (b'a' + file) as char;
    let rank_char = (b'1' + rank) as char;
    Some(format!("{file_char}{rank_char}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algebraic_round_trip() {
        let square = square_from_algebraic("e2").expect("square");
        assert_eq!(square.index(), 20);
        assert_eq!(algebraic_from_square(square).as_deref(), Some("e2"));
    }

    #[test]
    fn invalid_algebraic_returns_none() {
        assert!(square_from_algebraic("i9").is_none());
        assert!(square_from_algebraic("e0").is_none());
        assert!(square_from_algebraic("e22").is_none());
    }

    #[test]
    fn parse_uci_move() {
        let mv = move_from_uci("e2e4").expect("move");
        assert_eq!(algebraic_from_square(mv.from).as_deref(), Some("e2"));
        assert_eq!(algebraic_from_square(mv.to).as_deref(), Some("e4"));
        assert!(mv.promotion.is_none());
    }

    #[test]
    fn parse_promotion_move() {
        let mv = move_from_uci("e7e8q").expect("promotion");
        assert_eq!(algebraic_from_square(mv.from).as_deref(), Some("e7"));
        assert_eq!(algebraic_from_square(mv.to).as_deref(), Some("e8"));
        assert_eq!(mv.promotion, Some(PieceKind::Queen));
        assert_eq!(uci_from_move(mv).as_deref(), Some("e7e8q"));
    }

    #[test]
    fn reject_invalid_move_text() {
        assert!(move_from_uci("e2e").is_none());
        assert!(move_from_uci("e2e4qq").is_none());
        assert!(move_from_uci("e2e4x").is_none());
    }
}
