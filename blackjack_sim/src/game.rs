//! Module that focuses on the simulation of a single game of blackjack. In otherwords,
//!  this module provides all the functionality needed to test a single game of blackjack for a given counting strategy.

pub mod player;
pub mod strategy;
pub mod table;
pub mod prelude {
    pub use super::BlackjackGameSim;
    pub use crate::game::player::PlayerSim;
    pub use crate::game::strategy;
    pub use crate::game::table::BlackjackTableSim;
    pub use blackjack_lib::{BlackjackGameError, BlackjackTable, Card, Player, RANKS, SUITS};
    pub use std::io::{self, Write};
    // pub use BlackjackGameSim;
}

pub use prelude::*;
use rand::{self, Rng};
use std::sync::Arc;
use strategy::Strategy;

use self::strategy::{BettingStrategy, CountingStrategy, DecisionStrategy};

/// A struct to implement a thread safe deck of cards
pub struct DeckSim {
    cards: Vec<Arc<Card>>,
    n_decks: usize,
    deck_pos: usize,
    shuffle_flag_pos: usize,
    pub shuffle_flag: bool,
}

/// A struct to represent a deck of cards, is basically a collection of card structs that implements some specific logic related to a game of blackjack
impl DeckSim {
    /// An associated function that aids in the building of a deck of cards
    fn build_card_deck(n_decks: usize) -> Vec<Arc<Card>> {
        let mut cards = Vec::with_capacity(n_decks * 52);
        for _i in 0..n_decks {
            for suit in SUITS {
                for rank in RANKS {
                    cards.push(Arc::new(Card::new(suit, rank)));
                }
            }
        }
        cards
    }

    /// Creates and returns a new Deck struct
    pub fn new(n_decks: usize) -> DeckSim {
        assert!(n_decks > 0, "Cannot have a deck with zero cards");
        let cards = Self::build_card_deck(n_decks);
        let n_cards = cards.len();
        let shuffle_flag_pos = f32::floor(((n_cards - 1) as f32) * 0.8) as usize;

        DeckSim {
            cards,
            n_decks,
            deck_pos: 0,
            shuffle_flag_pos,
            shuffle_flag: true,
        }
    }

    /// Shuffles the deck of cards to simulate the random behavior of a shuffled deck of cards
    pub fn shuffle(&mut self, n_shuffles: u32) {
        assert!(n_shuffles > 0);
        let mut rng = rand::thread_rng();
        for _i in 0..n_shuffles {
            for j in 0..self.cards.len() {
                let random_idx = rng.gen_range(0..self.cards.len());
                self.cards.swap(j, random_idx);
            }
        }
        self.deck_pos = 0;
        self.shuffle_flag = false;
    }

    /// Returns the next card, i.e. the card that is at the top of the deck of cards
    pub fn get_next_card(&mut self) -> Option<Arc<Card>> {
        if self.deck_pos < self.cards.len() {
            let next_card = Some(Arc::clone(&self.cards[self.deck_pos]));
            self.deck_pos += 1;
            if self.deck_pos == self.shuffle_flag_pos {
                self.shuffle_flag = true;
            }
            return next_card;
        }

        None
    }
}

/// Struct that provides the functionality to simulate a game of blackjack using a specific counting strategy.
/// This struct saves all of the necessary data for reporting/logging the stats of the simulation as well.
pub struct BlackjackGameSim<S: Strategy> {
    table: BlackjackTableSim,
    player: PlayerSim<S>,
    min_bet: u32,
    num_hands: u32,
    pub total_wins: i32,
    pub total_pushes: i32,
    pub total_losses: i32,
    pub total_winnings: f32,
    pub num_player_blackjacks: i32,
    pub ended_early: bool,
}

impl<S: Strategy> BlackjackGameSim<S> {
    /// Associated method for building a new blackjack game.
    /// `table` is the `BlackjackTableSim` struct that will be used to simulate the blackjack logic,
    /// `player` is the `PlayerSim<S>` struct used to simulate a specific counting strategy during the simulation.
    /// `num_hands` is the number of hands that will be simulated during a single call to `self.run()`,
    /// the simulation will end in max `num_hands` and will only end sooner if the `player` runs out of funds sooner.
    /// `min_bet` decides what the minimum bet should be at the table.
    pub fn new(
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
            num_player_blackjacks: 0,
            ended_early: false,
        }
    }

    /// Method that runs the blackjack simulation the number of times specified during object creation.
    pub fn run(&mut self) -> Result<(), BlackjackGameError> {
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
                // let options = self.player.get_playing_options();
                let decision = self
                    .player
                    .decide_option(self.table.dealers_face_up_card())?;
                // Play the given option, return an error if it fails
                self.table.play_option(&mut self.player, decision)?;
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

            self.num_player_blackjacks += self.table.num_player_blackjacks;

            // Reset both player and table for another hand
            self.player.reset();
            self.table.reset();
        }

        Ok(())
    }

    /// Writes the stats the stats currently recorded to the given writer.
    // TODO: allow an arbitrary writer to be passed in
    pub fn display_stats(&self) {
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
            "total pushes:", self.total_pushes
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "total losses:", self.total_losses
        );
        println!(
            "{:<text_width$}{:>numeric_width$.2}",
            "total winnings:", self.total_winnings
        );
        println!(
            "{:<text_width$}{:>numeric_width$.2}",
            "players final balance:",
            self.player.balance()
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "number of player blackjacks:", self.num_player_blackjacks
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "ended early:", self.ended_early
        );
        println!("{}", "-".repeat(width));
    }

    pub fn reset(&mut self, new_table_balance: f32, new_player_balance: f32) {
        self.table.balance = new_table_balance;
        self.player.balance = new_player_balance;
        self.num_player_blackjacks = 0;
        self.table.num_player_blackjacks = 0;
        self.total_wins = 0;
        self.total_pushes = 0;
        self.total_losses = 0;
        self.total_winnings = 0.0;
        self.ended_early = false;
    }

    pub fn label(&self) -> String {
        self.player.label()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use strategy::{
        BasicStrategy, BettingStrategy, DecisionStrategy, HiLo, MarginBettingStrategy,
        PlayerStrategy, Strategy, TableState, WongHalves,
    };
    #[test]
    fn test_game() {
        const MIN_BET: u32 = 5;
        const NUM_HANDS: u32 = 300;
        const NUM_DECKS: u32 = 6;
        let counting_strategy = HiLo::new(NUM_DECKS);
        let decision_strategy = BasicStrategy::new();
        let betting_strategy = MarginBettingStrategy::new(3.0, MIN_BET);
        let strategy = PlayerStrategy::new(counting_strategy, decision_strategy, betting_strategy);
        let player = PlayerSim::new(500.0, strategy, true);
        // let table = <BlackjackTableSim as BlackjackTable<
        //     PlayerSim<PlayerStrategy<HiLo, BasicStrategy, MarginBettingStrategy>>,
        // >>::new(f32::MAX, 6, 7);
        let table = BlackjackTableSim::new(f32::MAX, 6, 7, false, false);
        let mut game = BlackjackGameSim::new(table, player, NUM_HANDS, MIN_BET);

        if let Err(e) = game.run() {
            panic!("error occured {e}");
        }

        game.display_stats();

        assert!(true);
    }
}
