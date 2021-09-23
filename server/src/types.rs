use rand::Rng;
use serde::Deserialize;

use crate::util::{i_to_xy, xy_to_i};

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
    pub id: u32,
    pub name: String,
    pub levels: Vec<LibFactory>,
}

#[derive(Deserialize, Debug)]
pub struct LibFactory {
    pub name: String,
    #[serde(default)]
    pub cost_buy: Vec<Amount>,
    pub cost_sell: Vec<Amount>,
    pub time: u32,
    pub drops: Vec<LibFactoryDrop>,
    pub sprite_path: String,
}

impl LibFactory {
    /// Check whether factory is buyable.
    fn can_buy(&self) -> bool {
        !self.cost_buy.is_empty()
    }
}

#[derive(Deserialize, Debug)]
pub struct LibFactoryDrop {
    item: ItemRef,
    chance: f32,
}

/// An amount of money or items.
// TODO: find better name for this
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Amount {
    /// Money amount.
    Money(u32),

    /// An item with quantity.
    Item(ItemRef, u32),
}

/// Item reference.
#[derive(Deserialize, Debug)]
pub struct ItemRef(String);
