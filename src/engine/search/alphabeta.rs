use crate::engine::board::Board;
use crate::engine::eval::Evaluator;
use crate::engine::movegen::{generate_legal, is_king_in_check};
use crate::engine::search::traits::{SearchAlgorithm, SearchResult};

const MATE_SCORE: i32 = 30_000;

pub struct AlphaBetaSearch;

impl SearchAlgorithm for AlphaBetaSearch {
    fn search(
        &mut self,
        board: &mut Board,
        evaluator: &impl Evaluator,
        depth: u32,
    ) -> SearchResult {
        let mut nodes = 0;
        let mut best_moves = Vec::new();
        let mut best_score = i32::MIN;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX;

        let moves = generate_legal(board);
        if moves.is_empty() {
            return SearchResult {
                best_moves: Vec::new(),
                score: evaluator.evaluate(board),
                nodes,
            };
        }

        let mut first_move = true;
        for mv in moves {
            let undo = match board.make_move(mv) {
                Ok(undo) => undo,
                Err(_) => continue,
            };
            let mut exact = false;
            let mut score = i32::MIN;
            if first_move {
                score = -alphabeta(
                    board,
                    evaluator,
                    depth.saturating_sub(1),
                    -beta,
                    -alpha,
                    &mut nodes,
                );
                exact = true;
                first_move = false;
            } else {
                let null_beta = alpha.saturating_add(1);
                score = -alphabeta(
                    board,
                    evaluator,
                    depth.saturating_sub(1),
                    -null_beta,
                    -alpha,
                    &mut nodes,
                );
                if score > alpha {
                    score = -alphabeta(
                        board,
                        evaluator,
                        depth.saturating_sub(1),
                        -beta,
                        -alpha,
                        &mut nodes,
                    );
                    exact = true;
                }
            }
            board.unmake_move(mv, undo);
            if exact {
                if score > best_score {
                    best_score = score;
                    best_moves.clear();
                    best_moves.push(mv);
                } else if score == best_score {
                    best_moves.push(mv);
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }

        SearchResult {
            best_moves,
            score: best_score,
            nodes,
        }
    }
}

fn alphabeta(
    board: &mut Board,
    evaluator: &impl Evaluator,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    nodes: &mut u64,
) -> i32 {
    *nodes += 1;
    let moves = generate_legal(board);
    if moves.is_empty() {
        if is_king_in_check(board, board.side_to_move) {
            // Subtract depth so faster mates score higher and slower losses are preferred.
            return -MATE_SCORE - depth as i32;
        }
        return 0;
    }

    if depth == 0 {
        return evaluator.evaluate(board);
    }

    let mut best = i32::MIN;
    for mv in moves {
        let undo = match board.make_move(mv) {
            Ok(undo) => undo,
            Err(_) => continue,
        };
        let score = -alphabeta(board, evaluator, depth - 1, -beta, -alpha, nodes);
        board.unmake_move(mv, undo);
        if score > best {
            best = score;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }

    best
}
