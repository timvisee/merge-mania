use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::types::{Amount, ItemRef};
use crate::util::{i_to_xy, one, xy_to_i};

/// Game configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct ConfigGame {
    /// Milliseconds per game tick.
    pub tick_millis: u64,

    /// Whether to reset the game state on start.
    pub reset: bool,

    /// Whether to immediately start new games.
    pub start: bool,
}

/// Outposts configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct ConfigOutposts {
    /// Secret value.
    pub secret: String,

    /// Minimum money to earn.
    pub money_min: u64,

    /// Maximum money to earn.
    pub money_max: u64,

    /// Money multiplier.
    pub money_multiplier: u64,

    /// Minimum energy to earn.
    pub energy_min: u64,

    /// Maximum energy to earn.
    pub energy_max: u64,

    /// Energy multiplier.
    pub energy_multiplier: u64,
}

/// Represents a configured user.
///
/// May be a game user or admin.
#[derive(Deserialize, Debug, Clone)]
pub struct ConfigUser {
    pub id: u32,

    /// Whether user has permission to play the game.
    pub role_game: bool,

    /// Whether user has permission to administer the game.
    #[serde(default)]
    pub role_admin: bool,
    pub name: String,
    pub password: String,
}

/// User defaults.
#[derive(Deserialize, Debug, Clone)]
pub struct ConfigDefaults {
    /// Default user money.
    pub money: u64,

    /// Default user energy.
    pub energy: u64,

    /// Default user inventory items.
    pub inventory: Vec<ItemRef>,

    /// Optional: to merge, merge cost
    pub merge_cost: Option<Vec<Amount>>,
}

/// Game item configuration.
// TODO: do not clone
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigItem {
    /// Unique item ID.
    #[serde(rename = "ref")]
    pub id: ItemRef,

    /// Optional: merge into item ID.
    pub merge: Option<ItemRef>,

    /// Item display name.
    pub name: String,

    /// Tier display name.
    pub tier: String,

    /// Optional: description to render on client.
    pub description: Option<String>,

    /// Optional: label to render on client.
    pub label: Option<String>,

    /// Optional: if buyable, buy cost
    pub buy: Option<Vec<Amount>>,

    /// Sell price.
    pub sell: u64,

    /// Optional: to merge, merge cost, overrides global default
    pub merge_cost: Option<Vec<Amount>>,

    /// Optional: drop item after number of ticks.
    pub drop_interval: Option<u64>,

    /// Optional: maximum number of drops before destruction.
    pub drop_limit: Option<u32>,

    /// Optional: possible drops.
    #[serde(default)]
    pub drops: Vec<ConfigDrop>,

    /// Sprite file path.
    #[serde(rename = "sprite", alias = "sprite_path")]
    pub sprite_path: String,

    /// Client ordering value.
    #[serde(default)]
    pub client_order: i16,
}

impl ConfigItem {
    /// Validate correctness.
    pub fn validate(&self, config: &Config) -> Result<(), ()> {
        // TODO: validate item is correct
        // TODO: - unique ID
        // TODO: - merge ID okay
        // TODO: - no empty tier name / name / label
        // TODO: - no drop interval for no drops
        // TODO: - any drops if drop interval is convered
        // TODO: - drop item IDs
        // TODO: - sprite path must exist
        Ok(())
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigDrop {
    /// Item to drop.
    pub item: ItemRef,

    /// Chance float.
    #[serde(default = "one")]
    pub chance: f64,
}

impl ConfigDrop {
    /// Resolve into config item.
    pub fn into_item(&self, config: &Config) -> Option<ConfigItem> {
        config.item(&self.item).cloned()
    }
}
