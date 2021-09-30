use std::collections::VecDeque;

use rand::prelude::*;
use serde::{Deserialize, Serialize};

use super::Update;
use crate::config::{
    Config, ConfigFactory, ConfigFactoryTier, ConfigItem, ConfigItemNew, ConfigProduct,
    ConfigProductTier, ConfigTeam,
};
use crate::types::ItemRef;
use crate::util::{i_to_xy, xy_to_i};

/// Maximum number of items in factory drop queue.
const FACTORY_QUEUE_SIZE: usize = 2;

/// Represents a team.
#[derive(Serialize, Deserialize, Debug)]
pub struct GameTeam {
    /// Team ID.
    pub id: u32,

    /// Team inventory.
    pub inventory: GameInventory,

    #[serde(skip)]
    pub config: Option<ConfigTeam>,
}

impl GameTeam {
    /// Construct a new team.
    pub fn new(tick: u64, config: &Config, id: u32) -> Self {
        Self {
            id,
            inventory: GameInventory::from_config(tick, config)
                .unwrap_or_else(|| GameInventory::default()),
            config: config.team(id).cloned(),
        }
    }

    /// Prepare configuration.
    pub fn prepare_config(&mut self, config: &Config) -> Result<(), ()> {
        self.config = Some(config.team(self.id).cloned().ok_or(())?);
        self.inventory.grid.prepare_config(config)
    }
}

impl Update for GameTeam {
    fn update(&mut self, config: &Config, tick: u64) -> bool {
        self.inventory.update(config, tick)
    }
}

/// Game item.
#[derive(Serialize, Deserialize, Debug)]
pub struct GameItemNew {
    /// Item ID.
    pub id: ItemRef,

    /// Next drop tick.
    tick: Option<u64>,

    /// Item drop queue.
    queue: VecDeque<ItemRef>,

    #[serde(skip)]
    pub config: Option<ConfigItemNew>,
}

impl GameItemNew {
    /// Construct item from configuration.
    ///
    /// Current game `tick` must be given.
    pub fn from_config(tick: u64, item: ConfigItemNew) -> Self {
        Self {
            id: item.id.clone(),
            tick: item.drop_interval.map(|t| t + tick),
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

        // TODO: update ticks!
        // TODO: what to do with the queue?

        self.id = item.id.clone();
        self.config = Some(item.clone());
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

impl Update for GameItemNew {
    fn update(&mut self, config: &Config, tick: u64) -> bool {
        // Do nothing if there's no tick, or if we didn't reach it yet
        if !self.tick.map(|t| t < tick).unwrap_or(false) {
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
        self.push_queue_drop(item)
    }
}

/// Inventory item.
#[derive(Serialize, Deserialize, Debug)]
pub enum GameItem {
    Product(GameProduct),
    Factory(GameFactory),
}

impl GameItem {
    pub fn from_config(tick: u64, item: ConfigItem) -> Self {
        match item {
            ConfigItem::Product(tier, item, level) => {
                GameItem::Product(GameProduct::from_config(tier, level))
            }
            ConfigItem::Factory(tier, item, level) => {
                GameItem::Factory(GameFactory::from_config(tick, tier, level))
            }
        }
    }

    /// Get sell amounts for item.
    // TODO: return amounts here, instead of just money
    pub fn sell_amounts(&self) -> Option<u64> {
        match self {
            GameItem::Product(product) => Some(product.config_item.as_ref()?.cost),
            GameItem::Factory(factory) => {
                crate::client::types::amount_only_money(&factory.config_item.as_ref()?.cost_sell)
            }
        }
    }

    /// Attemp to upgrade 1 level.
    ///
    /// Returns true if something changed, false if failed.
    #[must_use]
    pub fn upgrade(&mut self, config: &Config) -> bool {
        match self {
            GameItem::Product(product) => product.upgrade(config),
            GameItem::Factory(factory) => factory.upgrade(config),
        }
    }

    /// Prepare configuration.
    fn prepare_config(&mut self, config: &Config) -> Result<(), ()> {
        match self {
            GameItem::Product(product) => product.fetch_config(config),
            GameItem::Factory(factory) => factory.fetch_config(config),
        }
    }
}

impl Update for GameItem {
    fn update(&mut self, config: &Config, tick: u64) -> bool {
        match self {
            GameItem::Product(_) => false,
            GameItem::Factory(factory) => factory.update(config, tick),
        }
    }
}

/// Inventory product.
#[derive(Serialize, Deserialize, Debug)]
pub struct GameProduct {
    pub tier: u32,
    pub level: u16,

    #[serde(skip)]
    pub config_tier: Option<ConfigProductTier>,
    #[serde(skip)]
    pub config_item: Option<ConfigProduct>,
}

impl GameProduct {
    /// Construct from config.
    // TODO: return None for invalid level and no config item
    pub fn from_config(tier: ConfigProductTier, level: u16) -> Self {
        Self {
            tier: tier.id,
            level,
            config_item: tier.level(level).cloned(),
            config_tier: Some(tier),
        }
    }

    /// Attemp to upgrade 1 level.
    ///
    /// Returns true if something changed, false if failed.
    #[must_use]
    pub fn upgrade(&mut self, config: &Config) -> bool {
        // We must have config
        if self.fetch_config(config).is_err() {
            return false;
        }

        // We must be able to upgrade
        if !self.can_upgrade() {
            return false;
        }

        // Increase level, update config
        self.level += 1;
        self.config_tier.take();
        self.fetch_config(config);
        true
    }

    /// Check whether we can upgrade.
    ///
    /// Checks whether there is a next level.
    pub fn can_upgrade(&self) -> bool {
        // TODO: we must have the config

        self.config_tier.as_ref().unwrap().max_level() > self.level
    }

    /// Fetch config types if not set.
    fn fetch_config(&mut self, config: &Config) -> Result<(), ()> {
        // Skip if already set
        if self.config_tier.is_some() && self.config_item.is_some() {
            return Ok(());
        }

        // Find config models
        let reference = self.reference();
        match config.find_item(&reference).ok_or(())? {
            ConfigItem::Product(tier, item, _) => {
                self.config_tier.replace(tier);
                self.config_item.replace(item);
                Ok(())
            }
            _ => {
                warn!("Failed to resolve for product config: {:?}", reference);
                Err(())
            }
        }
    }

    /// Get `ItemRef` for current product.
    fn reference(&self) -> ItemRef {
        ItemRef::from(self.tier, self.level)
    }
}

/// Inventory factory.
#[derive(Serialize, Deserialize, Debug)]
pub struct GameFactory {
    /// Current tier ID.
    pub tier: u32,

    /// Current level.
    pub level: u16,

    /// Next drop tick.
    tick: u64,

    /// Item drop queue.
    // TODO: also serialize this
    #[serde(skip)]
    queue: VecDeque<GameItem>,

    #[serde(skip)]
    pub config_tier: Option<ConfigFactoryTier>,
    #[serde(skip)]
    pub config_item: Option<ConfigFactory>,
}

impl GameFactory {
    /// Construct from config.
    // TODO: return None for invalid level and no config item!
    pub fn from_config(tick: u64, factory: ConfigFactoryTier, level: u16) -> Self {
        Self {
            tier: factory.id,
            level,
            tick: tick + factory.level(level).as_ref().unwrap().time as u64,
            queue: VecDeque::default(),
            config_item: factory.level(level).cloned(),
            config_tier: Some(factory.clone()),
        }
    }

    /// Attemp to upgrade 1 level.
    ///
    /// Returns true if something changed, false if failed.
    #[must_use]
    pub fn upgrade(&mut self, config: &Config) -> bool {
        // We must have config
        if self.fetch_config(config).is_err() {
            return false;
        }

        // We must be able to upgrade
        let reference = self.reference();
        if !self.can_upgrade() {
            return false;
        }

        // Increase level, update config
        self.level += 1;
        self.config_tier.take();
        self.fetch_config(config);
        true
    }

    /// Check whether we can upgrade.
    ///
    /// Checks whether there is a next level.
    pub fn can_upgrade(&self) -> bool {
        // TODO: we must have the config

        self.config_tier.as_ref().unwrap().max_level() > self.level
    }

    /// Fetch config types if not set.
    fn fetch_config(&mut self, config: &Config) -> Result<(), ()> {
        // Skip if already set
        if self.config_tier.is_some() && self.config_item.is_some() {
            return Ok(());
        }

        // Find config models
        let reference = self.reference();
        match config.find_item(&reference).ok_or(())? {
            ConfigItem::Factory(tier, item, _) => {
                self.config_tier.replace(tier);
                self.config_item.replace(item);
                Ok(())
            }
            _ => {
                warn!("Failed to resolve for factory config: {:?}", reference);
                Err(())
            }
        }
    }

    /// Get `ItemRef` for current product.
    fn reference(&self) -> ItemRef {
        ItemRef::from(self.tier, self.level)
    }

    /// Attempt to add an item to the queue if there is sufficient queue space.
    ///
    /// Returns `false` if the item wasn't queued because there was no space.
    #[must_use]
    fn push_queue_drop(&mut self, item: GameItem) -> bool {
        if self.is_queue_space() {
            trace!("Add drop to factory queue");
            self.queue.push_back(item);
            return true;
        }

        false
    }

    /// Pop an item from the drop queue if there is any.
    #[must_use]
    fn pop_queue_drop(&mut self) -> Option<GameItem> {
        self.queue.pop_front()
    }

    /// Check whether there is space in the item drop queue.
    fn is_queue_space(&self) -> bool {
        self.queue.len() < FACTORY_QUEUE_SIZE
    }
}

impl Update for GameFactory {
    fn update(&mut self, config: &Config, tick: u64) -> bool {
        // We must have config
        if self.fetch_config(config).is_err() {
            return false;
        }

        // Do nothing if we didn't reach the tick
        // TODO: catch up to missed ticks
        if self.tick > tick {
            return false;
        }

        // Update tick to next
        self.tick = tick + self.config_item.as_ref().unwrap().time as u64;

        // Select config item to drop
        let item = match self.config_item.as_ref().unwrap().random_drop() {
            Some(item_ref) => item_ref,
            None => return false,
        };

        // Find config model
        let item = match config.find_item(&item) {
            Some(item) => item,
            None => {
                warn!("Failed to resolve for factory drop: {:?}", item);
                return false;
            }
        };

        // Transpose into game item, add to queue
        let item = GameItem::from_config(tick, item);
        self.push_queue_drop(item)
    }
}

/// An inventory.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GameInventory {
    pub money: u64,
    pub energy: u64,
    pub grid: GameInventoryGrid,
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
        })
    }
}

impl Update for GameInventory {
    fn update(&mut self, config: &Config, tick: u64) -> bool {
        self.grid.update(config, tick)
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
        let mut config_items: Vec<crate::config::ConfigItem> = Vec::with_capacity(refs.len());
        for item_ref in refs {
            config_items.push(config.find_item(&item_ref)?);
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

    /// Get item at grid position.
    ///
    /// Is `None` if cell is empty.
    pub fn get_at(&self, x: u32, y: u32) -> &Option<GameItem> {
        &self.items[xy_to_i(x, y)]
    }

    /// Get item at grid position.
    ///
    /// Is `None` if cell is empty.
    pub fn get_at_mut(&mut self, x: u32, y: u32) -> &mut Option<GameItem> {
        &mut self.items[xy_to_i(x, y)]
    }

    /// Place given item randomly in inventory.
    ///
    /// Returns `false` if there was no space.
    #[must_use]
    pub fn place_item(&mut self, item: GameItem) -> bool {
        match self.find_free_cell() {
            Some(coord) => {
                *self.get_at_mut(coord.0, coord.1) = Some(item);
                true
            }
            None => false,
        }
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
            .map(|(i, _)| i_to_xy(i))
    }

    /// Check whether inventory has any free cell.
    pub fn has_free_cell(&self) -> bool {
        self.items.iter().any(Option::is_none)
    }

    /// Count the number of free cells.
    pub fn count_free_cells(&self) -> usize {
        self.items.iter().filter(|i| i.is_none()).count()
    }

    /// Place factory queue items if there is space.
    fn place_queue_items(&mut self) -> bool {
        // There must be space
        let max = self.count_free_cells();
        if max <= 0 {
            return false;
        }

        // Obtain list of items to place
        let mut items: Vec<GameItem> = Vec::with_capacity(max);
        for item in self.items.iter_mut() {
            match item {
                Some(GameItem::Factory(factory)) => {
                    while items.len() < max {
                        match factory.pop_queue_drop() {
                            Some(item) => items.push(item),
                            None => break,
                        }
                    }

                    // Stop outer loop if we reached max
                    if items.len() >= max {
                        break;
                    }
                }
                _ => {}
            }
        }

        // We're done if we got no items
        if items.is_empty() {
            return false;
        }

        // Place all items
        // TODO: when factory is placed ensure tick setting is correct
        for item in items {
            if !self.place_item(item) {
                error!("failed to place selected item, no inventory space");
            }
        }

        true
    }

    /// Prepare configuration.
    fn prepare_config(&mut self, config: &Config) -> Result<(), ()> {
        for item in self.items.iter_mut() {
            match item {
                Some(item) => item.prepare_config(config)?,
                None => {}
            }
        }
        Ok(())
    }
}

impl Update for GameInventoryGrid {
    fn update(&mut self, config: &Config, tick: u64) -> bool {
        for item in self.items.iter_mut() {
            match item {
                Some(item) => {
                    item.update(config, tick);
                }
                None => {}
            }
        }

        // Place queued factory items onto field
        let changed = self.place_queue_items();

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
