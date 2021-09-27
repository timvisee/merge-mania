use rand::Rng;
use serde::Deserialize;

use crate::config::{Config, ConfigItem};
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
    pub tiers: Vec<ConfigProductTier>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigProductTier {
    pub id: u32,
    pub name: String,
    pub products: Vec<ConfigProduct>,
}

impl ConfigProductTier {
    /// Find a product tier product by its level.
    pub fn level(&self, level: u16) -> Option<&ConfigProduct> {
        self.products.get(level as usize)
    }

    /// Get the maximum level.
    // TODO: a product tier should always have at least one level
    pub fn max_level(&self) -> u16 {
        self.products.len().checked_sub(1).unwrap_or(0) as u16
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigProduct {
    pub name: String,
    pub cost: u32,
    pub sprite_path: String,
}

#[derive(Deserialize, Debug)]
pub struct ConfigFactories {
    pub tiers: Vec<ConfigFactoryTier>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigFactoryTier {
    pub id: u32,
    pub name: String,
    pub levels: Vec<ConfigFactory>,
}

impl ConfigFactoryTier {
    /// Find a factory tier product by its level.
    pub fn level(&self, level: u16) -> Option<&ConfigFactory> {
        self.levels.get(level as usize)
    }

    /// Get the maximum level.
    // TODO: a product tier should always have at least one level
    pub fn max_level(&self) -> u16 {
        self.levels.len().checked_sub(1).unwrap_or(0) as u16
    }
}

#[derive(Deserialize, Debug, Clone)]
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

    /// Select a random drop.
    ///
    /// This takes chance configuration into account.
    pub fn random_drop(&self) -> Option<ItemRef> {
        let mut rng = rand::thread_rng();

        let total = self.drops.iter().map(|d| d.chance).sum::<f64>();
        let mut value = rng.gen::<f64>();

        self.drops
            .iter()
            .skip_while(move |d| {
                value -= d.chance;
                value >= 0.0
            })
            .next()
            .map(|d| d.item.clone())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigFactoryDrop {
    item: ItemRef,

    /// Chance float.
    chance: f64,
}

impl ConfigFactoryDrop {
    /// Resolve into config item.
    pub fn into_item(&self, config: &Config) -> Option<ConfigItem> {
        config.find_item(&self.item)
    }
}