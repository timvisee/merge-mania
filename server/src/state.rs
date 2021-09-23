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
    _game: Game,
}

impl State {
    /// Construct new state.
    pub fn new(config: Config) -> Self {
        State {
            config,
            sessions: SessionManager::new(),
            clients: ClientManager::new(),
            _game: Game::default(),
        }
    }

    /// Transform into shared state.
    pub fn shared(self) -> SharedState {
        Arc::new(self)
    }
}
