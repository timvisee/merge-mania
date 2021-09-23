use rand::Rng;
use serde::Deserialize;

use crate::util::{i_to_xy, xy_to_i};

/// Represents runnable game state.
#[derive(Default)]
pub struct Game {
    /// Whether the game is running.
    running: bool,

    /// Team state.
    teams: Vec<Team>,
}

/// Represents a team.
#[derive(Debug)]
pub struct Team {
    /// Team ID.
    id: u32,

    /// Team inventory.
    inventory: Inventory,
}

/// Inventory item.
#[derive(Debug)]
pub enum Item {
    Product(Product),
    Factory(Factory),
}

/// Inventory product.
#[derive(Debug)]
pub struct Product {
    tier: u16,
    level: u16,
}

/// Inventory factory.
#[derive(Debug)]
pub struct Factory {
    tier: u32,
    level: u16,
}

impl Factory {
    pub fn from_lib(factory: &crate::types::LibFactoryTier) -> Self {
        Self {
            tier: factory.id,
            level: 0,
        }
    }
}

/// An inventory.
#[derive(Debug, Default)]
pub struct Inventory {
    money: usize,
    energy: usize,
    grid: InventoryGrid,
}

/// An inventory grid.
#[derive(Debug, Default)]
pub struct InventoryGrid {
    items: [Option<Item>; crate::INV_SIZE as usize],
}

impl InventoryGrid {
    /// Get item at grid position.
    ///
    /// Is `None` if cell is empty.
    pub fn get_at(&self, x: u32, y: u32) -> &Option<Item> {
        &self.items[xy_to_i(x, y)]
    }

    /// Get item at grid position.
    ///
    /// Is `None` if cell is empty.
    pub fn get_at_mut(&mut self, x: u32, y: u32) -> &mut Option<Item> {
        &mut self.items[xy_to_i(x, y)]
    }

    /// Place given item randomly in inventory.
    ///
    /// Returns `false` if there was no space.
    #[must_use]
    pub fn place_item(&mut self, item: Item) -> bool {
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
