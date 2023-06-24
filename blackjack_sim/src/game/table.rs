use crate::game::player::PlayerSim;
use crate::game::strategy::{
    BasicStrategy, BettingStrategy, DecisionStrategy, HiLo, MarginBettingStrategy, Strategy,
};
use blackjack_lib::{BlackjackGameError, BlackjackTable, Card, Deck};
use std::collections::HashSet;
use std::rc::Rc;

pub struct DealersHandSim {
    pub hand: Vec<Rc<Card>>,
    pub hand_value: Vec<u8>,
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

    /// Method to reset the hand after a complete hand
    pub fn reset(&mut self) {
        self.hand.clear();
        self.hand_value.clear();
    }
}

/// Struct for a simulated blackjack game
pub struct BlackjackTableSim {
    pub balance: f32,
    pub hand_log: Option<(i32, i32, i32, f32)>,
    final_cards: Vec<Rc<Card>>,
    pub dealers_hand: DealersHandSim,
    pub num_player_blackjacks: i32,
    // n_decks: usize,
    n_shuffles: u32,
    deck: Deck,
}

// impl BlackjackTableSim {}

/// TODO: Implement missing methods on the blackjack table interface
impl<S: Strategy> BlackjackTable<PlayerSim<S>> for BlackjackTableSim {
    fn new(starting_balance: f32, n_decks: usize, n_shuffles: u32) -> Self {
        let dealers_hand = DealersHandSim::new();
        let deck = Deck::new(n_decks);
        BlackjackTableSim {
            balance: starting_balance,
            hand_log: None,
            final_cards: vec![],
            dealers_hand,
            num_player_blackjacks: 0,
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
        } else if self.balance < 1.5 * bet {
            return Err(BlackjackGameError {
                message: "insufficient table balance to payout bet".to_string(),
            });
        }
        Ok(player.place_bet(bet))
    }

    /// Simulates dealing a hand of blackjack, the method may panic if `player` has not placed a valid bet.
    fn deal_hand(&mut self, player: &mut PlayerSim<S>) {
        assert!(!player.bets.is_empty());

        if self.deck.shuffle_flag {
            // For debugging purposes eventually remove this
            // println!("Shuffling...");
            self.deck.shuffle(self.n_shuffles);
            player.reset_strategy();
        }

        // Now deal cards to player and dealer
        let mut cur_card = self.deck.get_next_card().unwrap();
        player.receive_card(Rc::clone(&cur_card));
        player.update_strategy(Some(&cur_card));

        // First card to dealer is face up so the players strategy should be aware of it
        cur_card = self.deck.get_next_card().unwrap();
        self.dealers_hand.receive_card(Rc::clone(&cur_card));
        player.update_strategy(Some(&cur_card));

        cur_card = self.deck.get_next_card().unwrap();
        player.receive_card(Rc::clone(&cur_card));
        player.update_strategy(Some(&cur_card));

        // This card is face down so the players strategy should not take this card into account
        cur_card = self.deck.get_next_card().unwrap();
        self.dealers_hand.receive_card(cur_card);

        // Check for a blackjack, if the dealer has a blackjack we need to check whether the player has a blackjack or not as well
        // in addition we need to update the players strategy, i.e. the counting strategy
        if self.dealers_hand.has_blackjack() {
            player.update_strategy(Some(&self.dealers_hand.hand[1]));
            if player.has_blackjack() {
                player.push_current_hand();
                self.num_player_blackjacks += 1;
            } else {
                player.lose_current_hand();
            }
        } else if player.has_blackjack() {
            let current_bet = player.get_current_bet() as f32;
            self.balance -= current_bet * 1.5;
            player.blackjack(current_bet * 1.5);
            self.num_player_blackjacks += 1;
        }
    }

    /// Deals a card to the player, allows the player to update their strategy.
    /// If the player busted, then data about the hand is saved for logging purposes.
    fn hit(&mut self, player: &mut PlayerSim<S>) {
        // Deal another card to the player and make sure the player updates their strategy
        let card = self.deck.get_next_card().unwrap();
        player.receive_card(Rc::clone(&card));
        player.update_strategy(Some(&card));
        if player.busted() {
            player.lose_current_hand();
        }
    }

    /// Method for implementing the logic needed to double down on a bet
    fn double_down(&mut self, player: &mut PlayerSim<S>) {
        player.double_down();
        // Deal the player another card
        let card = self.deck.get_next_card().unwrap();
        player.receive_card(Rc::clone(&card));
        player.update_strategy(Some(&card));
        player.stand();
    }

    /// Method that implements the logic for splitting
    fn split(&mut self, player: &mut PlayerSim<S>) {
        let (card1, card2) = (
            self.deck.get_next_card().unwrap(),
            self.deck.get_next_card().unwrap(),
        );
        player.split(Rc::clone(&card1), Rc::clone(&card2));
        player.update_strategy(Some(&card1));
        player.update_strategy(Some(&card2));
    }

    /// Method that calls the `player`'s stand method.
    fn stand(&self, player: &mut PlayerSim<S>) {
        player.stand();
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

    /// Method for finishing the hand and deciding what bet(s) `player` wins or loses
    fn finish_hand(&mut self, player: &mut PlayerSim<S>) {
        if let Some(players_final_hands) = player.get_optimal_hands() {
            let dealers_optimal_hand =
                <BlackjackTableSim as BlackjackTable<PlayerSim<S>>>::get_dealers_optimal_final_hand(
                    self,
                );
            for (i, bet, hand) in players_final_hands {
                if dealers_optimal_hand > 21 || hand > dealers_optimal_hand {
                    self.balance -= bet as f32;
                    player.win_hand(i, bet);
                } else if dealers_optimal_hand == hand {
                    player.push_hand(i, bet);
                } else {
                    player.lose_hand(i, bet);
                }
            }
        }
        // Update the players strategy
        player.update_strategy(self.final_cards.iter());

        let (mut hands_won, mut hands_pushed, mut hands_lost, mut winnings) = (0, 0, 0, 0.0);
        for (_, bet) in player.bets_log.iter() {
            if *bet > 0.0 || *bet < 0.0 {
                winnings += *bet;
                if *bet < 0.0 {
                    hands_lost += 1;
                    self.balance -= *bet;
                } else {
                    hands_won += 1;
                }
            } else {
                hands_pushed += 1;
            }
        }

        if winnings > 0.0 {
            player.collect_winnings(winnings);
        }

        self.hand_log = Some((hands_won, hands_pushed, hands_lost, winnings));
    }
}

impl BlackjackTableSim {
    /// Takes a `PlayerSim<S>` struct, a HashMap<i32, String> representing the options available during the current turn (these options will be decided during runtime), and an i32 `option`.
    /// The method decides what method to call the implements the appropriate logic, returns a `Result<(), BlackjackGameError>` since the method is fallible.
    pub fn play_option<S: Strategy>(
        &mut self,
        player: &mut PlayerSim<S>,
        // options: HashSet<String>,
        option: String,
    ) -> Result<(), BlackjackGameError> {
        match option.as_str() {
            "stand" => Ok(self.stand(player)),
            "hit" => Ok(self.hit(player)),
            "split" => Ok(self.split(player)),
            "double down" => Ok(self.double_down(player)),
            "surrender" => Ok(self.surrender(player)),
            _ => Err(BlackjackGameError::new("option not available".to_string())),
        }
    }

    /// Getter method for the dealers face up card.
    pub fn dealers_face_up_card(&self) -> Rc<Card> {
        Rc::clone(&self.dealers_hand.hand[0])
    }

    /// Method for reseting the table for another round, does not reshuffle deck.
    pub fn reset(&mut self) {
        self.final_cards.clear();
        self.dealers_hand.reset();
        self.num_player_blackjacks = 0;
    }

    //TODO: implement surrender functionality eventually
    pub fn surrender<S: Strategy>(&self, player: &mut PlayerSim<S>) {}
}

#[test]
fn test_single_hand() {
    let betting_strategy = MarginBettingStrategy::new(3.0, 5);
    let decision_strategy = BasicStrategy::new();
    let hilo = HiLo::new(6, 5, betting_strategy, decision_strategy);
    let mut player = PlayerSim::new(500.0, hilo);
    let mut table = <BlackjackTableSim as BlackjackTable<
        PlayerSim<HiLo<MarginBettingStrategy, BasicStrategy>>,
    >>::new(f32::MAX, 6, 7);

    // Get the bet from the player and place a bet
    let bet = if let Ok(b) = player.bet() {
        b
    } else {
        panic!("player returned a bet of 0");
    };
    player.place_bet(bet as f32);

    // Display the player struct for debuggin purposes
    println!("{}", player);

    table.deal_hand(&mut player);

    println!("{}", player);

    // Display dealers hand for debugging purposes
    println!("dealers_hand: {:?}", table.dealers_hand.hand);
    println!("dealers_hand_value: {:?}", table.dealers_hand.hand_value);
    println!();

    if player.turn_is_over() || !player.continue_play(5) {
        println!("ended early, either player or dealer has blackjack");
        return;
    }

    // Get the options from the player
    let options = player.get_playing_options();

    println!("playing options = {:?}", options);

    let decision_result = player.decide_option(Rc::clone(&table.dealers_hand.hand[0]));

    if decision_result.is_ok() {
        println!("option chosen = {}", decision_result.as_ref().ok().unwrap());
    } else {
        panic!("player did not choose a valid option");
    }

    println!();

    // Play the current option
    if let Err(e) = table.play_option(&mut player, decision_result.unwrap()) {
        println!("error occurred: {e}");
        panic!();
    }

    // Display state of player
    println!("{}", player);

    assert!(true);
}

#[test]
fn test_single_hand_loop() {
    let betting_strategy = MarginBettingStrategy::new(3.0, 5);
    let decision_strategy = BasicStrategy::new();
    let hilo = HiLo::new(6, 5, betting_strategy, decision_strategy);
    let mut player = PlayerSim::new(500.0, hilo);
    let mut table = <BlackjackTableSim as BlackjackTable<
        PlayerSim<HiLo<MarginBettingStrategy, BasicStrategy>>,
    >>::new(f32::MAX, 6, 7);

    // Get bet from player
    let bet = match player.bet() {
        Ok(b) if b >= 5 => b,
        Ok(b) => {
            eprintln!("error: {b} is not a valid bet with a minimum bet of 5");
            return ();
        }
        Err(e) => {
            eprintln!("error: {e}");
            return ();
        }
    };

    player.place_bet(bet as f32);

    // Display player
    println!("{}", player);
    println!();

    table.deal_hand(&mut player);

    println!("{}", player);
    println!();

    while !player.turn_is_over() {
        println!("dealers_hand: {:?}", table.dealers_hand.hand);
        println!("dealers_hand_value: {:?}", table.dealers_hand.hand_value);
        println!();

        if player.turn_is_over() || !player.continue_play(5) {
            println!("ended early, either player or dealer has blackjack");
            return ();
        }

        // Get options
        let options = player.get_playing_options();
        println!("options: {:?}", options);

        let decision_result = player.decide_option(Rc::clone(&table.dealers_hand.hand[0]));

        let decision = match decision_result {
            Ok(d) => {
                println!("chosen option: {d}");
                d
            }
            Err(e) => {
                eprintln!("error: {e}");
                return ();
            }
        };

        println!();

        if let Err(e) = table.play_option(&mut player, decision) {
            eprintln!("error: {e}");
            return ();
        }

        // Display player again for debugging
        println!("{}", player);

        println!();
    }

    // Display player again
    println!("{}", player);
    println!();

    table.finish_hand(&mut player);

    println!("{}", player);
    println!();

    println!("dealers_hand: {:?}", table.dealers_hand.hand);
    println!("dealers_hand_value: {:?}", table.dealers_hand.hand_value);

    println!("bets_log: {:?}", table.hand_log);
    // // Display player
    // println!("{}", player);
    // println!();

    // table.deal_hand(&mut player);

    // println!("{}", player);
    // println!();

    // println!("dealers_hand: {:?}", table.dealers_hand.hand);
    // println!("dealers_hand_value: {:?}", table.dealers_hand.hand_value);
    // println!();

    // if player.turn_is_over() || !player.continue_play(5) {
    //     println!("ended early, either player or dealer has blackjack");
    //     return;
    // }

    // // Get options
    // let options = player.get_playing_options();
    // println!("options: {:?}", options);

    // let decision_result = player.decide_option(Rc::clone(&table.dealers_hand.hand[0]));

    // let decision = match decision_result {
    //     Ok(d) => {
    //         println!("chosen option: {d}");
    //         d
    //     }
    //     Err(e) => {
    //         eprintln!("error: {e}");
    //         return ();
    //     }
    // };

    // println!();

    // if let Err(e) = table.play_option(&mut player, options, decision) {
    //     eprintln!("error: {e}");
    //     return ();
    // }

    // // Display player again for debugging
    // println!("{}", player);

    // println!();

    assert!(true);
}
