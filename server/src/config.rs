use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use toml;

use crate::types::{LibFactories, LibProducts};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub teams: Vec<ConfigTeam>,
    pub products: LibProducts,
    pub factories: LibFactories,
}

/// Load config from disk.
// TODO: expose errors through error enum
pub fn load() -> Result<Config, ()> {
    println!("Loading game configuration...");

    let path = PathBuf::from(crate::CONFIG_PATH);

    let data = fs::read(path).expect("failed to read config.toml");

    let config = toml::from_slice(&data).expect("failed to parse config.toml");

    // TODO: validate config
    // TODO: - ensure unique IDs
    // TODO: - ensure sprite paths a correct

    println!("Game configuration loaded.");

    Ok(config)
}

/// Represents a configured team.
#[derive(Deserialize, Debug, Clone)]
pub struct ConfigTeam {
    pub id: u32,
    pub name: String,
    pub password: String,
}
