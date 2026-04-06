//! # rewind-bennett
//!
//! Bennett's algorithm for the Rewind SDK.
//!
//! Transforms arbitrary irreversible computations into reversible ones
//! using the pebbling game on a computation DAG, with configurable
//! space/time trade-offs via the epsilon parameter.

pub mod executor;
pub mod graph;
pub mod pebbling;

pub use executor::{BennettExecutor, BennettPlan};
pub use graph::{ComputationGraph, StepId};
pub use pebbling::{BennettConfig, PebblingStrategy};
