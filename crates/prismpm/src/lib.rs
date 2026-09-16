//! Prism Platform Model (PrismPM) Core Library.

#![deny(missing_docs)]
#![forbid(unsafe_code)]
// Public failures deliberately carry the complete registered span, labels,
// notes, help, and cause chain by value. Boxing the error would weaken the
// stable Controller API solely to satisfy a size heuristic.
#![allow(clippy::result_large_err)]

mod acceptance;
mod application_build;
pub mod authority;
mod browser_oracle;
pub mod cli;
pub mod config;
pub mod contracts;
pub mod controller;
pub mod deployment;
pub mod diagnostics;
pub mod error;
pub mod holo;
pub mod lifecycle;
pub mod oci;
pub mod operations;
pub mod sdk;
pub mod supply_chain;
pub mod system;
pub mod template;
pub mod upstream_conformance;
mod verification;

pub use controller::Controller;
pub use error::{DiagnosticCode, PrismError};
