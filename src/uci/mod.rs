use crate::engine::Engine;

mod commands;

pub use commands::{Command, GoCommand, PositionCommand};

pub fn run_loop(engine: &mut Engine) {
    let _ = engine;
}

pub fn parse_line(line: &str) -> Command {
    Command::Unknown(line.to_string())
}
