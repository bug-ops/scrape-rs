//! Parallel processing utilities.
//!
//! This module provides parallel processing capabilities for batch operations,
//! leveraging Rayon's work-stealing scheduler for efficient multi-threaded execution.
//!
//! # Feature Flag
//!
//! This module is only available when the `parallel` feature is enabled:
//!
//! ```toml
//! [dependencies]
//! scrape-core = { version = "0.1", features = ["parallel"] }
//! ```
//!
//! # Platform Support
//!
//! Parallel features are primarily designed for native platforms (Linux, macOS, Windows).
//! WASM builds should not enable the `parallel` feature as threading is not available
//! in the browser environment.

#[cfg(feature = "parallel")]
mod batch;

#[cfg(feature = "parallel")]
pub use batch::{parse_batch, parse_batch_owned, parse_batch_with_config};
