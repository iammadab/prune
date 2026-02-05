pub mod alphabeta;
pub mod minimax;
pub mod traits;

#[cfg(test)]
mod tests;

pub use alphabeta::AlphaBetaSearch;
pub use minimax::MinimaxSearch;
pub use traits::{SearchAlgorithm, SearchResult};
