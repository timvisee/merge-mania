use rand::prelude::*;
use serde::Serialize;

use crate::config::{
    Config, ConfigFactory, ConfigFactoryTier, ConfigItem, ConfigProduct, ConfigProductTier,
};
use crate::game::types::*;
use crate::types::ItemRef;

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

/// Inventory item.
#[derive(Serialize, Debug)]
pub enum ClientItem {
    Product(ClientProduct),
    Factory(ClientFactory),
}

impl ClientItem {
    pub fn from_game(game: &GameItem) -> Result<Self, ()> {
        Ok(match &game {
            GameItem::Product(product) => ClientItem::Product(ClientProduct::from_game(product)?),
            GameItem::Factory(factory) => ClientItem::Factory(ClientFactory::from_game(factory)?),
        })
    }
}

/// Inventory product.
#[derive(Serialize, Debug)]
pub struct ClientProduct {
    tier: u32,
    level: u16,
    sprite: String,
}

impl ClientProduct {
    pub fn from_game(game: &GameProduct) -> Result<Self, ()> {
        Ok(Self {
            tier: game.tier,
            level: game.level,
            sprite: game.config_item.as_ref().ok_or(())?.sprite_path.clone(),
        })
    }
}

/// Inventory factory.
#[derive(Serialize, Debug)]
pub struct ClientFactory {
    tier: u32,
    level: u16,
    sprite: String,
}

impl ClientFactory {
    pub fn from_game(game: &GameFactory) -> Result<Self, ()> {
        Ok(Self {
            tier: game.tier,
            level: game.level,
            sprite: game.config_item.as_ref().ok_or(())?.sprite_path.clone(),
        })
    }
}

/// An inventory.
#[derive(Serialize, Debug)]
pub struct ClientInventory {
    money: usize,
    energy: usize,
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
