#[macro_use]
extern crate rocket;

pub(crate) mod config;
pub(crate) mod game;
#[cfg(test)]
pub mod tests;
pub(crate) mod types;
pub(crate) mod util;

use game::Game;
use rocket::fs::{relative, FileServer};
use types::*;

/// Inventory width/height.
pub const INV_WIDTH: u16 = 8;

/// Inventory slot count.
pub const INV_SIZE: u16 = INV_WIDTH * 2;

// fn main() {
//     let config = config::load().expect("failed to load game config");

//     let game = Game::from(config);

//     let inventory = crate::types::Inventory::default();
// }

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![hello])
        .mount("/", FileServer::from(relative!("/public")))
}

#[get("/")]
async fn hello() -> &'static str {
    "Hello, world!"
}
