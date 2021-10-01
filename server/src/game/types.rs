use std::collections::VecDeque;

use rand::prelude::*;
use serde::{Deserialize, Serialize};

use super::Update;
use crate::config::{Config, ConfigItem, ConfigTeam};
use crate::types::{Amount, ItemRef};
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
    pub fn attach_config(&mut self, config: &Config) -> Result<(), ()> {
        self.config = Some(config.team(self.id).cloned().ok_or(())?);
        self.inventory.grid.attach_config(config)
    }
}

impl Update for GameTeam {
    fn update(&mut self, config: &Config, tick: u64) -> bool {
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

        // TODO: what to do with the queue?

        self.id = item.id.clone();
        self.config = Some(item.clone());

        // TODO: update ticks!
        // TODO: set to current game tick to instantly drop, instead of 0!
        self.tick = item.drop_interval.map(|_| 0);

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

impl Update for GameItem {
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
    /// Returns `false` if the inventory doesn't have enough resources, in which case it isn't
    /// modified.
    pub fn remove_amounts(&mut self, amounts: &[Amount]) -> bool {
        // User must have enough resources
        if !self.has_amounts(amounts) {
            return false;
        }

        // Remove all items from inventory
        amounts.iter().for_each(|amount| match amount {
            Amount::Money { money } => {
                self.money -= money;
            }
            Amount::Energy { energy } => {
                self.energy -= energy;
            }
            Amount::Item { item, quantity } => {
                for _ in 0..*quantity {
                    self.grid.remove_item(item);
                }
            }
        });

        true
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
    /// Returns `true` if succeeded.
    pub fn remove_item(&mut self, item: &ItemRef) -> bool {
        // TODO: use shared random source
        let mut rng = rand::thread_rng();

        // Find random cell index that holds this item
        let index = self
            .items
            .iter()
            .enumerate()
            .cycle()
            .skip(rng.gen_range(0..crate::INV_SIZE as usize))
            .take(crate::INV_SIZE as usize)
            .filter(|(_, i)| matches!(i, Some(i) if &i.id == item))
            .map(|(i, _)| i)
            .next();
        match index {
            Some(index) => self.items[index].take().is_some(),
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
    fn place_queue_items(&mut self, config: &Config, tick: u64) -> bool {
        // Get number of free grid cells, there must be space
        let max = self.count_free_cells();
        if max <= 0 {
            return false;
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
            let item = GameItem::from_config(tick, item.clone());
            if !self.place_item(item) {
                error!("Failed to place selected item, no inventory space");
            }
        }

        !items.is_empty()
    }
}

impl Update for GameInventoryGrid {
    fn update(&mut self, config: &Config, tick: u64) -> bool {
        // Update items, drop updated state
        for item in self.items.iter_mut() {
            if let Some(item) = item {
                item.update(config, tick);
            }
        }

        // Place queued factory items onto field
        self.place_queue_items(config, tick)
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
