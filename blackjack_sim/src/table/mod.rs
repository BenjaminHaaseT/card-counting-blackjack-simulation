mod dealer;
mod player;

use blackjack_lib::{BlackjackGameError, Card, Deck};
use dealer::DealersHandSim;
use player::PlayerSim;
pub struct BlackjackTableSim {
    deck: Deck,
    dealers_hand: DealersHandSim,
    balance: f64,
    n_decks: usize,
    n_shuffles: usize,
}

impl BlackjackTableSim {
    fn new(n_decks: usize, n_shuffles: usize, starting_balance: f64) -> Self {
        let deck = Deck::new(n_decks as u32);
        let dealers_hand = DealersHandSim::new();
        Self {
            deck,
            dealers_hand,
            balance: starting_balance,
            n_decks,
            n_shuffles,
        }
    }

    fn place_bet(&self, player: &mut PlayerSim) -> Result<(), BlackjackGameError> {}
}
