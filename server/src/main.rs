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

use std::pin::Pin;

use futures::future::Future;

use state::{SharedState, State};

/// Web server host.
pub const HOST: ([u8; 4], u16) = ([0, 0, 0, 0], 8000);

/// Config path.
pub const CONFIG_PATH: &str = "./../config/config.toml";

/// Inventory width/height.
pub const INV_WIDTH: u16 = 8;

/// Inventory slot count.
pub const INV_SIZE: u16 = INV_WIDTH.pow(2);

/// Sessions file path.
pub const SESSIONS_SAVE_PATH: &str = "save.sessions.json";

/// Game file path.
pub const GAME_SAVE_PATH: &str = "save.game.json";

/// Game autosave interval.
pub const GAME_SAVE_INTERVAL_SEC: u64 = 60;

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
            let game_loop = crate::game::run(state.clone());
            let quit_signal = quit_signal();

            type FutureType = Pin<Box<dyn Future<Output = ()>>>;
            let server: FutureType = Box::pin(server);
            let game_loop: FutureType = Box::pin(game_loop);
            let quit_signal: FutureType = Box::pin(quit_signal);

            futures::future::select_all([server, game_loop, quit_signal]).await;

            // Save game state before we quit
            if let Err(err) = state.game.save() {
                error!("Failed to save game state before quitting, this will lead to data loss");
            }
        })
}

/// Load shared state.
fn state() -> SharedState {
    info!("Initializing global state...");
    let config = config::load().expect("failed to load game config");

    let start = config.game.start;
    let mut state = State::new(config);

    // Start new games if configured
    if start && state.game.tick() == 0 {
        state.game.set_running(true);
    }

    state.shared()
}

/// Quit signal handler.
async fn quit_signal() {
    tokio::signal::ctrl_c().await;
    info!("Received quit signal");
}
