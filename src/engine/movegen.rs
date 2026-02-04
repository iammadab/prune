use crate::engine::types::{is_valid_square, Move, Square};

pub type MoveList = Vec<Move>;

pub const KNIGHT_OFFSETS: [i8; 8] = [-33, -31, -18, -14, 14, 18, 31, 33];
pub const BISHOP_OFFSETS: [i8; 4] = [-17, -15, 15, 17];
pub const ROOK_OFFSETS: [i8; 4] = [-16, -1, 1, 16];
pub const KING_OFFSETS: [i8; 8] = [-17, -16, -15, -1, 1, 15, 16, 17];

pub fn is_onboard(square: Square) -> bool {
    is_valid_square(square.index())
}

pub fn offset_square(square: Square, offset: i8) -> Option<Square> {
    let index = square.index() as i16 + offset as i16;
    if index < 0 || index > 127 {
        return None;
    }
    let candidate = Square(index as u8);
    if is_onboard(candidate) {
        Some(candidate)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::square_from_algebraic;

    #[test]
    fn offset_square_rejects_offboard() {
        let a1 = square_from_algebraic("a1").unwrap();
        assert!(offset_square(a1, -16).is_none());
        assert!(offset_square(a1, -1).is_none());
    }

    #[test]
    fn offset_square_allows_onboard() {
        let a1 = square_from_algebraic("a1").unwrap();
        let a2 = offset_square(a1, 16).expect("a2");
        assert_eq!(a2.index(), 16);
    }
}
