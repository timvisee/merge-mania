pub mod types;

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use toml;

pub use types::*;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub teams: Vec<ConfigTeam>,
    pub products: ConfigProducts,
    pub factories: ConfigFactories,
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
