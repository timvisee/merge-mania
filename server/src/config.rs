use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use toml;

use crate::types::{LibFactories, LibProducts};

const CONFIG_PATH: &str = "./../config/config.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub teams: Vec<ConfigTeam>,
    pub products: LibProducts,
    pub factories: LibFactories,
}

/// Load config from disk.
// TODO: expose errors through error enum
pub fn load() -> Result<Config, ()> {
    let path = PathBuf::from(CONFIG_PATH);

    let data = fs::read(path).expect("failed to read config.toml");

    let config = toml::from_slice(&data).expect("failed to parse config.toml");

    Ok(config)
}

/// Represents a configured team.
#[derive(Deserialize, Debug, Clone)]
pub struct ConfigTeam {
    pub id: u32,
    pub name: String,
    pub password: String,
}
