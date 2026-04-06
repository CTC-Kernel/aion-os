//! State types and typed indices for the reversible virtual machine.
//!
//! Uses newtype wrappers to prevent index confusion at compile time.
//!
//! # Examples
//!
//! ```
//! use rewind_core::state::{RegisterId, AncillaId, CheckpointId};
//!
//! let reg = RegisterId(0);
//! let anc = AncillaId(1);
//! let ckpt = CheckpointId(2);
//!
//! // These are different types — can't mix them up:
//! // let wrong: RegisterId = anc; // compile error!
//! ```

/// Index into the register file of the reversible VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RegisterId(pub u32);

/// Index into the ancilla bit storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AncillaId(pub u32);

/// Identifier for a saved checkpoint state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CheckpointId(pub u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_indices_are_distinct_types() {
        let r = RegisterId(0);
        let a = AncillaId(0);
        let c = CheckpointId(0);
        // Same inner value but different types
        assert_eq!(r.0, a.0);
        assert_eq!(a.0, c.0);
        // Debug works
        assert!(format!("{r:?}").contains("RegisterId"));
        assert!(format!("{a:?}").contains("AncillaId"));
        assert!(format!("{c:?}").contains("CheckpointId"));
    }

    #[test]
    fn indices_are_copy() {
        let r = RegisterId(5);
        let r2 = r; // Copy
        assert_eq!(r, r2);
    }
}
