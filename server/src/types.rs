use rand::Rng;
use serde::Deserialize;

use crate::util::{i_to_xy, xy_to_i};

/// Inventory item.
#[derive(Deserialize, Debug)]
pub enum Item {
    Product(Product),
    Factory(Factory),
}

/// Inventory product.
#[derive(Deserialize, Debug)]
pub struct Product {
    tier: u16,
    level: u16,
}

/// Inventory factory.
#[derive(Deserialize, Debug)]
pub struct Factory {
    tier: u16,
    level: u16,
}

/// An inventory.
#[derive(Default)]
pub struct Inventory {
    items: [Option<Item>; crate::INV_SIZE as usize],
}

impl Inventory {
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

#[derive(Deserialize, Debug)]
pub struct LibProducts {
    tiers: Vec<LibProductTier>,
}

#[derive(Deserialize, Debug)]
pub struct LibProductTier {
    id: u32,
    name: String,
    products: Vec<LibProduct>,
}

#[derive(Deserialize, Debug)]
pub struct LibProduct {
    name: String,
    cost: u32,
    sprite_path: String,
}

#[derive(Deserialize, Debug)]
pub struct LibFactories {
    tiers: Vec<LibFactoryTier>,
}

#[derive(Deserialize, Debug)]
pub struct LibFactoryTier {
    id: u32,
    name: String,
    levels: Vec<LibFactory>,
}

#[derive(Deserialize, Debug)]
pub struct LibFactory {
    name: String,
    cost_buy: u32,
    cost_sell: u32,
    time: u32,
    drops: Vec<LibFactoryDrop>,
    sprite_path: String,
}

#[derive(Deserialize, Debug)]
pub struct LibFactoryDrop {
    item: ItemRef,
    chance: f32,
}

/// Item reference.
#[derive(Deserialize, Debug)]
pub struct ItemRef(String);
