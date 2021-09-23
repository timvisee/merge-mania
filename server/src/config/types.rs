use rand::Rng;
use serde::Deserialize;

use crate::types::{Amount, ItemRef};
use crate::util::{i_to_xy, xy_to_i};

/// Represents a configured team.
#[derive(Deserialize, Debug, Clone)]
pub struct ConfigTeam {
    pub id: u32,
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct ConfigProducts {
    tiers: Vec<ConfigProductTier>,
}

#[derive(Deserialize, Debug)]
pub struct ConfigProductTier {
    id: u32,
    name: String,
    products: Vec<ConfigProduct>,
}

#[derive(Deserialize, Debug)]
pub struct ConfigProduct {
    name: String,
    cost: u32,
    sprite_path: String,
}

#[derive(Deserialize, Debug)]
pub struct ConfigFactories {
    tiers: Vec<ConfigFactoryTier>,
}

#[derive(Deserialize, Debug)]
pub struct ConfigFactoryTier {
    pub id: u32,
    pub name: String,
    pub levels: Vec<ConfigFactory>,
}

#[derive(Deserialize, Debug)]
pub struct ConfigFactory {
    pub name: String,
    #[serde(default)]
    pub cost_buy: Vec<Amount>,
    pub cost_sell: Vec<Amount>,
    pub time: u32,
    pub drops: Vec<ConfigFactoryDrop>,
    pub sprite_path: String,
}

impl ConfigFactory {
    /// Check whether factory is buyable.
    fn can_buy(&self) -> bool {
        !self.cost_buy.is_empty()
    }
}

#[derive(Deserialize, Debug)]
pub struct ConfigFactoryDrop {
    item: ItemRef,
    chance: f32,
}
