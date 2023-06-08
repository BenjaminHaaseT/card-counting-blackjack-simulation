use crate::sim::game::player::PlayerSim;
use crate::sim::game::strategy::CountingStrategy;
use blackjack_lib::{BlackjackTable, Card, Deck};
use std::rc::Rc;

// use super::strategy::CountingStrategy;

pub struct TableState<'a> {
    bet: u32,
    current_hand_value: &'a Vec<u8>,
    dealers_card: Rc<Card>,
}

struct DealersHandSim {
    hand: Vec<Rc<Card>>,
    hand_value: Vec<u8>,
}

impl DealersHandSim {
    /// Associated function to create a new `DealersHandSim` struct
    pub fn new() -> Self {
        DealersHandSim {
            hand: Vec::new(),
            hand_value: Vec::new(),
        }
    }
}

/// Struct for a simulated blackjack game
pub struct BlackjackTableSim {
    pub balance: f32,
    dealers_hand: DealersHandSim,
    n_decks: usize,
    n_shuffles: u32,
    deck: Deck,
}

/// TODO: Implement missing methods on the blackjack table interface
impl<S: CountingStrategy> BlackjackTable<PlayerSim<S>> for BlackjackTableSim {
    fn new(starting_balance: f32, n_decks: usize, n_shuffles: u32) -> Self {
        let dealers_hand = DealersHandSim::new();
        let deck = Deck::new(n_decks);
        BlackjackTableSim {
            balance: starting_balance,
            dealers_hand,
            n_decks,
            n_shuffles,
            deck,
        }
    }

    /// Simulates dealing a hand of blackjack, the method may panic if `player` has not placed a valid bet.
    fn deal_hand(&mut self, player: &mut PlayerSim<S>) {
        if self.deck.shuffle_flag {
            self.deck.shuffle(self.n_shuffles);
        }
        // Now deal cards to player and dealer
        let mut cur_card = self.deck.get_next_card().unwrap();
        player.receive_card(Rc::clone(&cur_card));
        player.update_strategy(cur_card);

        // First card to dealer is face up so the players strategy should be aware of it
        cur_card = self.deck.get_next_card().unwrap();
        self.dealers_hand.receive_card(Rc::clone(&cur_card));
        player.update_strategy(cur_card);

        cur_card = self.deck.get_next_card().unwrap();
        player.receive_card(Rc::clone(&cur_card));
        player.update_strategy(cur_card);

        // This card is face down so the players strategy should not take this card into account
        cur_card = self.deck.get_next_card().unwrap();
        self.dealers_hand.receive_card(cur_card)

        // TODO: implement updating the values of dealers/players hands, check for blackjack etc...
    }
}
