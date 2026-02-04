use crate::engine::types::Color;

pub const CASTLE_WHITE_KING: u8 = 1 << 0;
pub const CASTLE_WHITE_QUEEN: u8 = 1 << 1;
pub const CASTLE_BLACK_KING: u8 = 1 << 2;
pub const CASTLE_BLACK_QUEEN: u8 = 1 << 3;

pub fn has_kingside(rights: u8, color: Color) -> bool {
    match color {
        Color::White => rights & CASTLE_WHITE_KING != 0,
        Color::Black => rights & CASTLE_BLACK_KING != 0,
    }
}

pub fn has_queenside(rights: u8, color: Color) -> bool {
    match color {
        Color::White => rights & CASTLE_WHITE_QUEEN != 0,
        Color::Black => rights & CASTLE_BLACK_QUEEN != 0,
    }
}

pub fn revoke_kingside(rights: &mut u8, color: Color) {
    match color {
        Color::White => *rights &= !CASTLE_WHITE_KING,
        Color::Black => *rights &= !CASTLE_BLACK_KING,
    }
}

pub fn revoke_queenside(rights: &mut u8, color: Color) {
    match color {
        Color::White => *rights &= !CASTLE_WHITE_QUEEN,
        Color::Black => *rights &= !CASTLE_BLACK_QUEEN,
    }
}

pub fn revoke_all(rights: &mut u8, color: Color) {
    revoke_kingside(rights, color);
    revoke_queenside(rights, color);
}
