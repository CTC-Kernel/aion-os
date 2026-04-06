//! Core traits for reversible operations.
//!
//! The fundamental contract: for any `ReversibleOp` implementation,
//! `undo(execute(x)) == x` must hold for all valid inputs.
