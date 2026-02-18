use chess_engine::engine::eval::MaterialEvaluator;
use chess_engine::engine::search::{AlphaBetaSearch, MinimaxSearch};
use chess_engine::engine::Engine;
use std::collections::BTreeMap;
use std::fs;
use std::time::Instant;

#[derive(Debug, Clone)]
struct Puzzle {
    id: String,
    fen: String,
    moves: Vec<String>,
    mate: u8,
}

fn main() {
    let (depth, mate_counts) = parse_args();
    let mate_counts = if mate_counts.is_empty() {
        vec![1u8, 2, 3, 4, 5]
    } else {
        mate_counts
    };
    let mut puzzles_by_mate: BTreeMap<u8, Vec<Puzzle>> = BTreeMap::new();
    let mut total_puzzles = 0usize;

    for mate in mate_counts {
        let path = mate_to_path(mate);
        let mut file_puzzles =
            parse_puzzles_from_file(&path, mate).unwrap_or_else(|err| panic!("{path}: {err}"));
        println!("{path}: {} puzzles", file_puzzles.len());
        total_puzzles += file_puzzles.len();
        puzzles_by_mate
            .entry(mate)
            .or_default()
            .append(&mut file_puzzles);
    }

    println!("total puzzles: {total_puzzles}");

    let mut alphabeta = Engine::with_components(MaterialEvaluator, AlphaBetaSearch);
    print_engine_stats("alphabeta", &mut alphabeta, &puzzles_by_mate, depth);

    let mut minimax = Engine::with_components(MaterialEvaluator, MinimaxSearch);
    print_engine_stats("minimax", &mut minimax, &puzzles_by_mate, depth);
}

fn parse_args() -> (u32, Vec<u8>) {
    let mut depth = 6u32;
    let mut mate_counts = Vec::new();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--depth" => match args.next() {
                Some(value) => match value.parse::<u32>() {
                    Ok(parsed) => depth = parsed,
                    Err(_) => eprintln!("invalid --depth: {value}"),
                },
                None => eprintln!("missing value for --depth"),
            },
            "--mate" => match args.next() {
                Some(value) => match value.parse::<u8>() {
                    Ok(parsed) => mate_counts.push(parsed),
                    Err(_) => eprintln!("invalid --mate: {value}"),
                },
                None => eprintln!("missing value for --mate"),
            },
            _ => eprintln!("unknown argument: {arg}"),
        }
    }

    (depth, mate_counts)
}

fn mate_to_path(mate: u8) -> String {
    format!("bench/puzzles/mateIn{mate}.csv")
}

struct BenchStats {
    solved: usize,
    total: usize,
}

impl BenchStats {
    fn solve_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.solved as f64) * 100.0 / (self.total as f64)
        }
    }
}

fn print_engine_stats<E, S>(
    name: &'static str,
    engine: &mut Engine<E, S>,
    puzzles_by_mate: &BTreeMap<u8, Vec<Puzzle>>,
    depth: u32,
) where
    E: chess_engine::engine::eval::Evaluator,
    S: chess_engine::engine::search::SearchAlgorithm,
{
    let mut total_solved = 0usize;
    let mut total_puzzles = 0usize;
    let mut total_elapsed = 0.0f64;

    println!("engine: {name}");

    for (mate, puzzles) in puzzles_by_mate.iter() {
        let start = Instant::now();
        let stats = run_engine_on_puzzles(name, engine, puzzles, depth);
        let elapsed = start.elapsed().as_secs_f64();
        total_solved += stats.solved;
        total_puzzles += stats.total;
        total_elapsed += elapsed;
        println!(
            "mate {}: solved {}/{} ({:.2}%) in {:.2}s",
            mate,
            stats.solved,
            stats.total,
            stats.solve_rate(),
            elapsed
        );
    }

    let total_stats = BenchStats {
        solved: total_solved,
        total: total_puzzles,
    };
    println!(
        "total: solved {}/{} ({:.2}%) in {:.2}s",
        total_stats.solved,
        total_stats.total,
        total_stats.solve_rate(),
        total_elapsed
    );
}

fn run_engine_on_puzzles<E, S>(
    name: &'static str,
    engine: &mut Engine<E, S>,
    puzzles: &[Puzzle],
    depth: u32,
) -> BenchStats
where
    E: chess_engine::engine::eval::Evaluator,
    S: chess_engine::engine::search::SearchAlgorithm,
{
    let mut solved = 0usize;
    let total = puzzles.len();

    for puzzle in puzzles {
        if puzzle.moves.is_empty() {
            continue;
        }

        if let Err(err) = engine.set_position_fen(&puzzle.fen) {
            eprintln!("{name}: invalid FEN {}: {err}", puzzle.id);
            continue;
        }

        let mut solved_puzzle = true;
        engine.apply_move_list(&[puzzle.moves[0].clone()]);

        for (idx, expected) in puzzle.moves.iter().enumerate().skip(1) {
            let engine_turn = idx % 2 == 1;
            if engine_turn {
                let best = engine.search_depth(depth);
                if best != *expected {
                    solved_puzzle = false;
                    break;
                }
            }

            engine.apply_move_list(&[expected.to_string()]);
        }

        if solved_puzzle {
            solved += 1;
        }
    }

    BenchStats { solved, total }
}

fn parse_puzzles_from_file(path: &str, mate: u8) -> Result<Vec<Puzzle>, String> {
    let contents =
        fs::read_to_string(path).map_err(|err| format!("failed to read {}: {err}", path))?;
    let mut lines = contents.lines();
    let _ = lines
        .next()
        .ok_or_else(|| "missing header row".to_string())?;

    let mut puzzles = Vec::new();
    for (line_number, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let puzzle = parse_puzzle_row(line, mate).map_err(|err| {
            let display_line = line_number + 2;
            format!("line {display_line}: {err}")
        })?;
        puzzles.push(puzzle);
    }

    Ok(puzzles)
}

fn parse_puzzle_row(line: &str, mate: u8) -> Result<Puzzle, String> {
    let normalized = line.trim_end_matches('\r');
    let fields = parse_first_three_fields(normalized)?;

    let id = fields[0].to_string();
    let fen = fields[1].to_string();
    let moves_field = fields[2].as_str();

    let moves: Vec<String> = moves_field
        .split_whitespace()
        .map(|mv| mv.to_string())
        .collect();

    if moves.is_empty() {
        return Err("Moves value is empty".to_string());
    }

    Ok(Puzzle {
        id,
        fen,
        moves,
        mate,
    })
}

fn parse_first_three_fields(line: &str) -> Result<Vec<String>, String> {
    let parts: Vec<&str> = line.splitn(4, ',').collect();
    if parts.len() < 3 {
        return Err("expected at least 3 CSV fields".to_string());
    }

    Ok(parts[..3].iter().map(|part| (*part).to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sample_puzzle_row() {
        let line = "000rZ,2kr1b1r/p1p2pp1/2pqb3/7p/3N2n1/2NPB3/PPP2PPP/R2Q1RK1 w - - 2 13,d4e6 d6h2,822,85,100,420,kingsideAttack mate mateIn1 oneMove opening,https://lichess.org/seIMDWkD#25,Scandinavian_Defense Scandinavian_Defense_Modern_Variation";

        let puzzle = parse_puzzle_row(line, 1).expect("row parse");

        assert_eq!(puzzle.id, "000rZ");
        assert_eq!(
            puzzle.fen,
            "2kr1b1r/p1p2pp1/2pqb3/7p/3N2n1/2NPB3/PPP2PPP/R2Q1RK1 w - - 2 13"
        );
        assert_eq!(puzzle.moves, vec!["d4e6".to_string(), "d6h2".to_string()]);
        assert_eq!(puzzle.mate, 1);
    }
}
