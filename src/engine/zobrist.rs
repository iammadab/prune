use crate::engine::board::Board;
use crate::engine::types::{Color, Move, Piece, PieceKind, Square};
use std::sync::OnceLock;

const PIECE_TYPES: usize = 12;
const SQUARES: usize = 64;

#[derive(Clone, Copy)]
struct ZobristKeys {
    piece_square: [[u64; SQUARES]; PIECE_TYPES],
    side_to_move: u64,
    castling_rights: [u64; 16],
    en_passant_file: [u64; 8],
}

pub fn compute_hash(board: &Board) -> u64 {
    let keys = keys();
    let mut hash = 0u64;

    for (index, square) in board.squares.iter().enumerate() {
        if let Some(piece) = square {
            let square_index = square_index_from_0x88(index as u8);
            if let Some(sq) = square_index {
                let piece_index = piece_index(*piece);
                hash ^= keys.piece_square[piece_index][sq];
            }
        }
    }

    if board.side_to_move == Color::Black {
        hash ^= keys.side_to_move;
    }

    let castling_index = board.castling_rights as usize & 0x0f;
    hash ^= keys.castling_rights[castling_index];

    if let Some(ep) = board.en_passant {
        let file = ep.index() & 0x0f;
        if file < 8 {
            hash ^= keys.en_passant_file[file as usize];
        }
    }

    hash
}

pub fn update_hash_for_move(
    board: &Board,
    mv: Move,
    original_piece: Piece,
    moved_piece: Piece,
    captured: Option<Piece>,
    captured_square: Option<Square>,
    rook_move: Option<(Square, Square)>,
    previous_castling: u8,
    previous_en_passant: Option<Square>,
) -> u64 {
    let keys = keys();
    let mut hash = board.hash;

    if previous_castling <= 0x0f {
        hash ^= keys.castling_rights[previous_castling as usize];
    }
    if let Some(ep) = previous_en_passant {
        let file = ep.index() & 0x0f;
        if file < 8 {
            hash ^= keys.en_passant_file[file as usize];
        }
    }

    hash ^= keys.side_to_move;

    if let Some(from_sq) = square_index(mv.from) {
        let piece_idx = piece_index(original_piece);
        hash ^= keys.piece_square[piece_idx][from_sq];
    }

    if let Some(capture_sq) = captured_square {
        if let Some(captured_piece) = captured {
            if let Some(capture_index) = square_index(capture_sq) {
                let captured_idx = piece_index(captured_piece);
                hash ^= keys.piece_square[captured_idx][capture_index];
            }
        }
    }

    if let Some(to_sq) = square_index(mv.to) {
        let moved_idx = piece_index(moved_piece);
        hash ^= keys.piece_square[moved_idx][to_sq];
    }

    if let Some((rook_from, rook_to)) = rook_move {
        let rook_piece = Piece {
            color: moved_piece.color,
            kind: PieceKind::Rook,
        };
        if let Some(rook_from_idx) = square_index(rook_from) {
            let rook_index = piece_index(rook_piece);
            hash ^= keys.piece_square[rook_index][rook_from_idx];
        }
        if let Some(rook_to_idx) = square_index(rook_to) {
            let rook_index = piece_index(rook_piece);
            hash ^= keys.piece_square[rook_index][rook_to_idx];
        }
    }

    let new_castling = board.castling_rights as usize & 0x0f;
    hash ^= keys.castling_rights[new_castling];

    if let Some(ep) = board.en_passant {
        let file = ep.index() & 0x0f;
        if file < 8 {
            hash ^= keys.en_passant_file[file as usize];
        }
    }

    hash
}

fn square_index(square: Square) -> Option<usize> {
    square_index_from_0x88(square.index())
}

fn square_index_from_0x88(index: u8) -> Option<usize> {
    if (index & 0x88) != 0 {
        return None;
    }
    let file = index & 0x0f;
    let rank = index >> 4;
    if file > 7 || rank > 7 {
        return None;
    }
    Some((rank as usize) * 8 + file as usize)
}

fn piece_index(piece: Piece) -> usize {
    let base = match piece.kind {
        PieceKind::Pawn => 0,
        PieceKind::Knight => 1,
        PieceKind::Bishop => 2,
        PieceKind::Rook => 3,
        PieceKind::Queen => 4,
        PieceKind::King => 5,
    };
    match piece.color {
        Color::White => base,
        Color::Black => base + 6,
    }
}

fn keys() -> &'static ZobristKeys {
    static KEYS: OnceLock<ZobristKeys> = OnceLock::new();
    KEYS.get_or_init(|| {
        let mut rng = SplitMix64::new(0x9e37_79b9_7f4a_7c15);
        let mut piece_square = [[0u64; SQUARES]; PIECE_TYPES];
        for piece in 0..PIECE_TYPES {
            for square in 0..SQUARES {
                piece_square[piece][square] = rng.next_u64();
            }
        }

        let mut castling_rights = [0u64; 16];
        for value in castling_rights.iter_mut() {
            *value = rng.next_u64();
        }

        let mut en_passant_file = [0u64; 8];
        for value in en_passant_file.iter_mut() {
            *value = rng.next_u64();
        }

        ZobristKeys {
            piece_square,
            side_to_move: rng.next_u64(),
            castling_rights,
            en_passant_file,
        }
    })
}

#[derive(Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        let mut z = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        self.state = z;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}
