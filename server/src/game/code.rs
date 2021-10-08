use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

use sha1::{Digest, Sha1};

use crate::config::Config;

/// How often to change hash.
const INTERVAL_SECS: u64 = 30;

/// Number of hashes valid before and after the current hash.
const VALID_AROUND: i64 = 2;

/// Get current outpost token.
pub fn get_outpost_token(config: &Config, id: u32) -> String {
    let hash = get_hash_at(config, id, 0);
    let token = format!("{}:{}", id, hash);
    base64::encode(token)
}

/// Validate outpost token.
pub fn validate_outpost_token(config: &Config, token: &str) -> Option<u32> {
    let token = base64::decode(token).ok()?;
    let token = str::from_utf8(&token).ok()?;

    let (id, hash) = token.split_once(":")?;
    let id: u32 = id.parse().ok()?;

    // Validate, including offset times around it
    let valid =
        (-VALID_AROUND..=VALID_AROUND).any(|offset| get_hash_at(config, id, offset) == hash);
    if valid {
        Some(id)
    } else {
        None
    }
}

/// Get timestamp, with number of offsets.
fn time(offset: i64) -> i64 {
    (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / INTERVAL_SECS) as i64
        + offset
}

/// Get the hash for an outpost at a given time.
fn get_hash_at(config: &Config, id: u32, offset: i64) -> String {
    hash(&format!(
        "{}:{}:{}",
        id,
        time(offset),
        &config.outposts.secret
    ))
}

/// Hash a message, returns a string.
fn hash(msg: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(msg.as_bytes());
    let result = hasher.finalize();
    base64::encode(&result)
}
