pub mod types;

use std::sync::{Mutex, RwLock};

use rand::Rng;
use serde::Deserialize;
use tokio::time::{self, Duration};

use crate::config::{Config, ConfigFactoryTier};
use crate::state::SharedState;
use crate::util::{i_to_xy, xy_to_i};
use crate::ws;
pub use types::*;

/// A simple game loop.
pub(crate) async fn run(state: SharedState) {
    let mut interval = time::interval(Duration::from_secs(crate::TICK_SEC));

    loop {
        // Wait for tick
        interval.tick().await;

        // Process ticks
        // TODO: catch up to missed ticks here
        if state.game.running {
            state.game.process_ticks(&state, 1);
        }
    }
}

/// Represents runnable game state.
#[derive(Default)]
pub struct Game {
    /// Whether the game is running.
    pub running: bool,

    /// Current game tick.
    // TODO: switch to atomic?
    tick: Mutex<usize>,

    /// Team state.
    teams: Vec<RwLock<GameTeam>>,
}

impl Game {
    /// Add a new team.
    pub fn add_team(&mut self, team_id: u32) {
        // TODO: ensure team isn't added multiple times
        let team = GameTeam::new(team_id);
        self.teams.push(RwLock::new(team));
    }

    /// Process the game by the given amount of ticks.
    ///
    /// This should be invoked from a game loop.
    /// Calls `update` on the full game state afterwards.
    pub fn process_ticks(&self, state: &SharedState, ticks: usize) {
        let tick = {
            let mut lock = self.tick.lock().unwrap();
            *lock += ticks;
            *lock
        };

        error!("Processing game tick");

        // Update each team
        let mut changed = false;
        for team in self.teams.iter() {
            let mut team = team.write().unwrap();
            let team_changed = team.update(&state.config, tick);

            // Broadcast inventory update if team changed
            if team_changed {
                broadcast_team_inventory(state, &team);
            }

            changed = team_changed || changed;
        }

        // TODO: put factory items onto field
        // TODO: do not return true if only queue item was added
    }
}

pub trait Update {
    /// Update this state upto the given tick.
    ///
    /// Return `true` if internally changed.
    fn update(&mut self, config: &Config, tick: usize) -> bool;
}

/// Broadcast current inventory state to team clients.
fn broadcast_team_inventory(state: &SharedState, team: &GameTeam) {
    let inventory = serde_json::to_string(&team.inventory).expect("failed to serialize inventory");
    ws::send_to_team(&state, None, team.id, &inventory);
}
