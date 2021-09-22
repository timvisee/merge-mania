use std::iter;
use std::sync::RwLock;

use rand::{distributions::Alphanumeric, thread_rng, Rng};

/// Session token length.
const TOKEN_LENGTH: usize = 64;

/// A basic session manager.
pub struct SessionManager {
    sessions: RwLock<Vec<Session>>,
}

impl SessionManager {
    /// Construct a new session manager.
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(vec![]),
        }
    }

    /// Add session for a given team.
    ///
    /// Returns session with corresponding token.
    pub fn add(&self, team_id: u32) -> Session {
        let session = Session::new_random_token(team_id);
        self.sessions.write().unwrap().push(session.clone());
        session
    }

    /// Remove session with the given token.
    ///
    /// Returns `true` if a session was removed.
    pub fn remove(&self, token: &str) -> bool {
        let mut sessions = self.sessions.write().unwrap();
        match sessions.iter().position(|session| session.is_token(token)) {
            Some(i) => {
                sessions.remove(i);
                true
            }
            None => false,
        }
    }

    /// Get a session.
    pub fn get(&self, token: &str) -> Option<Session> {
        self.sessions
            .read()
            .unwrap()
            .iter()
            .find(|session| session.is_token_and_valid(token))
            .cloned()
    }

    /// Check whether the given token is valid.
    pub fn is_valid(&self, token: &str) -> bool {
        self.sessions
            .read()
            .unwrap()
            .iter()
            .any(|session| session.is_token_and_valid(token))
    }
}

/// A team session.
#[derive(Clone)]
pub struct Session {
    // Team this session is for.
    team_id: u32,

    // Session token.
    token: String,
}

impl Session {
    /// Construct a new session with a random token.
    fn new_random_token(team_id: u32) -> Self {
        Self {
            team_id,
            token: gen_token(),
        }
    }

    /// Check whether the given token matches this session.
    pub fn is_token(&self, token: &str) -> bool {
        valid_token_format(token) && self.token.trim() == token.trim()
    }

    /// Check whether the given token matches this session and this session is valid.
    pub fn is_token_and_valid(&self, token: &str) -> bool {
        self.is_token(token)
    }

    /// Get session token.
    pub fn token(&self) -> &str {
        &self.token
    }
}

/// Check whether the token format is valid.
fn valid_token_format(token: &str) -> bool {
    token.len() == TOKEN_LENGTH
        && token
            .chars()
            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'))
}

/// Generate a secure random token.
fn gen_token() -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(TOKEN_LENGTH)
        .collect()
}
