pub mod api;
pub mod magic;

use std::convert::Infallible;
use std::error::Error;

use serde::Serialize;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use crate::state::SharedState;

/// Build routes.
pub fn routes(
    state: SharedState,
) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone + Send + Sync + 'static {
    // Helper to transform state in shareable warp filter.
    let s = |s: SharedState| warp::any().map(move || s.clone());

    let heartbeat = warp::path("__heartbeat__").map(magic::heartbeat);

    let api_auth_users = warp::path("users")
        .and(s(state.clone()))
        .map(|state: SharedState| api::auth::users(state));

    let api_auth_login = warp::path("login").and(
        warp::post()
            .and(warp::body::json())
            .and(s(state.clone()))
            .map(api::auth::login),
    );

    let api_auth_logout = warp::path("logout").and(
        warp::post()
            .and(warp::body::json())
            .and(s(state.clone()))
            .map(api::auth::logout),
    );

    let api_auth_validate = warp::path("validate").and(
        warp::post()
            .and(warp::body::json())
            .and(s(state.clone()))
            .map(api::auth::validate),
    );

    let api_auth = warp::path("auth").and(
        api_auth_users
            .or(api_auth_login)
            .or(api_auth_logout)
            .or(api_auth_validate),
    );

    let api = warp::path("api").and(api_auth.recover(handle_api_rejection));

    let ws =
        warp::path("ws")
            .and(warp::ws())
            .and(s(state.clone()))
            .map(|ws: warp::ws::Ws, state| {
                // Start handling socket when websocket handshake succeeds
                ws.on_upgrade(move |socket| crate::ws::connected(state, socket))
            });

    let static_sprites = warp::path("sprites").and(warp::fs::dir("../sprites"));

    let static_client = warp::fs::dir("../client/dist");

    let static_server = warp::fs::dir("./public/");

    heartbeat
        .or(api)
        .or(ws)
        .or(static_sprites)
        .or(static_client)
        .or(static_server)
        .recover(handle_rejection)
}

/// Handle API route rejections.
///
/// Tries to nicely format and report errors, passes rejection along otherwise.
// TODO: clean this up
async fn handle_api_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    // } else if let Some(DivideByZero) = err.find() {
    //     code = StatusCode::BAD_REQUEST;
    //     message = "DIVIDE_BY_ZERO";
    } else if let Some(e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        // This error happens if the body could not be deserialized correctly
        // We can use the cause to analyze the error and customize the error message
        message = match e.source() {
            Some(cause) => {
                if cause.to_string().contains("denom") {
                    "FIELD_ERROR: denom"
                } else {
                    "BAD_REQUEST"
                }
            }
            None => "BAD_REQUEST",
        };
        code = StatusCode::BAD_REQUEST;
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        // We should have expected this... Just log and say its a 500
        error!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

/// Handle HTTP route rejections.
///
/// Tries to nicely format and report errors, passes rejection along otherwise.
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "HTTP 404 - Not found";
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "HTTP 405 - Method not allowed";
    } else {
        error!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "HTTP 500 - Internal server error (unhandled rejection)";
    }

    Ok(warp::reply::with_status(message, code))
}

/// An API error serializable to JSON.
#[derive(Serialize, Debug)]
pub(crate) struct ErrorMessage {
    pub code: u16,
    pub message: String,
}
