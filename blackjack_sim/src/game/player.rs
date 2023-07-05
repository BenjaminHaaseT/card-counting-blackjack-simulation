use crate::game::strategy::TableState;
use crate::game::strategy::{BettingStrategy, CountingStrategy, DecisionStrategy, Strategy};
use blackjack_lib::{compute_optimal_hand, BlackjackGameError, Card, Player};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::sync::Arc;

/// Struct for a simulated player
pub struct PlayerSim<S: Strategy> {
    hand: Vec<Vec<Arc<Card>>>,
    hand_values: Vec<Vec<u8>>,
    pub bets: Vec<u32>,
    pub bets_log: HashMap<usize, f32>,
    hand_idx: usize,
    pub balance: f32,
    strategy: S,
    surrender_flag: bool,
}

impl<S: Strategy> PlayerSim<S> {
    /// Associated function to create a new `PlayerSim` struct.
    pub fn new(starting_balance: f32, strategy: S, surrender_flag: bool) -> PlayerSim<S> {
        PlayerSim {
            hand: vec![vec![]],
            hand_values: vec![vec![]],
            bets: vec![],
            bets_log: HashMap::new(),
            hand_idx: 0,
            balance: starting_balance,
            strategy,
            surrender_flag,
        }
    }

    /// Method for determining whether or not the players turn is over
    pub fn turn_is_over(&self) -> bool {
        self.hand_idx == self.hand.len()
    }

    /// Method for determining whether the player can continue to play or not
    pub fn continue_play(&self, min_bet: u32) -> bool {
        (self.balance as u32) >= min_bet
    }

    /// Getter method for the players current bet
    pub fn get_current_bet(&self) -> u32 {
        self.bets[self.hand_idx]
    }

    /// Getter method for the players current balance.
    pub fn balance(&self) -> f32 {
        self.balance
    }

    /// Function for getting an initial bet
    pub fn bet(&mut self) -> Result<u32, BlackjackGameError> {
        let bet_state = self.strategy.get_current_bet_state(self.balance);
        let bet = self.strategy.bet(bet_state);
        if bet == 0 {
            return Err(BlackjackGameError::new("out of funds".to_string()));
        }

        Ok(bet)
    }

    /// Function to simluate the placing of a bet, updates the `PlayerSim`'s balance and bets
    /// Assumes the logic for checking whether or not the bet is valid has already been executed.
    pub fn place_bet(&mut self, bet: f32) {
        self.balance -= bet;
        self.bets.push(bet as u32);
    }

    /// Method to receive a card, updates the state of the `Player`
    pub fn receive_card(&mut self, card: Arc<Card>) {
        // Push new card onto current hand
        self.hand[self.hand_idx].push(Arc::clone(&card));

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
    pub fn get_playing_options(&self, dealers_up_card: Arc<Card>) -> HashSet<String> {
        let mut options = HashSet::new();
        options.insert("stand".to_string());
        options.insert("hit".to_string());
        if self.surrender_flag && self.can_surrender(dealers_up_card) {
            options.insert("surrender".to_string());
        }
        if self.can_split() {
            options.insert("split".to_string());
        }
        if self.can_double_down() {
            options.insert("double down".to_string());
        }

        options
    }

    /// Method to check whether or not the player can surrender

    /// Returns a boolean, true if the `PlayerSim` instance can split their hand, false otherwise.
    fn can_split(&self) -> bool {
        self.hand.len() < 4
            && self.hand[self.hand_idx].len() == 2
            && self.hand[self.hand_idx][0].rank == self.hand[self.hand_idx][1].rank
            && (self.bets[self.hand_idx] as f32) <= self.balance
    }

    /// Returns a boolean, true if the `PlayerSim` can double down, false otherwise.
    fn can_double_down(&self) -> bool {
        self.hand_idx == 0
            && (self.bets[self.hand_idx] as f32) <= self.balance
            && if self.hand_values[self.hand_idx].len() == 2 {
                self.hand_values[self.hand_idx][0] == 9
                    || self.hand_values[self.hand_idx][1] == 9
                    || self.hand_values[self.hand_idx][0] == 10
                    || self.hand_values[self.hand_idx][1] == 10
                    || self.hand_values[self.hand_idx][0] == 11
                    || self.hand_values[self.hand_idx][1] == 11
            } else {
                self.hand_values[self.hand_idx][0] == 9
                    || self.hand_values[self.hand_idx][0] == 10
                    || self.hand_values[self.hand_idx][0] == 11
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
    pub fn update_strategy<'a, I: IntoIterator<Item = &'a Arc<Card>>>(&mut self, cards: I) {
        for card in cards {
            self.strategy.update(Arc::clone(card));
        }
    }

    /// Method to stand on a current hand, increases the value of `self.hand_idx` to represent
    /// that the current hand at position `self.hand_idx` is now over.
    pub fn stand(&mut self) {
        self.hand_idx += 1;
    }

    /// Method that implements the logic for surrendering. Will return half the current bet that the player has on the table.
    pub fn can_surrender(&self, dealers_up_card: Arc<Card>) -> bool {
        self.hand_idx == 0
            && self.hand_values[self.hand_idx].len() == 2
            && (dealers_up_card.val == 1 || dealers_up_card.val == 10)
    }

    /// Method to update the state of the players hand when a push occurs.
    /// Change the bet of the current hand to 0, update the balance and return 0.
    pub fn push_current_hand(&mut self) {
        let bet = self.bets[self.hand_idx];
        self.balance += bet as f32;
        self.bets[self.hand_idx] = 0;
        self.bets_log.insert(self.hand_idx, 0.0);
        self.stand();
    }

    /// Method to update the state of the players hand when a bet is lost.
    /// Change the bet of the current hand to 0, and return the value negative value of the bet to indicate a loss occured
    pub fn lose_current_hand(&mut self) {
        let bet = -(self.bets[self.hand_idx] as i32);
        self.bets[self.hand_idx] = 0;
        self.bets_log.insert(self.hand_idx, bet as f32);
        self.stand();
    }

    /// Method for updating the internal bookeeping of won/lost bets when the player gets a blackjack
    pub fn blackjack(&mut self, winnings: f32) {
        let bet = self.bets[self.hand_idx] as f32;
        self.balance += bet;
        self.bets[self.hand_idx] = 0;
        self.bets_log.insert(self.hand_idx, winnings);
        self.stand();
    }

    /// Method to update the `PlayerSim` structs bets_log
    pub fn win_hand(&mut self, hand: usize, bet: u32) {
        self.balance += bet as f32;
        self.bets_log.insert(hand, bet as f32);
    }

    /// Method to update the `PlayerSim` structs bets_log
    pub fn lose_hand(&mut self, hand: usize, bet: u32) {
        self.bets_log.insert(hand, -(bet as f32));
    }

    /// Method to update the `PlayerSim` structs bets_log
    pub fn push_hand(&mut self, hand: usize, bet: u32) {
        self.balance += bet as f32;
        self.bets_log.insert(hand, 0.0);
    }

    /// Method for receiving winnings
    pub fn collect_winnings(&mut self, winnings: f32) {
        self.balance += winnings;
    }

    /// Method that returns a boolean, true if the player has busted on their current hand false if the current hand has not busted.
    /// Will panic if `self.hand_idx` > `self.hand.len()`
    pub fn busted(&self) -> bool {
        if self.hand_values[self.hand_idx].len() == 2 {
            self.hand_values[self.hand_idx][0] > 21 && self.hand_values[self.hand_idx][1] > 21
        } else {
            self.hand_values[self.hand_idx][0] > 21
        }
    }

    /// Method that will execute the logic for surrendering
    pub fn surrender(&mut self) -> f32 {
        let bet = self.bets[self.hand_idx] as f32;
        self.bets[self.hand_idx] = 0;
        self.balance += bet / 2.0;
        self.stand();
        bet / 2.0
    }

    /// Method that implements the logic for doubling down. Will panic if `self.balance` is not high enough to place the bet.
    pub fn double_down(&mut self) {
        assert!(self.bets[self.hand_idx] as f32 <= self.balance);
        self.balance -= self.bets[self.hand_idx] as f32;
        self.bets[self.hand_idx] *= 2;
    }

    /// Method that implements the logic for splitting.
    /// Will panic if `self.balance` is not high enough to place the bet or if the current hand is empty().
    pub fn split(&mut self, card1: Arc<Card>, card2: Arc<Card>) {
        assert!(self.bets[self.hand_idx] as f32 <= self.balance);
        // Get current bet and duplicate it for the new hand
        let cur_bet = self.bets[self.hand_idx];
        self.bets.insert(self.hand_idx + 1, cur_bet);

        // Split the current hand, and start with empty hand values
        let new_hand_start = self.hand[self.hand_idx].pop().unwrap();
        self.hand.insert(self.hand_idx + 1, vec![new_hand_start]);
        self.hand_values[self.hand_idx].clear();
        self.hand_values.insert(self.hand_idx + 1, vec![]);

        // receive a new card for each hand
        self.hand[self.hand_idx].push(card1);
        self.hand[self.hand_idx + 1].push(card2);

        // Now recompute the hand values
        let hand1: u8 = self.hand[self.hand_idx].iter().map(|c| c.val).sum();
        self.hand_values[self.hand_idx].push(hand1);
        if hand1 <= 11
            && (self.hand[self.hand_idx][0].rank == "A" || self.hand[self.hand_idx][1].rank == "A")
        {
            self.hand_values[self.hand_idx].push(hand1 + 10);
        }

        let hand2: u8 = self.hand[self.hand_idx + 1].iter().map(|c| c.val).sum();
        self.hand_values[self.hand_idx + 1].push(hand2);
        if hand2 <= 11
            && (self.hand[self.hand_idx + 1][0].rank == "A"
                || self.hand[self.hand_idx + 1][1].rank == "A")
        {
            self.hand_values[self.hand_idx + 1].push(hand2 + 10);
        }
    }

    pub fn get_optimal_hands(&mut self) -> Option<Vec<(usize, u32, u8)>> {
        let res = self
            .bets
            .iter()
            .zip(self.hand_values.iter())
            .enumerate()
            .filter(|(_i, (bet, _hand))| **bet > 0)
            .map(|(i, (bet, hand))| (i, *bet, compute_optimal_hand(hand)))
            .collect::<Vec<(usize, u32, u8)>>();
        if !res.is_empty() {
            Some(res)
        } else {
            None
        }
    }

    /// Method for returning a valid option given the state of the table
    pub fn decide_option(&self, dealers_up_card: Arc<Card>) -> Result<String, BlackjackGameError> {
        let options = self.get_playing_options(dealers_up_card.clone());
        let current_state = self.strategy.get_current_table_state(
            &self.hand[self.hand_idx],
            &self.hand_values[self.hand_idx],
            self.get_current_bet(),
            self.balance,
            dealers_up_card,
        );

        self.strategy.decide_option(current_state, options)
    }

    /// Method to get a string that describes the players strategy
    pub fn label(&self) -> String {
        self.strategy.label()
    }

    pub fn reset_strategy(&mut self) {
        self.strategy.reset();
    }

    pub fn reset(&mut self) {
        self.hand = vec![vec![]];
        self.hand_values = vec![vec![]];
        self.bets.clear();
        self.bets_log.clear();
        self.hand_idx = 0;
    }
}

impl<S: Strategy> Player for PlayerSim<S> {}

impl<S: Strategy + Display> Display for PlayerSim<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<21}{:?}\n\
                   {:<21}{:?}\n\
                   {:<21}{:?}\n\
                   {:<21}{:?}\n\
                   {:<21}{}\n\
                   {:<21}${:.2}\n\
                   {}",
            "hand:",
            self.hand,
            "hand_value:",
            self.hand_values,
            "bets:",
            self.bets,
            "bets_log:",
            self.bets_log,
            "hand_idx:",
            self.hand_idx,
            "balance:",
            self.balance,
            self.strategy,
        )
    }
}
