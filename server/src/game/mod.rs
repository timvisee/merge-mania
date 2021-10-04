pub mod types;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, RwLock};

use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::time::{self, Duration};

use crate::client::{ClientInventory, MsgSendKind};
use crate::config::{Config, ConfigItem};
use crate::state::SharedState;
use crate::types::Amount;
use crate::util::{i_to_xy, xy_to_i};
use crate::ws;
pub use types::*;

/// Threshold in number of changed items after which we should send the full inventory state,
/// rather than each changed cell individually.
const INV_CHANGE_PARTIAL_THRESHOLD: usize = 16;

/// Run game.
pub(crate) async fn run(state: SharedState) {
    let game = game_loop(state.clone());
    let save = save_loop(state);
    futures::future::select(Box::pin(game), Box::pin(save)).await;
}

/// Game logic loop.
pub(crate) async fn game_loop(state: SharedState) {
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

/// Game autosave loop.
pub(crate) async fn save_loop(state: SharedState) {
    let mut interval = time::interval(Duration::from_secs(crate::GAME_SAVE_INTERVAL_SEC));

    loop {
        // Wait for tick
        interval.tick().await;

        // Save game state
        if let Err(err) = state.game.save() {
            error!("Failed to autosave game state");
        }
    }
}

/// Represents runnable game state.
// TODO: when loading (deserializing) game, make sure all config properties get attached!
#[derive(Serialize, Deserialize, Default)]
pub struct Game {
    /// Whether the game is running.
    pub running: bool,

    /// Current game tick.
    tick: AtomicU64,

    /// Team state.
    // TODO: use better structure here, add team getter
    pub teams: RwLock<HashMap<u32, RwLock<GameTeam>>>,
}

impl Game {
    /// Make sure a given team is loaded, load it otherwise.
    pub fn ensure_team(&self, config: &Config, team_id: u32) {
        if !self.teams.read().unwrap().contains_key(&team_id) {
            self.add_team(config, team_id);
        }
    }

    /// Add a new team.
    fn add_team(&self, config: &Config, team_id: u32) {
        let team = GameTeam::new(self.tick(), config, team_id);
        self.teams
            .write()
            .unwrap()
            .insert(team_id, RwLock::new(team));
    }

    /// Get current game tick.
    pub fn tick(&self) -> u64 {
        self.tick.load(Ordering::Relaxed)
    }

    /// Process the game by the given amount of ticks.
    ///
    /// This should be invoked from a game loop.
    /// Calls `update` on the full game state afterwards.
    pub fn process_ticks(&self, state: &SharedState, ticks: u64) {
        trace!("Processing game tick");

        // Increase tick by 1
        let tick = self.tick.fetch_add(1, Ordering::Relaxed) + 1;

        // Update each team
        for team in self.teams.read().unwrap().values() {
            let mut team = team.write().unwrap();
            let (changed, discovered) = team.update(&state.config, tick);
            broadcast_team_cell_changes(state, &team, changed);

            // Send new inventory state if user discovered new items
            if discovered {
                debug!("Team discovered new drop, notifying client");
                let inventory = ClientInventory::from_game(&team.inventory)
                    .expect("failed to transpose game to client inventory");
                let msg = MsgSendKind::InventoryDiscovered(inventory.discovered);
                ws::send_to_team(&state, None, team.id, &msg.into());
            }
        }
    }

    /// Get the team client inventory.
    pub fn team_client_inventory(&self, config: &Config, team_id: u32) -> Option<ClientInventory> {
        self.ensure_team(config, team_id);
        let teams = self.teams.read().unwrap();
        let team = teams.get(&team_id)?.read().unwrap();
        let inventory = ClientInventory::from_game(&team.inventory)
            .expect("failed to transpose game to client inventory");
        Some(inventory)
    }

    /// Swap two items for a team.
    pub fn team_swap(
        &self,
        team_id: u32,
        config: &Config,
        cell: u8,
        other: u8,
    ) -> Option<ClientInventory> {
        self.ensure_team(config, team_id);
        let teams = self.teams.read().unwrap();
        let mut team = teams.get(&team_id).unwrap().write().unwrap();

        // TODO: validate indices
        // TODO: ensure first cell contains item

        // Swap cells
        let tmp = team.inventory.grid.items[cell as usize].take();
        team.inventory.grid.items[cell as usize] = team.inventory.grid.items[other as usize].take();
        team.inventory.grid.items[other as usize] = tmp;

        let inventory = ClientInventory::from_game(&team.inventory)
            .expect("failed to transpose game to client inventory");
        Some(inventory)
    }

    /// Merge two items for a team.
    ///
    /// Returns the new inventory state on success and `true` if a new item was discovered.
    pub fn team_merge(
        &self,
        team_id: u32,
        config: &Config,
        cell: u8,
        other: u8,
    ) -> Option<(ClientInventory, bool)> {
        self.ensure_team(config, team_id);
        let teams = self.teams.read().unwrap();
        let mut team = teams.get(&team_id).unwrap().write().unwrap();

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

        // Check for new item discovery
        let mut discovered = false;
        if let Some(item) = &team.inventory.grid.items[cell as usize] {
            let item_ref = item.id.clone();
            discovered = team.inventory.discover_item(item_ref);
        }

        let inventory = ClientInventory::from_game(&team.inventory)
            .expect("failed to transpose game to client inventory");
        Some((inventory, discovered))
    }

    /// Pay the given amounts.
    pub fn team_pay(
        &self,
        team_id: u32,
        config: &Config,
        amounts: &[Amount],
    ) -> Result<HashSet<u8>, ()> {
        self.ensure_team(config, team_id);
        let teams = self.teams.read().unwrap();
        let mut team = teams.get(&team_id).unwrap().write().unwrap();

        // Remove inventory amounts
        team.inventory.remove_amounts(amounts)
    }

    /// Buy an item for a team.
    ///
    /// Returns updated inventory on success and `true` if a new item was discovered.
    pub fn team_buy(
        &self,
        team_id: u32,
        config: &Config,
        cell: u8,
        item: ConfigItem,
    ) -> Option<(ClientInventory, bool)> {
        self.ensure_team(config, team_id);
        let teams = self.teams.read().unwrap();
        let mut team = teams.get(&team_id).unwrap().write().unwrap();

        // TODO: validate indices
        // TODO: ensure user has costs, pay costs

        let item_id = item.id.clone();
        let mut cell = &mut team.inventory.grid.items[cell as usize];

        // Cell must be empty
        if cell.is_some() {
            return None;
        }

        *cell = Some(GameItem::from_config(self.tick(), item));

        // Check for new item discovery
        let discovered = team.inventory.discover_item(item_id);

        let inventory = ClientInventory::from_game(&team.inventory)
            .expect("failed to transpose game to client inventory");
        Some((inventory, discovered))
    }

    /// Sell an item for a team.
    pub fn team_sell(&self, team_id: u32, config: &Config, cell: u8) -> Option<ClientInventory> {
        self.ensure_team(config, team_id);
        let teams = self.teams.read().unwrap();
        let mut team = teams.get(&team_id).unwrap().write().unwrap();

        // TODO: validate indices

        // Get sell, must contain item
        let mut cell = &mut team.inventory.grid.items[cell as usize];

        match cell.take() {
            Some(item) => team.inventory.money += item.config.as_ref().unwrap().sell,
            None => return None,
        }

        let inventory = ClientInventory::from_game(&team.inventory)
            .expect("failed to transpose game to client inventory");
        Some(inventory)
    }

    /// Scan a code for a team.
    pub fn team_scan_code(&self, team_id: u32, config: &Config) -> Option<ClientInventory> {
        self.ensure_team(config, team_id);
        let teams = self.teams.read().unwrap();
        let mut team = teams.get(&team_id).unwrap().write().unwrap();

        // TODO: implement this!
        warn!("Code scanning not yet implemented");

        // Gain some money and energy for now
        team.inventory.money += 10;
        team.inventory.energy += 5;

        let inventory = ClientInventory::from_game(&team.inventory)
            .expect("failed to transpose game to client inventory");
        Some(inventory)
    }

    /// Load game state from file.
    pub fn load(config: &Config) -> Result<Self, ()> {
        // Load default if file doesn't exist
        let path = PathBuf::from(crate::GAME_SAVE_PATH);
        if !path.is_file() {
            info!("No game state file, starting fresh");
            return Ok(Self::default());
        }

        // Load data from file
        info!("Loading game state from file");
        trace!("Reading game state file...");
        let data = fs::read(path).expect("failed to read game state file");

        // Deserialize
        trace!("Deserializing game state data...");
        let mut game: Self = match serde_json::from_slice(data.as_slice()) {
            Ok(state) => state,
            Err(err) => {
                error!(
                    "Failed to load game state from file, couldn't deserialize: {}",
                    err
                );
                return Err(());
            }
        };

        // Prepare configuration in game items
        debug!("Attaching game item configuration models...");
        if let Err(err) = game.attach_config(config) {
            error!("Failed to link configuration to game objects, config might have changed?",);
            return Err(());
        }
        Ok(game)
    }

    /// Save game state to file.
    pub fn save(&self) -> Result<(), ()> {
        info!("Saving game state to file");

        // Serialize state
        trace!("Serializing game state...");
        let data = if cfg!(debug_assertions) {
            serde_json::to_vec_pretty(self)
        } else {
            serde_json::to_vec(self)
        };
        let data = match data {
            Ok(data) => data,
            Err(err) => {
                error!("Failed to save game to file, couldn't serialize: {}", err);
                return Err(());
            }
        };

        // Write to file
        trace!("Writing game state to file...");
        match fs::write(crate::GAME_SAVE_PATH, data.as_slice()) {
            Ok(result) => Ok(result),
            Err(err) => {
                error!("Failed to save game state to file: {}", err);
                Err(())
            }
        }
    }

    /// Attach configuration.
    pub fn attach_config(&mut self, config: &Config) -> Result<(), ()> {
        for team in self.teams.read().unwrap().values() {
            let mut team = team.write().unwrap();
            team.attach_config(config)?;
        }
        Ok(())
    }
}

/// Broadcast cell changes to team.
///
/// This does some smart checks to figure out the best way of sending these changes.
fn broadcast_team_cell_changes(state: &SharedState, team: &GameTeam, changed: HashSet<u8>) {
    // Send full inventory state when a lot of cells have changed
    if changed.len() >= INV_CHANGE_PARTIAL_THRESHOLD {
        broadcast_team_inventory(state, team);
        return;
    }

    // Obtain user inventory
    let inventory = ClientInventory::from_game(&team.inventory)
        .expect("failed to transpose game to client inventory");

    // Send each change
    for cell in changed {
        let msg = MsgSendKind::InventoryCell {
            index: cell,
            item: inventory.grid.items[cell as usize].clone(),
        };
        ws::send_to_team(&state, None, team.id, &msg.into());
    }
}

/// Broadcast current inventory state to team clients.
fn broadcast_team_inventory(state: &SharedState, team: &GameTeam) {
    let inventory = ClientInventory::from_game(&team.inventory)
        .expect("failed to transpose game to client inventory");
    let msg = MsgSendKind::Inventory(inventory);
    ws::send_to_team(&state, None, team.id, &msg.into());
}
