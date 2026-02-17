use std::fs;

#[derive(Debug, Clone)]
struct Puzzle {
    id: String,
    fen: String,
    moves: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct HeaderMap {
    id_idx: usize,
    fen_idx: usize,
    moves_idx: usize,
}

const HEADER_MAP: HeaderMap = HeaderMap {
    id_idx: 0,
    fen_idx: 1,
    moves_idx: 2,
};

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
        let puzzle = parse_puzzle_row(line, HEADER_MAP).map_err(|err| {
            let display_line = line_number + 2;
            format!("line {display_line}: {err}")
        })?;
        puzzles.push(puzzle);
    }

    Ok(puzzles)
}

fn parse_puzzle_row(line: &str, header: HeaderMap) -> Result<Puzzle, String> {
    let normalized = line.trim_end_matches('\r');
    let fields = parse_csv_line(normalized)?;

    let id = fields
        .get(header.id_idx)
        .ok_or_else(|| "missing PuzzleId value".to_string())?
        .to_string();
    let fen = fields
        .get(header.fen_idx)
        .ok_or_else(|| "missing FEN value".to_string())?
        .to_string();
    let moves_field = fields
        .get(header.moves_idx)
        .ok_or_else(|| "missing Moves value".to_string())?;

    let moves: Vec<String> = moves_field
        .split_whitespace()
        .map(|mv| mv.to_string())
        .collect();

    if moves.is_empty() {
        return Err("Moves value is empty".to_string());
    }

    Ok(Puzzle { id, fen, moves })
}

fn parse_csv_line(line: &str) -> Result<Vec<String>, String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes {
                    if matches!(chars.peek(), Some('"')) {
                        chars.next();
                        current.push('"');
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                }
            }
            ',' if !in_quotes => {
                fields.push(current);
                current = String::new();
            }
            _ => current.push(ch),
        }
    }

    if in_quotes {
        return Err("unterminated quoted field".to_string());
    }

    fields.push(current);
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sample_puzzle_row() {
        let line = "000rZ,2kr1b1r/p1p2pp1/2pqb3/7p/3N2n1/2NPB3/PPP2PPP/R2Q1RK1 w - - 2 13,d4e6 d6h2,822,85,100,420,kingsideAttack mate mateIn1 oneMove opening,https://lichess.org/seIMDWkD#25,Scandinavian_Defense Scandinavian_Defense_Modern_Variation";

        let puzzle = parse_puzzle_row(line, HEADER_MAP).expect("row parse");

        assert_eq!(puzzle.id, "000rZ");
        assert_eq!(
            puzzle.fen,
            "2kr1b1r/p1p2pp1/2pqb3/7p/3N2n1/2NPB3/PPP2PPP/R2Q1RK1 w - - 2 13"
        );
        assert_eq!(puzzle.moves, vec!["d4e6".to_string(), "d6h2".to_string()]);
    }
}
