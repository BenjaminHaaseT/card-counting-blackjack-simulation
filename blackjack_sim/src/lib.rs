pub mod game;

use blackjack_lib::{BlackjackTable, Card, Deck};
pub use game::prelude::*;

/// Struct for running a number of simulations for a specific strategy. 
/// This struct has the functionality for recording and writing specific stats of interest from the simulation as well.
pub struct BlackjackGameSimulator<S: Strategy> {
    game: BlackjackGameSim<S>,
    player_starting_balance: f32,
    table_starting_balance: f32,
    num_simulations: u32,
    num_hands_per_simulation: u32,
    accumulated_wins: i32,
    accumulated_pushes: i32,
    accumulated_losses: i32
    accumulated_winnings: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
}
