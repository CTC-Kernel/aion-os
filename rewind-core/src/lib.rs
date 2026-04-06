//! # rewind-core
//!
//! Core types and traits for the Rewind reversible computing SDK.
//!
//! This crate provides the foundational abstractions:
//! - `QuantumCell` — Linear type that must be consumed exactly once
//! - `ReversibleOp` — Trait guaranteeing operation reversibility
//! - `ExecutionBackend` — Hardware abstraction for execution targets
//! - `BitPlane` — SIMD-friendly bit storage (Structure of Arrays layout)

pub mod backend;
pub mod bitplane;
pub mod cell;
pub mod error;
pub mod state;
pub mod traits;

pub use cell::QuantumCell;
