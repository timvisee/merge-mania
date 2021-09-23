pub mod types;

use rand::Rng;
use serde::Deserialize;

use crate::config::ConfigFactoryTier;
use crate::util::{i_to_xy, xy_to_i};
pub use types::*;

/// Represents runnable game state.
#[derive(Default)]
pub struct Game {
    /// Whether the game is running.
    running: bool,

    /// Team state.
    teams: Vec<GameTeam>,
}
