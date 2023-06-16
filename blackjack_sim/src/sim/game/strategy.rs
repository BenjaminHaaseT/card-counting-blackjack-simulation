use crate::sim::game::player::PlayerSimState;
use crate::sim::game::table::TableState;
use blackjack_lib::console::player;
use blackjack_lib::{BlackjackGameError, Card};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// A trait that allows for a fully customizeable blackjack strategy. Can accomodate complex card counting strategies.
pub trait Strategy {
    fn update(&mut self, card: Rc<Card>);
    fn bet(&self, balance: f32) -> u32;
    fn decide_option<'a>(
        &self,
        decision_state: &'a TableState,
        options: &HashMap<i32, String>,
    ) -> String;
}

/// Trait for a generic decision strategy. Has only one required method `decide_option()`,
/// the method that will take in the current state of the table i.e. the dealers face upcard and the state of the player and return a decsion.
/// Allows for composibility and customizability for specific card counting strategies.
/// The implementer may implement a custom decision strategy based on the state of the table
pub trait DecisionStrategy {
    fn decide_option<'a>(
        dealers_card: Rc<Card>,
        players_state: PlayerSimState<'a>,
        options: &HashSet<i32, String>,
    ) -> String;
}

/// Trait for a generic betting strategy. Allows greater composibility and customizeability for any playing strategy.
pub trait BettingStrategy {}

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

    /// Method for deciding how to play the current hand given the appropriate data
    pub fn decide_option<'a>(
        &self,
        dealers_card: Rc<Card>,
        player_state: PlayerSimState<'a>,
        options: &HashSet<String>,
    ) -> Result<String, BlackjackGameError> {
        let mut option = String::new();
        let dealers_card = dealers_card.val;

        // First check if we should surrender or not
        if options.contains("surrender") {
            if let Some(o) = self
                .surrender
                .get(&(player_state.hand_value[0], dealers_card))
            {
                option.push_str(0);
            }
        }

        if option.is_empty() && options.contains("split") {
            if let Some(o) = self
                .pair_totals
                .get(&(player_state.hand_value[0], dealers_card))
            {
                if o == "split" {
                    option.push_str(o);
                }
            }
        }

        // Check if players hand is a soft total, if so default ot soft totals lookup table
        if options.is_empty() && player_state.hand_value.len() == 2 {
            match self
                .soft_totals
                .get(&(player_state.hand_value[0], dealers_card))
            {
                Some(o) if o == "double down" && options.contains("double down") => {
                    option.push_str(o.as_str());
                }
                Some(o) if o == "double down" && !options.contains("double down") => {
                    option.push_str("hit");
                }
                Some(o) if options.contains(o.as_str()) => option.push_str(o.as_str()),
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
                .get(&(player_state.hand_value[0], dealers_card))
            {
                Some(o) if o == "double down" && options.contains("double down") => {
                    option.push_str(o.as_str());
                }
                Some(o) if o == "double down" && !options.contains("double down") {
                    option.push_str("hit");
                }
                Some(o) if options.contains(o.as_str()) => option.push_str(o.as_str()),
                _ => {
                    return Err(BlackjackGameError {
                        message: "option {o} not a valid choice".to_string(),
                    })
                }
            }
        }

        match option.is_empty() {
            true => Err(BlackjackGameError { message: "valid option was not selected".to_string() }),
            false => Ok(option)
        }
    }
}

impl DecisionStrategy for BasicStrategy {
    fn decide_option<'a>(
        dealers_card: Rc<Card>,
        player_state: PlayerSimState<'a>,
        options: &HashSet<String>,
    ) -> String {
        // Call self decide
        self.decide_option(dealers_card, player_state, options)
    }
}

/// Struct that implements a simple HiLo betting strategy
pub struct HiLo<B: BettingStrategy, D: DecisionStrategy> {
    running_count: i32,
    total_cards_counted: u32,
    n_decks: usize,
    min_bet: u32,
    betting_margin: f32,
    sliding_margin: Option<f32>,
    lookup_table: HashMap<u8, i32>,
    betting_strategy: B,
    decision_strategy: D,
}

impl<B: BettingStrategy, D: DecisionStrategy> HiLo<B, D> {
    /// Associated method for creating a new HiLo struct
    fn new(n_decks: usize, min_bet: u32, betting_margin: f32, sliding_margin: Option<f32>) -> Self {
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
            total_cards_counted: 0,
            n_decks,
            min_bet,
            betting_margin,
            sliding_margin,
            lookup_table,
        }
    }
}

impl<B: BettingStrategy, D: DecisionStrategy> Strategy for HiLo<B, D> {
    fn update(&mut self, card: Rc<Card>) {
        self.running_count += self.lookup_table[&card.val];
        self.total_cards_counted += 1;
    }

    fn bet(&self, balance: f32) -> u32 {}

    fn decide_option<'a>(
        &self,
        decision_state: &'a TableState,
        options: &HashMap<i32, String>,
    ) -> String {
    }
}
