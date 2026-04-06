//! # rewind-core
//!
//! Core types and traits for the Rewind reversible computing SDK.
//!
//! This crate provides the foundational abstractions:
//! - `QuantumCell` — Linear type that must be consumed exactly once
//! - `ReversibleOp` — Trait guaranteeing operation reversibility
//! - `BitPlane` — SIMD-friendly bit storage (Structure of Arrays layout)
//! - `RewindError` — Typed, actionable error types

pub mod backend;
pub mod bitplane;
pub mod builder;
pub mod cell;
pub mod engine;
pub mod error;
pub mod fuzzer;
pub mod program;
pub mod runtime;
pub mod state;
pub mod traits;

pub use backend::{ExecutionBackend, RecordingBackend, SimulatedCpu};
pub use bitplane::BitPlane;
pub use builder::ProgramBuilder;
pub use cell::QuantumCell;
pub use engine::{ExecutionEngine, Op, ReversibleProgram};
pub use error::RewindError;
pub use runtime::{ReversibleRuntime, RuntimeStats};
pub use state::{AncillaId, CheckpointId, RegisterId};
pub use traits::{ReversibleOp, assert_reversible, check_reversible};
