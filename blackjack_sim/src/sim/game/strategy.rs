use crate::sim::game::table::TableState;
use blackjack_lib::Card;
use std::collections::HashMap;
use std::rc::Rc;

pub trait CountingStrategy {
    fn update(&mut self, card: Rc<Card>);
    fn bet(&self, balance: f32) -> u32;
    fn decide_option<'a>(
        &self,
        decision_state: &'a TableState,
        options: &HashMap<i32, String>,
    ) -> String;
}

/// Struct that implements a simple HiLo betting strategy
pub struct HiLo {
    running_count: i32,
    total_cards_counted: u32,
    n_decks: usize,
    min_bet: u32,
    betting_margin: f32,
    sliding_margin: Option<f32>,
    lookup_table: HashMap<u8, i32>,
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
