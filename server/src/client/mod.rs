//! Merge Mania client types.
//!
//! Models that are send to the client.

pub mod action;
pub mod types;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::ConfigItem;
use crate::types::ItemRef;
pub use action::*;
pub use types::*;

/// A message to send to a client.
#[derive(Serialize, Debug)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MsgSend {
    Ok(MsgSendKind),
    Err(String),
}

impl From<MsgSendKind> for MsgSend {
    fn from(kind: MsgSendKind) -> Self {
        MsgSend::Ok(kind)
    }
}

/// A message to receive from a client.
#[derive(Deserialize, Debug)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MsgRecv {
    Ok(MsgRecvKind),
    Err(String),
}

impl From<MsgRecvKind> for MsgRecv {
    fn from(kind: MsgRecvKind) -> Self {
        MsgRecv::Ok(kind)
    }
}

/// Message kinds to send to a client.
#[derive(Serialize, Debug)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum MsgSendKind {
    /// Ping request.
    Ping,

    /// Game item configuration.
    ConfigItems(HashMap<ItemRef, ConfigItem>),

    /// Inventory state for current client team.
    Inventory(ClientInventory),

    /// Inventory cell state for current client team.
    InventoryCell { index: u8, item: Option<ClientItem> },

    /// Toast notification.
    Toast(String),
}

/// Message kinds to receive from a client.
#[derive(Deserialize, Debug)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum MsgRecvKind {
    // TODO: kind for authentication token
    /// Action: swap two cells
    ActionSwap(ClientActionSwap),

    /// Action: merge two cells
    ActionMerge(ClientActionMerge),

    /// Action: buy item at cell
    ActionBuy(ClientActionBuy),

    /// Action: sell item at cell
    ActionSell(ClientActionSell),

    /// Action: scan a code to gain energy.
    ActionScanCode,

    /// Request inventory state update.
    GetInventory,
}
