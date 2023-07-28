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

fn add_simulation_helper(simulator: &mut MulStrategyBlackjackSimulator, sim_params: SimConfig) {}

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
        // TODO: needs trait object implementation for creating a strategy and then a simulation
        todo!();
    }

    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    Ok(())
}
