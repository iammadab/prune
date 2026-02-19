use crate::engine::board::Board;
use crate::engine::eval::Evaluator;
use crate::engine::movegen::{generate_legal, is_king_in_check};
#[cfg(feature = "qsearch")]
use crate::engine::search::quiescence::quiesce_ab;
use crate::engine::search::traits::{SearchAlgorithm, SearchResult};
use crate::engine::search::tt::{Bound, TTEntry, TranspositionTable};
use crate::engine::types::Move;

const MATE_SCORE: i32 = 30_000;
const QUIESCE_DEPTH: u32 = 4;
const TT_SIZE: usize = 1 << 20;

pub struct AlphaBetaSearch {
    tt: TranspositionTable,
}

impl Default for AlphaBetaSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl AlphaBetaSearch {
    pub fn new() -> Self {
        Self {
            tt: TranspositionTable::new(TT_SIZE),
        }
    }
}

impl SearchAlgorithm for AlphaBetaSearch {
    fn search(
        &mut self,
        board: &mut Board,
        evaluator: &impl Evaluator,
        depth: u32,
    ) -> SearchResult {
        self.search_root(board, evaluator, depth, None)
    }

    fn search_with_root_ordering(
        &mut self,
        board: &mut Board,
        evaluator: &impl Evaluator,
        depth: u32,
        preferred_root: Option<&[crate::engine::types::Move]>,
    ) -> SearchResult {
        self.search_root(board, evaluator, depth, preferred_root)
    }
}

impl AlphaBetaSearch {
    fn search_root(
        &mut self,
        board: &mut Board,
        evaluator: &impl Evaluator,
        depth: u32,
        preferred_root: Option<&[Move]>,
    ) -> SearchResult {
        let mut nodes = 0;
        let mut best_moves = Vec::new();
        let mut best_score = i32::MIN;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX;
        let alpha_orig = alpha;

        let mut moves = generate_legal(board);
        let tt_best = self
            .tt
            .probe(board.hash())
            .and_then(|entry| entry.best_move);
        moves = reorder_moves(&moves, tt_best, preferred_root);

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
                    self,
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
                    self,
                    board,
                    evaluator,
                    depth.saturating_sub(1),
                    -null_beta,
                    -alpha,
                    &mut nodes,
                );
                if score > alpha {
                    score = -alphabeta(
                        self,
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

        let bound = if best_score <= alpha_orig {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };
        let key = board.hash();
        self.tt.store(TTEntry {
            key,
            depth,
            score: best_score,
            bound,
            best_move: best_moves.first().copied(),
        });

        SearchResult {
            best_moves,
            score: best_score,
            nodes,
        }
    }
}

fn alphabeta(
    search: &mut AlphaBetaSearch,
    board: &mut Board,
    evaluator: &impl Evaluator,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    nodes: &mut u64,
) -> i32 {
    *nodes += 1;
    let alpha_orig = alpha;

    if let Some(entry) = search.tt.probe(board.hash()) {
        if entry.depth >= depth {
            match entry.bound {
                Bound::Exact => return entry.score,
                Bound::Lower if entry.score >= beta => return entry.score,
                Bound::Upper if entry.score <= alpha => return entry.score,
                _ => {}
            }
        }
    }

    if depth == 0 {
        if !is_king_in_check(board, board.side_to_move) {
            #[cfg(feature = "qsearch")]
            {
                return quiesce_ab(board, evaluator, alpha, beta, nodes, QUIESCE_DEPTH);
            }
            #[cfg(not(feature = "qsearch"))]
            {
                return evaluator.evaluate(board);
            }
        }

        let moves = generate_legal(board);
        if moves.is_empty() {
            // Subtract depth so faster mates score higher and slower losses are preferred.
            return -MATE_SCORE - depth as i32;
        }
        #[cfg(feature = "qsearch")]
        {
            return quiesce_ab(board, evaluator, alpha, beta, nodes, QUIESCE_DEPTH);
        }
        #[cfg(not(feature = "qsearch"))]
        {
            return evaluator.evaluate(board);
        }
    }

    let moves = generate_legal(board);
    if moves.is_empty() {
        if is_king_in_check(board, board.side_to_move) {
            // Subtract depth so faster mates score higher and slower losses are preferred.
            return -MATE_SCORE - depth as i32;
        }
        return 0;
    }

    let tt_best = search
        .tt
        .probe(board.hash())
        .and_then(|entry| entry.best_move);
    let moves = reorder_moves(&moves, tt_best, None);

    let mut best = i32::MIN;
    let mut best_move = None;
    for mv in moves {
        let undo = match board.make_move(mv) {
            Ok(undo) => undo,
            Err(_) => continue,
        };
        let score = -alphabeta(search, board, evaluator, depth - 1, -beta, -alpha, nodes);
        board.unmake_move(mv, undo);
        if score > best {
            best = score;
            best_move = Some(mv);
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }

    let bound = if best <= alpha_orig {
        Bound::Upper
    } else if best >= beta {
        Bound::Lower
    } else {
        Bound::Exact
    };
    search.tt.store(TTEntry {
        key: board.hash(),
        depth,
        score: best,
        bound,
        best_move,
    });

    best
}

fn reorder_moves(moves: &[Move], primary: Option<Move>, preferred: Option<&[Move]>) -> Vec<Move> {
    let mut ordered = Vec::with_capacity(moves.len());
    if let Some(primary) = primary {
        if moves.iter().any(|candidate| *candidate == primary) {
            ordered.push(primary);
        }
    }

    if let Some(preferred) = preferred {
        for mv in preferred {
            if moves.iter().any(|candidate| candidate == mv)
                && !ordered.iter().any(|candidate| candidate == mv)
            {
                ordered.push(*mv);
            }
        }
    }

    for mv in moves {
        if !ordered.iter().any(|candidate| candidate == mv) {
            ordered.push(*mv);
        }
    }

    ordered
}
