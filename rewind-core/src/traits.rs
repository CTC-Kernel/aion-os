//! Core traits for reversible operations.
//!
//! The fundamental contract: for any `ReversibleOp` implementation,
//! `undo(execute(x)) == x` must hold for all valid inputs.
//!
//! # Examples
//!
//! ```
//! use rewind_core::traits::ReversibleOp;
//!
//! struct NotGate;
//!
//! impl ReversibleOp for NotGate {
//!     type State = u64;
//!     type Ancilla = ();
//!
//!     fn execute(&self, state: Self::State) -> (Self::State, Self::Ancilla) {
//!         (!state, ())
//!     }
//!
//!     fn undo(&self, state: Self::State, _ancilla: Self::Ancilla) -> Self::State {
//!         !state // NOT is self-inverse
//!     }
//! }
//!
//! let gate = NotGate;
//! let input = 42u64;
//! let (output, ancilla) = gate.execute(input);
//! let restored = gate.undo(output, ancilla);
//! assert_eq!(input, restored);
//! ```

/// A reversible operation that can be executed forward and undone backward.
///
/// Every implementation **must** satisfy the reversibility property:
/// `∀ state: undo(execute(state)) == state`
///
/// Use the [`assert_reversible!`] macro or [`check_reversible`] function
/// to verify this property with proptest.
pub trait ReversibleOp {
    /// The state type this operation transforms.
    type State;

    /// Ancilla data produced during forward execution, needed for undo.
    ///
    /// For self-inverse gates (e.g., NOT, XOR), this can be `()`.
    /// For gates that need to record intermediate values, this holds
    /// the bits required to reconstruct the original state.
    type Ancilla;

    /// Execute the operation forward, producing a new state and ancilla data.
    fn execute(&self, state: Self::State) -> (Self::State, Self::Ancilla);

    /// Undo the operation, restoring the original state from output + ancilla.
    fn undo(&self, state: Self::State, ancilla: Self::Ancilla) -> Self::State;
}

/// Checks that a `ReversibleOp` satisfies `undo(execute(x)) == x` for a given input.
///
/// Returns `true` if the property holds, `false` otherwise.
pub fn check_reversible<Op>(op: &Op, input: Op::State) -> bool
where
    Op: ReversibleOp,
    Op::State: Clone + PartialEq,
{
    let original = input.clone();
    let (output, ancilla) = op.execute(input);
    let restored = op.undo(output, ancilla);
    restored == original
}

/// Asserts that a `ReversibleOp` satisfies the reversibility property for a given input.
///
/// Panics with a descriptive message if `undo(execute(input)) != input`.
pub fn assert_reversible<Op>(op: &Op, input: Op::State)
where
    Op: ReversibleOp,
    Op::State: Clone + PartialEq + std::fmt::Debug,
{
    let original = input.clone();
    let (output, ancilla) = op.execute(input);
    let restored = op.undo(output, ancilla);
    assert_eq!(
        restored, original,
        "Reversibility violated: undo(execute(x)) != x"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NotGate;

    impl ReversibleOp for NotGate {
        type State = u64;
        type Ancilla = ();

        fn execute(&self, state: u64) -> (u64, ()) {
            (!state, ())
        }

        fn undo(&self, state: u64, _: ()) -> u64 {
            !state
        }
    }

    #[test]
    fn not_gate_is_reversible() {
        assert_reversible(&NotGate, 42u64);
        assert_reversible(&NotGate, 0u64);
        assert_reversible(&NotGate, u64::MAX);
    }

    #[test]
    fn check_reversible_returns_true_for_valid_gate() {
        assert!(check_reversible(&NotGate, 12345u64));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    struct NotGate;
    impl ReversibleOp for NotGate {
        type State = u64;
        type Ancilla = ();
        fn execute(&self, state: u64) -> (u64, ()) {
            (!state, ())
        }
        fn undo(&self, state: u64, _: ()) -> u64 {
            !state
        }
    }

    proptest! {
        #[test]
        fn not_gate_reversible_for_all_u64(x: u64) {
            prop_assert!(check_reversible(&NotGate, x));
        }
    }
}
