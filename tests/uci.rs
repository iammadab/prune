use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[test]
fn uci_handshake_works() {
    let exe = resolve_engine_exe();
    let mut child = Command::new(exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn engine binary");

    {
        let stdin = child.stdin.as_mut().expect("failed to open stdin");
        stdin
            .write_all(b"uci\nisready\nquit\n")
            .expect("failed to write to stdin");
    }

    let output = child
        .wait_with_output()
        .expect("failed to read engine output");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("id name prune"));
    assert!(stdout.contains("id author madab"));
    assert!(stdout.contains("uciok"));
    assert!(stdout.contains("readyok"));
}

#[test]
fn uci_reports_invalid_fen() {
    let exe = resolve_engine_exe();
    let mut child = Command::new(exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn engine binary");

    {
        let stdin = child.stdin.as_mut().expect("failed to open stdin");
        stdin
            .write_all(b"uci\nposition fen 8/8/8/8/8/8/8/8 w - - 0 1\nquit\n")
            .expect("failed to write to stdin");
    }

    let output = child
        .wait_with_output()
        .expect("failed to read engine output");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("info string invalid FEN:"));
    assert!(stdout.contains("missing white king"));
}

fn resolve_engine_exe() -> PathBuf {
    if let Some(exe) = option_env!("CARGO_BIN_EXE_chess_engine") {
        return PathBuf::from(exe);
    }

    let exe_name = format!("chess-engine{}", std::env::consts::EXE_SUFFIX);
    let current = std::env::current_exe().expect("failed to get current exe");
    let exe_dir = current
        .parent()
        .and_then(|dir| dir.parent())
        .expect("failed to resolve target directory");
    let candidate = exe_dir.join(exe_name);

    if candidate.exists() {
        candidate
    } else {
        panic!("engine binary not found at {}", candidate.display());
    }
}
