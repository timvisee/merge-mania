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

    /// Team ID if part of a game team.
    pub team_id: Option<u32>,
}

impl ClientSession {
    pub fn from_session(config: &Config, session: &Session) -> Self {
        Self {
            name: config.team(session.team_id).unwrap().name.clone(),
            team_id: Some(session.team_id),
        }
    }
}

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

/// Client team stats.
#[derive(Serialize, Default, Debug)]
pub struct ClientTeamStats {
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

impl ClientTeamStats {
    pub fn from_game(game: &GameTeamStats) -> Self {
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
