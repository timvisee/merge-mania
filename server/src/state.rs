use std::sync::Arc;

use crate::auth::{ClientManager, SessionManager};
use crate::config::Config;
use crate::game::Game;

pub type SharedState = Arc<State>;

/// Shared server state.
pub struct State {
    pub config: Config,
    pub sessions: SessionManager,
    pub clients: ClientManager,
    pub game: Game,
}

impl State {
    /// Construct new state.
    pub fn new(config: Config) -> Self {
        // Load game
        let mut game = if config.game.reset {
            info!("Resetting game state according to configuration");
            Game::default()
        } else {
            Game::load(&config).expect("failed to load game state")
        };

        State {
            config,
            sessions: SessionManager::load().expect("failed to load session manager"),
            clients: ClientManager::new(),
            game,
        }
    }

    /// Transform into shared state.
    pub fn shared(self) -> SharedState {
        Arc::new(self)
    }
}
