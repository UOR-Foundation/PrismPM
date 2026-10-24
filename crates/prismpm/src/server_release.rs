//! The release of a server application, verified and replayed without its source.
//!
//! A release carries the files of the build, among them the consumer's workspace
//! (`consumer/`) and the packages of the core and of the host (`server/`), and the
//! evidence of `prismpm verify`. This module checks that evidence against the
//! released files alone ([`check`]) and replays the acceptance by rebuilding the
//! released program and answering every vector with it ([`replay`]).

use crate::error::PrismError;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

/// Files by `/`-separated relative path.
pub type Files = BTreeMap<String, Vec<u8>>;

/// Check the retained verification of a released server application against the
/// released build files, with no filesystem, subprocess or network.
///
/// # Errors
/// Returns `PP6101` for any difference between the retained evidence and what the
/// model and the released files derive.
pub fn check(
    _build_manifest: &[u8],
    _build: &Files,
    _verification: &Files,
) -> Result<(), PrismError> {
    Err(PrismError::new(
        "PP9001",
        "the source-free check of a server release is not written",
    ))
}

/// Replay the acceptance of a released server application: check the release,
/// rebuild the program from the released files and answer every vector with it.
///
/// # Errors
/// Returns the refusals of [`check`], `PP4103` and `PP5007` for the workspace
/// build, and `PP6003` or `PP2021` for an acceptance the program does not meet.
pub fn replay(
    _build_manifest: &[u8],
    _build: &Files,
    _verification: &Files,
) -> Result<Value, PrismError> {
    Err(PrismError::new(
        "PP9001",
        "the replay of a server release is not written",
    ))
}

/// Replay the acceptance of the server application of a release in the local
/// OCI store of the project at `root`.
///
/// # Errors
/// Returns `PP6101` for a release that is not intact or holds no server
/// application, and the refusals of [`replay`].
pub fn replay_release(_root: &Path, _reference: &str) -> Result<Value, PrismError> {
    Err(PrismError::new(
        "PP9001",
        "the replay of a released server application is not written",
    ))
}
