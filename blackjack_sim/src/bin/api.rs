use actix_web::{
    body::{self, BoxBody},
    error, get,
    http::{header::ContentType, StatusCode},
    post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use blackjack_sim::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard};

#[derive(Serialize, Deserialize)]
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
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            UserError::InternalError => write!(f, "{}", "an internal error occured"),
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
        }
    }
}

/// Helper function to create a counting strategy i.e. a `CountingStrategy` trait object at runtime.
fn create_counting_strategy<S: AsRef<str>>(
    name: S,
    num_decks: u32,
) -> Result<Box<dyn CountingStrategy + 'static>, &'static str> {
    let counting_strategy: Box<dyn CountingStrategy> = match name.as_ref() {
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
) -> Result<Box<dyn DecisionStrategy + 'static>, &'static str> {
    let decision_strategy: Box<dyn DecisionStrategy> = match name.as_ref() {
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
) -> Result<Box<dyn BettingStrategy + 'static>, &'static str> {
    let betting_strategy: Box<dyn BettingStrategy> = match name.as_ref() {
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
) -> Result<Box<dyn Strategy + 'static>, &'static str> {
    let counting_strategy = create_counting_strategy(counting_strategy, num_decks)?;
    let decision_strategy = create_decision_strategy(decision_strategy)?;
    let betting_strategy = create_betting_strategy(betting_strategy, margin, min_bet)?;
    Ok(Box::new(
        PlayerStrategyDyn::new()
            .counting_strategy(counting_strategy)
            .decision_strategy(decision_strategy)
            .betting_strategy(betting_strategy)
            .build(),
    ))
}

/// A handler that will configure, and build a new `MulStrategyBlackjackSimulator` using the given parameters the body of the request
#[post("/config-game-params")]
async fn configure_simulation_parameters(
    params: web::Json<GameConfig>,
    app_sim: web::Data<Mutex<Option<MulStrategyBlackjackSimulator>>>,
) -> Result<HttpResponse, UserError> {
    let config = BlackjackSimulatorConfig::from(params.into_inner());
    let mut guard = if let Ok(g) = app_sim.lock() {
        g
    } else {
        return Err(UserError::InternalError);
    };

    *guard = Some(MulStrategyBlackjackSimulator::new(config).build());
    Ok(HttpResponse::Ok().finish())
}

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

        let strategy = if let Ok(s) = create_strategy(
            counting_strategy,
            decision_strategy,
            betting_strategy,
            num_decks as u32,
            min_bet,
            margin,
        ) {
            simulator.add_simulation(s);
            return Ok(HttpResponse::Ok().finish());
        };
    }

    return Err(UserError::InternalError);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    Ok(())
}
