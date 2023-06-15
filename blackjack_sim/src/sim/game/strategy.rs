use crate::sim::game::table::TableState;
use blackjack_lib::Card;
use std::collections::HashMap;
use std::rc::Rc;

pub trait Strategy {
    fn update(&mut self, card: Rc<Card>);
    fn bet(&self, balance: f32) -> u32;
    fn decide_option<'a>(
        &self,
        decision_state: &'a TableState,
        options: &HashMap<i32, String>,
    ) -> String;
}

pub trait DecisionStrategy {}

pub trait BettingStrategy {}

pub struct BasicStrategy {
    hard_totals: HashMap<(u8, u8), String>,
    soft_totals: HashMap<(u8, u8), String>,
    pair_totals: HashMap<(u8, u8), String>,
    surrender: HashMap<(u8, u8), String>,
}

impl BasicStrategy {
    fn build_lookup_tables() -> (
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
    }
    pub fn new() -> BasicStrategy {
        let mut soft_totals: HashMap<(u8, u8), String> = HashMap::new();
        for i in 3..=10 {
            for j in 1..=10 {
                match i {
                    3..=7 => soft_totals.insert((i, j), "hit".to_string()),
                    8 => match j {
                        2..=6 => soft_totals.insert((i, j), "double down".to_string()),
                        7 | 8 => soft_totals.insert((i, j), "stand".to_string()),
                        _ => soft_totals.insert((i, j), "hit".to_string()),
                    },
                    9 => match j {
                        6 => soft_totals.insert((i, j), "double down".to_string()),
                        _ => soft_totals.insert((i, j), "stand".to_string()),
                    },
                    _ => soft_totals.insert((i, j), "stand".to_string()),
                }
            }
        }

        let mut pair_totals: HashMap<(u8, u8), String> = HashMap::new();
        for i in (2..=20).step_by(2) {
            for j in 1..=10 {
                match i {
                    2 => pair_totals.insert((i, j), "split".to_string()),
                    4 | 6 => match j {
                        2..=7 => pair_totals.insert((i, j), "split".to_string()),
                        _ => pair_totals.insert((i, j), "default".to_string()),
                    },
                    8 => match j {
                        5 | 6 => pair_totals.insert((i, j), "split".to_string()),
                        _ => pair_totals.insert((i, j), "default".to_string()),
                    },
                    10 => pair_totals.insert((i, j), "default".to_string()),
                    12 => match j {
                        2..=6 => pair_totals.insert((i, j), "double down".to_string()),
                        _ => pair_totals.insert((i, j), "default".to_string()),
                    },
                    14 => match j {
                        2..=7 => pair_totals.insert((i, j), "split".to_string()),
                        _ => pair_totals.insert((i, j), "default".to_string()),
                    },
                    16 => pair_totals.insert((i, j), "split".to_string()),
                    18 => match j {
                        2..=6 | 8 | 9 => pair_totals.insert((i, j), "split".to_string()),
                        _ => pair_totals.insert((i, j), "default".to_string()),
                    },
                    20 => pair_totals.insert((i, j), "default".to_string()),
                }
            }
        }

        let mut surrender: HashMap<(u8, u8), String> = HashMap::new();
        surrender.insert((15, 10), "surrender".to_string());
        surrender.insert((16, 9), "surrender".to_string());
        surrender.insert((16, 10), "surrender".to_string());
        surrender.insert((16, 1), "surrender".to_string());
    }
}

impl DecisionStrategy for BasicStrategy {
    fn decide_option(decision_state: &'a TableState, options: &HashMap<i32, String>) -> String {}
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

impl HiLo {
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

impl CountingStrategy for HiLo {
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
