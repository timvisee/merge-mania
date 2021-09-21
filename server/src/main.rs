#[macro_use]
extern crate rocket;

pub(crate) mod config;
pub(crate) mod game;
#[cfg(test)]
pub mod tests;
pub(crate) mod types;
pub(crate) mod util;

use rocket::fs::{relative, FileServer, Options};
use rocket::http::Method;
use rocket::serde::json::Json;
use rocket_cors::{AllowedOrigins, CorsOptions};

use game::Game;
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
        .mount("/api", routes![api_teams])
        .mount(
            "/",
            FileServer::new(
                relative!("../client/dist"),
                Options::Index | Options::NormalizeDirs,
            ),
        )
}

#[get("/teams")]
async fn api_teams() -> Json<Vec<TestTeam>> {
    // Build list of test teams
    let teams = vec![
        TestTeam::new(1, "Team 1".into()),
        TestTeam::new(2, "Team 2".into()),
        TestTeam::new(3, "Team 3".into()),
    ];

    Json(teams)
}

#[derive(rocket::serde::Serialize, Debug)]
struct TestTeam {
    id: u32,
    name: String,
}

impl TestTeam {
    pub fn new(id: u32, name: String) -> Self {
        TestTeam { id, name }
    }
}
