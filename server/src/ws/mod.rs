use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;

use crate::auth::{generate_client_id, Client, Session};
use crate::state::SharedState;

/// New client connected.
pub async fn connected(state: SharedState, ws: WebSocket) {
    // Obtain unique client ID
    let client_id = generate_client_id();
    info!("WS({}): connect", client_id);

    // Split socket sender/receiver, use unbound channel for buffering/flushing
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Handle auth handshake, attempt to gracefully close otherwise
    let session = match handle_auth(state.clone(), client_id, &mut user_ws_rx).await {
        Some(session) => session,
        None => {
            if let Ok(ws) = user_ws_tx.reunite(user_ws_rx) {
                ws.close();
            }
            disconnected(state, client_id);
            return;
        }
    };

    // Use unbounded channel to handle buffering and flushing of messages
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    // Keep flushing client message queue to websocket
    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    warn!("WS({}): send error: {}", client_id, e);
                })
                .await;
        }
    });

    // Register client for a team
    state
        .clients
        .register(Client::new(client_id, session.team_id, tx));

    // Handle client messages
    handle(state.clone(), client_id, &mut user_ws_rx).await;

    // Socket disconnected when this is reached
    disconnected(state, client_id).await;
}

/// Handle authentication message.
///
/// Returns session on success, `None` on failure after which the socket should be closed.
async fn handle_auth(
    state: SharedState,
    client_id: usize,
    rx: &mut SplitStream<WebSocket>,
) -> Option<Session> {
    // TODO: introduce some timeout here

    while let Some(message) = rx.next().await {
        let msg = match message {
            Ok(msg) => msg,
            Err(e) => {
                warn!("WS({}): auth err: {}", client_id, e);
                break;
            }
        };

        // Receive messages
        let msg = match msg.to_str() {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        // Try to parse session token and validate
        let session: crate::auth::SessionData = match serde_json::from_str(msg) {
            Ok(session) => session,
            Err(err) => {
                warn!(
                    "WS({}): auth err, invalid session object: {}",
                    client_id, msg
                );
                continue;
            }
        };

        // Validate session
        let token = &session.token;
        let session = state.sessions.get_valid(token);
        if let Some(session) = &session {
            info!(
                "WS({}): auth success (team: {}, token: {}...)",
                client_id,
                session.team_id,
                &token[0..16]
            );
            // TODO: reply with success message
        } else {
            warn!(
                "WS({}): auth fail, session token invalid ({})",
                client_id, token
            );
        }
        return session;
    }

    None
}

/// Handle client messages.
async fn handle(state: SharedState, client_id: usize, user_ws_rx: &mut SplitStream<WebSocket>) {
    // TODO: timeout if not recieving heartbeat each minute

    while let Some(result) = user_ws_rx.next().await {
        // Parse message
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                warn!("WS({}): error: {}", client_id, e);
                break;
            }
        };

        // Get message text
        let msg = match msg.to_str() {
            Ok(msg) => msg,
            Err(err) => {
                warn!("WS({}): received non-text, skipping: {:?}", client_id, err);
                continue;
            }
        };

        handle_msg(&state, client_id, &msg).await;
    }
}

/// Handle client messages.
async fn handle_msg(state: &SharedState, client_id: usize, msg: &str) {
    debug!("WS({}): handle msg: {:?}", client_id, msg);

    // TODO: implement logic to handle client messages
}

/// Send message to a team.
///
/// Note: also sends to the current client as identified by `client_id`.
pub fn send_to_team(state: &SharedState, client_id: Option<usize>, team_id: u32, msg: &str) {
    debug!(
        "WS({}): sending to team {}: {}",
        client_id.unwrap_or(0),
        team_id,
        msg.chars().take(16).collect::<String>()
    );

    let clients = state.clients.clients.read().unwrap();
    let client_iter = clients.iter().filter(|c| c.team_id == team_id);
    for client in client_iter {
        // Send message, errors happen on disconnect, in which case disconnect logic will be
        // handled in other task
        let _ = client.tx.send(Message::text(msg));

        debug!(
            "WS({}): - msg queued for client {}",
            client_id.unwrap_or(0),
            client.client_id
        );
    }
}

/// Client disconnected.
async fn disconnected(state: SharedState, client_id: usize) {
    state.clients.unregister(client_id);
    info!("WS({}): disconnect", client_id);
}
