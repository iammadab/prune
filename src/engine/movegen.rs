use crate::engine::board::Board;
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

    moves
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board::Board;
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

    #[test]
    fn generate_startpos_pseudo_legal_count() {
        let mut board = Board::new();
        board.set_startpos();
        let moves = generate_pseudo_legal(&board);
        assert_eq!(moves.len(), 20);
    }
}
