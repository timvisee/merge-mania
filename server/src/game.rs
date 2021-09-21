use crate::config::Config;
use crate::types::Inventory;

/// Represents game state.
pub struct Game {
    config: Config,
    teams: Vec<Team>,
}

impl Game {
    /// Construct new game.
    pub fn from(config: Config) -> Game {
        Game {
            config,
            teams: vec![],
        }
    }
}

/// Represents a team.
pub struct Team {
    inventory: Inventory,
}
