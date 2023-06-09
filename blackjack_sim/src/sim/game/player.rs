use crate::sim::game::strategy::CountingStrategy;
use crate::sim::game::table::TableState;
use blackjack_lib::{BlackjackGameError, Card, Player};
use std::collections::HashMap;
use std::rc::Rc;

pub struct PlayerSim<S: CountingStrategy> {
    hand: Vec<Vec<Rc<Card>>>,
    hand_values: Vec<Vec<u8>>,
    pub bets: Vec<u32>,
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
    /// TODO: Create specific out of funds error to end the game early, do so in the main library
    pub fn place_bet(&mut self) -> Result<(), BlackjackGameError> {
        if self.balance == 0.0 {
            return Err(BlackjackGameError::new(String::from("out of funds")));
        }
        let bet = u32::min(self.strategy.bet(), self.balance as u32);
        self.balance -= bet as f64;
        self.bets.push(bet);
        Ok(())
    }

    /// Method to receive a card, updates the state of the `Player`
    pub fn receive_card(&mut self, card: Rc<Card>) {
        // Push new card onto current hand
        self.hand[self.hand_idx].push(Rc::clone(&card));

        // Update the value of the current hand
        let card_val = card.val;
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

    /// Method to return a formatted version of all the players hand(s)
    pub fn formatted_hand_values(&self) -> String {
        self.hand_values
            .iter()
            .map(|hand| {
                if hand.len() == 2 {
                    if hand[0] <= 21 && hand[1] <= 21 {
                        format!("{}/{}", hand[0], hand[1])
                    } else {
                        format!("{}", u8::min(hand[0], hand[1]))
                    }
                } else {
                    format!("{}", hand[0])
                }
            })
            .collect::<Vec<String>>()
            .join(", ")
    }

    /// Public method for producing the possible options a player can choose to player their current hand
    pub fn get_playing_options(&self) -> HashMap<i32, String> {
        let mut options = HashMap::new();
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

    /// Returns a boolean, true if the `PlayerSim` instance can split their hand, false otherwise.
    fn can_split(&self) -> bool {
        self.hand.len() < 4 && self.hand[self.hand_idx][0].rank == self.hand[self.hand_idx][1].rank
    }

    /// Returns a boolean, true if the `PlayerSim` can double down, false otherwise.
    fn can_double_down(&self) -> bool {
        self.hand_idx == 0
            && if self.hand_values[self.hand_idx].len() == 2 {
                self.hand_values[self.hand_idx][0] == 9
                    || self.hand_values[self.hand_idx][0] == 9
                    || self.hand_values[self.hand_idx][1] == 10
                    || self.hand_values[self.hand_idx][0] == 11
                    || self.hand_values[self.hand_idx][1] == 11
            } else {
                self.hand_values[self.hand_idx][0] == 9
                    || self.hand_values[self.hand_idx][1] == 9
                    || self.hand_values[self.hand_idx][0] == 10
                    || self.hand_values[self.hand_idx][1] == 10
                    || self.hand_values[self.hand_idx][0] == 11
                    || self.hand_values[self.hand_idx][1] == 11
            }
    }

    /// Returns a boolean representing whether the player has a blackjack or not.
    pub fn has_blackjack(&self) -> bool {
        self.hand_idx == 0
            && self.hand[self.hand_idx].len() == 2
            && ((self.hand[self.hand_idx][0].val == 10 && self.hand[self.hand_idx][1].rank == "A")
                || (self.hand[self.hand_idx][0].rank == "A"
                    && self.hand[self.hand_idx][1].val == 10))
    }

    /// Method that acts as a wrapper for accessing the `PlayerSim` struct instances `strategy`.
    pub fn update_strategy(&mut self, card: Rc<Card>) {
        self.strategy.update(card);
    }

    /// Method to stand on a current hand, increases the value of `self.hand_idx` to represent
    /// that the current hand at position `self.hand_idx` is now over.
    pub fn stand(&mut self) {
        self.hand_idx += 1;
    }

    /// Method to update the state of the players hand when a push occurs.
    /// Change the bet of the current hand to 0, update the balance and return 0.
    pub fn push(&mut self) -> i32 {
        let bet = self.bets[self.hand_idx];
        self.balance += bet as f64;
        self.bets[self.hand_idx] = 0;
        self.stand();
        0
    }

    /// Method to update the state of the players hand when a bet is lost.
    /// Change the bet of the current hand to 0, and return the value negative value of the bet to indicate a loss occured
    pub fn lose(&mut self) -> i32 {
        let bet = -(self.bets[self.hand_idx] as i32);
        self.bets[self.hand_idx] = 0;
        self.stand();
        bet
    }
}

impl<S: CountingStrategy> Player for PlayerSim<S> {}
