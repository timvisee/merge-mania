use rand::prelude::*;
use serde::Serialize;

use crate::config::Config;
use crate::game::types::*;
use crate::types::{Amount, ItemRef};

/// Represents a team.
#[derive(Serialize, Debug)]
pub struct ClientTeam {
    id: u32,
    name: String,
    inventory: ClientInventory,
}

impl ClientTeam {
    pub fn from_game(game: &GameTeam) -> Result<Self, ()> {
        Ok(Self {
            id: game.id,
            name: game.config.as_ref().ok_or(())?.name.clone(),
            inventory: ClientInventory::from_game(&game.inventory)?,
        })
    }
}

/// Game item.
#[derive(Serialize, Debug)]
pub struct ClientItem {
    /// Item ID.
    #[serde(rename = "ref")]
    pub id: ItemRef,

    /// Item display name.
    pub name: String,

    /// Tier display name.
    pub tier: String,

    /// Optional: label to render on client.
    pub label: Option<String>,

    /// Sell price.
    pub sell: u64,

    /// Optional: drop item after number of ticks.
    pub drop_interval: Option<u64>,

    /// Optional: maximum number of drops before destruction.
    pub drop_limit: Option<u32>,

    /// Sprite file path.
    pub sprite: String,

    /// Whether this item is mergeable.
    pub mergeable: bool,
}

impl ClientItem {
    pub fn from_game(game: &GameItem) -> Result<Self, ()> {
        let config = game.config.as_ref().unwrap();
        Ok(Self {
            id: game.id.clone(),
            tier: config.tier.clone(),
            name: config.name.clone(),
            label: config.label.clone(),
            sell: config.sell,
            drop_interval: config.drop_interval.clone(),
            drop_limit: config.drop_limit.clone(),
            sprite: config.sprite_path.clone(),
            mergeable: game.can_upgrade(),
        })
    }
}

/// An inventory.
#[derive(Serialize, Debug)]
pub struct ClientInventory {
    money: u64,
    energy: u64,

    #[serde(flatten)]
    grid: ClientInventoryGrid,
}

impl ClientInventory {
    pub fn from_game(game: &GameInventory) -> Result<Self, ()> {
        Ok(Self {
            money: game.money,
            energy: game.energy,
            grid: ClientInventoryGrid::from_game(&game.grid)?,
        })
    }
}

/// An inventory grid.
#[derive(Serialize, Debug)]
pub struct ClientInventoryGrid {
    items: Vec<Option<ClientItem>>,
}

impl ClientInventoryGrid {
    pub fn from_game(game: &GameInventoryGrid) -> Result<Self, ()> {
        let mut items = Vec::with_capacity(game.items.len());

        for item in &game.items {
            items.push(match item {
                Some(item) => Some(ClientItem::from_game(item)?),
                None => None,
            });
        }

        Ok(Self { items })
    }
}
