use serde::{Deserialize, Serialize};
use warp::reply::{json, Json, Reply};

use crate::auth::SessionToken;
use crate::client::ClientSession;
use crate::state::SharedState;

/// Get list of users.
pub fn users(state: SharedState) -> Json {
    let users: Vec<UserData> = state.config.users.iter().map(|t| t.into()).collect();
    json(&users)
}

#[derive(Serialize, Debug)]
pub struct UserData {
    id: u32,
    name: String,
}

impl From<&crate::config::ConfigUser> for UserData {
    fn from(user: &crate::config::ConfigUser) -> Self {
        UserData {
            id: user.id,
            name: user.name.clone(),
        }
    }
}

/// Login result.
#[derive(Serialize, Debug)]
pub struct LoginResult {
    token: SessionToken,
    session: ClientSession,
}

/// Login route.
pub fn login(data: LoginData, state: SharedState) -> Box<dyn Reply> {
    // Find user with ID
    let config_user = match state.config.users.iter().find(|t| t.id == data.user) {
        Some(user) => user,
        None => {
            return Box::new(crate::web::ApiError::from(crate::lang::USER_UNKNOWN).to_reply());
        }
    };

    // Validate password
    if config_user.password != data.password {
        return Box::new(crate::web::ApiError::from(crate::lang::USER_INCORRECT_PASS).to_reply());
    }

    // Create session
    let session = state.sessions.add(data.user);

    // Construct client session object
    let client_session = match ClientSession::from_session(&state.config, &session) {
        Some(session) => session,
        None => {
            error!("Failed to create client session object");
            return Box::new(crate::web::ApiError::from(crate::lang::INTERNAL_ERROR).to_reply());
        }
    };

    // Build response
    let response = LoginResult {
        token: SessionToken {
            token: session.token().into(),
        },
        session: client_session,
    };
    Box::new(json(&response))
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
    user: u32,
    #[serde(default)]
    password: String,
}

/// Session validation route.
pub fn validate(data: SessionToken, state: SharedState) -> impl Reply {
    // Get session, fail if unavailable
    let session = match state.sessions.get_valid(&data.token) {
        Some(session) => session,
        None => {
            warn!(
                "Client tried to validate with invalid session token: {}",
                data.token,
            );
            return json(&());
        }
    };

    // Construct client session object
    let session = match ClientSession::from_session(&state.config, &session) {
        Some(session) => session,
        None => {
            error!("Failed to create client session object");
            return json(&());
        }
    };

    json(&session)
}
