//! Error types for the Rewind SDK.
//!
//! All errors are explicit, typed, and actionable — the developer always knows
//! exactly what went wrong and how to fix it.
//!
//! # Examples
//!
//! ```
//! use rewind_core::error::RewindError;
//! use rewind_core::state::CheckpointId;
//!
//! let err = RewindError::CheckpointNotFound(CheckpointId(42));
//! assert!(format!("{err}").contains("42"));
//! ```

use crate::state::CheckpointId;
use std::fmt;

/// Errors that can occur during reversible computation.
#[derive(Debug)]
pub enum RewindError {
    /// A `QuantumCell` was dropped without being consumed.
    InformationLost,

    /// Attempted to restore a checkpoint that doesn't exist.
    CheckpointNotFound(CheckpointId),

    /// Ancilla stack is not empty after uncomputation.
    GarbageRemaining(usize),

    /// The ancilla memory budget has been exceeded.
    MemoryBudgetExceeded {
        /// Bytes currently used.
        used: usize,
        /// Configured maximum bytes.
        limit: usize,
    },
}

impl fmt::Display for RewindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InformationLost => {
                write!(f, "QuantumCell dropped without being consumed — information lost")
            }
            Self::CheckpointNotFound(id) => {
                write!(f, "Checkpoint {} not found", id.0)
            }
            Self::GarbageRemaining(bits) => {
                write!(
                    f,
                    "Ancilla stack not empty after uncomputation — {bits} bits remain"
                )
            }
            Self::MemoryBudgetExceeded { used, limit } => {
                write!(
                    f,
                    "Memory budget exceeded: {used} > {limit} bytes"
                )
            }
        }
    }
}

impl std::error::Error for RewindError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_are_descriptive() {
        let e1 = RewindError::InformationLost;
        assert!(e1.to_string().contains("information lost"));

        let e2 = RewindError::CheckpointNotFound(CheckpointId(42));
        assert!(e2.to_string().contains("42"));

        let e3 = RewindError::GarbageRemaining(5);
        assert!(e3.to_string().contains("5 bits remain"));

        let e4 = RewindError::MemoryBudgetExceeded { used: 1024, limit: 512 };
        assert!(e4.to_string().contains("1024 > 512"));
    }

    #[test]
    fn error_is_debug_printable() {
        let e = RewindError::InformationLost;
        let debug = format!("{e:?}");
        assert!(debug.contains("InformationLost"));
    }
}
