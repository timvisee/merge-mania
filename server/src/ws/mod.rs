use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt, TryFutureExt};
use rand::Rng;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;

use crate::auth::{generate_client_id, Client, Session};
use crate::client::{
    ClientActionBuy, ClientActionMerge, ClientActionSell, ClientActionSwap, ClientInventory,
    ClientSession, MsgRecv, MsgRecvKind, MsgSend, MsgSendKind,
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

    // Register client for a user
    state
        .clients
        .register(Client::new(client_id, session.user_id, tx));

    // Send game state to client
    send_initial(state.clone(), client_id, &session).await;

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
        // TODO: get MsgRecv here, instead of SessionToken directly
        let session: crate::auth::SessionToken = match serde_json::from_str(msg) {
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
                "WS({}): auth success (user: {}, token: {}...)",
                client_id,
                session.user_id,
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
async fn send_initial(state: SharedState, client_id: usize, session: &Session) {
    // Send game state
    let msg = MsgSendKind::GameState(state.game.running());
    send_to_client(&state, client_id, &msg.into());

    // Send session state
    let session = match ClientSession::from_session(&state.config, session) {
        Some(session) => {
            let msg = MsgSendKind::Session(session);
            send_to_client(&state, client_id, &msg.into());
        }
        None => {
            error!("Failed to send session state to user.");
        }
    };
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
        MsgRecvKind::GetGame => get_game(state, client_id),
        MsgRecvKind::SetGameRunning(running) => set_game_running(state, client_id, running),
        MsgRecvKind::ResetGame => reset_game(state, client_id),
        MsgRecvKind::GetInventory => get_inventory(state, client_id),
        MsgRecvKind::GetStats => get_stats(state, client_id),
        MsgRecvKind::ActionSwap(action) => action_swap(state, client_id, action),
        MsgRecvKind::ActionMerge(action) => action_merge(state, client_id, action),
        MsgRecvKind::ActionBuy(action) => action_buy(state, client_id, action),
        MsgRecvKind::ActionSell(action) => action_sell(state, client_id, action),
        MsgRecvKind::ActionScanCode(token) => action_scan_code(state, client_id, Some(token)),
        MsgRecvKind::MockScanCode => action_scan_code(state, client_id, None),
        MsgRecvKind::GetLeaderboard => get_leaderboard(state, client_id),
        MsgRecvKind::GetOutpostToken(id) => get_outpost_token(state, client_id, id),
    }
}

fn get_game(state: &SharedState, client_id: usize) {
    debug!("Client {} invoked get game", client_id);

    // Send game state
    let msg = MsgSendKind::GameState(state.game.running());
    send_to_client(&state, client_id, &msg.into());

    // Find client user ID
    let user_id = match state.clients.client_user_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // Check if user has user role
    let role_game = state
        .config
        .user(user_id)
        .map(|u| u.role_game)
        .unwrap_or(false);

    if role_game {
        // Send item configuration
        let msg = MsgSendKind::ConfigItems(state.config.items.clone());
        send_to_client(&state, client_id, &msg.into());

        // Also send inventory state
        get_inventory(state, client_id);
    }
}

fn set_game_running(state: &SharedState, client_id: usize, running: bool) {
    debug!("Client {} invoked set game running: {}", client_id, running);

    // Find client user ID
    let user_id = match state.clients.client_user_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // User must be admin
    let role_admin = state
        .config
        .user(user_id)
        .map(|u| u.role_admin)
        .unwrap_or(false);
    if !role_admin {
        warn!("Non-admin user tried to change game state");
        return;
    }

    // Set running state
    state.game.set_running(running);

    // Send game state to all clients
    let msg = MsgSendKind::GameState(running);
    send_to_all(&state, Some(client_id), &msg.into());
}

fn reset_game(state: &SharedState, client_id: usize) {
    debug!("Client {} invoked game reset", client_id);

    // Find client user ID
    let user_id = match state.clients.client_user_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // User must be admin
    let role_admin = state
        .config
        .user(user_id)
        .map(|u| u.role_admin)
        .unwrap_or(false);
    if !role_admin {
        warn!("Non-admin user tried to reset game");
        return;
    }

    // Reset game
    state.game.reset();

    info!("Game is reset by admin");

    // Update each client
    for client_id in state.clients.client_ids() {
        // Get user ID
        let user_id = match state.clients.client_user_id(client_id) {
            Some(id) => id,
            None => continue,
        };

        // Get and broadcast inventory to client
        let inventory = state.game.user_client_inventory(&state.config, user_id);
        if let Some(inventory) = inventory {
            let msg = MsgSendKind::Inventory(inventory);
            send_to_client(&state, client_id, &msg.into());
        }
    }
}

fn get_inventory(state: &SharedState, client_id: usize) {
    debug!("Client {} invoked get inventory", client_id);

    // Find client user ID
    let user_id = match state.clients.client_user_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // User must have game role
    let role_game = state
        .config
        .user(user_id)
        .map(|u| u.role_game)
        .unwrap_or(false);
    if !role_game {
        warn!("Non-game user tried to get inventory");
        return;
    }

    // Get inventory
    let mut inventory = match state.game.user_client_inventory(&state.config, user_id) {
        Some(inv) => inv,
        None => return,
    };

    // Send inventory state
    let msg = MsgSendKind::Inventory(inventory);
    send_to_client(&state, client_id, &msg.into());
}

fn get_stats(state: &SharedState, client_id: usize) {
    debug!("Client {} invoked get stats", client_id);

    // Find client user ID
    let user_id = match state.clients.client_user_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // User must have game role
    let role_game = state
        .config
        .user(user_id)
        .map(|u| u.role_game)
        .unwrap_or(false);
    if !role_game {
        warn!("Non-game user tried to get stats");
        return;
    }

    // Get stats
    let mut stats = match state.game.user_client_stats(&state.config, user_id) {
        Some(stats) => stats,
        None => return,
    };

    // Send stats
    let msg = MsgSendKind::Stats(stats);
    send_to_client(&state, client_id, &msg.into());
}

fn action_swap(state: &SharedState, client_id: usize, action: ClientActionSwap) {
    debug!("Client {} invoked swap action", client_id);

    // Find client user ID
    let user_id = match state.clients.client_user_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // User must have game role
    let role_game = state
        .config
        .user(user_id)
        .map(|u| u.role_game)
        .unwrap_or(false);
    if !role_game {
        warn!("Non-game user tried to swap items");
        return;
    }

    // Do swap, get inventory
    let mut inventory =
        match state
            .game
            .user_swap(user_id, &state.config, action.cell, action.other)
        {
            Some(inv) => inv,
            None => return,
        };

    // Send cell updates
    send_to_user_cell(&state, client_id, user_id, &inventory, action.cell);
    send_to_user_cell(&state, client_id, user_id, &inventory, action.other);
}

fn action_merge(state: &SharedState, client_id: usize, action: ClientActionMerge) {
    debug!("Client {} invoked merge action", client_id);

    // Find client user ID
    let user_id = match state.clients.client_user_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // User must have game role
    let role_game = state
        .config
        .user(user_id)
        .map(|u| u.role_game)
        .unwrap_or(false);
    if !role_game {
        warn!("Non-game user tried to merge items");
        return;
    }

    // Do merge, get inventory
    let (mut inventory, discovered) =
        match state
            .game
            .user_merge(user_id, &state.config, action.cell, action.other)
        {
            Some(inv) => inv,
            None => return,
        };

    // Send cell updates
    send_to_user_cell(&state, client_id, user_id, &inventory, action.cell);
    send_to_user_cell(&state, client_id, user_id, &inventory, action.other);

    // When a new item is discovered, notify the client
    if discovered {
        debug!("User discovered new item by merging, notifying client");
        let msg = MsgSendKind::InventoryDiscovered(inventory.discovered);
        send_to_client(&state, client_id, &msg.into());
    }
}

fn action_buy(state: &SharedState, client_id: usize, action: ClientActionBuy) {
    debug!("Client {} invoked buy action", client_id);

    // Find client user ID
    let user_id = match state.clients.client_user_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // User must have game role
    let role_game = state
        .config
        .user(user_id)
        .map(|u| u.role_game)
        .unwrap_or(false);
    if !role_game {
        warn!("Non-game user tried to buy item");
        return;
    }

    // Resolve item from config
    let item = match state.config.item(&action.item) {
        Some(item) => item,
        None => return,
    };

    // Get buy costs, cannot buy if no costs defined
    let costs = match &item.buy {
        Some(costs) => costs,
        None => return,
    };

    // Pay amounts, send notification if not enough resources
    let mut changed = match state.game.user_pay(user_id, &state.config, costs) {
        Ok(changed) => changed,
        Err(_) => {
            let msg = MsgSendKind::Toast(crate::lang::INSUFFICIENT_RESOURCES_TO_BUY.into());
            send_to_client(&state, client_id, &msg.into());

            // Broadcast inventory state to reset client state
            let inventory = state.game.user_client_inventory(&state.config, user_id);
            if let Some(inventory) = inventory {
                let msg = MsgSendKind::Inventory(inventory);
                send_to_user(&state, Some(client_id), user_id, &msg.into());
            }

            return;
        }
    };

    // Do buy, placing item in inventory, get inventory
    let (mut inventory, discovered) =
        match state
            .game
            .user_buy(user_id, &state.config, action.cell, item.clone())
        {
            Some(inv) => inv,
            None => return,
        };
    changed.insert(action.cell);

    // Send cell updates
    for cell in changed {
        send_to_user_cell(&state, client_id, user_id, &inventory, cell);
    }

    // Send user balances update
    let msg = MsgSendKind::InventoryBalances {
        money: inventory.money,
        energy: inventory.energy,
    };
    send_to_user(&state, Some(client_id), user_id, &msg.into());

    // When a new item is discovered, notify the client
    if discovered {
        debug!("User discovered new item by buying, notifying client");
        let msg = MsgSendKind::InventoryDiscovered(inventory.discovered);
        send_to_client(&state, client_id, &msg.into());
    }
}

fn action_sell(state: &SharedState, client_id: usize, action: ClientActionSell) {
    debug!("Client {} invoked sell action", client_id);

    // Find client user ID
    let user_id = match state.clients.client_user_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // User must have game role
    let role_game = state
        .config
        .user(user_id)
        .map(|u| u.role_game)
        .unwrap_or(false);
    if !role_game {
        warn!("Non-game user tried to sell item");
        return;
    }

    // Do sell, get inventory
    let mut inventory = match state.game.user_sell(user_id, &state.config, action.cell) {
        Some(inv) => inv,
        None => return,
    };

    // Send cell update
    send_to_user_cell(&state, client_id, user_id, &inventory, action.cell);

    // Send user balances update
    let msg = MsgSendKind::InventoryBalances {
        money: inventory.money,
        energy: inventory.energy,
    };
    send_to_user(&state, Some(client_id), user_id, &msg.into());
}

/// Invoke action to scan a QR code.
///
/// When the token is `None` it is always accepted if the user is admin.
fn action_scan_code(state: &SharedState, client_id: usize, token: Option<String>) {
    debug!("Client {} invoked scan code action", client_id);

    // Find client user ID
    let user_id = match state.clients.client_user_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // User must have correct roles
    let role_game = state
        .config
        .user(user_id)
        .map(|u| u.role_game)
        .unwrap_or(false);
    let role_admin = state
        .config
        .user(user_id)
        .map(|u| u.role_game)
        .unwrap_or(false);
    if !role_game {
        warn!("Non-game user tried to scan code");
        return;
    }
    if token.is_none() && !role_admin {
        warn!("Non-admin user tried to mock a code scan");
        return;
    }

    // Game must be running
    if !state.game.running() {
        warn!("User scanned code while game isn't running");
        let msg = MsgSendKind::CodeResult(false);
        send_to_client(&state, client_id, &msg.into());
        return;
    }

    // Validate token and get outpost ID
    let outpost_id = if let Some(token) = token {
        crate::game::code::validate_outpost_token(&state.config, &token)
    } else {
        Some(rand::thread_rng().gen_range(1..=10))
    };
    if outpost_id.is_none() {
        warn!("User scanned invalid code");
        let msg = MsgSendKind::CodeResult(false);
        send_to_client(&state, client_id, &msg.into());
        return;
    }

    // Run scan code action
    let inventory = match state
        .game
        .user_scan_code(user_id, &state.config, outpost_id.unwrap())
    {
        Some(inventory) => inventory,
        None => {
            warn!("User scanned same post as last time");
            let msg = MsgSendKind::CodeResult(false);
            send_to_client(&state, client_id, &msg.into());
            return;
        }
    };

    let msg = MsgSendKind::CodeResult(true);
    send_to_client(&state, client_id, &msg.into());

    // Send user balances update
    let msg = MsgSendKind::InventoryBalances {
        money: inventory.money,
        energy: inventory.energy,
    };
    send_to_user(&state, Some(client_id), user_id, &msg.into());
}

fn get_leaderboard(state: &SharedState, client_id: usize) {
    debug!("Client {} invoked get leaderboard", client_id);

    // Send game state
    let msg = MsgSendKind::GameState(state.game.running());
    send_to_client(&state, client_id, &msg.into());

    // Find client user ID
    let user_id = match state.clients.client_user_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // User must have admin role
    let role_admin = state
        .config
        .user(user_id)
        .map(|u| u.role_admin)
        .unwrap_or(false);
    if !role_admin {
        warn!("Non-game user tried to get leaderboard");
        return;
    }

    // Get leaderboard, send to client
    let msg = MsgSendKind::Leaderboard(state.game.leaderboard());
    send_to_client(&state, client_id, &msg.into());
}

fn get_outpost_token(state: &SharedState, client_id: usize, outpost_id: u32) {
    debug!(
        "Client {} invoked get outpost token for outpost {}",
        client_id, outpost_id
    );

    // Find client user ID
    let user_id = match state.clients.client_user_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // User must have admin role
    let role_admin = state
        .config
        .user(user_id)
        .map(|u| u.role_admin)
        .unwrap_or(false);
    if !role_admin {
        warn!("Non-game user tried to get outpost token");
        return;
    }

    // Generate outpost token and send it back
    let msg = MsgSendKind::OutpostToken(crate::game::code::get_outpost_token(
        &state.config,
        outpost_id,
    ));
    send_to_client(&state, client_id, &msg.into());
}

/// Send message to all clients.
///
/// Notes:
/// - also sends to the current client as identified by `client_id`.
/// - returns `Ok` even if the message reaches no client.
pub fn send_to_all(
    state: &SharedState,
    client_id: Option<usize>,
    msg: &MsgSend,
) -> serde_json::Result<()> {
    trace!("WS({}): send msg to all clients", client_id.unwrap_or(0),);

    // Serialize
    let msg = serde_json::to_string(msg)?;

    for client in state.clients.clients.read().unwrap().iter() {
        // Send message, errors happen on disconnect, in which case disconnect logic will be
        // handled in other task
        let _ = client.tx.send(Message::text(&msg));

        trace!(
            "WS({}): - msg queued for client {}",
            client_id.unwrap_or(0),
            client.client_id,
        );
    }

    Ok(())
}

/// Send message to client.
///
/// - returns `Ok` even if the message is never sent
pub fn send_to_client(
    state: &SharedState,
    client_id: usize,
    msg: &MsgSend,
) -> serde_json::Result<()> {
    trace!("WS({0}): send msg to client {0}", client_id);

    // Serialize
    let msg = serde_json::to_string(msg)?;

    let clients = state.clients.clients.read().unwrap();
    let client_iter = clients.iter().filter(|c| c.client_id == client_id);
    for client in client_iter {
        // Send message, errors happen on disconnect, in which case disconnect logic will be
        // handled in other task
        let _ = client.tx.send(Message::text(&msg));

        trace!("WS({0}): - msg queued for client {0}", client.client_id);
        return Ok(());
    }

    // TODO: return error, no client with this ID
    Ok(())
}

/// Send message to a user.
///
/// Notes:
/// - also sends to the current client as identified by `client_id`.
/// - returns `Ok` even if the message reaches no user.
pub fn send_to_user(
    state: &SharedState,
    client_id: Option<usize>,
    user_id: u32,
    msg: &MsgSend,
) -> serde_json::Result<()> {
    trace!(
        "WS({}): send msg to user {}",
        client_id.unwrap_or(0),
        user_id,
    );

    // Serialize
    let msg = serde_json::to_string(msg)?;

    let clients = state.clients.clients.read().unwrap();
    let client_iter = clients.iter().filter(|c| c.user_id == user_id);
    for client in client_iter {
        // Send message, errors happen on disconnect, in which case disconnect logic will be
        // handled in other task
        let _ = client.tx.send(Message::text(&msg));

        trace!(
            "WS({}): - msg queued for client {}",
            client_id.unwrap_or(0),
            client.client_id
        );
    }

    Ok(())
}

/// Send state of single inventory cell to user.
///
/// Notes:
/// - also sends to the current client as identified by `client_id`.
/// - returns `Ok` even if the message reaches no user.
fn send_to_user_cell(
    state: &SharedState,
    client_id: usize,
    user_id: u32,
    inventory: &ClientInventory,
    cell: u8,
) {
    let msg = MsgSendKind::InventoryCell {
        index: cell,
        item: inventory.grid.items[cell as usize].clone(),
    };
    send_to_user(&state, Some(client_id), user_id, &msg.into());
}

/// Client disconnected.
async fn disconnected(state: SharedState, client_id: usize) {
    info!("WS({}): disconnect", client_id);
    state.clients.unregister(client_id);
}
