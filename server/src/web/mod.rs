//! Webserver module.

use warp::http::StatusCode;
use warp::Reply;

use crate::routes::ErrorMessage;
use crate::state::SharedState;

/// HTTP server.
pub async fn server(state: SharedState) {
    println!("Initialzing server...");
    let routes = crate::routes::routes(state);
    warp::serve(routes).run(crate::HOST).await;
}

/// A custom and easily returnable API error.
#[derive(Debug)]
pub struct ApiError {
    code: StatusCode,
    message: String,
}

impl ApiError {
    /// Construct API error from given message.
    pub fn from<S: AsRef<str>>(message: S) -> Self {
        Self {
            code: StatusCode::BAD_REQUEST,
            message: message.as_ref().into(),
        }
    }

    /// Transform into warp reply.
    pub fn to_reply(self) -> impl Reply {
        // Transform into JSONable error
        let error = ErrorMessage {
            code: self.code.as_u16(),
            message: self.message,
        };

        warp::reply::with_status(warp::reply::json(&error), self.code)
    }
}
