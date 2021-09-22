pub(crate) mod auth;
pub(crate) mod config;
pub(crate) mod game;
pub(crate) mod lang;
pub(crate) mod routes;
pub(crate) mod server;
pub(crate) mod state;
#[cfg(test)]
pub mod tests;
pub(crate) mod types;
pub(crate) mod util;

use state::{SharedState, State};

/// Server host.
pub const HOST: ([u8; 4], u16) = ([127, 0, 0, 1], 8000);

/// Config path.
pub const CONFIG_PATH: &str = "./../config/config.toml";

/// Inventory width/height.
pub const INV_WIDTH: u16 = 8;

/// Inventory slot count.
pub const INV_SIZE: u16 = INV_WIDTH * 2;

/// Main entrypoint.
fn main() {
    let state = state();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            crate::server::server(state).await;
        })
}

/// Load shared state.
fn state() -> SharedState {
    let config = config::load().expect("failed to load game config");
    State::new(config).shared()
}
