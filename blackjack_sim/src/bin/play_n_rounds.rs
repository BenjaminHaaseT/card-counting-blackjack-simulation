use blackjack_lib::BlackjackTable;
use blackjack_sim::sim::game::player::{self, PlayerSim};
use blackjack_sim::sim::game::strategy::{
    self, BasicStrategy, BettingStrategy, DecisionStrategy, HiLo, MarginBettingStrategy, Strategy,
    TableState,
};
use blackjack_sim::sim::game::table::{self, BlackjackTableSim};
use std::f32::MIN;
use std::rc::Rc;

fn main() {
    let numb_rounds_result = if let Some(n) = std::env::args().skip(1).next() {
        n.parse::<u32>()
    } else {
        eprintln!("usage: play_n_rounds ROUNDS");
        std::process::exit(1);
    };

    let mut numb_rounds = match numb_rounds_result {
        Ok(n) => n,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    const MIN_BET: u32 = 5;
    let betting_strategy = MarginBettingStrategy::new(3.0, MIN_BET);
    let decision_strategy = BasicStrategy::new();
    let hilo = HiLo::new(6, MIN_BET, betting_strategy, decision_strategy);
    let mut player = PlayerSim::new(500.0, hilo);
    let mut table = <BlackjackTableSim as BlackjackTable<
        PlayerSim<HiLo<MarginBettingStrategy, BasicStrategy>>,
    >>::new(f32::MAX, 6, 7);

    let (mut total_wins, mut total_pushes, mut total_losses, mut total_winnings) = (0, 0, 0, 0.0);

    while numb_rounds > 0 && player.continue_play(MIN_BET) {
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

        table.deal_hand(&mut player);

        println!("{}", player);
        println!();

        while !player.turn_is_over() {
            println!("dealers_hand: {:?}", table.dealers_hand.hand);
            println!("dealers_hand_value: {:?}", table.dealers_hand.hand_value);
            println!();

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

        table.finish_hand(&mut player);

        println!("{}", player);
        println!();

        println!("dealers_hand: {:?}", table.dealers_hand.hand);
        println!("dealers_hand_value: {:?}", table.dealers_hand.hand_value);

        println!("bets_log: {:?}", table.hand_log);

        player.reset();
        table.reset();

        if let Some((wins, pushes, loses, winnings)) = table.hand_log {
            total_wins += wins;
            total_pushes += pushes;
            total_losses += loses;

            total_winnings += winnings;
        }

        numb_rounds -= 1;
    }

    let width = "number of player blackjacks:".len() + 20;
    let numeric_display_width = 80 - width;
    println!("{}", "-".repeat(80));
    println!("{:-^80}", "stats");
    let result_str = format!(
        "{:<width$}{:>numeric_display_width$}\n{:<width$}{:>numeric_display_width$}\n{:<width$}{:>numeric_display_width$}\n{:<width$}{:>numeric_display_width$}",
        "total wins:",
        total_wins,
        "total pushes:",
        total_pushes,
        "total losses:",
        total_losses,
        "total winnings:",
        total_winnings
    );
    println!("{result_str}");
    println!(
        "{:<width$}{:>numeric_display_width$.2}",
        "final balance",
        player.balance()
    );
    println!(
        "{:<width$}{:>numeric_display_width$}",
        "number of player blackjacks:", table.num_player_blackjacks,
    );
    println!("{}", "-".repeat(80));
}
