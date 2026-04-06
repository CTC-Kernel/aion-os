//! # Rewind
//!
//! **The first natively reversible computing SDK for Rust.**
//!
//! *Information is Sacred — it must never be destroyed.*
//!
//! Rewind guarantees that every operation is structurally invertible,
//! verified at compile-time, enabling bidirectional execution (forward/backward),
//! zero-cost rollbacks, and native time-travel debugging.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use rewind::prelude::*;
//!
//! #[reversible]
//! fn example(x: &mut u64, y: &mut u64) {
//!     *x += 42;
//!     *y ^= *x;
//! }
//! ```
//!
//! ## Crate Features
//!
//! - `simd` — Enable `std::simd` gate optimizations (nightly)
//! - `stable-simd` — Enable `pulp`-based SIMD (stable Rust)
//! - `bennett` — Enable Bennett's automatic reversible compilation

pub use rewind_core::*;
pub use rewind_dsl::*;
pub use rewind_gates::*;
