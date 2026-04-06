//! Scalar (non-SIMD) implementations of reversible gates.
//!
//! These operate on [`BitPlane`] and serve as the baseline implementation,
//! reference for correctness testing, and fallback when SIMD is unavailable.
//!
//! # Gates
//!
//! - [`PauliX`] — NOT gate (self-inverse)
//! - [`Cnot`] — Controlled-NOT (XOR target with control)
//! - [`Toffoli`] — Controlled-Controlled-NOT (universal reversible gate)
//!
//! # Examples
//!
//! ```
//! use rewind_core::{BitPlane, ReversibleOp, assert_reversible};
//! use rewind_gates::scalar::PauliX;
//!
//! let gate = PauliX;
//! let input = BitPlane::from_words(vec![0xDEADBEEF]);
//! assert_reversible(&gate, input);
//! ```

use rewind_core::bitplane::BitPlane;
use rewind_core::traits::ReversibleOp;

/// Pauli-X gate (NOT): inverts all bits. Self-inverse: `X(X(x)) == x`.
pub struct PauliX;

impl ReversibleOp for PauliX {
    type State = BitPlane;
    type Ancilla = ();

    fn execute(&self, state: BitPlane) -> (BitPlane, ()) {
        (state.not(), ())
    }

    fn undo(&self, state: BitPlane, _: ()) -> BitPlane {
        state.not() // Self-inverse
    }
}

/// CNOT gate: XORs the target with the control bits.
///
/// `CNOT(control, target) = (control, target XOR control)`
///
/// The control is unchanged; the ancilla stores it for undo.
pub struct Cnot;

/// State for CNOT: a (control, target) pair of BitPlanes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CnotState {
    pub control: BitPlane,
    pub target: BitPlane,
}

impl ReversibleOp for Cnot {
    type State = CnotState;
    type Ancilla = ();

    fn execute(&self, state: CnotState) -> (CnotState, ()) {
        let new_target = state.target.xor(&state.control);
        (
            CnotState {
                control: state.control,
                target: new_target,
            },
            (),
        )
    }

    fn undo(&self, state: CnotState, _: ()) -> CnotState {
        // CNOT is self-inverse: applying XOR again restores the original target
        let original_target = state.target.xor(&state.control);
        CnotState {
            control: state.control,
            target: original_target,
        }
    }
}

/// Toffoli gate (CCNOT): the universal reversible gate.
///
/// `Toffoli(c1, c2, target) = (c1, c2, target XOR (c1 AND c2))`
///
/// Both controls are unchanged. This gate is universal: any classical
/// reversible circuit can be built from Toffoli gates alone.
pub struct Toffoli;

/// State for Toffoli: two controls and a target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToffoliState {
    pub control1: BitPlane,
    pub control2: BitPlane,
    pub target: BitPlane,
}

impl ReversibleOp for Toffoli {
    type State = ToffoliState;
    type Ancilla = ();

    fn execute(&self, state: ToffoliState) -> (ToffoliState, ()) {
        let mask = state.control1.and(&state.control2);
        let new_target = state.target.xor(&mask);
        (
            ToffoliState {
                control1: state.control1,
                control2: state.control2,
                target: new_target,
            },
            (),
        )
    }

    fn undo(&self, state: ToffoliState, _: ()) -> ToffoliState {
        // Toffoli is self-inverse: XOR with (c1 AND c2) again restores target
        let mask = state.control1.and(&state.control2);
        let original_target = state.target.xor(&mask);
        ToffoliState {
            control1: state.control1,
            control2: state.control2,
            target: original_target,
        }
    }
}

/// Fredkin gate (CSWAP): controlled swap — the other universal reversible gate.
///
/// `Fredkin(control, a, b)`: if control bit is 1, swap a and b. Otherwise no-op.
/// Implemented as: `CNOT(a,b); Toffoli(control,b,a); CNOT(a,b)`.
pub struct Fredkin;

/// State for Fredkin: one control and two targets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FredkinState {
    pub control: BitPlane,
    pub target_a: BitPlane,
    pub target_b: BitPlane,
}

impl ReversibleOp for Fredkin {
    type State = FredkinState;
    type Ancilla = ();

    fn execute(&self, state: FredkinState) -> (FredkinState, ()) {
        // CSWAP: where control=1, swap a and b bits
        // Formula: diff = a XOR b; masked = diff AND control; a ^= masked; b ^= masked
        let diff = state.target_a.xor(&state.target_b);
        let masked = diff.and(&state.control);
        let new_a = state.target_a.xor(&masked);
        let new_b = state.target_b.xor(&masked);
        (
            FredkinState {
                control: state.control,
                target_a: new_a,
                target_b: new_b,
            },
            (),
        )
    }

    fn undo(&self, state: FredkinState, _: ()) -> FredkinState {
        // Fredkin is self-inverse
        let diff = state.target_a.xor(&state.target_b);
        let masked = diff.and(&state.control);
        let orig_a = state.target_a.xor(&masked);
        let orig_b = state.target_b.xor(&masked);
        FredkinState {
            control: state.control,
            target_a: orig_a,
            target_b: orig_b,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rewind_core::traits::{assert_reversible, check_reversible};

    // --- PauliX ---

    #[test]
    fn pauli_x_inverts_bits() {
        let input = BitPlane::from_words(vec![0xFF00FF00]);
        let (output, _) = PauliX.execute(input);
        assert_eq!(output.words()[0], !0xFF00FF00u64);
    }

    #[test]
    fn pauli_x_is_reversible() {
        assert_reversible(&PauliX, BitPlane::from_words(vec![0xDEADBEEF]));
        assert_reversible(&PauliX, BitPlane::from_words(vec![0, u64::MAX]));
    }

    // --- CNOT ---

    #[test]
    fn cnot_xors_target_with_control() {
        let state = CnotState {
            control: BitPlane::from_words(vec![0xFF]),
            target: BitPlane::from_words(vec![0x0F]),
        };
        let (result, _) = Cnot.execute(state);
        assert_eq!(result.target.words()[0], 0xFF ^ 0x0F);
        assert_eq!(result.control.words()[0], 0xFF); // Control unchanged
    }

    #[test]
    fn cnot_is_reversible() {
        let state = CnotState {
            control: BitPlane::from_words(vec![0xCAFE]),
            target: BitPlane::from_words(vec![0xBEEF]),
        };
        assert_reversible(&Cnot, state);
    }

    // --- Toffoli ---

    #[test]
    fn toffoli_xors_target_with_and_of_controls() {
        let state = ToffoliState {
            control1: BitPlane::from_words(vec![0xFF]),
            control2: BitPlane::from_words(vec![0x0F]),
            target: BitPlane::from_words(vec![0x00]),
        };
        let (result, _) = Toffoli.execute(state);
        // target XOR (0xFF AND 0x0F) = 0x00 XOR 0x0F = 0x0F
        assert_eq!(result.target.words()[0], 0x0F);
        assert_eq!(result.control1.words()[0], 0xFF); // Unchanged
        assert_eq!(result.control2.words()[0], 0x0F); // Unchanged
    }

    #[test]
    fn toffoli_is_reversible() {
        let state = ToffoliState {
            control1: BitPlane::from_words(vec![0xDEAD]),
            control2: BitPlane::from_words(vec![0xBEEF]),
            target: BitPlane::from_words(vec![0xCAFE]),
        };
        assert_reversible(&Toffoli, state);
    }

    #[test]
    fn toffoli_with_zero_controls_is_identity() {
        let state = ToffoliState {
            control1: BitPlane::from_words(vec![0x00]),
            control2: BitPlane::from_words(vec![0xFF]),
            target: BitPlane::from_words(vec![0xAB]),
        };
        let (result, _) = Toffoli.execute(state);
        assert_eq!(result.target.words()[0], 0xAB); // No change (one control is 0)
    }

    #[test]
    fn all_gates_check_reversible() {
        assert!(check_reversible(&PauliX, BitPlane::from_words(vec![42])));
        assert!(check_reversible(
            &Cnot,
            CnotState {
                control: BitPlane::from_words(vec![1]),
                target: BitPlane::from_words(vec![2]),
            }
        ));
        assert!(check_reversible(
            &Toffoli,
            ToffoliState {
                control1: BitPlane::from_words(vec![1]),
                control2: BitPlane::from_words(vec![2]),
                target: BitPlane::from_words(vec![3]),
            }
        ));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use rewind_core::traits::check_reversible;

    proptest! {
        #[test]
        fn pauli_x_reversible_for_all(val: u64) {
            prop_assert!(check_reversible(&PauliX, BitPlane::from_words(vec![val])));
        }

        #[test]
        fn cnot_reversible_for_all(ctrl: u64, tgt: u64) {
            let state = CnotState {
                control: BitPlane::from_words(vec![ctrl]),
                target: BitPlane::from_words(vec![tgt]),
            };
            prop_assert!(check_reversible(&Cnot, state));
        }

        #[test]
        fn toffoli_reversible_for_all(c1: u64, c2: u64, tgt: u64) {
            let state = ToffoliState {
                control1: BitPlane::from_words(vec![c1]),
                control2: BitPlane::from_words(vec![c2]),
                target: BitPlane::from_words(vec![tgt]),
            };
            prop_assert!(check_reversible(&Toffoli, state));
        }

        #[test]
        fn toffoli_multi_word_reversible(c1: u64, c2: u64, tgt: u64, c1b: u64, c2b: u64, tgtb: u64) {
            let state = ToffoliState {
                control1: BitPlane::from_words(vec![c1, c1b]),
                control2: BitPlane::from_words(vec![c2, c2b]),
                target: BitPlane::from_words(vec![tgt, tgtb]),
            };
            prop_assert!(check_reversible(&Toffoli, state));
        }
    }
}
