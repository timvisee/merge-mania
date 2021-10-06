use serde::{Deserialize, Serialize};
use warp::reply::{json, Json};

use crate::auth::SessionToken;
use crate::state::SharedState;

/// Get list of teams.
pub fn teams(state: SharedState) -> Json {
    let teams: Vec<TeamData> = state.config.teams.iter().map(|t| t.into()).collect();
    json(&teams)
}

#[derive(Serialize, Debug)]
pub struct TeamData {
    id: u32,
    name: String,
}

impl From<&crate::config::ConfigTeam> for TeamData {
    fn from(team: &crate::config::ConfigTeam) -> Self {
        TeamData {
            id: team.id,
            name: team.name.clone(),
        }
    }
}

use warp::Reply;

/// Login route.
pub fn login(data: LoginData, state: SharedState) -> Box<dyn Reply> {
    // Find team with ID
    let config_team = match state.config.teams.iter().find(|t| t.id == data.team) {
        Some(team) => team,
        None => {
            return Box::new(crate::web::ApiError::from(crate::lang::TEAM_UNKNOWN).to_reply());
        }
    };

    // Validate password
    if config_team.password != data.password {
        return Box::new(crate::web::ApiError::from(crate::lang::TEAM_INCORRECT_PASS).to_reply());
    }

    // Create session
    let session = state.sessions.add(data.team);

    Box::new(json(&SessionToken {
        token: session.token().into(),
    }))
}

/// Logout route.
pub fn logout(data: SessionToken, state: SharedState) -> impl Reply {
    // TODO: we might want to check session token validity here

    state.sessions.remove(&data.token);
    json(&true)
}

/// Login data.
#[derive(Deserialize, Debug)]
pub struct LoginData {
    team: u32,
    #[serde(default)]
    password: String,
}

/// Session validation route.
pub fn validate(data: SessionToken, state: SharedState) -> impl Reply {
    json(&state.sessions.is_valid(&data.token))
}
