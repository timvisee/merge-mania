use std::fs;
use std::iter;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use warp::filters::ws::{Message, WebSocket};

/// Session token length.
const TOKEN_LENGTH: usize = 64;

/// Unique client ID provider.
static CLIENT_IDS: AtomicUsize = AtomicUsize::new(1);

/// A basic session manager.
#[derive(Serialize, Deserialize, Debug)]
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

    /// Add session for a given user.
    ///
    /// Returns session with corresponding token.
    pub fn add(&self, user_id: u32) -> Session {
        let session = Session::new_random_token(user_id);
        self.sessions.write().unwrap().push(session.clone());

        // TODO: properly save, handle errors
        self.save().expect("failed to save");

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
            }
            None => return false,
        }

        // TODO: properly save, handle errors
        self.save().expect("failed to save");

        true
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

    /// Get a session and explicitly check it is valid.
    pub fn get_valid(&self, token: &str) -> Option<Session> {
        self.get(token)
    }

    /// Check whether the given token is valid.
    pub fn is_valid(&self, token: &str) -> bool {
        self.sessions
            .read()
            .unwrap()
            .iter()
            .any(|session| session.is_token_and_valid(token))
    }

    /// Load sessions from file.
    pub fn load() -> Result<Self, ()> {
        // Load default if file doesn't exist
        let path = PathBuf::from(crate::SESSIONS_SAVE_PATH);
        if !path.is_file() {
            info!("No sessions file, starting fresh");
            return Ok(Self::new());
        }

        // Load data from file
        info!("Loading sessions from file");
        trace!("Reading sessions file...");
        let data = fs::read(path).expect("failed to read sessions file");

        // Deserialize
        trace!("Deserializing sessions data...");
        match serde_json::from_slice(data.as_slice()) {
            Ok(state) => Ok(state),
            Err(err) => {
                error!(
                    "Failed to load sessions from file, couldn't deserialize: {}",
                    err
                );
                Err(())
            }
        }
    }

    /// Save sessions to file.
    pub fn save(&self) -> Result<(), ()> {
        info!("Saving sessions to file");

        // Serialize sessions
        trace!("Serializing sessions...");
        let data = match serde_json::to_vec(self) {
            Ok(data) => data,
            Err(err) => {
                error!(
                    "Failed to save sessions to file, couldn't serialize: {}",
                    err
                );
                return Err(());
            }
        };

        // Write to file
        trace!("Writing sessions to file...");
        match fs::write(crate::SESSIONS_SAVE_PATH, data.as_slice()) {
            Ok(result) => Ok(result),
            Err(err) => {
                error!("Failed to save sessions to file: {}", err);
                Err(())
            }
        }
    }
}

/// A user session.
#[derive(Serialize, Deserialize, Debug, Clone)]
// TODO: add admin property
pub struct Session {
    // User this session is for.
    // TODO: make option
    pub user_id: u32,

    // Session token.
    token: String,
}

impl Session {
    /// Construct a new session with a random token.
    fn new_random_token(user_id: u32) -> Self {
        Self {
            user_id,
            token: generate_token(),
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

/// A basic client connection manager.
///
/// Tracks active client websocket connections.
pub struct ClientManager {
    pub clients: RwLock<Vec<Client>>,
}

impl ClientManager {
    /// Construct a new client manager.
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(vec![]),
        }
    }

    /// Register a client.
    pub fn register(&self, client: Client) {
        self.clients.write().unwrap().push(client);
    }

    /// Unregister a client.
    pub fn unregister(&self, client_id: usize) -> bool {
        let mut clients = self.clients.write().unwrap();
        match clients.iter().position(|c| c.client_id == client_id) {
            Some(i) => {
                clients.remove(i);
                true
            }
            None => false,
        }
    }

    /// Find the user ID for a given client.
    pub fn client_user_id(&self, client_id: usize) -> Option<u32> {
        let clients = self.clients.read().unwrap();
        Some(clients.iter().find(|c| c.client_id == client_id)?.user_id)
    }
}

/// An active and authenticated client connection.
pub struct Client {
    /// Unique websocket client ID.
    pub client_id: usize,

    /// Authenticated user ID.
    pub user_id: u32,

    /// Message send queue.
    // TODO: make this private, send through JSON serialize function instead
    pub tx: mpsc::UnboundedSender<Message>,
}

impl Client {
    /// Construct a new client.
    pub fn new(client_id: usize, user_id: u32, tx: mpsc::UnboundedSender<Message>) -> Self {
        Self {
            client_id,
            user_id,
            tx,
        }
    }
}

/// Session data.
#[derive(Serialize, Deserialize, Debug)]
pub struct SessionToken {
    pub token: String,
}

/// Check whether the token format is valid.
fn valid_token_format(token: &str) -> bool {
    token.len() == TOKEN_LENGTH
        && token
            .chars()
            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'))
}

/// Generate a secure random token.
fn generate_token() -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(TOKEN_LENGTH)
        .collect()
}

/// Generate an unique client ID.
pub fn generate_client_id() -> usize {
    CLIENT_IDS.fetch_add(1, Ordering::Relaxed)
}
