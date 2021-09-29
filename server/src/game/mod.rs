pub mod types;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, RwLock};

use rand::Rng;
use serde::Deserialize;
use tokio::time::{self, Duration};

use crate::client::{ClientInventory, MsgSendKind};
use crate::config::{Config, ConfigFactoryTier, ConfigItem};
use crate::state::SharedState;
use crate::util::{i_to_xy, xy_to_i};
use crate::ws;
pub use types::*;

/// A simple game loop.
pub(crate) async fn run(state: SharedState) {
    let mut interval = time::interval(Duration::from_millis(state.config.game.tick_millis));

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

    /// Merge two items for a team.
    pub fn team_merge(
        &self,
        team_id: u32,
        config: &Config,
        cell: u8,
        other: u8,
    ) -> Option<ClientInventory> {
        let mut team = self
            .teams
            .iter()
            .map(|t| t.write().unwrap())
            .filter(|t| t.id == team_id)
            .next()?;

        // TODO: validate indices
        // TODO: ensure items are same type
        // TODO: ensure item can be upgraded

        let upgraded = team.inventory.grid.items[cell as usize]
            .as_mut()
            .unwrap()
            .upgrade(config);
        if upgraded {
            team.inventory.grid.items[other as usize] = None;
        }

        let inventory = ClientInventory::from_game(&team.inventory)
            .expect("failed to transpose game to client inventory");
        Some(inventory)
    }

    /// Buy an item for a team.
    pub fn team_buy(
        &self,
        team_id: u32,
        config: &Config,
        cell: u8,
        item: ConfigItem,
    ) -> Option<ClientInventory> {
        let mut team = self
            .teams
            .iter()
            .map(|t| t.write().unwrap())
            .filter(|t| t.id == team_id)
            .next()?;

        // TODO: validate indices
        // TODO: ensure user has costs, pay costs

        let mut cell = &mut team.inventory.grid.items[cell as usize];

        // Cell must be empty
        if cell.is_some() {
            return None;
        }

        *cell = Some(GameItem::from_config(self.tick(), item));

        let inventory = ClientInventory::from_game(&team.inventory)
            .expect("failed to transpose game to client inventory");
        Some(inventory)
    }

    /// Sell an item for a team.
    pub fn team_sell(&self, team_id: u32, config: &Config, cell: u8) -> Option<ClientInventory> {
        let mut team = self
            .teams
            .iter()
            .map(|t| t.write().unwrap())
            .filter(|t| t.id == team_id)
            .next()?;

        // TODO: validate indices

        // Get sell, must contain item
        let mut cell = &mut team.inventory.grid.items[cell as usize];

        match cell.take() {
            Some(item) => {
                // TODO: give proper money to user
                if let Some(money) = item.sell_amounts() {
                    team.inventory.money += money;
                }
            }
            None => return None,
        }

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
    let msg = MsgSendKind::Inventory(inventory);
    ws::send_to_team(&state, None, team.id, &msg.into());
}
