//! # labwc-core
//!
//! Core types, traits, and error types for labwc-rs.
//! This crate has zero dependencies on external crates beyond std,
//! bitflags, and thiserror. It defines the foundational abstractions
//! that all other crates build upon.
//!
//! ## Design
//!
//! - All geometry types are `Copy + Clone` plain-old-data structs.
//! - Enums replace C integer constants for type safety.
//! - Error types use `thiserror` for ergonomic `?` propagation.
//! - Bitflag types use the `bitflags` crate for composable options.

#![deny(unsafe_code)]

pub mod cursor;
pub mod edge;
pub mod error;
pub mod geometry;
pub mod state;
pub mod types;
pub mod view_state;

pub use cursor::*;
pub use edge::*;
pub use error::*;
pub use geometry::*;
pub use state::*;
pub use types::*;
pub use view_state::*;
