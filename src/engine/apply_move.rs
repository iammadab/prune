use crate::engine::board::Board;
use crate::engine::castling::{revoke_all, revoke_kingside, revoke_queenside};
use crate::engine::types::{Color, Move, Piece, PieceKind, Square};

pub fn apply_move(board: &mut Board, mv: Move) -> Result<(), String> {
    let ctx = MoveContext::new(board, mv)?;
    let moved_piece = match ctx.mv.promotion {
        Some(kind) => Piece {
            color: ctx.piece.color,
            kind,
        },
        None => ctx.piece,
    };

    let was_capture = apply_piece_move(board, &ctx, moved_piece)?;
    if ctx.is_castle {
        apply_castle_rook_move(board, &ctx)?;
    }

    update_en_passant(board, &ctx);
    update_castling_rights(&mut board.castling_rights, &ctx, was_capture);
    update_clocks(board, &ctx, was_capture);

    Ok(())
}

struct MoveContext {
    mv: Move,
    piece: Piece,
    from_index: u8,
    to_index: u8,
    from_file: u8,
    to_file: u8,
    from_rank: u8,
    to_rank: u8,
    was_capture: bool,
    is_pawn: bool,
    is_castle: bool,
    is_en_passant_capture: bool,
}

impl MoveContext {
    fn new(board: &Board, mv: Move) -> Result<Self, String> {
        let from_index = mv.from.index();
        let to_index = mv.to.index();
        let piece = board.squares[from_index as usize]
            .ok_or_else(|| "no piece on from square".to_string())?;
        if piece.color != board.side_to_move {
            return Err("piece does not match side to move".to_string());
        }
        let was_capture = board.squares[to_index as usize].is_some();
        let is_en_passant_capture =
            piece.kind == PieceKind::Pawn && board.en_passant == Some(mv.to) && !was_capture;
        let from_file = from_index & 0x0f;
        let to_file = to_index & 0x0f;
        let from_rank = from_index >> 4;
        let to_rank = to_index >> 4;
        let is_castle = piece.kind == PieceKind::King
            && from_rank == to_rank
            && (from_file as i8 - to_file as i8).abs() == 2;
        let is_pawn = piece.kind == PieceKind::Pawn;

        Ok(Self {
            mv,
            piece,
            from_index,
            to_index,
            from_file,
            to_file,
            from_rank,
            to_rank,
            was_capture,
            is_pawn,
            is_castle,
            is_en_passant_capture,
        })
    }
}

fn apply_piece_move(
    board: &mut Board,
    ctx: &MoveContext,
    moved_piece: Piece,
) -> Result<bool, String> {
    board.squares[ctx.from_index as usize] = None;
    let mut was_capture = ctx.was_capture;

    if ctx.is_en_passant_capture {
        let capture_index = match ctx.piece.color {
            Color::White => ctx.to_index - 16,
            Color::Black => ctx.to_index + 16,
        };
        board.squares[capture_index as usize] = None;
        was_capture = true;
    }

    board.squares[ctx.to_index as usize] = Some(moved_piece);
    Ok(was_capture)
}

fn apply_castle_rook_move(board: &mut Board, ctx: &MoveContext) -> Result<(), String> {
    let (rook_from_file, rook_to_file) = match ctx.to_file {
        6 => (7, 5),
        2 => (0, 3),
        _ => return Err("invalid castling target".to_string()),
    };
    let rook_rank = ctx.from_rank;
    let rook_from_index = (rook_rank * 16 + rook_from_file) as usize;
    let rook_to_index = (rook_rank * 16 + rook_to_file) as usize;
    let rook = board.squares[rook_from_index].ok_or_else(|| "no rook for castling".to_string())?;
    if rook.kind != PieceKind::Rook || rook.color != ctx.piece.color {
        return Err("invalid rook for castling".to_string());
    }
    board.squares[rook_from_index] = None;
    board.squares[rook_to_index] = Some(rook);
    Ok(())
}

fn update_en_passant(board: &mut Board, ctx: &MoveContext) {
    let mut new_en_passant = None;
    if ctx.is_pawn {
        if ctx.piece.color == Color::White && ctx.from_rank == 1 && ctx.to_rank == 3 {
            new_en_passant = Some(Square(ctx.from_index + 16));
        } else if ctx.piece.color == Color::Black && ctx.from_rank == 6 && ctx.to_rank == 4 {
            new_en_passant = Some(Square(ctx.from_index - 16));
        }
    }
    board.en_passant = new_en_passant;
}

fn update_castling_rights(rights: &mut u8, ctx: &MoveContext, was_capture: bool) {
    if ctx.piece.kind == PieceKind::King {
        revoke_all(rights, ctx.piece.color);
    }

    if ctx.piece.kind == PieceKind::Rook {
        match (ctx.piece.color, ctx.from_file, ctx.from_rank) {
            (Color::White, 0, 0) => revoke_queenside(rights, Color::White),
            (Color::White, 7, 0) => revoke_kingside(rights, Color::White),
            (Color::Black, 0, 7) => revoke_queenside(rights, Color::Black),
            (Color::Black, 7, 7) => revoke_kingside(rights, Color::Black),
            _ => {}
        }
    }

    if was_capture {
        match (ctx.to_file, ctx.to_rank) {
            (0, 0) => revoke_queenside(rights, Color::White),
            (7, 0) => revoke_kingside(rights, Color::White),
            (0, 7) => revoke_queenside(rights, Color::Black),
            (7, 7) => revoke_kingside(rights, Color::Black),
            _ => {}
        }
    }
}

fn update_clocks(board: &mut Board, ctx: &MoveContext, was_capture: bool) {
    if ctx.is_pawn || was_capture {
        board.halfmove_clock = 0;
    } else {
        board.halfmove_clock = board.halfmove_clock.saturating_add(1);
    }

    if board.side_to_move == Color::Black {
        board.fullmove_number = board.fullmove_number.saturating_add(1);
    }
    board.side_to_move = match board.side_to_move {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
}
