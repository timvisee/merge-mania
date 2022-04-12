use serde::Deserialize;

use crate::types::ItemRef;

/// Client action: swap two cells.
#[derive(Deserialize, Debug)]
pub struct ClientActionSwap {
    pub cell: u8,
    pub other: u8,
}

/// Client action: merge two cells.
#[derive(Deserialize, Debug)]
pub struct ClientActionMerge {
    pub cell: u8,
    pub other: u8,
}

/// Client action: buy item at cell.
#[derive(Deserialize, Debug)]
pub struct ClientActionBuy {
    pub cell: u8,
    pub item: ItemRef,
}

/// Client action: sell item at cell
#[derive(Deserialize, Debug)]
pub struct ClientActionSell {
    pub cell: u8,
}

/// Client action: reward the given user for an outpost.
#[derive(Deserialize, Debug)]
pub struct ClientActionRewardUser {
    pub outpost_id: u32,
    pub user_id: u32,
}
