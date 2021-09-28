pub mod types;

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use toml;

use crate::types::ItemRef;
pub use types::*;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub game: ConfigGame,
    pub teams: Vec<ConfigTeam>,
    pub products: ConfigProducts,
    pub factories: ConfigFactories,
    pub defaults: ConfigDefaults,
}

impl Config {
    /// Get a team.
    pub fn team(&self, team_id: u32) -> Option<&ConfigTeam> {
        self.teams.iter().find(|t| t.id == team_id)
    }

    /// Find an item by reference.
    ///
    /// Returns a configuration product or factory.
    // TODO: cache results by `item_ref`
    // TODO: return arc/ref
    pub fn find_item(&self, item_ref: &ItemRef) -> Option<ConfigItem> {
        let (tier, level) = item_ref.tier_level()?;

        // Find product
        if let Some(tier) = self.find_product_tier(tier) {
            return tier
                .level(level)
                .cloned()
                .map(|p| ConfigItem::Product(tier.clone(), p, level));
        }

        // Find factory
        if let Some(tier) = self.find_factory_tier(tier) {
            return tier
                .level(level)
                .cloned()
                .map(|f| ConfigItem::Factory(tier.clone(), f, level));
        }

        None
    }

    /// Find product tier configuration by tier ID.
    fn find_product_tier(&self, tier: u32) -> Option<&ConfigProductTier> {
        self.products.tiers.iter().find(|t| t.id == tier)
    }

    /// Find factory tier configuration by tier ID.
    fn find_factory_tier(&self, tier: u32) -> Option<&ConfigFactoryTier> {
        self.factories.tiers.iter().find(|t| t.id == tier)
    }
}

/// A config product or factory tier.
#[derive(Debug)]
pub enum ConfigTierItem {
    Product(ConfigProductTier),
    Factory(ConfigFactoryTier),
}

/// A config product or factory item.
///
/// Contains both tier and product.
#[derive(Debug)]
pub enum ConfigItem {
    Product(ConfigProductTier, ConfigProduct, u16),
    Factory(ConfigFactoryTier, ConfigFactory, u16),
}

/// Load config from disk.
// TODO: expose errors through error enum
pub fn load() -> Result<Config, ()> {
    debug!("Loading game configuration...");

    let path = PathBuf::from(crate::CONFIG_PATH);

    let data = fs::read(path).expect("failed to read config.toml");

    let config = toml::from_slice(&data).expect("failed to parse config.toml");

    // TODO: validate config
    // TODO: - ensure unique IDs
    // TODO: - ensure sprite paths a correct

    info!("Game configuration loaded");

    Ok(config)
}
