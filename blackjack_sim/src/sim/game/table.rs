use crate::sim::game::player::PlayerSim;
use crate::sim::game::strategy::CountingStrategy;
use blackjack_lib::{BlackjackGameError, BlackjackTable, Card, Deck};
use std::collections::HashMap;
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
        if self.hand_value.len() == 1 && self.hand_value[0] <= 11 && card_val == 1 {
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
    final_cards: Vec<Rc<Card>>,
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
            final_cards: vec![],
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
        player.update_strategy(Some(cur_card));

        // First card to dealer is face up so the players strategy should be aware of it
        cur_card = self.deck.get_next_card().unwrap();
        self.dealers_hand.receive_card(Rc::clone(&cur_card));
        player.update_strategy(Some(cur_card));

        cur_card = self.deck.get_next_card().unwrap();
        player.receive_card(Rc::clone(&cur_card));
        player.update_strategy(Some(cur_card));

        // This card is face down so the players strategy should not take this card into account
        cur_card = self.deck.get_next_card().unwrap();
        self.dealers_hand.receive_card(cur_card);

        // Check if either the dealer or player has blackjack if so the hand needs to end, so call player.stand()
        // TODO: Need to fix this for the simulation, hand needs to end
        if self.dealers_hand.has_blackjack() || player.has_blackjack() {
            
        }
    }

    /// Deals a card to the player, allows the player to update their strategy.
    /// If the player busted, then data about the hand is saved for logging purposes.
    fn hit(&mut self, player: &mut PlayerSim<S>) {
        // Deal another card to the player and make sure the player updates their strategy
        let card = self.deck.get_next_card().unwrap();
        player.receive_card(Rc::clone(&card));
        player.update_strategy(Some(card));
        if player.busted() {
            player.stand();
        }
    }

    /// Method for implementing the logic needed to double down on a bet
    fn double_down(&mut self, player: &mut PlayerSim<S>) {
        player.double_down();
        // Deal the player another card
        let card = self.deck.get_next_card().unwrap();
        player.receive_card(Rc::clone(&card));
        player.update_strategy(Some(card));
        player.stand();
    }

    /// Method that implements the logic for splitting
    fn split(&mut self, player: &mut PlayerSim<S>) {
        let (card1, card2) = (
            self.deck.get_next_card().unwrap(),
            self.deck.get_next_card().unwrap(),
        );
        player.split(Rc::clone(&card1), Rc::clone(&card2));
        player.update_strategy(Some(card1));
        player.update_strategy(Some(card2));
    }

    /// Method that calls the `player`'s stand method.
    fn stand(&self, player: &mut PlayerSim<S>) {
        player.stand();
    }

    /// Takes a `PlayerSim<S>` struct, a HashMap<i32, String> representing the options available during the current turn (these options will be decided during runtime), and an i32 `option`.
    /// The method decides what method to call the implements the appropriate logic, returns a `Result<(), BlackjackGameError>` since the method is fallible.
    fn play_option(
        &mut self,
        player: &mut PlayerSim<S>,
        options: &std::collections::HashMap<i32, String>,
        option: i32,
    ) -> Result<(), BlackjackGameError> {
        match options.get(&option) {
            Some(s) if s == "stand" => Ok(self.stand(player)),
            Some(s) if s == "hit" => Ok(self.hit(player)),
            Some(s) if s == "split" => Ok(self.split(player)),
            Some(s) if s == "double down" => Ok(self.double_down(player)),
            _ => Err(BlackjackGameError::new("option not available".to_string())),
        }
    }

    /// Method that computes and returns the optimal final hand for the dealer at the end of a hand of blackjack
    fn get_dealers_optimal_final_hand(&mut self) -> u8 {
        // Reveal dealers face down card here
        self.final_cards.push(Rc::clone(&self.dealers_hand.hand[1]));

        if self.dealers_hand.hand_value.len() == 2 {
            while self.dealers_hand.hand_value[0] < 17 && self.dealers_hand.hand_value[1] < 17 {
                let next_card = self.deck.get_next_card().unwrap();
                self.dealers_hand.receive_card(Rc::clone(&next_card));
                self.final_cards.push(next_card);
            }

            // Ensure we have a valid hand according to the rules of blackjack
            while (self.dealers_hand.hand_value[0] > 21 && self.dealers_hand.hand_value[1] < 17)
                || (self.dealers_hand.hand_value[0] < 17 && self.dealers_hand.hand_value[1] > 21)
            {
                let next_card = self.deck.get_next_card().unwrap();
                self.dealers_hand.receive_card(Rc::clone(&next_card));
                self.final_cards.push(next_card);
            }

            if self.dealers_hand.hand_value[0] <= 21 && self.dealers_hand.hand_value[1] <= 21 {
                return u8::max(
                    self.dealers_hand.hand_value[0],
                    self.dealers_hand.hand_value[1],
                );
            } else {
                return u8::min(
                    self.dealers_hand.hand_value[0],
                    self.dealers_hand.hand_value[1],
                );
            }
        }

        while self.dealers_hand.hand_value[0] < 17 {
            let next_card = self.deck.get_next_card().unwrap();
            self.dealers_hand.receive_card(Rc::clone(&next_card));
            self.final_cards.push(next_card);
        }

        self.dealers_hand.hand_value[0]
    }

    //TODO: Need to find a way of recording what bets were won/lost in an efficient way
    fn finish_hand(&mut self, player: &mut PlayerSim<S>) {
        let dealers_final_hand_value: Option<u8> = None;
        for (hand: u8, bet: u32) in player.final_hands() {
            if 
        }

        self.dealers_hand.reset();
        player.reset();
    }
}
