#[derive(Debug)]
pub enum Command {
    Uci,
    IsReady,
    UciNewGame,
    Position(PositionCommand),
    Go(GoCommand),
    Stop,
    Quit,
    Unknown(String),
}

#[derive(Debug, Default)]
pub struct PositionCommand {
    pub fen: Option<String>,
    pub moves: Vec<String>,
}

#[derive(Debug, Default)]
pub struct GoCommand {
    pub depth: Option<u32>,
    pub movetime: Option<u64>,
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
}
