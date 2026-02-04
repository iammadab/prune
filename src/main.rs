use chess_engine::engine::Engine;
use chess_engine::uci;

fn main() {
    let mut engine = Engine::new();
    uci::run_loop(&mut engine);
}
