use crate::engine::eval::Evaluator;
use crate::engine::Engine;
use std::io::{self, Write};

mod commands;

pub use commands::{Command, GoCommand, PositionCommand};

pub fn run_loop<E: Evaluator>(engine: &mut Engine<E>) {
    let stdin = io::stdin();

    loop {
        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match parse_line(line) {
            Command::Uci => {
                write_line("id name prune");
                write_line("id author madab");
                write_line("uciok");
            }
            Command::IsReady => {
                write_line("readyok");
            }
            Command::UciNewGame => {
                engine.reset_state();
            }
            Command::Position(cmd) => {
                match cmd.fen.as_deref() {
                    Some(fen) => engine.set_position_fen(fen),
                    None => engine.set_position_startpos(),
                }
                engine.apply_move_list(&cmd.moves);
            }
            Command::Go(cmd) => {
                let depth = cmd.depth.unwrap_or(1);
                let status = engine.game_status();
                match status {
                    crate::engine::types::GameStatus::Ongoing => {
                        let bestmove = engine.search_depth(depth);
                        write_line(&format!("bestmove {bestmove}"));
                    }
                    crate::engine::types::GameStatus::Checkmate
                    | crate::engine::types::GameStatus::Stalemate => {
                        write_line("bestmove 0000");
                    }
                }
            }
            Command::Stop => {
                engine.stop_search();
                write_line("bestmove 0000");
            }
            Command::Quit => {
                break;
            }
            Command::Unknown(_) => {}
        }
    }
}

pub fn parse_line(line: &str) -> Command {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let Some((first, rest)) = tokens.split_first() else {
        return Command::Unknown(line.to_string());
    };

    match *first {
        "uci" => Command::Uci,
        "isready" => Command::IsReady,
        "ucinewgame" => Command::UciNewGame,
        "position" => parse_position(rest).unwrap_or_else(|| Command::Unknown(line.to_string())),
        "go" => parse_go(rest).unwrap_or_else(|| Command::Unknown(line.to_string())),
        "stop" => Command::Stop,
        "quit" => Command::Quit,
        _ => Command::Unknown(line.to_string()),
    }
}

fn parse_position(tokens: &[&str]) -> Option<Command> {
    if tokens.is_empty() {
        return None;
    }

    let mut cmd = PositionCommand::default();

    match tokens[0] {
        "startpos" => {
            if let Some(moves_index) = tokens.iter().position(|&t| t == "moves") {
                if moves_index + 1 < tokens.len() {
                    cmd.moves = tokens[moves_index + 1..]
                        .iter()
                        .map(|m| (*m).to_string())
                        .collect();
                }
            }
        }
        "fen" => {
            if tokens.len() < 7 {
                return None;
            }

            let fen_fields = &tokens[1..7];
            cmd.fen = Some(fen_fields.join(" "));

            if tokens.len() > 7 {
                if tokens[7] == "moves" && tokens.len() > 8 {
                    cmd.moves = tokens[8..].iter().map(|m| (*m).to_string()).collect();
                }
            }
        }
        _ => return None,
    }

    Some(Command::Position(cmd))
}

fn parse_go(tokens: &[&str]) -> Option<Command> {
    let mut cmd = GoCommand::default();
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i] {
            "depth" => {
                if i + 1 < tokens.len() {
                    cmd.depth = tokens[i + 1].parse().ok();
                    i += 1;
                }
            }
            "movetime" => {
                if i + 1 < tokens.len() {
                    cmd.movetime = tokens[i + 1].parse().ok();
                    i += 1;
                }
            }
            "wtime" => {
                if i + 1 < tokens.len() {
                    cmd.wtime = tokens[i + 1].parse().ok();
                    i += 1;
                }
            }
            "btime" => {
                if i + 1 < tokens.len() {
                    cmd.btime = tokens[i + 1].parse().ok();
                    i += 1;
                }
            }
            "winc" => {
                if i + 1 < tokens.len() {
                    cmd.winc = tokens[i + 1].parse().ok();
                    i += 1;
                }
            }
            "binc" => {
                if i + 1 < tokens.len() {
                    cmd.binc = tokens[i + 1].parse().ok();
                    i += 1;
                }
            }
            _ => {}
        }

        i += 1;
    }

    Some(Command::Go(cmd))
}

fn write_line(line: &str) {
    println!("{line}");
    let _ = io::stdout().flush();
}
