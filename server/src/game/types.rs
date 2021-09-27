use std::collections::VecDeque;

use rand::Rng;
use serde::Serialize;

use super::Update;
use crate::config::{
    Config, ConfigFactory, ConfigFactoryTier, ConfigItem, ConfigProduct, ConfigProductTier,
};
use crate::types::ItemRef;
use crate::util::{i_to_xy, xy_to_i};

/// Maximum number of items in factory drop queue.
const FACTORY_QUEUE_SIZE: usize = 4;

/// Represents a team.
#[derive(Serialize, Debug)]
pub struct GameTeam {
    /// Team ID.
    pub id: u32,

    /// Team inventory.
    pub inventory: GameInventory,
}

impl GameTeam {
    /// Construct a new team.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            // TODO: use default inventory items instead
            inventory: GameInventory::default(),
        }
    }
}

impl Update for GameTeam {
    fn update(&mut self, config: &Config, tick: usize) -> bool {
        self.inventory.update(config, tick)
    }
}

/// Inventory item.
#[derive(Serialize, Debug)]
pub enum GameItem {
    Product(GameProduct),
    Factory(GameFactory),
}

impl GameItem {
    pub fn from_config(item: ConfigItem) -> Self {
        match item {
            ConfigItem::Product(tier, item, level) => {
                GameItem::Product(GameProduct::from_config(tier, level))
            }
            ConfigItem::Factory(tier, item, level) => {
                GameItem::Factory(GameFactory::from_config(tier, level))
            }
        }
    }
}

impl Update for GameItem {
    fn update(&mut self, config: &Config, tick: usize) -> bool {
        match self {
            GameItem::Product(_) => false,
            GameItem::Factory(factory) => factory.update(config, tick),
        }
    }
}

/// Inventory product.
#[derive(Serialize, Debug)]
pub struct GameProduct {
    tier: u32,
    level: u16,

    #[serde(skip)]
    config_tier: Option<ConfigProductTier>,
    #[serde(skip)]
    config_item: Option<ConfigProduct>,
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
        // We must be able to upgrade
        if !self.can_upgrade(config) {
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
    pub fn can_upgrade(&mut self, config: &Config) -> bool {
        // We must have config
        if self.fetch_config(config).is_err() {
            return false;
        }

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
#[derive(Serialize, Debug)]
pub struct GameFactory {
    /// Current tier ID.
    tier: u32,

    /// Current level.
    level: u16,

    /// Last processing tick.
    tick: usize,

    /// Item drop queue.
    // TODO: also serialize this
    #[serde(skip)]
    queue: VecDeque<GameItem>,

    #[serde(skip)]
    config_tier: Option<ConfigFactoryTier>,
    #[serde(skip)]
    config_item: Option<ConfigFactory>,
}

impl GameFactory {
    /// Construct from config.
    // TODO: return None for invalid level and no config item
    pub fn from_config(factory: ConfigFactoryTier, level: u16) -> Self {
        Self {
            tier: factory.id,
            level,
            tick: 0,
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
        // We must be able to upgrade
        let reference = self.reference();
        if !self.can_upgrade(config) {
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
    pub fn can_upgrade(&mut self, config: &Config) -> bool {
        // We must have config
        if self.fetch_config(config).is_err() {
            return false;
        }

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
    fn queue_drop(&mut self, item: GameItem) -> bool {
        if self.is_queue_space() {
            info!("Added factory queue item");
            self.queue.push_back(item);
            return true;
        }

        false
    }

    /// Check whether there is space in the item drop queue.
    fn is_queue_space(&self) -> bool {
        self.queue.len() < FACTORY_QUEUE_SIZE
    }
}

impl Update for GameFactory {
    fn update(&mut self, config: &Config, tick: usize) -> bool {
        // We must have config
        if self.fetch_config(config).is_err() {
            return false;
        }

        // TODO: make sure it is time to drop an item

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
        let item = GameItem::from_config(item);
        self.queue_drop(item)
    }
}

/// An inventory.
#[derive(Serialize, Debug, Default)]
pub struct GameInventory {
    money: usize,
    energy: usize,
    grid: GameInventoryGrid,
}

impl Update for GameInventory {
    fn update(&mut self, config: &Config, tick: usize) -> bool {
        self.grid.update(config, tick)
    }
}

/// An inventory grid.
#[derive(Serialize, Debug)]
pub struct GameInventoryGrid {
    items: Vec<Option<GameItem>>,
}

impl GameInventoryGrid {
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
}

impl Update for GameInventoryGrid {
    fn update(&mut self, config: &Config, tick: usize) -> bool {
        let mut changed = false;
        for item in self.items.iter_mut() {
            match item {
                Some(item) => changed = item.update(config, tick) || changed,
                None => {}
            }
        }

        changed
    }
}

// TODO: remove this, used for testing
impl Default for GameInventoryGrid {
    fn default() -> Self {
        // let mut items = Vec::with_capacity(crate::INV_SIZE as usize);
        let mut items = (0..crate::INV_SIZE as usize)
            .map(|_| None)
            .collect::<Vec<_>>();
        items[0] = Some(GameItem::Product(GameProduct {
            tier: 1,
            level: 0,
            config_tier: None,
            config_item: None,
        }));
        items[1] = Some(GameItem::Factory(GameFactory {
            tier: 101,
            level: 0,
            tick: 0,
            queue: VecDeque::new(),
            config_tier: None,
            config_item: None,
        }));
        Self { items }
    }
}
