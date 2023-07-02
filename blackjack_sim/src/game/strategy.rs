// use crate::sim::game::player::PlayerSimState;
// use crate::sim::game::table::TableState;
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
// use std::fmt::Display;
use std::sync::Arc;

pub mod prelude {
    pub use super::*;
    pub use blackjack_lib::console::player;
    pub use blackjack_lib::{BlackjackGameError, Card};
}

pub use prelude::*;

/// Struct for encapsulating all the necessary information for a struct that implements `Strategy` to make a decsion and or place a bet.
/// Meant as a conveince for reducing the number of arguments passed to methods to a struct that implements `Strategy`.
pub struct TableState<'a> {
    hand: &'a Vec<Arc<Card>>,
    hand_value: &'a Vec<u8>,
    bet: u32,
    balance: f32,
    running_count: f32,
    true_count: f32,
    num_decks: u32,
    dealers_up_card: Arc<Card>,
}

impl<'a> TableState<'a> {
    fn new(
        hand: &'a Vec<Arc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        running_count: f32,
        true_count: f32,
        num_decks: u32,
        dealers_up_card: Arc<Card>,
    ) -> TableState<'a> {
        TableState {
            hand,
            hand_value,
            bet,
            balance,
            running_count,
            true_count,
            num_decks,
            dealers_up_card,
        }
    }
}

/// Trait for a generic decision strategy. Has only one required method `decide_option()`,
/// the method that will take in the current state of the table i.e. the dealers face upcard and the state of the player and return a decsion.
/// Allows for composibility and customizability for specific card counting strategies.
/// The implementer may implement a custom decision strategy based on the state of the table
pub trait DecisionStrategy {
    /// Method that takes `self` by reference, `decision_state` representing the state of the table and the count,
    /// and `options` a `HashSet<String>` representing the valid options to a player may choose to play their current hand.
    /// This method returns a string representing the most optimal way to play the current hand given its inputs
    fn decide_option<'a>(
        &self,
        decision_state: TableState<'a>,
        options: HashSet<String>,
    ) -> Result<String, BlackjackGameError>;
}

/// Trait for a generic betting strategy. Allows greater composibility and customizeability for any playing strategy.
pub trait BettingStrategy {
    fn bet(&self, running_count: f32, true_count: f32, balance: f32) -> u32;
}

/// Trait for a specific counting srategy. Can be implemented by any object that can be used to implement a counting strategy
pub trait CountingStrategy {
    fn new(num_decks: u32) -> Self;
    fn update(&mut self, card: Arc<Card>);
    fn get_current_table_state<'a>(
        &self,
        hand: &'a Vec<Arc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        dealers_up_card: Arc<Card>,
    ) -> TableState<'a>;
    fn reset(&mut self);
    fn running_count(&self) -> f32;
    fn true_count(&self) -> f32;
}

/// Struct that encapsulates the logic needed for a simple margin based betting strategy, i.e. for each positive value that the true count takes it will compute the bet as
/// `self.min_bet` * `self.margin` * true_count
pub struct MarginBettingStrategy {
    margin: f32,
    min_bet: u32,
}

impl MarginBettingStrategy {
    /// Associated method for returning a new `MarginBettingStrategy` struct
    pub fn new(margin: f32, min_bet: u32) -> MarginBettingStrategy {
        MarginBettingStrategy { margin, min_bet }
    }
}

impl BettingStrategy for MarginBettingStrategy {
    /// Returns the bet based on the true count, if the true count is greater than zero the product of the true count minimum bet and the margin is returned
    fn bet(&self, running_count: f32, true_count: f32, balance: f32) -> u32 {
        if true_count > 0.0 {
            let scalar = f32::ceil(true_count);
            u32::min(
                balance as u32,
                ((self.min_bet as f32) * scalar * self.margin) as u32,
            )
        } else {
            u32::min(balance as u32, self.min_bet)
        }
    }
}

/// A struct that implments the `DecisionStrategy` trait. Decides playing option according to strict basic strategy only.
/// The decision strategy only requires what knowing what the dealers face up card is and the players current cards.
pub struct BasicStrategy {
    hard_totals: HashMap<(u8, u8), String>,
    soft_totals: HashMap<(u8, u8), String>,
    pair_totals: HashMap<(u8, u8), String>,
    surrender: HashMap<(u8, u8), String>,
}

impl BasicStrategy {
    /// Associated method for populating the lookup tables used in basic strategy, intended to be a helper method.
    fn build_lookup_tables() -> (
        HashMap<(u8, u8), String>,
        HashMap<(u8, u8), String>,
        HashMap<(u8, u8), String>,
        HashMap<(u8, u8), String>,
    ) {
        // Populate hard_totals lookup table
        let mut hard_totals: HashMap<(u8, u8), String> = HashMap::new();
        for i in 2..=21 {
            for j in 1..=10 {
                let mut option = String::new();
                match i {
                    9 => match j {
                        3..=6 => option.push_str("double down"),
                        _ => option.push_str("hit"),
                    },
                    10 => match j {
                        2..=9 => option.push_str("double down"),
                        _ => option.push_str("hit"),
                    },
                    11 => option.push_str("double down"),
                    12 => match j {
                        1..=3 | 7..=10 => option.push_str("hit"),
                        _ => option.push_str("stand"),
                    },
                    13..=16 => match j {
                        2..=6 => option.push_str("stand"),
                        _ => option.push_str("hit"),
                    },
                    17..=21 => option.push_str("stand"),
                    _ => option.push_str("hit"),
                }
                hard_totals.insert((i, j), option);
            }
        }

        // Populate soft totals i.e. hand that contains an ace
        let mut soft_totals: HashMap<(u8, u8), String> = HashMap::new();
        for i in 3..=10 {
            for j in 1..=10 {
                let mut option = String::new();
                match i {
                    3..=7 => option.push_str("hit"),
                    8 => match j {
                        2..=6 => option.push_str("double down"),
                        7 | 8 => option.push_str("stand"),
                        _ => option.push_str("hit"),
                    },
                    9 => match j {
                        6 => option.push_str("double down"),
                        _ => option.push_str("stand"),
                    },
                    _ => option.push_str("stand"),
                }

                soft_totals.insert((i, j), option);
            }
        }

        // Populate pair totals
        let mut pair_totals: HashMap<(u8, u8), String> = HashMap::new();
        for i in (2..=20).step_by(2) {
            for j in 1..=10 {
                let mut option = String::new();
                match i {
                    2 => option.push_str("split"),
                    4 | 6 => match j {
                        2..=7 => option.push_str("split"),
                        _ => option.push_str("default"),
                    },
                    8 => match j {
                        5 | 6 => option.push_str("split"),
                        _ => option.push_str("default"),
                    },
                    10 => option.push_str("default"),
                    12 => match j {
                        2..=6 => option.push_str("split"),
                        _ => option.push_str("default"),
                    },
                    14 => match j {
                        2..=7 => option.push_str("split"),
                        _ => option.push_str("default"),
                    },
                    16 => option.push_str("split"),
                    18 => match j {
                        2..=6 | 8 | 9 => option.push_str("split"),
                        _ => option.push_str("default"),
                    },
                    20 => option.push_str("default"),
                    _ => todo!(),
                }

                pair_totals.insert((i, j), option);
            }
        }

        // Populate surrender options if available or necessary
        let mut surrender: HashMap<(u8, u8), String> = HashMap::new();
        surrender.insert((15, 10), "surrender".to_string());
        surrender.insert((16, 9), "surrender".to_string());
        surrender.insert((16, 10), "surrender".to_string());
        surrender.insert((16, 1), "surrender".to_string());

        (hard_totals, soft_totals, pair_totals, surrender)
    }

    /// Associated method for creating a new `BasicStrategy` struct.
    pub fn new() -> BasicStrategy {
        let (hard_totals, soft_totals, pair_totals, surrender) =
            BasicStrategy::build_lookup_tables();

        BasicStrategy {
            hard_totals,
            soft_totals,
            pair_totals,
            surrender,
        }
    }
}

impl DecisionStrategy for BasicStrategy {
    /// Method for deciding how to play the current hand given the appropriate data
    fn decide_option<'a>(
        &self,
        decision_state: TableState<'a>,
        options: HashSet<String>,
    ) -> Result<String, BlackjackGameError> {
        let mut option = String::new();
        let dealers_card = decision_state.dealers_up_card.val;

        // First check if we should surrender or not
        if options.contains("surrender") {
            if let Some(o) = self
                .surrender
                .get(&(decision_state.hand_value[0], dealers_card))
            {
                option.push_str(o.as_str());
            }
        }

        if option.is_empty() && options.contains("split") {
            if let Some(o) = self
                .pair_totals
                .get(&(decision_state.hand_value[0], dealers_card))
            {
                if o == "split" {
                    option.push_str(o);
                }
            }
        }

        // Check if players hand is a soft total, if so default ot soft totals lookup table
        if option.is_empty()
            && decision_state.hand_value.len() == 2
            && decision_state.hand_value[1] <= 21
        {
            if let Some(opt) = self
                .soft_totals
                .get(&(decision_state.hand_value[0], dealers_card))
            {
                if options.contains(opt.as_str()) {
                    option.push_str(opt.as_str());
                } else if opt == "double down" && !options.contains("double down") {
                    option.push_str("hit");
                } else {
                    return Err(BlackjackGameError {
                        message: format!("option chosen: {}, not available for valid options {:?} with soft total of {}", opt, options, decision_state.hand_value[0])
                    });
                }
            }
        }

        if option.is_empty() {
            match self
                .hard_totals
                .get(&(decision_state.hand_value[0], dealers_card))
            {
                Some(o) if options.contains(o.as_str()) => option.push_str(o.as_str()),
                Some(o) if o == "double down" && !options.contains("double down") => {
                    option.push_str("hit");
                }
                _ => {
                    return Err(BlackjackGameError {
                        message: "option {o} not a valid choice".to_string(),
                    })
                }
            }
        }

        if option.is_empty() {
            return Err(BlackjackGameError {
                message: "no valid option was selected".to_string(),
            });
        }

        Ok(option)
    }
}

pub struct HiLo {
    running_count: i32,
    true_count: f32,
    num_decks: u32,
    total_cards_counted: i32,
    lookup_table: HashMap<u8, i32>,
}

impl CountingStrategy for HiLo {
    /// Associated Method for building a new HiLo counting object
    fn new(num_decks: u32) -> Self {
        // Initialize lookup table
        let mut lookup_table = HashMap::new();
        for i in 2..7 {
            lookup_table.insert(i, 1);
        }
        for i in 7..10 {
            lookup_table.insert(i, 0);
        }
        lookup_table.insert(1, -1);
        lookup_table.insert(10, -1);

        HiLo {
            running_count: 0,
            true_count: 0.0,
            num_decks,
            total_cards_counted: 0,
            lookup_table,
        }
    }

    fn update(&mut self, card: Arc<Card>) {
        self.running_count += self.lookup_table[&card.val];
        self.total_cards_counted += 1;
        let estimated_decks_counted =
            (self.num_decks as f32) - ((self.total_cards_counted as f32) / 52.0);
        self.true_count = (self.running_count as f32) / estimated_decks_counted;
    }

    fn get_current_table_state<'a>(
        &self,
        hand: &'a Vec<Arc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        dealers_up_card: Arc<Card>,
    ) -> TableState<'a> {
        TableState {
            hand,
            hand_value,
            bet,
            balance,
            running_count: self.running_count as f32,
            true_count: self.true_count,
            num_decks: self.num_decks,
            dealers_up_card,
        }
    }

    fn running_count(&self) -> f32 {
        self.running_count as f32
    }

    fn true_count(&self) -> f32 {
        self.true_count
    }

    fn reset(&mut self) {
        self.running_count = 0;
        self.total_cards_counted = 0;
        self.true_count = 0.0;
    }
}

impl Display for HiLo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = "total cards counted:".len();
        let num_width = f32::ceil(f32::log10(self.total_cards_counted as f32)) as usize;
        write!(
            f,
            "{:<width$}{:>num_width$}\n{:<width$}{:>num_width$}\n{:<width$}{:>num_width$.2}",
            "running count:",
            self.running_count,
            "total cards counted:",
            self.total_cards_counted,
            "true count",
            self.true_count,
        )
    }
}

/// A struct that implements the famous Wong Halves card counting strategy.
pub struct WongHalves {
    running_count: f32,
    true_count: f32,
    num_decks: u32,
    total_cards_counted: i32,
    lookup_table: HashMap<u8, f32>,
}

impl CountingStrategy for WongHalves {
    fn new(num_decks: u32) -> Self {
        // Build lookup table with card values counted according to Wong Halves counting strategy.
        let mut lookup_table = HashMap::new();
        lookup_table.insert(1, -1.0);
        lookup_table.insert(10, -1.0);
        lookup_table.insert(2, 0.5);
        lookup_table.insert(7, 0.5);
        lookup_table.insert(3, 1.0);
        lookup_table.insert(4, 1.0);
        lookup_table.insert(6, 1.0);
        lookup_table.insert(5, 1.5);
        lookup_table.insert(8, 0.0);
        lookup_table.insert(9, -0.5);

        WongHalves {
            running_count: 0.0,
            true_count: 0.0,
            num_decks,
            total_cards_counted: 0,
            lookup_table,
        }
    }

    fn get_current_table_state<'a>(
        &self,
        hand: &'a Vec<Arc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        dealers_up_card: Arc<Card>,
    ) -> TableState<'a> {
        TableState {
            hand,
            hand_value,
            bet,
            balance,
            running_count: self.running_count,
            true_count: self.true_count,
            num_decks: self.num_decks,
            dealers_up_card,
        }
    }

    fn update(&mut self, card: Arc<Card>) {
        self.running_count += self.lookup_table[&card.val];
        self.total_cards_counted += 1;
        let estimated_decks_counted =
            (self.num_decks as f32) - ((self.total_cards_counted as f32) / 52.0);
        self.true_count = self.running_count / estimated_decks_counted;
    }

    fn reset(&mut self) {
        self.running_count = 0.0;
        self.true_count = 0.0;
        self.total_cards_counted = 0;
    }

    fn running_count(&self) -> f32 {
        self.running_count
    }

    fn true_count(&self) -> f32 {
        self.true_count
    }
}

/// Struct that implements the popular Knockout card counting strategy. No need to compute a true count.
pub struct KO {
    running_count: i32,
    num_decks: u32,
    lookup_table: HashMap<u8, i32>,
}

impl CountingStrategy for KO {
    /// Associated method to build a new KO struct
    fn new(num_decks: u32) -> Self {
        let mut lookup_table = HashMap::new();
        for i in 2u8..=7 {
            lookup_table.insert(i, 1);
        }
        lookup_table.insert(8, 0);
        lookup_table.insert(9, 0);
        lookup_table.insert(1, -1);
        lookup_table.insert(10, -1);
        let running_count = 4 - 4 * (num_decks as i32);

        KO {
            running_count,
            num_decks,
            lookup_table,
        }
    }

    /// Update the count for the strategy. Since there is no need to compute true count, we only need to update the running count.
    fn update(&mut self, card: Arc<Card>) {
        self.running_count += self.lookup_table[&card.val];
    }

    /// Getter for the true count. Since the true count and running count are the same we only need to return the running count.
    fn true_count(&self) -> f32 {
        self.running_count as f32
    }

    /// Getter for the running count.
    fn running_count(&self) -> f32 {
        self.running_count as f32
    }

    /// Method that takes data about the current state of the table and returns a `TableState` object that holds all relevant information for a player to make a decision
    fn get_current_table_state<'a>(
        &self,
        hand: &'a Vec<Arc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        dealers_up_card: Arc<Card>,
    ) -> TableState<'a> {
        TableState {
            hand,
            hand_value,
            bet,
            balance,
            running_count: self.running_count as f32,
            true_count: self.running_count as f32,
            num_decks: self.num_decks,
            dealers_up_card,
        }
    }

    /// Reset the counting strategy. We only need to reset the running count to 4 - total number of decks * 4.
    fn reset(&mut self) {
        self.running_count = 4 - (self.num_decks as i32) * 4;
    }
}

/// A struct that implements the HiOpt1 counting method
pub struct HiOptI {
    running_count: i32,
    true_count: f32,
    num_decks: u32,
    total_cards_counted: i32,
    lookup_table: HashMap<u8, i32>,
}

impl CountingStrategy for HiOptI {
    fn new(num_decks: u32) -> Self {
        let mut lookup_table = HashMap::new();
        lookup_table.insert(2, 0);
        for i in 3..=6_u8 {
            lookup_table.insert(i, 1);
        }
        for i in 7..=9_u8 {
            lookup_table.insert(i, 0);
        }
        lookup_table.insert(1, 0);
        lookup_table.insert(10, -1);

        HiOptI {
            running_count: 0,
            true_count: 0.0,
            num_decks,
            total_cards_counted: 0,
            lookup_table,
        }
    }

    fn update(&mut self, card: Arc<Card>) {
        self.running_count += self.lookup_table[&card.val];
        self.total_cards_counted += 1;
        let estimated_decks_played =
            (self.num_decks as f32) - ((self.total_cards_counted as f32) / 52.0);
        self.true_count = (self.running_count as f32) / estimated_decks_played;
    }

    fn get_current_table_state<'a>(
        &self,
        hand: &'a Vec<Arc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        dealers_up_card: Arc<Card>,
    ) -> TableState<'a> {
        TableState {
            hand,
            hand_value,
            bet,
            balance,
            running_count: self.running_count as f32,
            true_count: self.true_count,
            num_decks: self.num_decks,
            dealers_up_card,
        }
    }

    fn running_count(&self) -> f32 {
        self.running_count as f32
    }

    fn true_count(&self) -> f32 {
        self.true_count
    }

    fn reset(&mut self) {
        self.running_count = 0;
        self.total_cards_counted = 0;
        self.true_count = 0.0;
    }
}

/// A struct that implements the HiOptII counting method
pub struct HiOptII {
    running_count: i32,
    true_count: f32,
    num_decks: u32,
    total_cards_counted: i32,
    lookup_table: HashMap<u8, i32>,
}

impl CountingStrategy for HiOptII {
    fn new(num_decks: u32) -> Self {
        let mut lookup_table = HashMap::new();
        lookup_table.insert(2, 1);
        lookup_table.insert(3, 1);
        lookup_table.insert(4, 2);
        lookup_table.insert(5, 2);
        lookup_table.insert(6, 1);
        lookup_table.insert(7, 1);
        lookup_table.insert(8, 0);
        lookup_table.insert(9, 0);
        lookup_table.insert(10, -2);
        lookup_table.insert(1, 0);

        HiOptII {
            running_count: 0,
            true_count: 0.0,
            num_decks,
            total_cards_counted: 0,
            lookup_table,
        }
    }

    fn update(&mut self, card: Arc<Card>) {
        self.running_count += self.lookup_table[&card.val];
        self.total_cards_counted += 1;
        let estimated_decks_played =
            (self.num_decks as f32) - ((self.total_cards_counted as f32) / 52.0);
        self.true_count = (self.running_count as f32) / estimated_decks_played;
    }

    fn get_current_table_state<'a>(
        &self,
        hand: &'a Vec<Arc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        dealers_up_card: Arc<Card>,
    ) -> TableState<'a> {
        TableState {
            hand,
            hand_value,
            bet,
            balance,
            running_count: self.running_count as f32,
            true_count: self.true_count,
            num_decks: self.num_decks,
            dealers_up_card,
        }
    }

    fn running_count(&self) -> f32 {
        self.running_count as f32
    }

    fn true_count(&self) -> f32 {
        self.true_count
    }

    fn reset(&mut self) {
        self.running_count = 0;
        self.total_cards_counted = 0;
        self.true_count = 0.0;
    }
}

/// A struct that implements Red Seven counting method
pub struct RedSeven {
    running_count: i32,
    true_count: f32,
    num_decks: u32,
    total_cards_counted: i32,
    lookup_table: HashMap<u8, i32>,
}

impl CountingStrategy for RedSeven {
    fn new(num_decks: u32) -> Self {
        let mut lookup_table = HashMap::new();
        for i in 2..=6_u8 {
            lookup_table.insert(i, -1);
        }
        for i in 8..=9_u8 {
            lookup_table.insert(i, 0);
        }
        lookup_table.insert(10, -1);
        lookup_table.insert(1, -1);

        RedSeven {
            running_count: 0,
            true_count: 0.0,
            num_decks,
            total_cards_counted: 0,
            lookup_table,
        }
    }

    fn update(&mut self, card: Arc<Card>) {
        let card_index = match self.lookup_table.get(&card.val) {
            Some(v) => *v,
            None => {
                if card.suit == "H" || card.suit == "D" {
                    1
                } else {
                    0
                }
            }
        };

        self.running_count += card_index;
        self.total_cards_counted += 1;
        let estimated_decks = (self.num_decks as f32) - ((self.total_cards_counted as f32) / 52.0);
        self.true_count = (self.running_count as f32) / estimated_decks;
    }

    fn get_current_table_state<'a>(
        &self,
        hand: &'a Vec<Arc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        dealers_up_card: Arc<Card>,
    ) -> TableState<'a> {
        TableState {
            hand,
            hand_value,
            bet,
            balance,
            running_count: self.running_count as f32,
            true_count: self.true_count,
            num_decks: self.num_decks,
            dealers_up_card,
        }
    }

    fn running_count(&self) -> f32 {
        self.running_count as f32
    }

    fn true_count(&self) -> f32 {
        self.true_count
    }

    fn reset(&mut self) {
        self.running_count = 0;
        self.true_count = 0.0;
        self.total_cards_counted = 0;
    }
}
/// A struct that encapsulates everything needed to implement a specific playing to test in a simulation.
#[derive(Debug)]
pub struct PlayerStrategy<C, D, B>
where
    C: CountingStrategy,
    D: DecisionStrategy,
    B: BettingStrategy,
{
    counting_strategy: C,
    decision_strategy: D,
    betting_strategy: B,
}

impl<C, D, B> PlayerStrategy<C, D, B>
where
    C: CountingStrategy,
    D: DecisionStrategy,
    B: BettingStrategy,
{
    pub fn new(counting_strategy: C, decision_strategy: D, betting_strategy: B) -> Self {
        PlayerStrategy {
            counting_strategy,
            decision_strategy,
            betting_strategy,
        }
    }
}

impl<C, D, B> Display for PlayerStrategy<C, D, B>
where
    C: CountingStrategy + Display,
    D: DecisionStrategy,
    B: BettingStrategy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.counting_strategy)
    }
}

impl<C, D, B> Strategy for PlayerStrategy<C, D, B>
where
    C: CountingStrategy,
    D: DecisionStrategy,
    B: BettingStrategy,
{
    fn bet(&self, balance: f32) -> u32 {
        self.betting_strategy.bet(
            self.counting_strategy.running_count() as f32,
            self.counting_strategy.true_count(),
            balance,
        )
    }

    fn decide_option<'a>(
        &self,
        current_state: TableState<'a>,
        options: HashSet<String>,
    ) -> Result<String, BlackjackGameError> {
        self.decision_strategy.decide_option(current_state, options)
    }

    fn reset(&mut self) {
        self.counting_strategy.reset();
    }

    fn update(&mut self, card: Arc<Card>) {
        self.counting_strategy.update(card);
    }

    fn get_current_table_state<'a>(
        &self,
        hand: &'a Vec<Arc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        dealers_up_card: Arc<Card>,
    ) -> TableState<'a> {
        self.counting_strategy.get_current_table_state(
            hand,
            hand_value,
            bet,
            balance,
            dealers_up_card,
        )
    }
}

/// A trait for creating dynamic strategy trait objects. Use full for when testing multiple strategies against eachother
pub trait Strategy {
    // fn new() -> Self;
    fn bet(&self, balance: f32) -> u32;
    fn decide_option<'a>(
        &self,
        current_state: TableState<'a>,
        options: HashSet<String>,
    ) -> Result<String, BlackjackGameError>;
    fn reset(&mut self);
    fn update(&mut self, card: Arc<Card>);
    fn get_current_table_state<'a>(
        &self,
        hand: &'a Vec<Arc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        dealers_up_card: Arc<Card>,
    ) -> TableState<'a>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dynamic_strategy_creation() {
        let mut strategies: Vec<Box<dyn Strategy>> = vec![];
        let dyn_strategy1: Box<dyn Strategy> = Box::new(PlayerStrategy::new(
            HiLo::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ));

        let dyn_strategy2: Box<dyn Strategy> = Box::new(PlayerStrategy::new(
            WongHalves::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ));

        strategies.push(dyn_strategy1);
        strategies.push(dyn_strategy2);
        // println!("{:#?}", strategies);
        assert!(true);
    }
}
