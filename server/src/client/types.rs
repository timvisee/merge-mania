use rand::prelude::*;
use serde::Serialize;

use crate::config::{
    Config, ConfigFactory, ConfigFactoryTier, ConfigItem, ConfigProduct, ConfigProductTier,
};
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
    name: String,

    /// Sell price in money. May be None if price cannot be represented by money.
    sell_price: Option<u64>,
    sprite: String,

    /// Whether there is a higher level of this tier available.
    can_upgrade: bool,
}

impl ClientProduct {
    pub fn from_game(game: &GameProduct) -> Result<Self, ()> {
        let config = game.config_item.as_ref().ok_or(())?;
        Ok(Self {
            tier: game.tier,
            level: game.level,
            name: config.name.clone(),
            sell_price: Some(config.cost),
            sprite: config.sprite_path.clone(),
            can_upgrade: game.can_upgrade(),
        })
    }
}

/// Inventory factory.
#[derive(Serialize, Debug)]
pub struct ClientFactory {
    tier: u32,
    level: u16,
    name: String,
    interval: u32,

    /// Sell price in money. May be None if price cannot be represented by money.
    sell_price: Option<u64>,
    sprite: String,

    /// Whether there is a higher level of this tier available.
    can_upgrade: bool,
}

impl ClientFactory {
    pub fn from_game(game: &GameFactory) -> Result<Self, ()> {
        let config = game.config_item.as_ref().ok_or(())?;
        Ok(Self {
            tier: game.tier,
            level: game.level,
            name: config.name.clone(),
            interval: config.time,
            sell_price: amount_only_money(&config.cost_sell),
            sprite: config.sprite_path.clone(),
            can_upgrade: game.can_upgrade(),
        })
    }
}

/// An inventory.
#[derive(Serialize, Debug)]
pub struct ClientInventory {
    money: u64,
    energy: u64,
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

/// For given amounts, if all are money, get money sum.
pub fn amount_only_money(amounts: &[Amount]) -> Option<u64> {
    // All amounts must be money
    if amounts.iter().all(|a| matches!(a, Amount::Money(_))) {
        return Some(
            amounts
                .iter()
                .map(|a| match a {
                    Amount::Money(money) => money,
                    _ => unreachable!(),
                })
                .sum(),
        );
    }
    None
}
