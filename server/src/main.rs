pub(crate) mod config;
pub(crate) mod game;
pub(crate) mod types;

use game::Game;
use types::*;

/// Inventory width/height.
pub const INV_WIDTH: u16 = 8;

/// Inventory slot count.
pub const INV_SIZE: u16 = INV_WIDTH * 2;

fn main() {
    let config = config::load().expect("failed to load game config");

    let game = Game::from(config);
}
