pub mod game;
pub mod write;

use blackjack_lib::{BlackjackTable, Card, Deck};
pub use game::prelude::*;
use game::strategy::CountingStrategy;
use prelude::PlayerStrategyDyn;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;
use std::iter::FromIterator;
use std::sync::mpsc::{self, channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

use strategy::{
    BasicStrategy, BettingStrategy, DecisionStrategy, HiLo, MarginBettingStrategy, Strategy,
};

pub mod prelude {
    pub use super::{
        strategy::prelude::*, BlackjackSimulation, BlackjackSimulator, BlackjackSimulatorConfig,
        BlackjackSimulatorConfigBuilder, MulStrategyBlackjackSimulator,
        MulStrategyBlackjackSimulatorBuilder, SimulationError,
    };
}

/// Simple struct for recording all of the interesting data points accumulated during a simulation
pub struct SimulationSummary {
    pub wins: i32,
    pub pushes: i32,
    pub losses: i32,
    pub early_endings: i32,
    pub winnings: f32,
    pub num_hands: u32,
    pub player_blackjacks: i32,
    pub label: String,
}

impl Display for SimulationSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const width: usize = 80;
        const text_width: usize = "number of player blackjacks".len() + 20;
        const num_width: usize = width - text_width;
        let total_hands = self.wins + self.losses + self.pushes;
        let body = format!(
            "{}{}\n\
        {:<text_width$}{:>num_width$}\n\
        {:<text_width$}{:>num_width$}\n\
        {:<text_width$}{:>num_width$}\n\
        {:<text_width$}{:>num_width$.2}\n\
        {:<text_width$}{:>num_width$}\n\
        {:<text_width$}{:>num_width$}\n\
        {:<text_width$}{:>num_width$}\n\
        {:<text_width$}{:>num_width$.2}\n\
        {:<text_width$}{:>num_width$.2}\n\
        {:<text_width$}{:>num_width$.2}\n\
        {:<text_width$}{:>num_width$.2}\n",
            "strategy: ",
            self.label,
            "hands won",
            self.wins,
            "hands pushed",
            self.pushes,
            "hands lost",
            self.losses,
            "winnings",
            self.winnings,
            "number of player blackjacks",
            self.player_blackjacks,
            "number of early endings",
            self.early_endings,
            "total hands played",
            total_hands,
            "win percentage",
            (self.wins as f32) / (total_hands as f32),
            "push percentage",
            (self.pushes as f32) / (total_hands as f32),
            "loss percentage",
            (self.losses as f32) / (total_hands as f32),
            "average winnings per hand",
            self.winnings / (total_hands as f32)
        );
        write!(f, "{}", body)
    }
}

#[derive(Debug)]
pub enum SimulationError {
    GameError(String),
    SendingError(String),
    WriteError(String),
}

impl Display for SimulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulationError::GameError(s)
            | SimulationError::SendingError(s)
            | SimulationError::WriteError(s) => write!(f, "{}", s),
        }
    }
}

impl Error for SimulationError {}
pub trait BlackjackSimulation: Send {
    /// Required method, the method that will be called to run all simulations.
    fn run(&mut self) -> Result<(), BlackjackGameError>;
    ///Required method, the method that will be called to run a single simulation.
    fn run_single_simulation(&mut self) -> Result<(), BlackjackGameError>;
    /// Required method, the method that will display the stats recorded for a given simulation.
    fn display_stats(&self);
    /// Required method, the method that will reset the simulation
    fn reset(&mut self);
    /// Required method, the method for producing output statistics/data recorded during the simulation
    fn summary(&self) -> SimulationSummary;
}

/// Struct for running a number of simulations for a specific strategy.
/// This struct has the functionality for recording and writing specific stats of interest from the simulation as well.
/// A `BlackjackSimulator` object's main purupose is conveince, it acts as a wrapper for all the other structs needed to run a blackjack simulation to test a specific strategy.
/// It allows the user of the object to control specific parameters of a typical blackjack game that one would find at a casino,
/// such as number of decks, the counting strategy, number of shuffles, minimum bet, etc...
pub struct BlackjackSimulator<S>
where
    S: Strategy,
{
    game: BlackjackGameSim<S>,
    player_starting_balance: f32,
    table_starting_balance: f32,
    num_simulations: u32,
    hands_per_simulation: u32,
    accumulated_wins: i32,
    accumulated_pushes: i32,
    accumulated_losses: i32,
    accumulated_winnings: f32,
    num_early_endings: i32,
    num_player_blackjacks: i32,
    silent: bool,
}

impl<S: Strategy> BlackjackSimulator<S> {
    pub fn new(
        strategy: S,
        player_starting_balance: f32,
        table_starting_balance: f32,
        num_simulations: u32,
        num_decks: usize,
        num_shuffles: u32,
        min_bet: u32,
        hands_per_simulation: u32,
        silent: bool,
        surrender: bool,
        soft_seventeen: bool,
        insurance: bool,
    ) -> Self {
        let player = PlayerSim::new(player_starting_balance, strategy, surrender);
        // let table = <BlackjackTableSim as BlackjackTable<PlayerSim<S>>>::new(
        //     table_starting_balance,
        //     num_decks,
        //     num_shuffles,
        //     soft_seventeen,
        // );
        let table = BlackjackTableSim::new(
            table_starting_balance,
            num_decks,
            num_shuffles,
            soft_seventeen,
            insurance,
        );
        let game = BlackjackGameSim::new(table, player, hands_per_simulation, min_bet);
        Self {
            game,
            player_starting_balance,
            table_starting_balance,
            num_simulations,
            hands_per_simulation,
            accumulated_wins: 0,
            accumulated_pushes: 0,
            accumulated_losses: 0,
            accumulated_winnings: 0.0,
            num_early_endings: 0,
            num_player_blackjacks: 0,
            silent,
        }
    }
}

impl<S: Strategy + Send> BlackjackSimulation for BlackjackSimulator<S> {
    /// Method that will run the simulation, recording the necessary data. Returns a `Result<(), BlackjackGameError> if an error occurs during any simulation.
    fn run(&mut self) -> Result<(), BlackjackGameError> {
        // Run the simulation
        for i in 0..self.num_simulations {
            if let Err(e) = self.game.run() {
                return Err(e);
            }
            // Record data from simulation
            self.accumulated_wins += self.game.total_wins;
            self.accumulated_pushes += self.game.total_pushes;
            self.accumulated_losses += self.game.total_losses;
            self.accumulated_winnings += self.game.total_winnings;
            self.num_player_blackjacks += self.game.num_player_blackjacks;
            if self.game.ended_early {
                self.num_early_endings += 1;
            }
            if !self.silent {
                println!("simulation #{}", i + 1);
                self.game.display_stats();
            }

            // Reset balances for next simulation
            self.game
                .reset(self.table_starting_balance, self.player_starting_balance);
        }
        Ok(())
    }

    /// Method to run a single simulation. The state of the simulation is not reset afterwards, nor is any output displayed to the console.
    fn run_single_simulation(&mut self) -> Result<(), BlackjackGameError> {
        if let Err(e) = self.game.run() {
            return Err(e);
        }
        // Record the data from the simulation
        self.accumulated_wins += self.game.total_wins;
        self.accumulated_pushes += self.game.total_pushes;
        self.accumulated_losses += self.game.total_losses;
        self.accumulated_winnings += self.game.total_winnings;
        self.num_player_blackjacks += self.game.num_player_blackjacks;
        if self.game.ended_early {
            self.num_early_endings += 1;
        }
        if !self.silent {
            self.game.display_stats();
        }
        Ok(())
    }

    /// Method that will display the accumulated data recorded from running all simulations.
    fn display_stats(&self) {
        const width: usize = 80;
        const text_width: usize = "number of player blackjacks:".len() + 20;
        const numeric_width: usize = width - text_width;

        println!("{}", "-".repeat(width));
        println!(
            "{:-^width$}",
            format!("running {} simulations", self.num_simulations)
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "total wins:", self.accumulated_wins
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "total pushes:", self.accumulated_pushes
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "total losses:", self.accumulated_losses
        );
        println!(
            "{:<text_width$}{:>numeric_width$.2}",
            "total winnings:", self.accumulated_winnings
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "number of player blackjacks:", self.num_player_blackjacks
        );
        println!(
            "{:<text_width$}{:>numeric_width$}",
            "number of early endings", self.num_early_endings
        );
        println!("{}", "-".repeat(width));
    }

    /// Method to get a `SimulationSummary` object derived from the current data recorded in `self`.
    fn summary(&self) -> SimulationSummary {
        SimulationSummary {
            wins: self.accumulated_wins,
            losses: self.accumulated_losses,
            pushes: self.accumulated_pushes,
            early_endings: self.num_early_endings,
            winnings: self.accumulated_winnings,
            num_hands: self.num_simulations * self.hands_per_simulation,
            player_blackjacks: self.num_player_blackjacks,
            label: self.game.label(),
        }
    }

    /// Method for reseting the state of the simulation, so it can be run again.
    /// Note that a simulation must be reset before running another simulation, otherwise the data produced is not meaningful.
    fn reset(&mut self) {
        self.game
            .reset(self.table_starting_balance, self.player_starting_balance);
    }
}

/// This struct is for testing multiple strategies at once, designed to give the use options to customize different parameters of the
/// game while testing multiple strategies. Tests each strategy in parallel to speed up computation.
pub struct MulStrategyBlackjackSimulator {
    simulations: Vec<Box<dyn BlackjackSimulation>>,
    pub config: BlackjackSimulatorConfig,
}

impl MulStrategyBlackjackSimulator {
    /// Method that returns a new `MulStrategyBlackjackSimulatorBuilder` object.
    pub fn new(config: BlackjackSimulatorConfig) -> MulStrategyBlackjackSimulatorBuilder {
        MulStrategyBlackjackSimulatorBuilder {
            simulations: None,
            config: config,
        }
    }

    /// The method that will run each of the strategies in a configured simulation. Each strategy gets tested in a new thread,
    /// the output of each simulation gets sent to the stats module for writing a summary of results to a chosen destination.
    pub fn run(
        &mut self,
        file_out: Box<dyn Write + Send + 'static>,
    ) -> Result<(), SimulationError> {
        // Open channel
        let (write_sender, write_receiver) = mpsc::channel::<(Option<SimulationSummary>, usize)>();

        // Collect thread handles
        let mut handles = vec![];
        self.simulations.reverse();
        let mut id = 1usize;

        // Create unique id's for each simulation, that way the writing thread knows when one simulation is done
        let ids = HashSet::from_iter(1..=self.simulations.len());

        // Spawn thread for writing recorded information
        let write_handle =
            thread::spawn(move || write::write_summaries(write_receiver, ids, file_out));

        while let Some(mut simulation) = self.simulations.pop() {
            // Clone the sender to the write_receiver
            let write_sender_clone = write_sender.clone();
            let num_simulations = self.config.num_simulations;

            // Spawn the thread for each simulation
            let handle = thread::spawn(move || {
                for _i in 0..num_simulations {
                    if let Err(e) = simulation.run_single_simulation() {
                        return Err(SimulationError::GameError(e.message));
                    }
                    // record data from simulation
                    let summary = simulation.summary();
                    // send data to stats module
                    if let Err(e) = write_sender_clone.send((Some(summary), id)) {
                        return Err(SimulationError::SendingError(format!("{}", e)));
                    }
                    // reset simulation
                    simulation.reset();
                }
                // Tell the stats thread we are finished with this simulation
                if let Err(e) = write_sender_clone.send((None, id)) {
                    return Err(SimulationError::SendingError(format!("{}", e)));
                }
                Ok(())
            });

            handles.push(handle);
            id += 1;
        }

        for (i, handle) in handles.into_iter().enumerate() {
            if let Err(e) = handle.join().unwrap() {
                eprintln!("error occured for simulation #{i}");
                return Err(e);
            }
        }

        // Make sure write_handle has finished as well
        if let Err(e) = write_handle.join().unwrap() {
            return Err(SimulationError::WriteError(format!("{}", e)));
        }

        Ok(())
    }

    /// A method for adding a simulation to the simulator, takes `strategy` and then creates a new simulation which is represented as trait object of type `BlackjackSimulation`,
    ///  the adding it to `self.simulations`.
    pub fn add_simulation(&mut self, strategy: Box<PlayerStrategyDyn>) {
        // Create trait object
        let simulation: Box<dyn BlackjackSimulation> = Box::new(BlackjackSimulator::new(
            strategy,
            self.config.player_starting_balance,
            self.config.table_starting_balance,
            self.config.num_simulations,
            self.config.num_decks,
            self.config.num_shuffles,
            self.config.min_bet,
            self.config.hands_per_simulation,
            self.config.silent,
            self.config.surrender,
            self.config.soft_seventeen,
            self.config.insurance,
        ));
        self.simulations.push(simulation);
    }
}

unsafe impl Send for MulStrategyBlackjackSimulator {}
/// Struct for building a `MulStrategyBlackjackSimulator` object
pub struct MulStrategyBlackjackSimulatorBuilder {
    simulations: Option<Vec<Box<dyn BlackjackSimulation>>>,
    config: BlackjackSimulatorConfig,
}

impl MulStrategyBlackjackSimulatorBuilder {
    /// Method for adding a new simulation to the vector of simulations, the only required input is struct that implements the `Strategy` trait,
    /// the rest of the configurations for the simulation are taken from the preset `BlackjackSimulatorConfig` object that was passed during object creation.
    pub fn simulation<S: Strategy + Send + 'static>(&mut self, strategy: S) -> &mut Self {
        let simulation = Box::new(BlackjackSimulator::new(
            strategy,
            self.config.player_starting_balance,
            self.config.table_starting_balance,
            self.config.num_simulations,
            self.config.num_decks,
            self.config.num_shuffles,
            self.config.min_bet,
            self.config.hands_per_simulation,
            self.config.silent,
            self.config.surrender,
            self.config.soft_seventeen,
            self.config.insurance,
        ));
        if let Some(ref mut sim_vec) = self.simulations {
            sim_vec.push(simulation);
        } else {
            self.simulations = Some(vec![simulation]);
        }
        self
    }

    /// Method that builds a `MulStrategyBlackjackSimulator` object
    pub fn build(&mut self) -> MulStrategyBlackjackSimulator {
        MulStrategyBlackjackSimulator {
            simulations: self.simulations.take().unwrap_or(vec![]),
            config: self.config,
        }
    }
}

/// Struct for configuring a single `BlackjackSimulator` object
#[derive(Clone, Copy)]
pub struct BlackjackSimulatorConfig {
    pub player_starting_balance: f32,
    pub table_starting_balance: f32,
    pub num_simulations: u32,
    pub num_decks: usize,
    pub num_shuffles: u32,
    pub min_bet: u32,
    pub hands_per_simulation: u32,
    pub silent: bool,
    pub surrender: bool,
    pub soft_seventeen: bool,
    pub insurance: bool,
}

impl BlackjackSimulatorConfig {
    /// Associated method for returning a new `BlackjackSimulatorConfigBuilder` object. Allows customization of the BlackjackSimulator
    /// i.e. allows the user to choose the hyperparameters of the blackjack simulation such as the players starting balance, the number of simulations run,
    /// the minimum bet per hand, and how many decks are used.
    pub fn new() -> BlackjackSimulatorConfigBuilder {
        BlackjackSimulatorConfigBuilder {
            player_starting_balance: None,
            table_starting_balance: None,
            num_simulations: None,
            num_decks: None,
            num_shuffles: None,
            min_bet: None,
            hands_per_simulation: None,
            silent: None,
            surrender: None,
            soft_seventeen: None,
            insurance: None,
        }
    }
}

impl Default for BlackjackSimulatorConfig {
    /// Returns the standard configurations for a game of blackjack.
    fn default() -> Self {
        BlackjackSimulatorConfig::new().build()
    }
}

/// Struct to implement builder pattern for `BlackjackSimulatorConfig`
#[derive(Clone, Copy)]
pub struct BlackjackSimulatorConfigBuilder {
    player_starting_balance: Option<f32>,
    table_starting_balance: Option<f32>,
    num_simulations: Option<u32>,
    num_decks: Option<usize>,
    num_shuffles: Option<u32>,
    min_bet: Option<u32>,
    hands_per_simulation: Option<u32>,
    silent: Option<bool>,
    surrender: Option<bool>,
    soft_seventeen: Option<bool>,
    insurance: Option<bool>,
}

impl BlackjackSimulatorConfigBuilder {
    /// Method for changing the starting balance of the player.
    pub fn player_starting_balance(&mut self, balance: f32) -> &mut Self {
        self.player_starting_balance = Some(balance);
        self
    }

    /// Method for changing the starting balance of the table
    pub fn table_starting_balance(&mut self, balance: f32) -> &mut Self {
        self.table_starting_balance = Some(balance);
        self
    }

    /// Method for settign the number of simulations run.
    pub fn num_simulations(&mut self, n: u32) -> &mut Self {
        self.num_simulations = Some(n);
        self
    }

    /// Method for choosing the number of decks used in the game
    pub fn num_decks(&mut self, decks: usize) -> &mut Self {
        self.num_decks = Some(decks);
        self
    }

    /// Method for setting the number of shuffles when shuffling is needed during the simulation
    pub fn num_shuffles(&mut self, shuffles: u32) -> &mut Self {
        self.num_shuffles = Some(shuffles);
        self
    }

    /// Method for setting the minimum bet for the game
    pub fn min_bet(&mut self, bet: u32) -> &mut Self {
        self.min_bet = Some(bet);
        self
    }

    /// Method for setting the maximum number of hands that will be played for each simulation
    pub fn hands_per_simulation(&mut self, hands: u32) -> &mut Self {
        self.hands_per_simulation = Some(hands);
        self
    }

    /// Method for setting a boolean flag, if set to false the `BlackjackSimulator` that is configured with these configurations will display its summary
    ///  output for each simulation run, otherwise it will remain silent.
    pub fn silent(&mut self, silent: bool) -> &mut Self {
        self.silent = Some(silent);
        self
    }

    /// Method for setting a flag that determines if the game allows surrender or not
    pub fn surrender(&mut self, surrender: bool) -> &mut Self {
        self.surrender = Some(surrender);
        self
    }

    /// Method for setting the flag that determines if the dealer must hit soft seventeens, default is false
    pub fn soft_seventeen(&mut self, seventeen: bool) -> &mut Self {
        self.soft_seventeen = Some(seventeen);
        self
    }

    /// Method for setting the flag that determines if the game allows insurance bets to be taken. If insurance is set to true,
    /// insurance bets are allowed to be placed only if the dealer's up card is an ace.
    pub fn insurance(&mut self, insurance: bool) -> &mut Self {
        self.insurance = Some(insurance);
        self
    }

    /// Method for building a `BlackjackSimulatorCofig` object from the given `BlackjackSimulatorConfigBuilder` object.
    pub fn build(&mut self) -> BlackjackSimulatorConfig {
        BlackjackSimulatorConfig {
            player_starting_balance: self.player_starting_balance.unwrap_or(500.0),
            table_starting_balance: self.table_starting_balance.unwrap_or(f32::MAX),
            num_simulations: self.num_simulations.unwrap_or(100),
            num_decks: self.num_decks.unwrap_or(6),
            num_shuffles: self.num_shuffles.unwrap_or(7),
            min_bet: self.min_bet.unwrap_or(5),
            hands_per_simulation: self.hands_per_simulation.unwrap_or(50),
            silent: self.silent.unwrap_or(true),
            surrender: self.surrender.unwrap_or(true),
            soft_seventeen: self.soft_seventeen.unwrap_or(false),
            insurance: self.insurance.unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strategy::{
        BasicStrategy, BettingStrategy, CountingStrategy, DecisionStrategy, HiLo,
        MarginBettingStrategy, PlayerStrategy, Strategy, WongHalves, KO,
    };

    #[test]
    fn simple_simulation_test() {
        const MIN_BET: u32 = 5;
        const NUM_DECKS: u32 = 6;
        let counting_strategy = KO::new(NUM_DECKS);
        let decision_strategy = BasicStrategy::new();
        let betting_strategy = MarginBettingStrategy::new(3.0, MIN_BET);
        let strategy = PlayerStrategy::new(counting_strategy, decision_strategy, betting_strategy);

        let mut simulator = BlackjackSimulator::new(
            strategy,
            500.0,
            f32::MAX,
            50,
            6,
            7,
            MIN_BET,
            400,
            false,
            true,
            false,
            false,
        );

        if let Err(e) = simulator.run() {
            panic!("error: {}", e);
        }

        simulator.display_stats();
        assert!(true);
    }

    #[test]
    fn run_multiple_simulations() {
        let mut simulator = MulStrategyBlackjackSimulator::new(BlackjackSimulatorConfig::default())
            .simulation(PlayerStrategy::new(
                KO::new(6),
                BasicStrategy::new(),
                MarginBettingStrategy::new(3.0, 5),
            ))
            .simulation(PlayerStrategy::new(
                WongHalves::new(6),
                BasicStrategy::new(),
                MarginBettingStrategy::new(3.0, 5),
            ))
            .simulation(PlayerStrategy::new(
                HiLo::new(6),
                BasicStrategy::new(),
                MarginBettingStrategy::new(3.0, 5),
            ))
            .build();

        if let Err(e) = simulator.run(Box::new(std::io::stdout())) {
            eprintln!("{}", e);
            panic!();
        }

        // test passed if we get to this point
        assert!(true);
    }
}
