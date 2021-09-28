pub mod types;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, RwLock};

use rand::Rng;
use serde::Deserialize;
use tokio::time::{self, Duration};

use crate::client::{ClientInventory, MsgKind};
use crate::config::{Config, ConfigFactoryTier};
use crate::state::SharedState;
use crate::util::{i_to_xy, xy_to_i};
use crate::ws;
pub use types::*;

/// A simple game loop.
pub(crate) async fn run(state: SharedState) {
    let mut interval = time::interval(Duration::from_millis(crate::TICK_MILLIS));

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
// TODO: when loading (deserializing) game, make sure all config properties get attached!
#[derive(Default)]
pub struct Game {
    /// Whether the game is running.
    pub running: bool,

    /// Current game tick.
    tick: AtomicUsize,

    /// Team state.
    teams: Vec<RwLock<GameTeam>>,
}

impl Game {
    /// Add a new team.
    pub fn add_team(&mut self, config: &Config, team_id: u32) {
        // TODO: dynamically load team when data is requested
        // TODO: ensure team isn't added multiple times
        let team = GameTeam::new(self.tick(), config, team_id);
        self.teams.push(RwLock::new(team));
    }

    /// Get current game tick.
    pub fn tick(&self) -> usize {
        self.tick.load(Ordering::Relaxed)
    }

    /// Process the game by the given amount of ticks.
    ///
    /// This should be invoked from a game loop.
    /// Calls `update` on the full game state afterwards.
    pub fn process_ticks(&self, state: &SharedState, ticks: usize) {
        trace!("Processing game tick");

        // Increase tick by 1
        let tick = self.tick.fetch_add(1, Ordering::Relaxed) + 1;

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

    /// Get the team client inventory.
    pub fn team_client_inventory(&self, team_id: u32) -> Option<ClientInventory> {
        let team = self
            .teams
            .iter()
            .map(|t| t.read().unwrap())
            .filter(|t| t.id == team_id)
            .next()?;
        let inventory = ClientInventory::from_game(&team.inventory)
            .expect("failed to transpose game to client inventory");
        Some(inventory)
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
    let inventory = ClientInventory::from_game(&team.inventory)
        .expect("failed to transpose game to client inventory");
    let msg = MsgKind::Inventory(inventory);
    ws::send_to_team(&state, None, team.id, &msg.into());
}
