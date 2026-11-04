use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

/// Opaque pagination cursor: the sort key of the last row plus its id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPayload {
    /// The sort key encoded as a string (RFC3339 timestamp, or title text).
    pub k: String,
    pub id: Uuid,
}

pub fn encode_cursor(key: &str, id: Uuid) -> String {
    let payload = CursorPayload {
        k: key.to_string(),
        id,
    };
    match serde_json::to_vec(&payload) {
        Ok(bytes) => URL_SAFE_NO_PAD.encode(bytes),
        Err(_) => String::new(),
    }
}

pub fn decode_cursor(raw: &str) -> Result<CursorPayload, AppError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(raw)
        .map_err(|_| AppError::invalid_cursor())?;
    serde_json::from_slice(&bytes).map_err(|_| AppError::invalid_cursor())
}
