pub mod types;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, RwLock};

use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::time::{self, Duration};

use crate::client::{ClientInventory, ClientUserStats, MsgSendKind};
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
        if state.game.running() {
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
    running: AtomicBool,

    /// Current game tick.
    tick: AtomicU64,

    /// User state.
    // TODO: use better structure here, add user getter
    pub users: RwLock<HashMap<u32, RwLock<GameUser>>>,
}

impl Game {
    /// Make sure a given user is loaded, load it otherwise.
    pub fn ensure_user(&self, config: &Config, user_id: u32) {
        if !self.users.read().unwrap().contains_key(&user_id) {
            self.add_user(config, user_id);
        }
    }

    /// Add a new user.
    fn add_user(&self, config: &Config, user_id: u32) {
        let user = GameUser::new(self.tick(), config, user_id);
        self.users
            .write()
            .unwrap()
            .insert(user_id, RwLock::new(user));
    }

    /// Check if game is running.
    pub fn running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Set whether the game is running.
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Relaxed)
    }

    /// Reset the game.
    pub fn reset(&self) {
        // Grab users lock
        let mut users = self.users.write().unwrap();

        // Drop all user states
        users.clear();

        // Reset game tick
        self.tick.store(0, Ordering::Relaxed);

        drop(users);
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

        // Update each user
        for user in self.users.read().unwrap().values() {
            let mut user = user.write().unwrap();
            let (changed, discovered, drop_count) = user.update(&state.config, tick);

            // Broadcast cell changes
            broadcast_user_cell_changes(state, &user, changed);

            // Send new inventory state if user discovered new items
            if discovered {
                debug!("User discovered new drop, notifying client");
                let inventory = ClientInventory::from_game(&user.inventory)
                    .expect("failed to transpose game to client inventory");
                let msg = MsgSendKind::InventoryDiscovered(inventory.discovered);
                ws::send_to_user(&state, None, user.id, &msg.into());
            }

            // Increase stats
            user.stats.inc_drop(drop_count);
        }
    }

    /// Get the user client inventory.
    pub fn user_client_inventory(&self, config: &Config, user_id: u32) -> Option<ClientInventory> {
        self.ensure_user(config, user_id);
        let users = self.users.read().unwrap();
        let user = users.get(&user_id)?.read().unwrap();
        let inventory = ClientInventory::from_game(&user.inventory)
            .expect("failed to transpose game to client inventory");
        Some(inventory)
    }

    /// Get the user client stats.
    pub fn user_client_stats(&self, config: &Config, user_id: u32) -> Option<ClientUserStats> {
        self.ensure_user(config, user_id);
        let users = self.users.read().unwrap();
        let user = users.get(&user_id)?.read().unwrap();
        let stats = ClientUserStats::from_game(&user.stats);
        Some(stats)
    }

    /// Swap two items for a user.
    pub fn user_swap(
        &self,
        user_id: u32,
        config: &Config,
        cell: u8,
        other: u8,
    ) -> Option<ClientInventory> {
        self.ensure_user(config, user_id);
        let users = self.users.read().unwrap();
        let mut user = users.get(&user_id).unwrap().write().unwrap();

        // TODO: validate indices
        // TODO: ensure first cell contains item

        // Swap cells
        let tmp = user.inventory.grid.items[cell as usize].take();
        user.inventory.grid.items[cell as usize] = user.inventory.grid.items[other as usize].take();
        user.inventory.grid.items[other as usize] = tmp;

        // Increase stats
        user.stats.inc_swap();

        let inventory = ClientInventory::from_game(&user.inventory)
            .expect("failed to transpose game to client inventory");
        Some(inventory)
    }

    /// Merge two items for a user.
    ///
    /// Returns the new inventory state on success and `true` if a new item was discovered.
    pub fn user_merge(
        &self,
        user_id: u32,
        config: &Config,
        cell: u8,
        other: u8,
    ) -> Option<(ClientInventory, bool)> {
        self.ensure_user(config, user_id);
        let users = self.users.read().unwrap();
        let mut user = users.get(&user_id).unwrap().write().unwrap();

        // TODO: validate indices
        // TODO: ensure items are same type
        // TODO: ensure item can be upgraded

        let upgraded = match user.inventory.grid.items[cell as usize].as_mut() {
            Some(cell) => cell.upgrade(config),
            None => {
                warn!("Failed to upgrade item, cell is empty. Possible data race?");
                return None;
            }
        };
        if upgraded {
            user.inventory.grid.items[other as usize] = None;
        }

        // Check for new item discovery
        let mut discovered = false;
        if let Some(item) = &user.inventory.grid.items[cell as usize] {
            let item_ref = item.id.clone();
            discovered = user.inventory.discover_item(item_ref);
        }

        // Increase stats
        user.stats.inc_merge();

        let inventory = ClientInventory::from_game(&user.inventory)
            .expect("failed to transpose game to client inventory");
        Some((inventory, discovered))
    }

    /// Pay the given amounts.
    pub fn user_pay(
        &self,
        user_id: u32,
        config: &Config,
        amounts: &[Amount],
    ) -> Result<HashSet<u8>, ()> {
        self.ensure_user(config, user_id);
        let users = self.users.read().unwrap();
        let mut user = users.get(&user_id).unwrap().write().unwrap();

        // Increase stats
        user.stats.inc_money_spent(amounts_money(&amounts));
        user.stats.inc_energy_spent(amounts_energy(&amounts));

        // Remove inventory amounts
        user.inventory.remove_amounts(amounts)
    }

    /// Buy an item for a user.
    ///
    /// Returns updated inventory on success and `true` if a new item was discovered.
    pub fn user_buy(
        &self,
        user_id: u32,
        config: &Config,
        cell: u8,
        item: ConfigItem,
    ) -> Option<(ClientInventory, bool)> {
        self.ensure_user(config, user_id);
        let users = self.users.read().unwrap();
        let mut user = users.get(&user_id).unwrap().write().unwrap();

        // TODO: validate indices
        // TODO: ensure user has costs, pay costs

        let item_id = item.id.clone();
        let mut cell = &mut user.inventory.grid.items[cell as usize];

        // Cell must be empty, put game item in it
        if cell.is_some() {
            return None;
        }
        *cell = Some(GameItem::from_config(self.tick(), item));

        // Check for new item discovery
        let discovered = user.inventory.discover_item(item_id);

        // Increase stats
        user.stats.inc_buy();

        let inventory = ClientInventory::from_game(&user.inventory)
            .expect("failed to transpose game to client inventory");
        Some((inventory, discovered))
    }

    /// Sell an item for a user.
    pub fn user_sell(&self, user_id: u32, config: &Config, cell: u8) -> Option<ClientInventory> {
        self.ensure_user(config, user_id);
        let users = self.users.read().unwrap();
        let mut user = users.get(&user_id).unwrap().write().unwrap();

        // TODO: validate indices

        // Get sell, must contain item
        let mut cell = &mut user.inventory.grid.items[cell as usize];

        // Clear cell, get sell amount to earn
        let amount = match cell.take() {
            Some(item) => item.config.as_ref().unwrap().sell,
            None => return None,
        };
        user.inventory.money += amount;

        // Increase stats
        user.stats.inc_sell();
        user.stats.inc_money_earned(amount);

        let inventory = ClientInventory::from_game(&user.inventory)
            .expect("failed to transpose game to client inventory");
        Some(inventory)
    }

    /// Scan a code for a user.
    pub fn user_scan_code(&self, user_id: u32, config: &Config) -> Option<ClientInventory> {
        self.ensure_user(config, user_id);
        let users = self.users.read().unwrap();
        let mut user = users.get(&user_id).unwrap().write().unwrap();

        // TODO: implement this!
        warn!("Code scanning not yet implemented");

        // Gain some money and energy for now
        // TODO: grab these values from configuration
        const MONEY_INC: u64 = 10;
        const ENERGY_INC: u64 = 5;
        // user.inventory.money += MONEY_INC;
        user.inventory.energy += ENERGY_INC;

        // Increase stats
        user.stats.inc_scan_code();
        user.stats.inc_energy_earned(ENERGY_INC);

        let inventory = ClientInventory::from_game(&user.inventory)
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
        for user in self.users.read().unwrap().values() {
            let mut user = user.write().unwrap();
            user.attach_config(config)?;
        }
        Ok(())
    }
}

/// Broadcast cell changes to user.
///
/// This does some smart checks to figure out the best way of sending these changes.
fn broadcast_user_cell_changes(state: &SharedState, user: &GameUser, changed: HashSet<u8>) {
    // Send full inventory state when a lot of cells have changed
    if changed.len() >= INV_CHANGE_PARTIAL_THRESHOLD {
        broadcast_user_inventory(state, user);
        return;
    }

    // Obtain user inventory
    let inventory = ClientInventory::from_game(&user.inventory)
        .expect("failed to transpose game to client inventory");

    // Send each change
    for cell in changed {
        let msg = MsgSendKind::InventoryCell {
            index: cell,
            item: inventory.grid.items[cell as usize].clone(),
        };
        ws::send_to_user(&state, None, user.id, &msg.into());
    }
}

/// Broadcast current inventory state to user clients.
fn broadcast_user_inventory(state: &SharedState, user: &GameUser) {
    let inventory = ClientInventory::from_game(&user.inventory)
        .expect("failed to transpose game to client inventory");
    let msg = MsgSendKind::Inventory(inventory);
    ws::send_to_user(&state, None, user.id, &msg.into());
}

/// Get money amount for given list of amounts.
fn amounts_money(amounts: &[Amount]) -> u64 {
    amounts
        .iter()
        .map(|amount| match amount {
            Amount::Money { money } => *money,
            _ => 0,
        })
        .sum()
}

/// Get energy amount for given list of amounts.
fn amounts_energy(amounts: &[Amount]) -> u64 {
    amounts
        .iter()
        .map(|amount| match amount {
            Amount::Energy { energy } => *energy,
            _ => 0,
        })
        .sum()
}
