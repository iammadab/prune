use chess_engine::engine::eval::MaterialEvaluator;
use chess_engine::engine::search::{AlphaBetaSearch, MinimaxSearch};
use chess_engine::engine::Engine;
use chess_engine::uci;
use std::env;

fn main() {
    let (default_depth, seed) = parse_args();
    // let mut engine = Engine::with_components(MaterialEvaluator, MinimaxSearch);
    let mut engine = Engine::with_components(MaterialEvaluator, AlphaBetaSearch::new());
    if let Some(seed) = seed {
        engine.set_rng_seed(seed);
    }
    uci::run_loop(&mut engine, default_depth);
}

fn parse_args() -> (u32, Option<u64>) {
    let mut default_depth = 6u32;
    let mut seed = None;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--depth" => match args.next() {
                Some(value) => match value.parse::<u32>() {
                    Ok(parsed) => default_depth = parsed,
                    Err(_) => eprintln!("invalid --depth: {value}"),
                },
                None => eprintln!("missing value for --depth"),
            },
            "--seed" => match args.next() {
                Some(value) => match value.parse::<u64>() {
                    Ok(parsed) => seed = Some(parsed),
                    Err(_) => eprintln!("invalid --seed: {value}"),
                },
                None => eprintln!("missing value for --seed"),
            },
            _ => eprintln!("unknown argument: {arg}"),
        }
    }

    (default_depth, seed)
}
