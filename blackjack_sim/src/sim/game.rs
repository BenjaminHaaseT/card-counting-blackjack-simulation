//! Module that focuses on the simulation of a single game of blackjack. In otherwords,
//!  this module provides all the functionality needed to test a single game of blackjack for a given counting strategy.

use crate::sim::game::player::PlayerSim;
use crate::sim::game::strategy::{
    BasicStrategy, BettingStrategy, DecisionStrategy, HiLo, MarginBettingStrategy, Strategy,
    TableState,
};
use crate::sim::game::table::BlackjackTableSim;
use blackjack_lib::{BlackjackGameError, BlackjackTable, Player};
use std::io::{self, Write};

pub mod player;
pub mod strategy;
pub mod table;

/// Struct that provides the functionality to simulate a game of blackjack using a specific counting strategy.
/// This struct saves all of the necessary data for reporting/logging the stats of the simulation as well.
pub struct BlackjackGameSim<S: Strategy> {
    table: BlackjackTableSim,
    player: PlayerSim<S>,
    min_bet: u32,
    num_hands: u32,
    total_wins: i32,
    total_pushes: i32,
    total_losses: i32,
    total_winnings: f32,
    number_of_player_blackjacks: i32,
    ended_early: bool,
}

impl<S: Strategy> BlackjackGameSim<S> {
    /// Associated method for building a new blackjack game.
    /// `table` is the `BlackjackTableSim` struct that will be used to simulate the blackjack logic,
    /// `player` is the `PlayerSim<S>` struct used to simulate a specific counting strategy during the simulation.
    /// `num_hands` is the number of hands that will be simulated during a single call to `self.run()`,
    /// the simulation will end in max `num_hands` and will only end sooner if the `player` runs out of funds sooner.
    /// `min_bet` decides what the minimum bet should be at the table.
    fn new(
        table: BlackjackTableSim,
        player: PlayerSim<S>,
        num_hands: u32,
        min_bet: u32,
    ) -> BlackjackGameSim<S> {
        BlackjackGameSim {
            table,
            player,
            min_bet,
            num_hands,
            total_wins: 0,
            total_pushes: 0,
            total_losses: 0,
            total_winnings: 0.0,
            number_of_player_blackjacks: 0,
            ended_early: false,
        }
    }

    /// Method that runs the blackjack simulation the number of times specified during object creation.
    fn run(&mut self) -> Result<(), BlackjackGameError> {
        for _i in 0..self.num_hands {
            // Check if player can continue
            if !self.player.continue_play(self.min_bet) {
                self.ended_early = true;
                break;
            }
            // Get bet from player
            let bet = match self.player.bet() {
                Ok(b) if b >= self.min_bet => b,
                Ok(_) => {
                    // eprintln!("error: player cannot bet less than the minimum of {}", self.min_bet);
                    return Err(BlackjackGameError::new(
                        "player tried to bet less than table minimum".to_string(),
                    ));
                }
                Err(e) => {
                    // eprintln!("error: {e}")
                    return Err(e);
                }
            };

            // Have player place bet
            self.player.place_bet(bet as f32);

            // Deal hand
            self.table.deal_hand(&mut self.player);

            // Let player decide options until they are no longer able to
            while !self.player.turn_is_over() {
                // Get the chosen option from the player, return if it is an error
                let options = self.player.get_playing_options();
                let decision = self
                    .player
                    .decide_option(self.table.dealers_face_up_card())?;
                // Play the given option, return an error if it fails
                self.table
                    .play_option(&mut self.player, options, decision)?;
            }

            // Finish the hand
            self.table.finish_hand(&mut self.player);

            // Log the data from the game
            if let Some((wins, pushes, losses, winnings)) = self.table.hand_log {
                self.total_wins += wins;
                self.total_pushes += pushes;
                self.total_losses += losses;
                self.total_winnings += winnings;
            }

            // Reset both player and table for another hand
            self.player.reset();
            self.table.reset();
        }

        Ok(())
    }

    fn write_stats(&self, destination: Write) -> io::Result<()> {
        const width: usize = 80;
        const text_width: usize = "number of player blackjacks:".len() + 20;
        const numeric_width: usize = width - text_width;

        println!("{}", "-".repeat(width));
        println!("{:-^width$}", "stats");
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "total wins:", self.total_wins
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "total pushes:", self.pushes
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "total losses:", self.losses
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "total winnings:", self.total_winnings
        );
        println!(
            "{:<text_width$}{:>numeric_width$.2}",
            "players final balance:",
            self.player.balance()
        );
        println!(
            "{:<text_width$}{:numeric_width$}",
            "number of player blackjacks:", self.number_of_player_blackjacks
        );
        println!("{}", "-".repeat(width));
    }
}
