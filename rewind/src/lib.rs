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
//! ```
//! use rewind::prelude::*;
//!
//! // Build a reversible program
//! let program = ReversibleProgram::new(vec![
//!     Op::Not(0),
//!     Op::Cnot { control: 0, target: 1 },
//!     Op::Toffoli { c1: 0, c2: 1, target: 2 },
//! ]);
//!
//! // Create registers and run
//! let regs = vec![
//!     BitPlane::from_words(vec![0xFF]),
//!     BitPlane::from_words(vec![0x0F]),
//!     BitPlane::from_words(vec![0x00]),
//! ];
//! let mut engine = ExecutionEngine::new(regs);
//! let original = engine.registers().to_vec();
//!
//! program.forward(&mut engine);   // Execute
//! program.backward(&mut engine);  // Rewind
//! assert_eq!(engine.registers().to_vec(), original); // Restored!
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

/// Convenient imports for common usage.
///
/// ```
/// use rewind::prelude::*;
/// ```
pub mod prelude {
    pub use rewind_core::bitplane::BitPlane;
    pub use rewind_core::cell::QuantumCell;
    pub use rewind_core::engine::{ExecutionEngine, Op, ReversibleProgram};
    pub use rewind_core::error::RewindError;
    pub use rewind_core::runtime::ReversibleRuntime;
    pub use rewind_core::state::{AncillaId, CheckpointId, RegisterId};
    pub use rewind_core::traits::{ReversibleOp, assert_reversible, check_reversible};
    pub use rewind_dsl::reversible;
    pub use rewind_gates::scalar::{Cnot, CnotState, PauliX, Toffoli, ToffoliState};
}
