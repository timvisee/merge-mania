use serde::Deserialize;

/// An amount of money or items.
// TODO: find better name for this
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Amount {
    /// Money amount.
    Money(usize),

    /// An item with quantity.
    Item(ItemRef, u32),
}

/// Item reference.
#[derive(Deserialize, Debug)]
pub struct ItemRef(String);
