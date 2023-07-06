use blackjack_sim::strategy::{
    AceFive, BasicStrategy, CountingStrategy, Halves, HiLo, HiOptI, HiOptII, JNoir,
    MarginBettingStrategy, OmegaII, PlayerStrategy, RedSeven, S17DeviationStrategy, SilverFox,
    UnbalancedZen2, WongHalves, ZenCount, KISS, KISSII, KISSIII, KO,
};

use blackjack_sim::{
    BlackjackSimulatorConfig, BlackjackSimulatorConfigBuilder, MulStrategyBlackjackSimulator,
    MulStrategyBlackjackSimulatorBuilder,
};
use clap::Parser;
use std::fs::File;
use std::io::Write;

#[derive(Parser)]
#[command(name = "Card Counting Simulator")]
#[command(author = "Benjamin Haase")]
#[command(version = "0.1.0")]
#[command(
    about = "Simulates the common card counting strategies, and records/displays the data produced by each simulation"
)]
struct Cli {
    /// Optional argument to set the starting balance of the table
    #[arg(short = 't', long, value_name = "TABLE")]
    table_starting_balance: Option<f32>,

    /// Optional argument, sets the output file name
    #[arg(short = 'f', long, value_name = "FILE")]
    file_out: Option<std::path::PathBuf>,

    /// Sets the players starting balance for each simulation
    #[arg(short = 'p', long, value_name = "PLAYER")]
    player_starting_balance: f32,

    /// Sets the total number of simulations that will be run
    #[arg(short = 'n', long, value_name = "SIMULATIONS")]
    num_simulations: u32,

    /// Sets the number of decks that are used in the blackjack game
    #[arg(short = 'd', long, value_name = "DECKS")]
    num_decks: usize,

    /// Determines the maximum number of hands played for any given simulation
    #[arg(short = 'r', long, value_name = "HANDS")]
    hands_per_simulation: u32,

    /// Determines the minimum bet required
    #[arg(short = 'b', long, value_name = "BET")]
    min_bet: u32,

    /// Decides whether or not to display output from each simulation run
    #[arg(short = 'g', long, value_name = "SILENT")]
    silent_game: Option<bool>,

    /// Decides whether surrender is a valid play at the blackjack table
    #[arg(short = 's', long, value_name = "SURRENDER")]
    surrender: bool,

    /// Decides the margin to increase bets by
    #[arg(short = 'm', long, value_name = "MARGIN")]
    betting_margin: Option<f32>,

    /// Decides whether or not the dealer hits on soft seventeens
    #[arg(short = 'e', long, value_name = "SEVENTEEN")]
    soft_seventeen: Option<bool>,
}

fn main() -> std::io::Result<()> {
    // Get command line arguments to
    let cli = Cli::parse();
    // Build configuration for simulation
    let config = BlackjackSimulatorConfig::new()
        .player_starting_balance(cli.player_starting_balance)
        .table_starting_balance(cli.table_starting_balance.unwrap_or(f32::MAX))
        .num_simulations(cli.num_simulations)
        .num_decks(cli.num_decks)
        .hands_per_simulation(cli.hands_per_simulation)
        .min_bet(cli.min_bet)
        .silent(cli.silent_game.unwrap_or(true))
        .surrender(cli.surrender)
        .soft_seventeen(cli.soft_seventeen.unwrap_or(false))
        .build();

    // Get other configurations out of cli
    let out_writer: Box<dyn Write + Send + 'static> = if cli.file_out.is_some() {
        Box::new(File::create(cli.file_out.unwrap())?)
    } else {
        Box::new(std::io::stdout())
    };

    let betting_margin = match cli.betting_margin {
        Some(b) => b,
        None => 2.0,
    };

    let num_decks = cli.num_decks as u32;
    let min_bet = cli.min_bet;

    // Build the simulator
    let mut simulator = MulStrategyBlackjackSimulator::new(config)
        .simulation(PlayerStrategy::new(
            HiLo::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            WongHalves::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            KO::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            RedSeven::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            HiOptI::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            HiOptII::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            AceFive::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            OmegaII::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            ZenCount::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            Halves::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            KISS::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            KISSII::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            KISSIII::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            SilverFox::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            JNoir::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .simulation(PlayerStrategy::new(
            UnbalancedZen2::new(num_decks),
            S17DeviationStrategy::new(),
            MarginBettingStrategy::new(betting_margin, min_bet),
        ))
        .build();

    // Run simulation and check for error
    println!("Running simulations...");

    if let Err(err) = simulator.run(out_writer) {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }

    println!("Simulations complete.");

    Ok(())
}
