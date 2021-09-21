#[macro_use]
extern crate rocket;

pub(crate) mod config;
pub(crate) mod game;
#[cfg(test)]
pub mod tests;
pub(crate) mod types;
pub(crate) mod util;

use rocket::http::Method;
use rocket_cors::{AllowedOrigins, CorsOptions};

use game::Game;
use rocket::fs::{relative, FileServer, Options};
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
    // Set up CORS configuration
    let cors = CorsOptions::default();
    // .allowed_origins(AllowedOrigins::all())
    // .allowed_methods(
    //     vec![Method::Get, Method::Post, Method::Patch]
    //         .into_iter()
    //         .map(From::from)
    //         .collect(),
    // )
    // .allow_credentials(true);

    // Set up and ignite rocket
    rocket::build()
        .attach(cors.to_cors().expect("failed to build CORS config"))
        .mount("/", routes![hello])
        .mount(
            "/",
            FileServer::new(
                relative!("../client/dist"),
                Options::Index | Options::NormalizeDirs,
            ),
        )
}

#[get("/test")]
async fn hello() -> &'static str {
    "Hello, world!"
}
