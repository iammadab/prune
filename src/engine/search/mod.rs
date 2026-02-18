pub mod alphabeta;
pub mod minimax;
#[cfg(feature = "qsearch")]
pub mod quiescence;
pub mod traits;

pub use alphabeta::AlphaBetaSearch;
pub use minimax::MinimaxSearch;
pub use traits::{SearchAlgorithm, SearchResult};

#[cfg(test)]
mod tests;
