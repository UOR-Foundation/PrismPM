//! Prism Platform Model (PrismPM) Core Library.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub mod cli;
pub mod controller;
pub mod error;
pub mod holo;

pub use controller::Controller;
pub use error::PrismError;
