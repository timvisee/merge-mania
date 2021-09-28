//! Merge Mania client types.
//!
//! Models that are send to the client.

pub mod types;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use types::*;

/// A message.
#[derive(Serialize, Debug)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Msg {
    Ok(MsgKind),
    Err(String),
}

impl From<MsgKind> for Msg {
    fn from(kind: MsgKind) -> Self {
        Msg::Ok(kind)
    }
}

/// Message kinds.
#[derive(Serialize, Debug)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum MsgKind {
    /// Ping request.
    Ping,

    /// Inventory state for current client team.
    Inventory(ClientInventory),
}
