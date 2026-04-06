//! # rewind-gc
//!
//! Garbage-Free Collector for the Rewind SDK.
//!
//! Instead of freeing memory (which destroys information), this module
//! "uncomputes" intermediate steps to restore ancilla registers to their
//! initial zero state, preserving information integrity.

pub mod budget;
pub mod stack;
