use crate::sim::game::player::PlayerSimState;
// use crate::sim::game::table::TableState;
use blackjack_lib::console::player;
use blackjack_lib::{BlackjackGameError, Card};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Struct for encapsulating all the necessary information for a struct that implements `Strategy` to make a decsion and or place a bet.
/// Meant as a conveince for reducing the number of arguments passed to methods ot a struct that implements `Strategy`.
pub struct TableState<'a> {
    hand: &'a Vec<Rc<Card>>,
    hand_value: &'a Vec<u8>,
    bet: u32,
    balance: f32,
    running_count: i32,
    true_count: i32,
    dealers_up_card: Rc<Card>,
}

impl<'a> TableState<'a> {
    fn new(
        hand: &'a Vec<Rc<Card>>,
        hand_value: &'a Vec<u8>,
        bet: u32,
        balance: f32,
        running_count: i32,
        true_count: i32,
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

/// A trait that allows for a fully customizeable blackjack strategy. Can accomodate complex card counting strategies.
pub trait Strategy {
    fn update(&mut self, card: Rc<Card>);
    fn bet<'a>(&self, decision_state: TableState<'a>) -> u32;
    fn decide_option<'a>(
        &self,
        decision_state: TableState<'a>,
        options: &HashSet<String>,
    ) -> Result<String, BlackjackGameError>;
}

/// Trait for a generic decision strategy. Has only one required method `decide_option()`,
/// the method that will take in the current state of the table i.e. the dealers face upcard and the state of the player and return a decsion.
/// Allows for composibility and customizability for specific card counting strategies.
/// The implementer may implement a custom decision strategy based on the state of the table
pub trait DecisionStrategy {
    fn decide_option<'a>(
        &self,
        decision_state: TableState<'a>,
        options: &HashSet<String>,
    ) -> Result<String, BlackjackGameError>;
}

/// Trait for a generic betting strategy. Allows greater composibility and customizeability for any playing strategy.
pub trait BettingStrategy {
    fn bet<'a>(&self, decision_state: TableState<'a>) -> u32;
}

/// Struct that encapsulates the logic needed for a simple margin based betting strategy, will take in a `TableState`
/// struct and decide to increase/decrease the bet according to the margin and minimum bet
pub struct MarginBettingStrategy {
    margin: f32,
    min_bet: u32,
}

impl MarginBettingStrategy {
    /// A simple margin betting strategy, for each positive value of true count this strategy returns the product of the minimum bet, true count and margin
    fn bet<'a>(&self, decision_state: TableState<'a>) -> u32 {
        if decision_state.true_count > 0 {
            u32::min(
                decision_state.balance as u32,
                f32::floor((self.min_bet as f32) * self.margin * decision_state.true_count as f32)
                    as u32,
            )
        } else {
            u32::min(decision_state.balance as u32, self.min_bet)
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
        for i in 9..=21 {
            for j in 1..=10 {
                let mut option = String::new();
                match i {
                    2..=8 => option.push_str("hit"),
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
                    17 => option.push_str("stand"),
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

    // /// Method for deciding how to play the current hand given the appropriate data
    // pub fn decide_option<'a>(
    //     &self,
    //     decision_state: TableState<'a>,
    //     options: &HashSet<String>,
    // ) -> Result<String, BlackjackGameError> {
    //     let mut option = String::new();
    //     let dealers_card = decision_state.dealers_up_card.val;

    //     // First check if we should surrender or not
    //     if options.contains("surrender") {
    //         if let Some(o) = self
    //             .surrender
    //             .get(&(decision_state.hand_value[0], dealers_card))
    //         {
    //             option.push_str(o.as_str());
    //         }
    //     }

    //     if option.is_empty() && options.contains("split") {
    //         if let Some(o) = self
    //             .pair_totals
    //             .get(&(decision_state.hand_value[0], dealers_card))
    //         {
    //             if o == "split" {
    //                 option.push_str(o);
    //             }
    //         }
    //     }

    //     // Check if players hand is a soft total, if so default ot soft totals lookup table
    //     if options.is_empty() && decision_state.hand_value.len() == 2 {
    //         match self
    //             .soft_totals
    //             .get(&(decision_state.hand_value[0], dealers_card))
    //         {
    //             Some(o) if options.contains(o.as_str()) => option.push_str(o.as_str()),
    //             Some(o) if o == "double down" && !options.contains("double down") => {
    //                 option.push_str("hit");
    //             }
    //             _ => {
    //                 return Err(BlackjackGameError {
    //                     message: "option {o} not a valid choice".to_string(),
    //                 })
    //             }
    //         }
    //     }

    //     if options.is_empty() {
    //         match self
    //             .hard_totals
    //             .get(&(decision_state.hand_value[0], dealers_card))
    //         {
    //             Some(o) if options.contains(o.as_str()) => option.push_str(o.as_str()),
    //             Some(o) if o == "double down" && !options.contains("double down") => {
    //                 option.push_str("hit");
    //             }
    //             _ => {
    //                 return Err(BlackjackGameError {
    //                     message: "option {o} not a valid choice".to_string(),
    //                 })
    //             }
    //         }
    //     }

    //     match option.is_empty() {
    //         true => Err(BlackjackGameError {
    //             message: "valid option was not selected".to_string(),
    //         }),
    //         false => Ok(option),
    //     }
    // }
}

impl DecisionStrategy for BasicStrategy {
    /// Method for deciding how to play the current hand given the appropriate data
    fn decide_option<'a>(
        &self,
        decision_state: TableState<'a>,
        options: &HashSet<String>,
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
        if options.is_empty() && decision_state.hand_value.len() == 2 {
            match self
                .soft_totals
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

        if options.is_empty() {
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

        match option.is_empty() {
            true => Err(BlackjackGameError {
                message: "valid option was not selected".to_string(),
            }),
            false => Ok(option),
        }
    }
}

/// Struct that implements a simple HiLo betting strategy
pub struct HiLo<B: BettingStrategy, D: DecisionStrategy> {
    running_count: i32,
    true_count: i32,
    total_cards_counted: u32,
    n_decks: u32,
    betting_strategy: B,
    decision_strategy: D,
    lookup_table: HashMap<u8, i32>,
}

impl<B: BettingStrategy, D: DecisionStrategy> HiLo<B, D> {
    /// Associated method for creating a new HiLo struct
    fn new(n_decks: u32, min_bet: u32, betting_strategy: B, decision_strategy: D) -> Self {
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
            true_count: 0,
            total_cards_counted: 0,
            n_decks,
            betting_strategy,
            decision_strategy,
            lookup_table,
        }
    }
}

impl<B: BettingStrategy, D: DecisionStrategy> Strategy for HiLo<B, D> {
    /// Method updates the running count and the true count respectively
    fn update(&mut self, card: Rc<Card>) {
        let card_val = card.val;
        self.running_count += self.lookup_table[&card_val];
        self.total_cards_counted += 1;
        let decks_played = self.n_decks - ((self.total_cards_counted / 52) as u32);
        self.true_count = self.running_count / (decks_played as i32);
    }

    /// Method that returns a bet according to `decision_state`
    fn bet<'a>(&self, decision_state: TableState<'a>) -> u32 {
        self.betting_strategy.bet(decision_state)
    }

    /// Method for making a decision with regards to playing a hand.
    /// The method is potentiall fallible so it returns a `Result<String, BlackjackGameError>` representing whether or not a valid option was chosen or not
    fn decide_option<'a>(
        &self,
        decision_state: TableState<'a>,
        options: &HashSet<String>,
    ) -> Result<String, BlackjackGameError> {
        self.decision_strategy
            .decide_option(decision_state, options)
    }
}
