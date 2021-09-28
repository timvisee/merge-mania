// TODO: remove this before release
#![allow(unused)]

#[macro_use]
extern crate log;

pub(crate) mod auth;
pub(crate) mod client;
pub(crate) mod config;
pub(crate) mod game;
pub(crate) mod lang;
pub(crate) mod routes;
pub(crate) mod state;
#[cfg(test)]
pub mod tests;
pub(crate) mod types;
pub(crate) mod util;
pub(crate) mod web;
pub(crate) mod ws;

use state::{SharedState, State};

/// Web server host.
pub const HOST: ([u8; 4], u16) = ([0, 0, 0, 0], 8000);

/// Config path.
pub const CONFIG_PATH: &str = "./../config/config.toml";

/// Inventory width/height.
pub const INV_WIDTH: u16 = 8;

/// Inventory slot count.
pub const INV_SIZE: u16 = INV_WIDTH.pow(2);

/// Number of milliseconds for each tick.
pub const TICK_MILLIS: u64 = 1000;

/// Main entrypoint.
fn main() {
    // Initialize logging
    dotenv::dotenv();
    pretty_env_logger::init();

    let state = state();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let server = crate::web::server(state.clone());
            let game_loop = crate::game::run(state);

            // Run server and game loop
            futures::future::select(Box::pin(server), Box::pin(game_loop)).await;
        })
}

/// Load shared state.
fn state() -> SharedState {
    info!("Initializing global state...");
    let config = config::load().expect("failed to load game config");

    let mut state = State::new(config);
    state.game.running = true;

    state.shared()
}
