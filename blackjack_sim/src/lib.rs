pub mod game;

use blackjack_lib::{BlackjackTable, Card, Deck};
pub use game::prelude::*;

// type Result<T, E> = Result<T, BlackjackGameError>;

/// Struct for running a number of simulations for a specific strategy.
/// This struct has the functionality for recording and writing specific stats of interest from the simulation as well.
/// A `BlackjackSimulator` object's main purupose is conveince, it acts as a wrapper for all the other structs needed to run a blackjack simulation to test a specific strategy.
/// It allows the user of the object to control specific parameters of a typical blackjack game that one would find at a casino,
/// such as number of decks, the counting strategy, number of shuffles, minimum bet, etc...
pub struct BlackjackSimulator<S: Strategy> {
    // strategy: S,
    game: BlackjackGameSim<S>,
    player_starting_balance: f32,
    table_starting_balance: f32,
    num_simulations: u32,
    // hands_per_simulation: u32,
    accumulated_wins: i32,
    accumulated_pushes: i32,
    accumulated_losses: i32,
    accumulated_winnings: f32,
    num_early_endings: i32,
    num_player_blackjacks: i32,
    silent: bool,
}

impl<S: Strategy> BlackjackSimulator<S> {
    /// Associated function for creating a new blackjack simulation. Takes in the necessary parameters for s
    /// tarting a blackjack Simulation and returns a new `BlackjackSimulator` object.
    pub fn new(
        strategy: S,
        player_starting_balance: f32,
        table_starting_balance: f32,
        num_simulations: u32,
        num_decks: usize,
        num_shuffles: u32,
        min_bet: u32,
        hands_per_simulation: u32,
        silent: bool,
    ) -> Self {
        let player = PlayerSim::new(player_starting_balance, strategy);
        let table = <BlackjackTableSim as BlackjackTable<PlayerSim<S>>>::new(
            table_starting_balance,
            num_decks,
            num_shuffles,
        );
        let game = BlackjackGameSim::new(table, player, hands_per_simulation, min_bet);
        Self {
            game,
            player_starting_balance,
            table_starting_balance,
            num_simulations,
            // hands_per_simulation,
            accumulated_wins: 0,
            accumulated_pushes: 0,
            accumulated_losses: 0,
            accumulated_winnings: 0.0,
            num_early_endings: 0,
            num_player_blackjacks: 0,
            silent,
        }
    }

    /// Method that will run the simulation, recording the necessary data. Returns a `Result<(), BlackjackGameError> if an error occurs during any simulation.
    pub fn run(&mut self) -> Result<(), BlackjackGameError> {
        // Run the simulation
        for i in 0..self.num_simulations {
            if let Err(e) = self.game.run() {
                return Err(e);
            }
            // Record data from simulation
            self.accumulated_wins += self.game.total_wins;
            self.accumulated_pushes += self.game.total_pushes;
            self.accumulated_losses += self.game.total_losses;
            self.accumulated_winnings += self.game.total_winnings;
            self.num_player_blackjacks += self.game.num_player_blackjacks;
            if self.game.ended_early {
                self.num_early_endings += 1;
            }
            if !self.silent {
                println!("simulation #{}", i + 1);
                self.game.display_stats();
            }
            // Reset balances for next simulation
            self.game
                .simulation_reset(self.table_starting_balance, self.player_starting_balance);
        }
        Ok(())
    }

    /// Method that will display the accumulated data recorded from running all simulations.
    pub fn display_stats(&self) {
        const width: usize = 80;
        const text_width: usize = "number of player blackjacks:".len() + 20;
        const numeric_width: usize = width - text_width;

        println!("{}", "-".repeat(width));
        println!(
            "{:-^width$}",
            format!("running {} simulations", self.num_simulations)
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "total wins:", self.accumulated_wins
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "total pushes:", self.accumulated_pushes
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "total losses:", self.accumulated_losses
        );
        println!(
            "{:<text_width$}{:>numeric_width$.2}",
            "total winnings:", self.accumulated_winnings
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "number of player blackjacks:", self.num_player_blackjacks
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "number of early endings", self.num_early_endings
        );
        println!("{}", "-".repeat(width));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple_simulation_test() {
        const MIN_BET: u32 = 5;
        let betting_strategy = MarginBettingStrategy::new(4.0, MIN_BET);
        let decision_strategy = BasicStrategy::new();
        let strategy = HiLo::new(6, MIN_BET, betting_strategy, decision_strategy);
        let mut simulator =
            BlackjackSimulator::new(strategy, 500.0, f32::MAX, 30, 6, 7, MIN_BET, 200, false);

        if let Err(e) = simulator.run() {
            panic!("error: {}", e);
        }

        simulator.display_stats();
        assert!(true);
    }
}
