//! load-rs is a performance-oriented HTTP load testing library and CLI tool.
//!
//! It provides a flexible and efficient way to execute load tests with support for
//! custom request generation, multiple input sources (files, manifests), and
//! real-time statistics reporting using HdrHistogram.

pub mod generator;
pub mod models;
pub mod runner;

pub use generator::*;
pub use models::*;
pub use runner::*;
