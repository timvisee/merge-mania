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
use crate::client::{
    ClientActionBuy, ClientActionMerge, ClientActionSell, MsgRecv, MsgRecvKind, MsgSend,
    MsgSendKind,
};
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

    // Send game state to client
    send_initial(state.clone(), client_id).await;

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
        // TODO: get MsgRecv here, instead of SessionData directly
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

/// Send current state to client.
async fn send_initial(state: SharedState, client_id: usize) {
    // TODO: send initial data
    // TODO: - team info
    // TODO: - game state

    // Send item configuration
    let msg = MsgSendKind::ConfigItems(state.config.items.clone());
    send_to_client(&state, client_id, &msg.into());

    // Find client team ID
    let team_id = match state.clients.client_team_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // Get team client inventory
    let inventory = match state.game.team_client_inventory(team_id) {
        Some(inv) => inv,
        None => return,
    };

    // Send inventory state
    let msg = MsgSendKind::Inventory(inventory);
    send_to_client(&state, client_id, &msg.into());
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

        // Parse message
        let msg: MsgRecv = match serde_json::from_str(msg) {
            Ok(msg) => msg,
            Err(err) => {
                warn!(
                    "WS({}): could not parse client message: {:?}",
                    client_id, err
                );
                continue;
            }
        };

        handle_msg(&state, client_id, msg).await;
    }
}

/// Handle client messages.
async fn handle_msg(state: &SharedState, client_id: usize, msg: MsgRecv) {
    // Report error kinds
    let msg = match msg {
        MsgRecv::Ok(msg) => msg,
        MsgRecv::Err(err) => {
            warn!(
                "WS({}): received error from client, unhandled: {:?}",
                client_id, err
            );
            return;
        }
    };

    // Handle specific message
    match msg {
        MsgRecvKind::ActionMerge(action) => action_merge(state, client_id, action),
        MsgRecvKind::ActionBuy(action) => action_buy(state, client_id, action),
        MsgRecvKind::ActionSell(action) => action_sell(state, client_id, action),
        // _ => {
        //     warn!("WS({}): unhandled client message: {:?}", client_id, msg);
        // }
    }
}

fn action_merge(state: &SharedState, client_id: usize, action: ClientActionMerge) {
    // TODO: log merge

    // Find client team ID
    let team_id = match state.clients.client_team_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // TODO: ensure we can merge

    // Do merge, get inventory
    let mut inventory =
        match state
            .game
            .team_merge(team_id, &state.config, action.cell, action.other)
        {
            Some(inv) => inv,
            None => return,
        };

    // Broadcast inventory state
    let msg = MsgSendKind::Inventory(inventory);
    send_to_team(&state, Some(client_id), team_id, &msg.into());
}

fn action_buy(state: &SharedState, client_id: usize, action: ClientActionBuy) {
    // TODO: log buy

    // Find client team ID
    let team_id = match state.clients.client_team_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // TODO: ensure we can buy
    // TODO: resolve item

    // Resolve item from config
    let item = match state.config.item(&action.item) {
        Some(item) => item,
        None => return,
    };

    // Do buy, get inventory
    let mut inventory = match state
        .game
        .team_buy(team_id, &state.config, action.cell, item.clone())
    {
        Some(inv) => inv,
        None => return,
    };

    // Broadcast inventory state
    let msg = MsgSendKind::Inventory(inventory);
    send_to_team(&state, Some(client_id), team_id, &msg.into());
}

fn action_sell(state: &SharedState, client_id: usize, action: ClientActionSell) {
    // TODO: log sell

    // Find client team ID
    let team_id = match state.clients.client_team_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // Do sell, get inventory
    let mut inventory = match state.game.team_sell(team_id, &state.config, action.cell) {
        Some(inv) => inv,
        None => return,
    };

    // Broadcast inventory state
    let msg = MsgSendKind::Inventory(inventory);
    send_to_team(&state, Some(client_id), team_id, &msg.into());
}

/// Send message to client.
///
/// - returns `Ok` even if the message is never sent
pub fn send_to_client(
    state: &SharedState,
    client_id: usize,
    msg: &MsgSend,
) -> serde_json::Result<()> {
    debug!("WS({0}): send msg to client {0}", client_id);

    // Serialize
    let msg = serde_json::to_string(msg)?;

    let clients = state.clients.clients.read().unwrap();
    let client_iter = clients.iter().filter(|c| c.client_id == client_id);
    for client in client_iter {
        // Send message, errors happen on disconnect, in which case disconnect logic will be
        // handled in other task
        let _ = client.tx.send(Message::text(&msg));

        debug!("WS({0}): - msg queued for client {0}", client.client_id);
        return Ok(());
    }

    // TODO: return error, no client with this ID
    Ok(())
}

/// Send message to a team.
///
/// Notes:
/// - also sends to the current client as identified by `client_id`.
/// - returns `Ok` even if the message reaches no team.
pub fn send_to_team(
    state: &SharedState,
    client_id: Option<usize>,
    team_id: u32,
    msg: &MsgSend,
) -> serde_json::Result<()> {
    debug!(
        "WS({}): send msg to team {}",
        client_id.unwrap_or(0),
        team_id,
    );

    // Serialize
    let msg = serde_json::to_string(msg)?;

    let clients = state.clients.clients.read().unwrap();
    let client_iter = clients.iter().filter(|c| c.team_id == team_id);
    for client in client_iter {
        // Send message, errors happen on disconnect, in which case disconnect logic will be
        // handled in other task
        let _ = client.tx.send(Message::text(&msg));

        debug!(
            "WS({}): - msg queued for client {}",
            client_id.unwrap_or(0),
            client.client_id
        );
    }

    Ok(())
}

/// Client disconnected.
async fn disconnected(state: SharedState, client_id: usize) {
    info!("WS({}): disconnect", client_id);
    state.clients.unregister(client_id);
}
