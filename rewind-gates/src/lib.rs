//! # rewind-gates
//!
//! Reversible logic gates for the Rewind SDK.
//!
//! Provides Pauli-X (NOT), CNOT (Controlled-NOT), and Toffoli (CCNOT) gates
//! as the universal building blocks for reversible computation.
//!
//! ## Feature Flags
//!
//! - `simd` — Enable `std::simd` optimizations (requires nightly)
//! - `stable-simd` — Enable `pulp`-based SIMD optimizations (stable Rust)

pub mod algorithms;
pub mod circuits;
pub mod scalar;

#[cfg(feature = "simd")]
pub mod simd;

#[cfg(feature = "stable-simd")]
pub mod stable_simd;
