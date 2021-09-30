pub mod types;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer};
use toml;

use crate::types::ItemRef;
pub use types::*;

// TODO: remove this?
#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct ItemContainer {
    items: Vec<ConfigItem>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    /// Game settings.
    pub game: ConfigGame,

    /// Teams.
    pub teams: Vec<ConfigTeam>,

    /// Team defaults.
    pub defaults: ConfigDefaults,

    /// Game items.
    #[serde(deserialize_with = "vec_to_map")]
    pub items: HashMap<ItemRef, ConfigItem>,
}

impl Config {
    /// Get a team by ID.
    pub fn team(&self, team_id: u32) -> Option<&ConfigTeam> {
        self.teams.iter().find(|t| t.id == team_id)
    }

    /// Get item by reference.
    ///
    /// Returns `None` if it doesn't exist.
    pub fn item(&self, item_ref: &ItemRef) -> Option<&ConfigItem> {
        self.items.get(item_ref)
    }
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
    // TODO: - call item.validate();

    info!("Game configuration loaded");

    Ok(config)
}

/// Deserialize a `Vec` into a `HashMap` by key.
fn vec_to_map<'de, D>(d: D) -> Result<HashMap<ItemRef, ConfigItem>, D::Error>
where
    D: Deserializer<'de>,
{
    let items: Vec<ConfigItem> = Vec::deserialize(d)?;
    Ok(items.into_iter().map(|i| (i.id.clone(), i)).collect())
}
