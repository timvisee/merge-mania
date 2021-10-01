use serde::{Deserialize, Serialize};

/// An amount of money or items.
// TODO: find better name for this
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Amount {
    /// Money amount.
    Money { money: u64 },

    /// Energy amount.
    Energy { energy: u64 },

    /// An item with quantity.
    Item { item: ItemRef, quantity: u8 },
}

/// Item reference.
// TODO: don't use string, use integer properties instead
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ItemRef(String);

impl ItemRef {
    /// Construct from raw tier and level.
    pub fn from(tier: u32, level: u16) -> Self {
        Self(format!("{}.{}", tier, level))
    }

    pub fn tier_level(&self) -> Option<(u32, u16)> {
        let (tier, part) = self.0.split_once('.')?;
        Some((tier.parse().ok()?, part.parse().ok()?))
    }

    /// Get item tier number.
    pub fn tier(&self) -> Option<u32> {
        self.0.split_once('.')?.0.parse().ok()
    }

    /// Get item level.
    pub fn level(&self) -> Option<u16> {
        self.0.split_once('.')?.1.parse().ok()
    }
}
