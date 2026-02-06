# Chess Engine Worklog

This is my chess engine worklog as I explore different search techniques and the fundamentals that make a chess engine: board representation, move generation, evaluation, and search.

## Worklog
- Search: Minimax (negamax) baseline with a pluggable Alpha-Beta pruning variant and node counting.
- Evaluation: material-only scoring from the side-to-move perspective.
- Movegen: 0x88 board, pseudo-legal generation, legal filtering, en passant, and castling moves.
- Rules/Legality: castling through/out of/into check blocked; game status for mate/stalemate.
- FEN: parsing plus semantic validation (king count, pawn ranks, castling rights, en passant).
- UCI: basic loop with `position`, `go depth`, `stop`, and terminal `bestmove 0000`.
- Tests: perft depths, castling/en passant cases, FEN validation, evaluation sanity checks.
