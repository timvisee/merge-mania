pub mod types;

use std::sync::Mutex;

use rand::Rng;
use serde::Deserialize;
use tokio::time::{self, Duration};

use crate::config::ConfigFactoryTier;
use crate::state::SharedState;
use crate::util::{i_to_xy, xy_to_i};
pub use types::*;

/// A simple game loop.
pub(crate) async fn run(state: SharedState) {
    let mut interval = time::interval(Duration::from_secs(crate::TICK_SEC));

    loop {
        // Wait for tick
        interval.tick().await;

        // TODO: do not progress in game.running is false

        // Process ticks
        // TODO: catch up to missed ticks here
        state.game.process_ticks(1);
    }
}

/// Represents runnable game state.
#[derive(Default)]
pub struct Game {
    /// Whether the game is running.
    running: bool,

    /// Current game tick.
    // TODO: switch to atomic?
    tick: Mutex<usize>,

    /// Team state.
    teams: Vec<GameTeam>,
}

impl Game {
    /// Process the game by the given amount of ticks.
    ///
    /// This should be invoked from a game loop.
    /// Calls `update` on the full game state afterwards.
    pub fn process_ticks(&self, ticks: usize) {
        let tick = {
            let mut lock = self.tick.lock().unwrap();
            *lock += ticks;
            *lock
        };

        println!("TODO: run team ticks");
        // TODO: self.update(tick);
    }
}

impl Update for Game {
    fn update(&mut self, tick: usize) {
        for team in self.teams.iter_mut() {
            team.update(tick);
        }
    }
}

pub trait Update {
    /// Update this state upto the given tick.
    fn update(&mut self, tick: usize);
}
