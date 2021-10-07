use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{Config, ConfigItem, ConfigUser};
use crate::types::{Amount, ItemRef};
use crate::util::{i_to_xy, xy_to_i};

/// Maximum number of items in factory drop queue.
const FACTORY_QUEUE_SIZE: usize = 2;

/// Represents a game user.
#[derive(Serialize, Deserialize, Debug)]
pub struct GameUser {
    /// User ID.
    pub id: u32,

    /// User inventory.
    pub inventory: GameInventory,

    #[serde(skip)]
    pub config: Option<ConfigUser>,

    #[serde(default)]
    pub stats: GameUserStats,
}

impl GameUser {
    /// Construct a new user.
    pub fn new(tick: u64, config: &Config, id: u32) -> Self {
        Self {
            id,
            inventory: GameInventory::from_config(tick, config)
                .unwrap_or_else(|| GameInventory::default()),
            config: config.user(id).cloned(),
            stats: GameUserStats::default(),
        }
    }

    /// Prepare configuration.
    pub fn attach_config(&mut self, config: &Config) -> Result<(), ()> {
        let user = config.user(self.id).cloned().ok_or(())?;

        // // User must have game permission
        // if !user.role_game {
        //     error!("Could not attach config to game user, because user has no game permission");
        //     return Err(());
        // }

        self.config = Some(user);
        self.inventory.grid.attach_config(config)
    }

    /// Update game user.
    ///
    /// Returns list of changed inventory cells and `true` if a new item is discovered.
    pub fn update(&mut self, config: &Config, tick: u64) -> (HashSet<u8>, bool, u32) {
        self.inventory.update(config, tick)
    }
}

/// Game item.
#[derive(Serialize, Deserialize, Debug)]
pub struct GameItem {
    /// Item ID.
    pub id: ItemRef,

    /// Next drop tick.
    tick: Option<u64>,

    /// Number of times left to drop.
    drop_limit: Option<u32>,

    /// Item drop queue.
    queue: VecDeque<ItemRef>,

    #[serde(skip)]
    pub config: Option<ConfigItem>,
}

impl GameItem {
    /// Construct item from configuration.
    ///
    /// Current game `tick` must be given.
    pub fn from_config(tick: u64, item: ConfigItem) -> Self {
        Self {
            id: item.id.clone(),
            tick: item.drop_interval.map(|t| tick + t),
            drop_limit: item.drop_limit.clone(),
            queue: Default::default(),
            config: Some(item),
        }
    }

    /// Attach configuration to this item.
    fn attach_config(&mut self, config: &Config) -> Result<(), ()> {
        match config.item(&self.id) {
            Some(config) => {
                self.config = Some(config.clone());
                Ok(())
            }
            None => {
                warn!("Failed to resolve config for item: {:?}", self.id);
                Err(())
            }
        }
    }

    /// Update game item.
    ///
    /// Returns `true` if changed.
    fn update(&mut self, config: &Config, tick: u64) -> bool {
        // We must have reached tick, and have not reached the drop limit
        // TODO: queue must have space
        let reached_tick = self.tick.map(|t| t < tick).unwrap_or(false);
        let reached_drop_limit = self.drop_limit.map(|l| l == 0).unwrap_or(false);
        if !reached_tick || reached_drop_limit {
            return false;
        }

        // Update tick to next
        self.tick = self
            .config
            .as_ref()
            .unwrap()
            .drop_interval
            .map(|t| tick + t);

        // Select config item to drop
        let item = match self.config.as_ref().unwrap().random_drop() {
            Some(item_ref) => item_ref,
            None => return false,
        };
        if !self.push_queue_drop(item) {
            return false;
        }

        // Decrease drop limit
        if let Some(limit) = self.drop_limit {
            self.drop_limit = Some(limit.saturating_sub(1));
        }

        true
    }

    /// Check whether this is upgradable (mergeable).
    pub fn can_upgrade(&self) -> bool {
        self.config.as_ref().unwrap().merge.is_some()
    }

    /// Attempt to upgrade (merge) a level.
    ///
    /// Returns true on success, if something has changed, false otherwise.
    #[must_use]
    pub fn upgrade(&mut self, config: &Config) -> bool {
        // Ensure we can upgrade
        if !self.can_upgrade() {
            return false;
        }

        // Find upgraded config item
        let upgrade_id = match &self.config.as_ref().unwrap().merge {
            Some(id) => id,
            None => return false,
        };
        let item = match config.item(upgrade_id) {
            Some(item) => item,
            None => {
                warn!(
                    "Failed to resolve config for item upgrade: {:?}",
                    upgrade_id
                );
                return false;
            }
        };

        // TODO: what to do with the queue?

        self.id = item.id.clone();
        self.config = Some(item.clone());

        // TODO: update ticks!
        // TODO: set to current game tick to instantly drop, instead of 0!
        self.tick = item.drop_interval.map(|_| 0);
        self.drop_limit = item.drop_limit.clone();

        // TODO: ensure to set all values!

        true
    }

    /// Attempt to add an item to the queue if there is sufficient queue space.
    ///
    /// Returns `false` if the item wasn't queued because there was no space.
    #[must_use]
    fn push_queue_drop(&mut self, item: ItemRef) -> bool {
        if self.queue.len() < FACTORY_QUEUE_SIZE {
            trace!("Add drop to factory queue: {:?}", item);
            self.queue.push_back(item);
            return true;
        }
        false
    }

    /// Pop an item from the drop queue if there is any.
    #[must_use]
    fn pop_queue_drop(&mut self) -> Option<ItemRef> {
        self.queue.pop_front()
    }
}

/// An inventory.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GameInventory {
    pub money: u64,
    pub energy: u64,
    pub grid: GameInventoryGrid,
    #[serde(default)]
    pub discovered: HashSet<ItemRef>,
}

impl GameInventory {
    /// Get a default inventory from configuration.
    ///
    /// Returns `None` on failure.
    pub fn from_config(tick: u64, config: &Config) -> Option<Self> {
        Some(Self {
            money: config.defaults.money,
            energy: config.defaults.energy,
            grid: GameInventoryGrid::from_config(tick, config)?,
            discovered: config.defaults.inventory.iter().cloned().collect(),
        })
    }

    /// Update game inventory.
    ///
    /// Returns list of changed cells and `true` if a new item is discovered.
    fn update(&mut self, config: &Config, tick: u64) -> (HashSet<u8>, bool, u32) {
        // Update grid, collect changed cells and discovered items
        let (changed, discovered, drop_count) = self.grid.update(config, tick);

        // Check wheher new items are discovered
        let discovered = self.discover_items(discovered);

        (changed, discovered, drop_count)
    }

    /// Discover an item.
    ///
    /// Returns `true` if it is newly discovered.
    pub fn discover_item(&mut self, item: ItemRef) -> bool {
        // TODO: remove debug here
        let discovered = self.discovered.insert(item.clone());
        if discovered {
            trace!("User discovered new item: {:?}", item);
        }
        discovered
    }

    /// Discover a list of items.
    ///
    /// Returns `true` if any new item was discovered.
    fn discover_items(&mut self, items: HashSet<ItemRef>) -> bool {
        let mut discovered = false;
        for item in items {
            discovered = self.discover_item(item) || discovered;
        }
        discovered
    }

    /// Check whether the inventory contains the given amounts.
    pub fn has_amounts(&self, amounts: &[Amount]) -> bool {
        // TODO: fails if multiple amounts of same type are given, fix this!

        amounts.iter().all(|amount| match amount {
            Amount::Money { money } => self.money >= *money,
            Amount::Energy { energy } => self.energy >= *energy,
            Amount::Item { item, quantity } => self.grid.has_item_quantity(item, *quantity),
        })
    }

    /// Remove the given amounts from the inventory.
    ///
    /// Returns `Ok(map)` listing changed cells.
    /// Returns `Err(())` if the inventory doesn't have enough resources, in which case it isn't
    /// modified.
    pub fn remove_amounts(&mut self, amounts: &[Amount]) -> Result<HashSet<u8>, ()> {
        // User must have enough resources
        if !self.has_amounts(amounts) {
            return Err(());
        }

        // Remove all items from inventory
        Ok(amounts.iter().fold(HashSet::new(), |mut changed, amount| {
            match amount {
                Amount::Money { money } => {
                    self.money -= money;
                }
                Amount::Energy { energy } => {
                    self.energy -= energy;
                }
                Amount::Item { item, quantity } => {
                    for _ in 0..*quantity {
                        if let Some(cell) = self.grid.remove_item(item) {
                            changed.insert(cell);
                        }
                    }
                }
            }
            changed
        }))
    }
}

/// An inventory grid.
#[derive(Serialize, Deserialize, Debug)]
pub struct GameInventoryGrid {
    pub items: Vec<Option<GameItem>>,
}

impl GameInventoryGrid {
    /// Get a default inventory from configuration.
    ///
    /// Returns `None` on failure.
    pub fn from_config(tick: u64, config: &Config) -> Option<Self> {
        let refs = &config.defaults.inventory;

        // Get config items from refs
        // TODO: remove type from vec
        let mut config_items: Vec<crate::config::ConfigItem> = Vec::with_capacity(refs.len());
        for item_ref in refs {
            config_items.push(config.item(&item_ref)?.clone());
        }

        // Transpose config into game items
        let mut items: Vec<Option<GameItem>> = config_items
            .into_iter()
            .map(|i| Some(GameItem::from_config(tick, i)))
            .collect();

        // Give list correct length
        items.truncate(crate::INV_SIZE as usize);
        items.extend((0..crate::INV_SIZE as usize - items.len()).map(|_| None));

        // Shuffle items
        let mut rng = rand::thread_rng();
        items.shuffle(&mut rng);

        Some(Self { items })
    }

    /// Attach configuration.
    fn attach_config(&mut self, config: &Config) -> Result<(), ()> {
        for item in self.items.iter_mut() {
            match item {
                Some(item) => item.attach_config(config)?,
                None => {}
            }
        }
        Ok(())
    }

    /// Get item at grid position.
    ///
    /// Is `None` if cell is empty.
    pub fn get_at(&self, x: u32, y: u32) -> &Option<GameItem> {
        &self.items[xy_to_i(x, y) as usize]
    }

    /// Get item at grid position.
    ///
    /// Is `None` if cell is empty.
    pub fn get_at_mut(&mut self, x: u32, y: u32) -> &mut Option<GameItem> {
        &mut self.items[xy_to_i(x, y) as usize]
    }

    /// Place given item randomly in inventory.
    ///
    /// Returns cell index, returns `None` if there was no space.
    #[must_use]
    pub fn place_item(&mut self, item: GameItem) -> Option<u8> {
        self.find_free_cell().map(|coord| {
            *self.get_at_mut(coord.0, coord.1) = Some(item);
            xy_to_i(coord.0, coord.1)
        })
    }

    /// Check whether the grid contains the given number of items.
    pub fn has_item_quantity(&self, item: &ItemRef, quantity: u8) -> bool {
        self.items
            .iter()
            .filter(|i| matches!(i, Some(i) if &i.id == item))
            .take(quantity as usize)
            .count()
            >= quantity as usize
    }

    /// Remove an item from the grid.
    ///
    /// Returns cell index of removed item on success, `None` on failure.
    pub fn remove_item(&mut self, item: &ItemRef) -> Option<u8> {
        // TODO: use shared random source
        let mut rng = rand::thread_rng();

        // Find random cell index that holds this item
        self.items
            .iter()
            .enumerate()
            .cycle()
            .skip(rng.gen_range(0..crate::INV_SIZE as usize))
            .take(crate::INV_SIZE as usize)
            .filter(|(_, i)| matches!(i, Some(i) if &i.id == item))
            .map(|(i, _)| i)
            .next()
            .and_then(|index| self.items[index].take().map(|_| index as u8))
    }

    /// Find a random free cell in the inventory.
    ///
    /// Returns `None` if no cell is available.
    pub fn find_free_cell(&self) -> Option<(u32, u32)> {
        // TODO: use shared random source
        let mut rng = rand::thread_rng();

        // Walk through all items, find first empty cell from random position
        self.items
            .iter()
            .enumerate()
            .cycle()
            .skip(rng.gen_range(0..crate::INV_SIZE as usize))
            .take(crate::INV_SIZE as usize)
            .filter(|(_, item)| item.is_none())
            .next()
            .map(|(i, _)| i_to_xy(i as u8))
    }

    /// Check whether inventory has any free cell.
    pub fn has_free_cell(&self) -> bool {
        self.items.iter().any(Option::is_none)
    }

    /// Count the number of free cells.
    pub fn count_free_cells(&self) -> usize {
        self.items.iter().filter(|i| i.is_none()).count()
    }

    /// Update game inventory.
    ///
    /// Return list of changed cell indices.
    pub fn update(&mut self, config: &Config, tick: u64) -> (HashSet<u8>, HashSet<ItemRef>, u32) {
        // Update items, drop updated state
        for item in self.items.iter_mut() {
            if let Some(item) = item {
                item.update(config, tick);
            }
        }

        // Place queued factory items onto field
        let (mut changed, discovered) = self.place_queue_items(config, tick);
        let drop_count = changed.len() as u32;

        // Remove items that reached their drop limit
        changed.extend(self.remove_drop_limit_items(config));

        (changed, discovered, drop_count)
    }

    /// Place factory queue items if there is space.
    fn place_queue_items(&mut self, config: &Config, tick: u64) -> (HashSet<u8>, HashSet<ItemRef>) {
        // Get number of free grid cells, there must be space
        let max = self.count_free_cells();
        if max <= 0 {
            return (HashSet::new(), HashSet::new());
        }

        // Obtain list of items to place
        let mut items: Vec<ItemRef> = Vec::with_capacity(max);
        for item in self.items.iter_mut() {
            if let Some(item) = item {
                while items.len() < max {
                    match item.pop_queue_drop() {
                        Some(item) => items.push(item),
                        None => break,
                    }
                }
            }

            // Stop outer loop if we reached max
            if items.len() >= max {
                break;
            }
        }

        // Place all items
        // TODO: when factory is placed ensure tick setting is correct
        let mut changed = HashSet::new();
        let mut discovered = HashSet::new();
        for item in &items {
            // Resolve item from config
            let item = match config.item(&item) {
                Some(item) => item,
                None => {
                    warn!(
                        "Failed to place queued item, item ref does not resolve: {:?}",
                        item
                    );
                    continue;
                }
            };

            // Transpose into game item, place it
            let game_item = GameItem::from_config(tick, item.clone());
            match self.place_item(game_item) {
                Some(index) => {
                    changed.insert(index);
                    discovered.insert(item.id.clone());
                }
                None => {
                    error!("Failed to place selected item, no inventory space");
                }
            }
        }

        (changed, discovered)
    }

    /// Remove items that reached their drop limit, if their drop queue is cleared.
    fn remove_drop_limit_items(&mut self, config: &Config) -> HashSet<u8> {
        let mut changed = HashSet::new();
        for (index, item) in self.items.iter_mut().enumerate() {
            match item {
                Some(i) => match i.drop_limit {
                    Some(limit) if limit <= 0 && i.queue.is_empty() => {
                        // This item reached it limit, remove it
                        // TODO: do we need a special item remove routine?
                        *item = None;
                        changed.insert(index as u8);
                    }
                    _ => {}
                },
                None => {}
            }
        }
        changed
    }
}

impl Default for GameInventoryGrid {
    fn default() -> Self {
        Self {
            items: (0..crate::INV_SIZE as usize)
                .map(|_| None)
                .collect::<Vec<_>>(),
        }
    }
}

/// Game user stats.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GameUserStats {
    /// Number of merges by user.
    pub merge_count: AtomicU32,

    /// Number of items bought by user.
    pub buy_count: AtomicU32,

    /// Number of items sold by user.
    pub sell_count: AtomicU32,

    /// Number of item swaps (moves) by user.
    pub swap_count: AtomicU32,

    /// Number of codes scanned by user.
    pub code_count: AtomicU32,

    /// Number of items dropped by factories.
    pub drop_count: AtomicU32,

    /// Money spent by user.
    pub money_spent: AtomicU64,

    /// Money earned by user from selling.
    pub money_earned: AtomicU64,

    /// Energy spent by user.
    pub energy_spent: AtomicU64,

    /// Energy earned by user from scanning codes.
    pub energy_earned: AtomicU64,
}

impl GameUserStats {
    /// Increase merge counter by one.
    pub fn inc_merge(&self) {
        self.merge_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Increase buy counter by one.
    pub fn inc_buy(&self) {
        self.buy_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Increase sell counter by one.
    pub fn inc_sell(&self) {
        self.sell_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Increase swap counter by one.
    pub fn inc_swap(&self) {
        self.swap_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Increase code scan counter by one.
    pub fn inc_scan_code(&self) {
        self.code_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Increase drop counter by one.
    pub fn inc_drop(&self, amount: u32) {
        self.drop_count.fetch_add(amount, Ordering::Relaxed);
    }

    /// Increase money spent.
    pub fn inc_money_spent(&self, amount: u64) {
        self.money_spent.fetch_add(amount, Ordering::Relaxed);
    }

    /// Increase money earned.
    pub fn inc_money_earned(&self, amount: u64) {
        self.money_earned.fetch_add(amount, Ordering::Relaxed);
    }

    /// Increase energy spent.
    pub fn inc_energy_spent(&self, amount: u64) {
        self.energy_spent.fetch_add(amount, Ordering::Relaxed);
    }

    /// Increase energy earned.
    pub fn inc_energy_earned(&self, amount: u64) {
        self.energy_earned.fetch_add(amount, Ordering::Relaxed);
    }
}
