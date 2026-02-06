# prune

This is my chess engine worklog as I explore different search techniques and the fundamentals that make a chess engine: board representation, move generation, evaluation, and search.

## Usage
Define the engine with an evaluator and a search algorithm:

```rust
use chess_engine::engine::Engine;
use chess_engine::engine::eval::MaterialEvaluator;
use chess_engine::engine::search::{AlphaBetaSearch, MinimaxSearch};

let minimax_engine = Engine::with_components(MaterialEvaluator, MinimaxSearch);
let alphabeta_engine = Engine::with_components(MaterialEvaluator, AlphaBetaSearch);
```

Then run the UCI loop with the engine you want to use:

```rust
use chess_engine::uci;

let mut engine = Engine::with_components(MaterialEvaluator, AlphaBetaSearch);
uci::run_loop(&mut engine);
```
