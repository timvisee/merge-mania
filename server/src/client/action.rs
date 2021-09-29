use serde::Deserialize;

use crate::types::ItemRef;

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
