//! Canonical JSON encoding and hashing for Holo format.

use sha2::{Digest, Sha256};
use crate::error::PrismError;
use super::dto::HoloDocument;

/// Encode HoloDocument into canonical JSON bytes.
pub fn encode_canonical(doc: &HoloDocument) -> Result<Vec<u8>, PrismError> {
    let mut bytes = serde_json::to_vec(doc).map_err(|e| PrismError::new("PP3002", e.to_string()))?;
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    Ok(bytes)
}

/// Compute content ID SHA-256 hash.
pub fn content_id(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

