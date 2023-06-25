// use crate::sim::game::player::PlayerSimState;
// use crate::sim::game::table::TableState;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::rc::Rc;

pub mod prelude {
    pub use super::*;
    pub use blackjack_lib::console::player;
    pub use blackjack_lib::{BlackjackGameError, Card};
}

pub use prelude::*;

/// Struct for encapsulating all the necessary information for a struct that implements `Strategy` to make a decsion and or place a bet.
/// Meant as a conveince for reducing the number of arguments passed to methods to a struct that implements `Strategy`.
pub struct TableState<'a> {
    hand: &'a Vec<Rc<Card>>,
    hand_value: &'a Vec<u8>,
    bet: u32,
    balance: f32,
    running_count: i32,
    true_count: f32,
    dealers_up_card: Rc<Card>,
}

impl<'a> TableState<'a> {
    fn new(
        hand: &'a Vec<Rc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        running_count: i32,
        true_count: f32,
        dealers_up_card: Rc<Card>,
    ) -> TableState<'a> {
        TableState {
            hand,
            hand_value,
            bet,
            balance,
            running_count,
            true_count,
            dealers_up_card,
        }
    }
}

/// Trait for a generic decision strategy. Has only one required method `decide_option()`,
/// the method that will take in the current state of the table i.e. the dealers face upcard and the state of the player and return a decsion.
/// Allows for composibility and customizability for specific card counting strategies.
/// The implementer may implement a custom decision strategy based on the state of the table
pub trait DecisionStrategy {
    fn decide_option<'a>(
        &self,
        decision_state: TableState<'a>,
        options: HashSet<String>,
    ) -> Result<String, BlackjackGameError>;
}

/// Trait for a generic betting strategy. Allows greater composibility and customizeability for any playing strategy.
pub trait BettingStrategy {
    fn bet(&self, running_count: i32, true_count: f32, balance: f32) -> u32;
}

/// Trait for a specific counting srategy. Can be implemented by any object that can be used to implement a counting strategy
pub trait CountingStrategy {
    fn new(num_decks: u32) -> Self;
    fn update(&mut self, card: Rc<Card>);
    fn get_current_table_state<'a>(
        &self,
        hand: &'a Vec<Rc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        dealers_up_card: Rc<Card>,
    ) -> TableState<'a>;
    fn reset(&mut self);
    fn running_count(&self) -> i32;
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
    fn bet(&self, running_count: i32, true_count: f32, balance: f32) -> u32 {
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

// /// Struct that implements a simple HiLo betting strategy
// pub struct HiLo<B: BettingStrategy, D: DecisionStrategy> {
//     running_count: i32,
//     true_count: f32,
//     total_cards_counted: u32,
//     n_decks: u32,
//     betting_strategy: B,
//     decision_strategy: D,
//     lookup_table: HashMap<u8, i32>,
// }

// // TODO: Implement builder pattern for HiLo strategy.
// impl<B: BettingStrategy, D: DecisionStrategy> HiLo<B, D> {
//     /// Associated method for creating a new HiLo struct
//     pub fn new(n_decks: u32, min_bet: u32, betting_strategy: B, decision_strategy: D) -> Self {
//         // Initialize lookup table
//         let mut lookup_table = HashMap::new();
//         for i in 2..7 {
//             lookup_table.insert(i, 1);
//         }
//         for i in 7..10 {
//             lookup_table.insert(i, 0);
//         }
//         lookup_table.insert(1, -1);
//         lookup_table.insert(10, -1);

//         HiLo {
//             running_count: 0,
//             true_count: 0.0,
//             total_cards_counted: 0,
//             n_decks,
//             betting_strategy,
//             decision_strategy,
//             lookup_table,
//         }
//     }
// }

// impl<B: BettingStrategy, D: DecisionStrategy> Strategy for HiLo<B, D> {
//     /// Method updates the running count and the true count respectively
//     fn update(&mut self, card: Rc<Card>) {
//         let card_val = card.val;
//         self.running_count += self.lookup_table[&card_val];
//         self.total_cards_counted += 1;
//         let remaining_decks = (self.n_decks as f32) - ((self.total_cards_counted as f32) / 52.0);
//         self.true_count = (self.running_count as f32) / remaining_decks;
//     }

//     /// Method that returns a bet according to `decision_state`
//     fn bet<'a>(&self, balance: f32) -> u32 {
//         self.betting_strategy
//             .bet(self.running_count, self.true_count, balance)
//     }

//     /// Method for making a decision with regards to playing a hand.
//     /// The method is potentiall fallible so it returns a `Result<String, BlackjackGameError>` representing whether or not a valid option was chosen or not
//     fn decide_option<'a>(
//         &self,
//         decision_state: TableState<'a>,
//         options: HashSet<String>,
//     ) -> Result<String, BlackjackGameError> {
//         self.decision_strategy
//             .decide_option(decision_state, options)
//     }

//     /// Method for getting the current state of the table
//     fn get_current_table_state<'a>(
//         &self,
//         hand: &'a Vec<Rc<Card>>,
//         hand_value: &'a Vec<u8>,
//         bet: u32,
//         balance: f32,
//         dealers_up_card: Rc<Card>,
//     ) -> TableState<'a> {
//         TableState::new(
//             hand,
//             hand_value,
//             bet,
//             balance,
//             self.running_count,
//             self.true_count,
//             dealers_up_card,
//         )
//     }

//     /// Method for reseting the count of the current strategy. A counting strategy needs to be reset whenenver the deck gets shuffled again,
//     /// this method provides that functionality.
//     fn reset(&mut self) {
//         self.running_count = 0;
//         self.true_count = 0.0;
//         self.total_cards_counted = 0;
//     }
// }

// impl<B: BettingStrategy, D: DecisionStrategy> Display for HiLo<B, D> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{}",
//             format!(
//                 "{:<21}:{}\n{:<21}:{}\n{:<21}:{}",
//                 "running_count:",
//                 self.running_count,
//                 "true_count:",
//                 self.true_count,
//                 "total_cards_counted:",
//                 self.total_cards_counted,
//             )
//         )
//     }
// }

pub struct HiLo {
    running_count: i32,
    true_count: f32,
    num_decks: u32,
    total_cards_counted: i32,
    lookup_table: HashMap<u8, i32>,
}

impl HiLo {
    // pub fn new(num_decks: u32) -> Self {
    //     // Initialize lookup table
    //     let mut lookup_table = HashMap::new();
    //     for i in 2..7 {
    //         lookup_table.insert(i, 1);
    //     }
    //     for i in 7..10 {
    //         lookup_table.insert(i, 0);
    //     }
    //     lookup_table.insert(1, -1);
    //     lookup_table.insert(10, -1);

    //     HiLo {
    //         running_count: 0,
    //         true_count: 0.0,
    //         num_decks,
    //         total_cards_counted: 0,
    //         lookup_table,
    //     }
    // }
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
    fn update(&mut self, card: Rc<Card>) {
        self.running_count += self.lookup_table[&card.val];
        self.total_cards_counted += 1;
        let estimated_decks_counted =
            (self.num_decks as f32) - ((self.total_cards_counted as f32) / 52.0);
        self.true_count = (self.running_count as f32) / estimated_decks_counted;
    }

    fn get_current_table_state<'a>(
        &self,
        hand: &'a Vec<Rc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        dealers_up_card: Rc<Card>,
    ) -> TableState<'a> {
        TableState {
            hand,
            hand_value,
            bet,
            balance,
            running_count: self.running_count,
            true_count: self.true_count,
            dealers_up_card,
        }
    }

    fn running_count(&self) -> i32 {
        self.running_count
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

/// A struct that encapsulates everything needed to implement a specific playing to test in a simulation.
pub struct Strategy<C, D, B>
where
    C: CountingStrategy,
    D: DecisionStrategy,
    B: BettingStrategy,
{
    counting_strategy: C,
    decision_strategy: D,
    betting_strategy: B,
}

impl<C, D, B> Strategy<C, D, B>
where
    C: CountingStrategy,
    D: DecisionStrategy,
    B: BettingStrategy,
{
    pub fn new(num_decks: u32, min_bet: u32) -> StrategyBuilder<C, D, B> {
        StrategyBuilder {
            num_decks,
            min_bet,
            counting_strategy: None,
            decision_strategy: None,
            betting_strategy: None,
        }
    }

    pub fn bet(&self, balance: f32) -> u32 {
        self.betting_strategy.bet(
            self.counting_strategy.running_count(),
            self.counting_strategy.true_count(),
            balance,
        )
    }

    pub fn decide_option<'a>(
        &self,
        current_state: TableState<'a>,
        options: HashSet<String>,
    ) -> Result<String, BlackjackGameError> {
        self.decision_strategy.decide_option(current_state, options)
    }

    pub fn reset(&mut self) {
        self.counting_strategy.reset();
    }

    pub fn update(&mut self, card: Rc<Card>) {
        self.counting_strategy.update(card);
    }

    pub fn get_current_table_state<'a>(
        &self,
        hand: &'a Vec<Rc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        dealers_up_card: Rc<Card>,
    ) -> TableState<'a> {
        TableState {
            hand,
            hand_value,
            bet,
            balance,
            running_count: self.counting_strategy.running_count(),
            true_count: self.counting_strategy.true_count(),
            dealers_up_card,
        }
    }
}

impl<C, D, B> Display for Strategy<C, D, B>
where
    C: CountingStrategy + Display,
    D: DecisionStrategy,
    B: BettingStrategy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.counting_strategy)
    }
}

/// A struct that aids in building a customized strategy
pub struct StrategyBuilder<C, D, B>
where
    C: CountingStrategy,
    D: DecisionStrategy,
    B: BettingStrategy,
{
    num_decks: u32,
    min_bet: u32,
    counting_strategy: Option<C>,
    decision_strategy: Option<D>,
    betting_strategy: Option<B>,
}

impl<C, D, B> StrategyBuilder<C, D, B>
where
    C: CountingStrategy,
    D: DecisionStrategy,
    B: BettingStrategy,
{
    /// Method that sets `self.counting_strategy` of a `StrategyBuilder` object.
    pub fn counting_strategy(&mut self, counter: C) -> &mut Self {
        self.counting_strategy = Some(counter);
        self
    }

    /// Method that sets `self.decision_strategy` of a `StrategyBuilder` object.
    pub fn decision_strategy(&mut self, decider: D) -> &mut Self {
        self.decision_strategy = Some(decider);
        self
    }

    /// Method that sets `self.betting_strategy` of a `StrategyBuilder` object.
    pub fn betting_strategy(&mut self, better: B) -> &mut Self {
        self.betting_strategy = Some(better);
        self
    }

    /// Method that builds a `Strategy` object. Will supply counting_strategy, decision_strategy and bettting strategy with
    /// `HiLo`, `BasicStrategy` and `MarginBettingStrategy` respectively by default if these fields have not been set.
    pub fn build(&mut self) -> Strategy<C, D, B> {
        let counting_strategy = self
            .counting_strategy
            .take()
            .expect("counting strategy should be set");
        let decision_strategy = self
            .decision_strategy
            .take()
            .expect("decision strategy should have been set");
        let betting_strategy = self
            .betting_strategy
            .take()
            .expect("betting strategy should have been set");
        Strategy {
            counting_strategy,
            decision_strategy,
            betting_strategy,
        }
    }
}
