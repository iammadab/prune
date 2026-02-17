use std::fs;

#[derive(Debug, Clone)]
struct Puzzle {
    id: String,
    fen: String,
    moves: Vec<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let puzzle_paths = if args.is_empty() {
        vec![
            "bench/puzzles/mateIn1.csv".to_string(),
            "bench/puzzles/mateIn2.csv".to_string(),
            "bench/puzzles/mateIn3.csv".to_string(),
            "bench/puzzles/mateIn4.csv".to_string(),
            "bench/puzzles/mateIn5.csv".to_string(),
        ]
    } else {
        args
    };

    let mut total = 0usize;
    for path in puzzle_paths {
        let puzzles = parse_puzzles_from_file(&path).unwrap_or_else(|err| panic!("{path}: {err}"));
        println!("{path}: {} puzzles", puzzles.len());
        total += puzzles.len();
    }

    println!("total puzzles: {total}");
}

fn parse_puzzles_from_file(path: &str) -> Result<Vec<Puzzle>, String> {
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
        let puzzle = parse_puzzle_row(line).map_err(|err| {
            let display_line = line_number + 2;
            format!("line {display_line}: {err}")
        })?;
        puzzles.push(puzzle);
    }

    Ok(puzzles)
}

fn parse_puzzle_row(line: &str) -> Result<Puzzle, String> {
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

    Ok(Puzzle { id, fen, moves })
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

        let puzzle = parse_puzzle_row(line).expect("row parse");

        assert_eq!(puzzle.id, "000rZ");
        assert_eq!(
            puzzle.fen,
            "2kr1b1r/p1p2pp1/2pqb3/7p/3N2n1/2NPB3/PPP2PPP/R2Q1RK1 w - - 2 13"
        );
        assert_eq!(puzzle.moves, vec!["d4e6".to_string(), "d6h2".to_string()]);
    }
}
