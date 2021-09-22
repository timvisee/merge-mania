use crate::types::Inventory;

/// Represents runnable game state.
#[derive(Default)]
pub struct Game {
    teams: Vec<Team>,
}

/// Represents a team.
#[derive(Debug)]
pub struct Team {
    id: u32,
    inventory: Inventory,
}
