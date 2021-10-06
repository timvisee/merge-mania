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

    // Register client for a team
    state
        .clients
        .register(Client::new(client_id, session.team_id, tx));

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
async fn send_initial(state: SharedState, client_id: usize, session: &Session) {
    // Send session state
    let msg = MsgSendKind::Session(ClientSession::from_session(&state.config, session));
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
        MsgRecvKind::GetGame => get_game(state, client_id),
        MsgRecvKind::GetInventory => get_inventory(state, client_id),
        MsgRecvKind::GetStats => get_stats(state, client_id),
        MsgRecvKind::ActionSwap(action) => action_swap(state, client_id, action),
        MsgRecvKind::ActionMerge(action) => action_merge(state, client_id, action),
        MsgRecvKind::ActionBuy(action) => action_buy(state, client_id, action),
        MsgRecvKind::ActionSell(action) => action_sell(state, client_id, action),
        MsgRecvKind::ActionScanCode => action_scan_code(state, client_id),
    }
}

fn get_game(state: &SharedState, client_id: usize) {
    debug!("Client {} invoked get game", client_id);

    // Send item configuration
    let msg = MsgSendKind::ConfigItems(state.config.items.clone());
    send_to_client(&state, client_id, &msg.into());

    // Also send inventory state
    get_inventory(state, client_id);
}

fn get_inventory(state: &SharedState, client_id: usize) {
    debug!("Client {} invoked get inventory", client_id);

    // Find client team ID
    let team_id = match state.clients.client_team_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // Get inventory
    let mut inventory = match state.game.team_client_inventory(&state.config, team_id) {
        Some(inv) => inv,
        None => return,
    };

    // Send inventory state
    let msg = MsgSendKind::Inventory(inventory);
    send_to_client(&state, client_id, &msg.into());
}

fn get_stats(state: &SharedState, client_id: usize) {
    debug!("Client {} invoked get stats", client_id);

    // Find client team ID
    let team_id = match state.clients.client_team_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // Get stats
    let mut stats = match state.game.team_client_stats(&state.config, team_id) {
        Some(stats) => stats,
        None => return,
    };

    // Send stats
    let msg = MsgSendKind::Stats(stats);
    send_to_client(&state, client_id, &msg.into());
}

fn action_swap(state: &SharedState, client_id: usize, action: ClientActionSwap) {
    debug!("Client {} invoked swap action", client_id);

    // Find client team ID
    let team_id = match state.clients.client_team_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // Do swap, get inventory
    let mut inventory =
        match state
            .game
            .team_swap(team_id, &state.config, action.cell, action.other)
        {
            Some(inv) => inv,
            None => return,
        };

    // Send cell updates
    send_to_team_cell(&state, client_id, team_id, &inventory, action.cell);
    send_to_team_cell(&state, client_id, team_id, &inventory, action.other);
}

fn action_merge(state: &SharedState, client_id: usize, action: ClientActionMerge) {
    debug!("Client {} invoked merge action", client_id);

    // Find client team ID
    let team_id = match state.clients.client_team_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // TODO: ensure we can merge

    // Do merge, get inventory
    let (mut inventory, discovered) =
        match state
            .game
            .team_merge(team_id, &state.config, action.cell, action.other)
        {
            Some(inv) => inv,
            None => return,
        };

    // Send cell updates
    send_to_team_cell(&state, client_id, team_id, &inventory, action.cell);
    send_to_team_cell(&state, client_id, team_id, &inventory, action.other);

    // When a new item is discovered, notify the client
    if discovered {
        debug!("Team discovered new item by merging, notifying client");
        let msg = MsgSendKind::InventoryDiscovered(inventory.discovered);
        send_to_client(&state, client_id, &msg.into());
    }
}

fn action_buy(state: &SharedState, client_id: usize, action: ClientActionBuy) {
    debug!("Client {} invoked buy action", client_id);

    // Find client team ID
    let team_id = match state.clients.client_team_id(client_id) {
        Some(id) => id,
        None => return,
    };

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
    let mut changed = match state.game.team_pay(team_id, &state.config, costs) {
        Ok(changed) => changed,
        Err(_) => {
            let msg = MsgSendKind::Toast(crate::lang::INSUFFICIENT_RESOURCES_TO_BUY.into());
            send_to_client(&state, client_id, &msg.into());

            // Broadcast inventory state to reset client state
            let inventory = state.game.team_client_inventory(&state.config, team_id);
            if let Some(inventory) = inventory {
                // TODO: only broadcast changed values (money, energy)
                let msg = MsgSendKind::Inventory(inventory);
                send_to_team(&state, Some(client_id), team_id, &msg.into());
            }

            return;
        }
    };

    // Do buy, placing item in inventory, get inventory
    let (mut inventory, discovered) =
        match state
            .game
            .team_buy(team_id, &state.config, action.cell, item.clone())
        {
            Some(inv) => inv,
            None => return,
        };
    changed.insert(action.cell);

    // Send cell updates
    for cell in changed {
        send_to_team_cell(&state, client_id, team_id, &inventory, cell);
    }

    // Send team balances update
    let msg = MsgSendKind::InventoryBalances {
        money: inventory.money,
        energy: inventory.energy,
    };
    send_to_team(&state, Some(client_id), team_id, &msg.into());

    // When a new item is discovered, notify the client
    if discovered {
        debug!("Team discovered new item by buying, notifying client");
        let msg = MsgSendKind::InventoryDiscovered(inventory.discovered);
        send_to_client(&state, client_id, &msg.into());
    }
}

fn action_sell(state: &SharedState, client_id: usize, action: ClientActionSell) {
    debug!("Client {} invoked sell action", client_id);

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

    // Send cell update
    send_to_team_cell(&state, client_id, team_id, &inventory, action.cell);

    // Send team balances update
    let msg = MsgSendKind::InventoryBalances {
        money: inventory.money,
        energy: inventory.energy,
    };
    send_to_team(&state, Some(client_id), team_id, &msg.into());
}

fn action_scan_code(state: &SharedState, client_id: usize) {
    debug!("Client {} invoked scan code action", client_id);

    // Find client team ID
    let team_id = match state.clients.client_team_id(client_id) {
        Some(id) => id,
        None => return,
    };

    // Run scan code action
    let inventory = match state.game.team_scan_code(team_id, &state.config) {
        Some(inventory) => inventory,
        None => return,
    };

    // Send team balances update
    let msg = MsgSendKind::InventoryBalances {
        money: inventory.money,
        energy: inventory.energy,
    };
    send_to_team(&state, Some(client_id), team_id, &msg.into());

    // Send not yet implemented toast
    let msg = MsgSendKind::Toast(crate::lang::NO_CODE_FREE_ENERGY.into());
    send_to_client(&state, client_id, &msg.into());
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

        trace!("WS({0}): - msg queued for client {0}", client.client_id);
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
    trace!(
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

        trace!(
            "WS({}): - msg queued for client {}",
            client_id.unwrap_or(0),
            client.client_id
        );
    }

    Ok(())
}

/// Send state of single inventory cell to team.
///
/// Notes:
/// - also sends to the current client as identified by `client_id`.
/// - returns `Ok` even if the message reaches no team.
fn send_to_team_cell(
    state: &SharedState,
    client_id: usize,
    team_id: u32,
    inventory: &ClientInventory,
    cell: u8,
) {
    let msg = MsgSendKind::InventoryCell {
        index: cell,
        item: inventory.grid.items[cell as usize].clone(),
    };
    send_to_team(&state, Some(client_id), team_id, &msg.into());
}

/// Client disconnected.
async fn disconnected(state: SharedState, client_id: usize) {
    info!("WS({}): disconnect", client_id);
    state.clients.unregister(client_id);
}
