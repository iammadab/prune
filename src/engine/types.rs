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
}
