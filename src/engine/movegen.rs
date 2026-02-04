use crate::engine::board::Board;
use crate::engine::castling::{has_kingside, has_queenside};
use crate::engine::types::{is_valid_square, Color, Move, Piece, PieceKind, Square};

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

pub fn generate_pseudo_legal(board: &Board) -> MoveList {
    let mut moves = Vec::new();
    let side = board.side_to_move;

    for index in 0u8..128u8 {
        if !is_valid_square(index) {
            continue;
        }
        let piece = match board.squares[index as usize] {
            Some(piece) if piece.color == side => piece,
            _ => continue,
        };

        let from = Square(index);
        match piece.kind {
            PieceKind::Pawn => generate_pawn_moves(board, from, piece, &mut moves),
            PieceKind::Knight => {
                generate_jump_moves(board, from, piece, &KNIGHT_OFFSETS, &mut moves)
            }
            PieceKind::Bishop => {
                generate_slider_moves(board, from, piece, &BISHOP_OFFSETS, &mut moves)
            }
            PieceKind::Rook => generate_slider_moves(board, from, piece, &ROOK_OFFSETS, &mut moves),
            PieceKind::Queen => {
                generate_slider_moves(board, from, piece, &BISHOP_OFFSETS, &mut moves);
                generate_slider_moves(board, from, piece, &ROOK_OFFSETS, &mut moves);
            }
            PieceKind::King => generate_jump_moves(board, from, piece, &KING_OFFSETS, &mut moves),
        }
    }

    generate_castling_moves(board, &mut moves);

    moves
}

pub fn generate_legal(board: &mut Board) -> MoveList {
    let pseudo = generate_pseudo_legal(board);
    let mut legal = Vec::new();
    for mv in pseudo {
        let undo = match board.make_move(mv) {
            Ok(undo) => undo,
            Err(_) => continue,
        };
        let mover = match board.side_to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
        let in_check = is_king_in_check(board, mover);
        board.unmake_move(mv, undo);
        if !in_check {
            legal.push(mv);
        }
    }

    legal
}

pub fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_legal(board);
    let mut nodes = 0u64;
    for mv in moves {
        let undo = match board.make_move(mv) {
            Ok(undo) => undo,
            Err(_) => continue,
        };
        nodes += perft(board, depth - 1);
        board.unmake_move(mv, undo);
    }

    nodes
}

fn generate_pawn_moves(board: &Board, from: Square, piece: Piece, moves: &mut MoveList) {
    let from_rank = from.index() >> 4;
    match piece.color {
        Color::White => {
            let one = offset_square(from, 16);
            if let Some(to) = one {
                if board.squares[to.index() as usize].is_none() {
                    add_pawn_advance(from, to, moves);
                    if from_rank == 1 {
                        let two = offset_square(from, 32);
                        if let Some(to2) = two {
                            if board.squares[to2.index() as usize].is_none() {
                                moves.push(Move {
                                    from,
                                    to: to2,
                                    promotion: None,
                                });
                            }
                        }
                    }
                }
            }

            generate_pawn_capture(board, from, 15, moves);
            generate_pawn_capture(board, from, 17, moves);
            generate_en_passant(board, from, 15, moves);
            generate_en_passant(board, from, 17, moves);
        }
        Color::Black => {
            let one = offset_square(from, -16);
            if let Some(to) = one {
                if board.squares[to.index() as usize].is_none() {
                    add_pawn_advance(from, to, moves);
                    if from_rank == 6 {
                        let two = offset_square(from, -32);
                        if let Some(to2) = two {
                            if board.squares[to2.index() as usize].is_none() {
                                moves.push(Move {
                                    from,
                                    to: to2,
                                    promotion: None,
                                });
                            }
                        }
                    }
                }
            }

            generate_pawn_capture(board, from, -15, moves);
            generate_pawn_capture(board, from, -17, moves);
            generate_en_passant(board, from, -15, moves);
            generate_en_passant(board, from, -17, moves);
        }
    }
}

fn add_pawn_advance(from: Square, to: Square, moves: &mut MoveList) {
    let to_rank = to.index() >> 4;
    if to_rank == 0 || to_rank == 7 {
        for kind in [
            PieceKind::Queen,
            PieceKind::Rook,
            PieceKind::Bishop,
            PieceKind::Knight,
        ] {
            moves.push(Move {
                from,
                to,
                promotion: Some(kind),
            });
        }
    } else {
        moves.push(Move {
            from,
            to,
            promotion: None,
        });
    }
}

fn generate_pawn_capture(board: &Board, from: Square, offset: i8, moves: &mut MoveList) {
    let target = match offset_square(from, offset) {
        Some(square) => square,
        None => return,
    };
    let target_piece = match board.squares[target.index() as usize] {
        Some(piece) => piece,
        None => return,
    };
    let from_piece = board.squares[from.index() as usize].expect("pawn missing");
    if target_piece.color == from_piece.color {
        return;
    }

    add_pawn_advance(from, target, moves);
}

fn generate_en_passant(board: &Board, from: Square, offset: i8, moves: &mut MoveList) {
    let ep = match board.en_passant {
        Some(square) => square,
        None => return,
    };
    let target = match offset_square(from, offset) {
        Some(square) => square,
        None => return,
    };
    if target != ep {
        return;
    }

    moves.push(Move {
        from,
        to: ep,
        promotion: None,
    });
}

fn generate_jump_moves(
    board: &Board,
    from: Square,
    piece: Piece,
    offsets: &[i8],
    moves: &mut MoveList,
) {
    for offset in offsets {
        let to = match offset_square(from, *offset) {
            Some(square) => square,
            None => continue,
        };
        match board.squares[to.index() as usize] {
            None => moves.push(Move {
                from,
                to,
                promotion: None,
            }),
            Some(target) if target.color != piece.color => moves.push(Move {
                from,
                to,
                promotion: None,
            }),
            _ => {}
        }
    }
}

fn generate_slider_moves(
    board: &Board,
    from: Square,
    piece: Piece,
    offsets: &[i8],
    moves: &mut MoveList,
) {
    for offset in offsets {
        let mut current = from;
        loop {
            let next = match offset_square(current, *offset) {
                Some(square) => square,
                None => break,
            };
            match board.squares[next.index() as usize] {
                None => {
                    moves.push(Move {
                        from,
                        to: next,
                        promotion: None,
                    });
                    current = next;
                }
                Some(target) => {
                    if target.color != piece.color {
                        moves.push(Move {
                            from,
                            to: next,
                            promotion: None,
                        });
                    }
                    break;
                }
            }
        }
    }
}

fn generate_castling_moves(board: &Board, moves: &mut MoveList) {
    let side = board.side_to_move;
    match side {
        Color::White => generate_castling_for_color(board, side, 0, moves),
        Color::Black => generate_castling_for_color(board, side, 7, moves),
    }
}

fn generate_castling_for_color(board: &Board, color: Color, rank: u8, moves: &mut MoveList) {
    let king_square = Square(rank * 16 + 4);
    let king_piece = match board.squares[king_square.index() as usize] {
        Some(piece) if piece.kind == PieceKind::King && piece.color == color => piece,
        _ => return,
    };

    if has_kingside(board.castling_rights, color) {
        let f_square = Square(rank * 16 + 5);
        let g_square = Square(rank * 16 + 6);
        let rook_square = Square(rank * 16 + 7);
        let rook_ok = matches!(board.squares[rook_square.index() as usize], Some(Piece { color: c, kind: PieceKind::Rook }) if c == color);
        if rook_ok
            && board.squares[f_square.index() as usize].is_none()
            && board.squares[g_square.index() as usize].is_none()
        {
            moves.push(Move {
                from: king_square,
                to: g_square,
                promotion: None,
            });
        }
    }

    if has_queenside(board.castling_rights, color) {
        let b_square = Square(rank * 16 + 1);
        let c_square = Square(rank * 16 + 2);
        let d_square = Square(rank * 16 + 3);
        let rook_square = Square(rank * 16 + 0);
        let rook_ok = matches!(board.squares[rook_square.index() as usize], Some(Piece { color: c, kind: PieceKind::Rook }) if c == color);
        if rook_ok
            && board.squares[b_square.index() as usize].is_none()
            && board.squares[c_square.index() as usize].is_none()
            && board.squares[d_square.index() as usize].is_none()
        {
            moves.push(Move {
                from: king_square,
                to: c_square,
                promotion: None,
            });
        }
    }

    let _ = king_piece;
}

fn is_king_in_check(board: &Board, color: Color) -> bool {
    let king_square = match find_king(board, color) {
        Some(square) => square,
        None => return false,
    };
    is_square_attacked(board, king_square, opposite_color(color))
}

fn find_king(board: &Board, color: Color) -> Option<Square> {
    for index in 0u8..128u8 {
        if !is_valid_square(index) {
            continue;
        }
        match board.squares[index as usize] {
            Some(piece) if piece.color == color && piece.kind == PieceKind::King => {
                return Some(Square(index));
            }
            _ => {}
        }
    }
    None
}

fn is_square_attacked(board: &Board, square: Square, by_color: Color) -> bool {
    if is_attacked_by_pawn(board, square, by_color) {
        return true;
    }
    if is_attacked_by_jump(board, square, by_color, PieceKind::Knight, &KNIGHT_OFFSETS) {
        return true;
    }
    if is_attacked_by_slider(board, square, by_color, PieceKind::Bishop, &BISHOP_OFFSETS) {
        return true;
    }
    if is_attacked_by_slider(board, square, by_color, PieceKind::Rook, &ROOK_OFFSETS) {
        return true;
    }
    if is_attacked_by_slider(board, square, by_color, PieceKind::Queen, &BISHOP_OFFSETS) {
        return true;
    }
    if is_attacked_by_slider(board, square, by_color, PieceKind::Queen, &ROOK_OFFSETS) {
        return true;
    }
    if is_attacked_by_jump(board, square, by_color, PieceKind::King, &KING_OFFSETS) {
        return true;
    }

    false
}

fn is_attacked_by_pawn(board: &Board, square: Square, by_color: Color) -> bool {
    let offsets: [i8; 2] = match by_color {
        Color::White => [-15, -17],
        Color::Black => [15, 17],
    };
    for offset in offsets {
        let attacker = match offset_square(square, offset) {
            Some(attacker) => attacker,
            None => continue,
        };
        if let Some(piece) = board.squares[attacker.index() as usize] {
            if piece.color == by_color && piece.kind == PieceKind::Pawn {
                return true;
            }
        }
    }
    false
}

fn is_attacked_by_jump(
    board: &Board,
    square: Square,
    by_color: Color,
    kind: PieceKind,
    offsets: &[i8],
) -> bool {
    for offset in offsets {
        let attacker = match offset_square(square, *offset) {
            Some(attacker) => attacker,
            None => continue,
        };
        if let Some(piece) = board.squares[attacker.index() as usize] {
            if piece.color == by_color && piece.kind == kind {
                return true;
            }
        }
    }
    false
}

fn is_attacked_by_slider(
    board: &Board,
    square: Square,
    by_color: Color,
    kind: PieceKind,
    offsets: &[i8],
) -> bool {
    for offset in offsets {
        let mut current = square;
        loop {
            let next = match offset_square(current, *offset) {
                Some(square) => square,
                None => break,
            };
            match board.squares[next.index() as usize] {
                None => {
                    current = next;
                }
                Some(piece) => {
                    if piece.color == by_color && piece.kind == kind {
                        return true;
                    }
                    break;
                }
            }
        }
    }
    false
}

fn opposite_color(color: Color) -> Color {
    match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board::Board;
    use crate::engine::types::{square_from_algebraic, uci_from_move};

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

    #[test]
    fn generate_startpos_pseudo_legal_count() {
        let mut board = Board::new();
        board.set_startpos();
        let moves = generate_pseudo_legal(&board);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn generate_en_passant_move() {
        let mut board = Board::new();
        board.set_fen("8/8/8/3pP3/8/8/8/8 w - d6 0 1").expect("fen");
        let moves = generate_pseudo_legal(&board);
        let has_ep = moves
            .iter()
            .filter_map(|mv| uci_from_move(*mv))
            .any(|uci| uci == "e5d6");
        assert!(has_ep);
    }

    #[test]
    fn generate_castling_moves() {
        let mut board = Board::new();
        board
            .set_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
            .expect("fen");
        let moves = generate_pseudo_legal(&board);
        let uci_moves: Vec<String> = moves.iter().filter_map(|mv| uci_from_move(*mv)).collect();
        assert!(uci_moves.iter().any(|mv| mv == "e1g1"));
        assert!(uci_moves.iter().any(|mv| mv == "e1c1"));
    }

    #[test]
    fn generate_legal_startpos_count() {
        let mut board = Board::new();
        board.set_startpos();
        let moves = generate_legal(&mut board);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn perft_startpos_depths() {
        let mut board = Board::new();
        board.set_startpos();
        assert_eq!(perft(&mut board, 1), 20);
        assert_eq!(perft(&mut board, 2), 400);
        assert_eq!(perft(&mut board, 3), 8902);
        assert_eq!(perft(&mut board, 4), 197281);
    }
}
