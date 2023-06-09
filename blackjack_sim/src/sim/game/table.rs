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

    /// Method for receiving a card, changes the state of the `DealersHandSim` instance
    pub fn receive_card(&mut self, card: Rc<Card>) {
        let card_val = card.val;
        self.hand.push(card);
        if self.hand_value.is_empty() {
            self.hand_value.push(card_val);
        } else {
            self.hand_value[0] += card_val;
            if self.hand_value.len() == 2 {
                self.hand_value[1] += card_val;
            }
        }

        // Check if we need to add an alternative hand value
        if self.hand.len() == 1 && self.hand_value[0] <= 11 && card_val == 1 {
            let alternative_hand_val = self.hand_value[0] + 10;
            self.hand_value.push(alternative_hand_val);
        }
    }

    /// Methods that checks if the dealer has a blackjack
    pub fn has_blackjack(&self) -> bool {
        self.hand.len() == 2
            && ((self.hand[0].val == 10 && self.hand[1].rank == "A")
                || (self.hand[0].rank == "A" && self.hand[1].val == 10))
    }
}

/// Struct for a simulated blackjack game
pub struct BlackjackTableSim {
    pub balance: f32,
    pub hand_data: Vec<(i32, String, String)>,
    bet_data: Vec<i32>,
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
            hand_data: vec![],
            bet_data: vec![],
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
        self.dealers_hand.receive_card(cur_card);

        // Check if dealer has blackjack, if so check if player has blackjack
        //  in either case the hand needs to end. Log the bet, and the hand values of dealer, player respectively
        // We need to update self.hand_data for logging purposes
        // TODO: implement .formatted_hand_values() for dealers_hand and reset methods as well
        if self.dealers_hand.has_blackjack() {
            let (amount, players_formatted_hand, dealers_formatted_hand) = if player.has_blackjack()
            {
                (
                    player.push(),
                    player.formatted_hand_values(),
                    self.dealers_hand.formatted_hand_values(),
                )
            } else {
                (
                    player.lose(),
                    player.formatted_hand_values(),
                    self.dealers_hand.formatted_hand_values(),
                )
            };
            // Save this for logging purposes
            self.hand_data
                .push((amount, players_formatted_hand, dealers_formatted_hand));
            player.reset();
            self.dealers_hand.reset();
        }
    }
}
