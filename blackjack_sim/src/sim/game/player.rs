use crate::sim::game::strategy::CountingStrategy;
use crate::sim::game::table::TableState;
use blackjack_lib::{Card, Player};
use std::collections::HashMap;
use std::rc::Rc;

struct PlayerSim<S: CountingStrategy> {
    hand: Vec<Vec<Rc<Card>>>,
    hand_values: Vec<Vec<u8>>,
    bets: Vec<u32>,
    hand_idx: usize,
    balance: f64,
    strategy: S,
}

impl<S: CountingStrategy> PlayerSim<S> {
    /// Associated function to create a new `PlayerSim` struct.
    pub fn new(starting_balance: f64, strategy: S) -> PlayerSim<S> {
        PlayerSim {
            hand: vec![vec![]],
            hand_values: vec![vec![]],
            bets: vec![],
            hand_idx: 0,
            balance: starting_balance,
            strategy,
        }
    }

    /// Function to simluate the placing of a bet, updates the `PlayerSim`'s balance and bets
    pub fn place_bet(&mut self) {
        let bet = u32::min(self.strategy.bet(), self.balance as u32);
        self.balance -= bet as f64;
        self.bets.push(bet);
    }

    /// Method to receive a card
    pub fn receive_card(&mut self, card: Rc<Card>) {
        // Push new card onto current hand
        self.hand[self.hand_idx].push(Rc::clone(&card));

        // Update the value of the current hand
        let card_val = card.get_card_value();
        if self.hand_values[self.hand_idx].is_empty() {
            self.hand_values[self.hand_idx].push(card_val);
        } else {
            self.hand_values[self.hand_idx][0] += card_val;
            if self.hand_values[self.hand_idx].len() == 2 {
                self.hand_values[self.hand_idx][1] += card_val;
            }
        }

        // Check if we need to add an alternative hand value to the hand
        if self.hand_values[self.hand_idx].len() == 1
            && self.hand_values[self.hand_idx][0] <= 11
            && card_val == 1
        {
            let alt_val = self.hand_values[self.hand_idx][0] + 10;
            self.hand_values[self.hand_idx].push(alt_val);
        }
    }

    /// Public method for producing the possible options a player can choose to player their current hand
    pub fn get_playing_options(&self) -> HashMap<i32, String> {
        let options = HashMap::new();
        options.insert(1i32, "stand".to_string());
        options.insert(2, "hit".to_string());
        let mut option_count = 3;
        if self.can_split() {
            options.insert(option_count, "split".to_string());
            option_count += 1;
        }
        if self.can_double_down() {
            options.insert(option_count, "double down".to_string());
        }

        options
    }

    //TODO: implement can_split(), can_double_down(), etc...
}
