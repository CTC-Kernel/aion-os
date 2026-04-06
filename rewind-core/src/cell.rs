//! QuantumCell — a linear type that must be consumed exactly once.
//!
//! Unlike standard Rust types, a `QuantumCell` cannot be duplicated (`Clone` is forbidden)
//! or silently destroyed (dropping without consumption triggers a panic).
//! This enforces the "Information is Sacred" principle at the type level.
