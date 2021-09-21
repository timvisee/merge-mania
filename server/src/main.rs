pub(crate) mod config;
pub(crate) mod types;

use types::*;

/// Inventory width/height.
pub const INV_WIDTH: u16 = 8;

/// Inventory slot count.
pub const INV_SIZE: u16 = INV_WIDTH * 2;

fn main() {
    let config = config::load();
    dbg!(config);
}
