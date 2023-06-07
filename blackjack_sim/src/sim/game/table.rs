use blackjack_lib::Card;
use std::rc::Rc;

pub struct TableState<'a> {
    bet: u32,
    current_hand_value: &'a Vec<u8>,
    dealers_card: Rc<Card>,
}
