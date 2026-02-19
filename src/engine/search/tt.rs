use crate::engine::types::Move;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
    pub key: u64,
    pub depth: u32,
    pub score: i32,
    pub bound: Bound,
    pub best_move: Option<Move>,
}

pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
    mask: usize,
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        let size = size.next_power_of_two().max(1);
        Self {
            entries: vec![None; size],
            mask: size - 1,
        }
    }

    pub fn probe(&self, key: u64) -> Option<TTEntry> {
        let index = self.index(key);
        match self.entries[index] {
            Some(entry) if entry.key == key => Some(entry),
            _ => None,
        }
    }

    pub fn store(&mut self, entry: TTEntry) {
        let index = self.index(entry.key);
        match self.entries[index] {
            None => self.entries[index] = Some(entry),
            Some(existing) => {
                if entry.depth >= existing.depth {
                    self.entries[index] = Some(entry);
                }
            }
        }
    }

    fn index(&self, key: u64) -> usize {
        (key as usize) & self.mask
    }
}
