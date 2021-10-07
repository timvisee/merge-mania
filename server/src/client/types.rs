use std::collections::HashSet;
use std::sync::atomic::Ordering;

use rand::prelude::*;
use serde::Serialize;

use crate::auth::Session;
use crate::config::Config;
use crate::game::types::*;
use crate::types::{Amount, ItemRef};

/// Client session state.
// TODO: add admin boolean property
#[derive(Serialize, Debug)]
pub struct ClientSession {
    /// Account display name.
    pub name: String,

    /// User ID.
    pub user_id: u32,

    /// Whether user has permission to play the game.
    pub role_game: bool,

    /// Whether user has permission to administer the game.
    pub role_admin: bool,
}

impl ClientSession {
    pub fn from_session(config: &Config, session: &Session) -> Option<Self> {
        // Get config user
        let user = config.user(session.user_id)?;

        Some(Self {
            name: config.user(session.user_id).unwrap().name.clone(),
            user_id: session.user_id,
            role_game: user.role_game,
            role_admin: user.role_admin,
        })
    }
}

/// Represents a user.
#[derive(Serialize, Debug)]
pub struct ClientUser {
    id: u32,
    name: String,
    inventory: ClientInventory,
}

impl ClientUser {
    pub fn from_game(game: &GameUser) -> Result<Self, ()> {
        Ok(Self {
            id: game.id,
            name: game.config.as_ref().ok_or(())?.name.clone(),
            inventory: ClientInventory::from_game(&game.inventory)?,
        })
    }
}

/// Game item.
#[derive(Serialize, Debug, Clone)]
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
    pub money: u64,
    pub energy: u64,

    #[serde(flatten)]
    pub grid: ClientInventoryGrid,

    pub discovered: HashSet<ItemRef>,
}

impl ClientInventory {
    pub fn from_game(game: &GameInventory) -> Result<Self, ()> {
        Ok(Self {
            money: game.money,
            energy: game.energy,
            grid: ClientInventoryGrid::from_game(&game.grid)?,
            discovered: game.discovered.clone(),
        })
    }
}

/// An inventory grid.
#[derive(Serialize, Debug)]
pub struct ClientInventoryGrid {
    pub items: Vec<Option<ClientItem>>,
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

/// Client user stats.
#[derive(Serialize, Default, Debug)]
pub struct ClientUserStats {
    /// Number of merges by user.
    merge_count: u32,

    /// Number of items bought by user.
    buy_count: u32,

    /// Number of items sold by user.
    sell_count: u32,

    /// Number of item swaps (moves) by user.
    swap_count: u32,

    /// Number of codes scanned by user.
    code_count: u32,

    /// Number of items dropped by factories.
    drop_count: u32,

    /// Money spent by user.
    money_spent: u64,

    /// Money earned by user from selling.
    money_earned: u64,

    /// Energy spent by user.
    energy_spent: u64,

    /// Energy earned by user from scanning codes.
    energy_earned: u64,
}

impl ClientUserStats {
    pub fn from_game(game: &GameUserStats) -> Self {
        Self {
            merge_count: game.merge_count.load(Ordering::Relaxed),
            buy_count: game.buy_count.load(Ordering::Relaxed),
            sell_count: game.sell_count.load(Ordering::Relaxed),
            swap_count: game.swap_count.load(Ordering::Relaxed),
            code_count: game.code_count.load(Ordering::Relaxed),
            drop_count: game.drop_count.load(Ordering::Relaxed),
            money_spent: game.money_spent.load(Ordering::Relaxed),
            money_earned: game.money_earned.load(Ordering::Relaxed),
            energy_spent: game.energy_spent.load(Ordering::Relaxed),
            energy_earned: game.energy_earned.load(Ordering::Relaxed),
        }
    }
}

/// Client leaderboard user.
#[derive(Serialize, Debug)]
pub struct ClientLeaderboardUser {
    /// Account display name.
    pub name: String,

    /// User money.
    pub money: u64,
}

impl ClientLeaderboardUser {
    pub fn from_game(game: &GameUser) -> Result<Self, ()> {
        Ok(Self {
            name: game.config.as_ref().ok_or(())?.name.clone(),
            money: game.inventory.money,
        })
    }
}
