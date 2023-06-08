use crate::sim::game::table::TableState;
use blackjack_lib::Card;
use std::collections::HashMap;
use std::rc::Rc;

pub trait CountingStrategy {
    fn update(&mut self, cards: Rc<Card>);
    fn bet(&self) -> u32;
    fn decide_option<'a>(
        &self,
        decision_state: &'a TableState,
        options: &HashMap<i32, String>,
    ) -> String;
}
