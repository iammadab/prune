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
uci::run_loop(&mut engine, 6);
```

The binary supports optional CLI flags:

```sh
cargo run -- --depth 8 --seed 12345
```

- `--depth` sets the default search depth when `go depth` is not provided.
- `--seed` sets the RNG seed so best-move sampling is deterministic; omit for nondeterministic sampling.

## Bench
Run the puzzle bench (defaults to mateIn1-5 CSVs):

```sh
cargo run --bin bench
```

Defaults:
- `--depth` defaults to 2.
- If no `--mate` is provided, it runs mateIn1-5.

Specify depth and mate counts:

```sh
cargo run --bin bench -- --depth 8
cargo run --bin bench -- --depth 8 --mate 3
cargo run --bin bench -- --mate 2 --mate 4
```
