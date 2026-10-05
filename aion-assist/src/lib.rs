//! Timer-based skill rotation sender.
//!
//! The rotation clock lives in [`rotation`] and can be checked without a
//! game running. On Windows, [`send`] turns the next skill into a scan-code
//! key press aimed at the focused window.

pub mod aim;
mod app;
pub mod cli;
pub mod config;
pub mod keys;
pub mod rotation;
pub mod send;
pub mod timing;

pub use app::{run, Outcome};
