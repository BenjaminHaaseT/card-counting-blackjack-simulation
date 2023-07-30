use actix_web::{
    body::{self, BoxBody},
    error, get,
    http::{header::ContentType, StatusCode},
    post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use blackjack_sim::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{BufWriter, Write};
use std::sync::mpsc::Receiver;
use std::sync::{Mutex, MutexGuard};

/// A struct for handling the configurations of the game. Meant to be deserialized from JSON.
#[derive(Debug, Deserialize)]
struct GameConfig {
    player_starting_balance: f32,
    table_starting_balance: Option<f32>,
    num_simulations: u32,
    num_decks: usize,
    hands_per_simulation: u32,
    min_bet: u32,
    surrender: bool,
    soft_seventeen: Option<bool>,
    insurance: Option<bool>,
}

impl From<GameConfig> for BlackjackSimulatorConfig {
    fn from(value: GameConfig) -> Self {
        BlackjackSimulatorConfig::new()
            .player_starting_balance(value.player_starting_balance)
            .table_starting_balance(value.table_starting_balance.unwrap_or(f32::MAX))
            .num_simulations(value.num_simulations)
            .num_decks(value.num_decks)
            .hands_per_simulation(value.hands_per_simulation)
            .min_bet(value.min_bet)
            .surrender(value.surrender)
            .soft_seventeen(value.soft_seventeen.unwrap_or(false))
            .insurance(value.insurance.unwrap_or(false))
            .build()
    }
}

/// A struct for deserializing the strategy configuration from json.
#[derive(Deserialize)]
struct SimConfig {
    counting_strategy: String,
    decision_strategy: String,
    betting_strategy: String,
    betting_margin: f32,
}

/// An enum that will handle user facing errors
#[derive(Debug)]
enum UserError {
    InternalError,
    SimulationCreationError(String),
    SimulatorNotCreated,
    BadInput(String),
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::InternalError => write!(f, "{}", "an internal error occured"),
            UserError::SimulationCreationError(ref s) => write!(f, "{}", s),
            UserError::SimulatorNotCreated => write!(
                f,
                "{}",
                "unable to add simulation, a simulator has not been created"
            ),
            UserError::BadInput(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for UserError {}

impl error::ResponseError for UserError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code())
            .content_type(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::SimulationCreationError(_) => StatusCode::BAD_REQUEST,
            UserError::SimulatorNotCreated => StatusCode::BAD_REQUEST,
            UserError::BadInput(_) => StatusCode::BAD_REQUEST,
        }
    }
}

/// A struct for collecting simulation `SimulationSummary` data into something that can deserialize into JSON
#[derive(Serialize)]
struct SimulationSummaryJson {
    pub counting_strategy: String,
    pub wins: i32,
    pub pushes: i32,
    pub losses: i32,
    pub early_endings: i32,
    pub winnings: f32,
    pub num_hands: u32,
    pub player_blackjacks: i32,
    pub total_hands_played: u32,
    pub win_pct: f32,
    pub push_pct: f32,
    pub lose_pct: f32,
    pub avg_winnings_per_hand: f32,
}

impl SimulationSummaryJson {
    fn new(counting_strategy: String) -> Self {
        SimulationSummaryJson {
            counting_strategy,
            wins: 0,
            pushes: 0,
            losses: 0,
            early_endings: 0,
            winnings: 0.0,
            num_hands: 0,
            player_blackjacks: 0,
            total_hands_played: 0,
            win_pct: 0.0,
            push_pct: 0.0,
            lose_pct: 0.0,
            avg_winnings_per_hand: 0.0,
        }
    }
}

unsafe impl Send for SimulationSummaryJson {}

/// A struct for collecting all of the simulation summaries into a format that can be
#[derive(Serialize)]
struct SimulationSummaryMap {
    summaries: HashMap<usize, SimulationSummaryJson>,
}

impl SimulationSummaryMap {
    fn new() -> Self {
        SimulationSummaryMap {
            summaries: HashMap::new(),
        }
    }
}

unsafe impl Send for SimulationSummaryMap {}

/// A function for writing data that can be passed as a write function to the `MulStrategyBlackjackSimulator` run method.
fn write_simulation_summary_as_json(
    receiver: Receiver<(Option<SimulationSummary>, usize)>,
    mut ids: HashSet<usize>,
) -> Result<String, Box<dyn std::error::Error + Send + 'static>> {
    let mut summaries_map = SimulationSummaryMap::new();

    'outer: loop {
        match receiver.recv().unwrap() {
            (Some(cur_summary), id) => {
                let summary = summaries_map
                    .summaries
                    .entry(id)
                    .or_insert(SimulationSummaryJson::new(cur_summary.label));
                summary.wins += cur_summary.wins;
                summary.pushes += cur_summary.pushes;
                summary.losses += cur_summary.losses;
                summary.winnings += cur_summary.winnings;
                summary.player_blackjacks += cur_summary.player_blackjacks;
                summary.early_endings += cur_summary.early_endings;
            }
            (None, id) => {
                // Remove from ids
                ids.remove(&id);
                // Check if we are done processing simulations
                if ids.is_empty() {
                    break 'outer;
                }
            }
        }
    }

    // Compute final statistics
    for (_, v) in &mut summaries_map.summaries {
        let total_hands_played = v.wins + v.pushes + v.losses;
        let win_pct = (v.wins as f32) / (total_hands_played as f32);
        let push_pct = (v.pushes as f32) / (total_hands_played as f32);
        let lose_pct = (v.losses as f32) / (total_hands_played as f32);
        let avg_winnings_per_hand = (v.winnings as f32) / (total_hands_played as f32);
        v.win_pct = win_pct;
        v.push_pct = push_pct;
        v.lose_pct = lose_pct;
        v.avg_winnings_per_hand = avg_winnings_per_hand;
    }

    match serde_json::to_string(&summaries_map) {
        Ok(res) => Ok(res),
        Err(_) => Err(Box::new(UserError::InternalError)),
    }
}

/// Helper function to create a counting strategy i.e. a `CountingStrategy` trait object at runtime.
fn create_counting_strategy<S: AsRef<str>>(
    name: S,
    num_decks: u32,
) -> Result<Box<dyn CountingStrategy + Send + 'static>, &'static str> {
    let counting_strategy: Box<dyn CountingStrategy + Send + 'static> = match name.as_ref() {
        "HiLo" => Box::new(HiLo::new(num_decks)),
        "Wong Halves" => Box::new(WongHalves::new(num_decks)),
        "KO" => Box::new(KO::new(num_decks)),
        "HiOptI" => Box::new(HiOptI::new(num_decks)),
        "HiOptII" => Box::new(HiOptII::new(num_decks)),
        "Red Seven" => Box::new(RedSeven::new(num_decks)),
        "OmegaII" => Box::new(OmegaII::new(num_decks)),
        "AceFive" => Box::new(AceFive::new(num_decks)),
        "Zen Count" => Box::new(ZenCount::new(num_decks)),
        "Halves" => Box::new(Halves::new(num_decks)),
        "KISS" => Box::new(KISS::new(num_decks)),
        "KISSII" => Box::new(KISSII::new(num_decks)),
        "KISSIII" => Box::new(KISSIII::new(num_decks)),
        "JNoir" => Box::new(JNoir::new(num_decks)),
        "Silver Fox" => Box::new(SilverFox::new(num_decks)),
        "Unbalanced Zen 2" => Box::new(UnbalancedZen2::new(num_decks)),
        _ => return Err("counting strategy not recognized"),
    };

    Ok(counting_strategy)
}

/// Helper function to create a decsion strategy i.e. a `DecisionStrategy` trait object at runtime.
fn create_decision_strategy<S: AsRef<str>>(
    name: S,
) -> Result<Box<dyn DecisionStrategy + Send + 'static>, &'static str> {
    let decision_strategy: Box<dyn DecisionStrategy + Send + 'static> = match name.as_ref() {
        "Basic Strategy" => Box::new(BasicStrategy::new()),
        "S17 Deviations" => Box::new(S17DeviationStrategy::new()),
        "H17 Deviations" => Box::new(H17DeviationStrategy::new()),
        _ => return Err("decision strategy not recognized"),
    };

    Ok(decision_strategy)
}

/// Helper function to create a betting strategy at runtime i.e. a `BettingStrategy` trait object.
fn create_betting_strategy<S: AsRef<str>>(
    name: S,
    margin: f32,
    min_bet: u32,
) -> Result<Box<dyn BettingStrategy + Send + 'static>, &'static str> {
    let betting_strategy: Box<dyn BettingStrategy + Send + 'static> = match name.as_ref() {
        "Margin" => Box::new(MarginBettingStrategy::new(margin, min_bet)),
        _ => return Err("betting startegy not recognized"),
    };

    Ok(betting_strategy)
}

/// Helper function to create a `Strategy` trait object at runtime
fn create_strategy<S: AsRef<str>>(
    counting_strategy: S,
    decision_strategy: S,
    betting_strategy: S,
    num_decks: u32,
    min_bet: u32,
    margin: f32,
) -> Result<PlayerStrategyDyn, &'static str> {
    let counting_strategy = create_counting_strategy(counting_strategy, num_decks)?;
    let decision_strategy = create_decision_strategy(decision_strategy)?;
    let betting_strategy = create_betting_strategy(betting_strategy, margin, min_bet)?;
    Ok(PlayerStrategyDyn::new()
        .counting_strategy(counting_strategy)
        .decision_strategy(decision_strategy)
        .betting_strategy(betting_strategy)
        .build())
}

/// A handler that will configure, and build a new `MulStrategyBlackjackSimulator` using the given parameters the body of the request
#[post("/config-game-params")]
async fn configure_simulation_parameters(
    params: web::Json<GameConfig>,
    app_sim: web::Data<Mutex<Option<MulStrategyBlackjackSimulator>>>,
) -> Result<HttpResponse, UserError> {
    // let config = params.into_inner();
    let config = BlackjackSimulatorConfig::from(params.into_inner());
    let mut guard = if let Ok(g) = app_sim.lock() {
        g
    } else {
        return Err(UserError::InternalError);
    };

    *guard = Some(MulStrategyBlackjackSimulator::new(config).build());
    Ok(HttpResponse::Ok().body("simulator created successfully"))
}

/// A handler that will add a simulation to the simulator.
#[post("/add-sim")]
async fn add_simulation(
    sim_params: web::Json<SimConfig>,
    app_sim: web::Data<Mutex<Option<MulStrategyBlackjackSimulator>>>,
) -> Result<HttpResponse, UserError> {
    let mut guard = if let Ok(g) = app_sim.lock() {
        g
    } else {
        return Err(UserError::InternalError);
    };

    if let Some(simulator) = guard.as_mut() {
        let (num_decks, min_bet) = (simulator.config.num_decks, simulator.config.min_bet);
        let (counting_strategy, decision_strategy, betting_strategy, margin) = (
            sim_params.counting_strategy.as_str(),
            sim_params.decision_strategy.as_str(),
            sim_params.betting_strategy.as_str(),
            sim_params.betting_margin,
        );

        match create_strategy(
            counting_strategy,
            decision_strategy,
            betting_strategy,
            num_decks as u32,
            min_bet,
            margin,
        ) {
            Ok(s) => {
                simulator.add_simulation(s);
                return Ok(HttpResponse::Ok().body("simulation added successfully"));
            }
            Err(msg) => return Err(UserError::SimulationCreationError(msg.to_owned())),
        }
    }

    return Err(UserError::SimulatorNotCreated);
}

/// A handler that will run the simulation given the configurations.
/// Will return an error resposne if the game has not been configured and/or no simulations have been added.
#[get("/run-sim")]
async fn run_simulation(
    app_sim: web::Data<Mutex<Option<MulStrategyBlackjackSimulator>>>,
) -> Result<HttpResponse, UserError> {
    // Attempt to lock the mutex
    if let Ok(mut guard) = app_sim.lock() {
        // Check if we have a valid simulator
        if let Some(simulator) = guard.as_mut() {
            if simulator.simulations().is_empty() {
                return Err(UserError::BadInput(String::from(
                    "no simulations have been added, unable to run.",
                )));
            }
            match simulator.run_return_out(Box::new(write_simulation_summary_as_json)) {
                Ok(res_as_json) => {
                    return Ok(HttpResponse::Ok()
                        .content_type(ContentType::json())
                        .body(res_as_json));
                }
                Err(_e) => return Err(UserError::InternalError),
            }
        }
    }

    Err(UserError::InternalError)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address = "127.0.0.1";
    let port = 8080;
    println!("Listenting at {}:{}...", address, port);

    let app_sim: web::Data<Mutex<Option<MulStrategyBlackjackSimulator>>> =
        web::Data::new(Mutex::new(None));

    HttpServer::new(move || {
        App::new()
            .app_data(app_sim.clone())
            .service(configure_simulation_parameters)
            .service(add_simulation)
            .service(run_simulation)
    })
    .bind((address, port))?
    .run()
    .await
}
