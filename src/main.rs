use chess_engine::engine::eval::MaterialEvaluator;
use chess_engine::engine::search::MinimaxSearch;
use chess_engine::engine::Engine;
use chess_engine::uci;

fn main() {
    let mut engine = Engine::with_components(MaterialEvaluator, MinimaxSearch);
    uci::run_loop(&mut engine);
}
