use crate::sim::game::player::PlayerSim;
use crate::sim::game::strategy::CountingStrategy;
use blackjack_lib::{BlackjackGameError, BlackjackTable, Card, Deck};
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

    /// Method for getting the formatted hand value of the dealer, intended for logging purposes
    pub fn formatted_hand_values(&self) -> String {
        if self.hand_value.len() == 2 {
            if self.hand_value[0] <= 21 && self.hand_value[1] <= 21 {
                format!("{}/{}", self.hand_value[0], self.hand_value[1])
            } else {
                format!("{}", u8::min(self.hand_value[0], self.hand_value[1]))
            }
        } else {
            format!("{}", self.hand_value[0])
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
    /// Takes a player and a bet and handles the logic for placing a bet before a hand is dealt
    fn place_bet(
        &self,
        player: &mut PlayerSim<S>,
        bet: f32,
    ) -> Result<(), blackjack_lib::BlackjackGameError> {
        if bet <= 0.0 {
            return Err(BlackjackGameError {
                message: "bet must be a positive amount".to_string(),
            });
            // return Err("Bet must be a positive amount".to_string());
        } else if self.balance < 1.5 * bet {
            return Err(BlackjackGameError {
                message: "insufficient table balance to payout bet".to_string(),
            });
        }
        player.place_bet(bet)
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
        }
    }

    /// Deals a card to the player, allows the player to update their strategy.
    /// If the player busted, then data about the hand is saved for logging purposes.
    fn hit(&mut self, player: &mut PlayerSim<S>) {
        // Deal another card to the player and make sure the player updates their strategy
        let card = self.deck.get_next_card().unwrap();
        player.receive_card(Rc::clone(&card));
        player.update_strategy(card);
        if player.busted() {
            // Log the hand data
            self.hand_data.push((
                player.lose(),
                player.formatted_hand_values(),
                String::from("NA"),
            ));
        }
    }

    /// Method for implementing the logic needed to double down on a bet
    fn double_down(&mut self, player: &mut PlayerSim<S>) {
        player.double_down();
        // Deal the player another card
        let card = self.deck.get_next_card().unwrap();
        player.receive_card(Rc::clone(&card));
        player.update_strategy(card);
        // Check if player has busted or not
        if !player.busted() {
            player.stand();
        } else {
            // The player lost the hand, data needs to be logged
            self.hand_data.push((
                player.lose(),
                player.formatted_hand_values(),
                String::from("NA"),
            ));
        }
    }

    fn split(&mut self, player: &mut PlayerSim<S>) {}

    fn stand(&self, player: &mut PlayerSim<S>) {}
}
