//! Pre-built reversible circuits — common algorithms composed from gates.
//!
//! These circuits demonstrate that Rewind can perform real computation,
//! not just individual gate operations. Each circuit is fully reversible.
//!
//! # Examples
//!
//! ```
//! use rewind_core::engine::Op;
//! use rewind_gates::circuits;
//!
//! // Half-adder: computes sum and carry from two input bits
//! let ops = circuits::half_adder(0, 1, 2);
//! assert_eq!(ops.len(), 2); // CNOT + Toffoli
//! ```

use rewind_core::engine::Op;

/// Reversible half-adder circuit.
///
/// Computes:
/// - `sum_reg = input_a XOR input_b` (via CNOT)
/// - `carry_reg ^= input_a AND input_b` (via Toffoli)
///
/// Both input registers are preserved. The sum and carry registers
/// must be initialized to 0 for correct results.
///
/// Requires 3 registers: `input_a`, `input_b`, `carry_reg`.
/// The sum is stored in `input_b` (XORed with `input_a`).
pub fn half_adder(input_a: usize, input_b: usize, carry: usize) -> Vec<Op> {
    vec![
        Op::Cnot {
            control: input_a,
            target: input_b,
        }, // input_b becomes sum (a XOR b)
        Op::Toffoli {
            c1: input_a,
            c2: input_b,
            target: carry,
        }, // carry = a AND (a XOR b) — note: this is after CNOT
    ]
}

/// Reversible full-adder circuit.
///
/// Computes sum and carry-out from two inputs and a carry-in.
/// Requires 4 registers: `a`, `b`, `carry_in`, `carry_out`.
///
/// After execution:
/// - `b` contains the sum bit (a XOR b XOR carry_in)
/// - `carry_out` contains the carry (majority function)
pub fn full_adder(a: usize, b: usize, carry_in: usize, carry_out: usize) -> Vec<Op> {
    vec![
        // Step 1: b ^= a (partial sum)
        Op::Cnot {
            control: a,
            target: b,
        },
        // Step 2: carry_out ^= a AND b (partial carry from a,b)
        Op::Toffoli {
            c1: a,
            c2: b,
            target: carry_out,
        },
        // Step 3: b ^= carry_in (full sum)
        Op::Cnot {
            control: carry_in,
            target: b,
        },
        // Step 4: carry_out ^= carry_in AND b (carry from carry_in)
        Op::Toffoli {
            c1: carry_in,
            c2: b,
            target: carry_out,
        },
        // Step 5: undo step 1 partial on carry path
        Op::Cnot {
            control: a,
            target: b,
        },
    ]
}

/// Reversible SWAP circuit using 3 CNOTs.
///
/// Swaps the contents of two registers using the identity:
/// `CNOT(a,b); CNOT(b,a); CNOT(a,b)` = SWAP(a,b)
pub fn swap(a: usize, b: usize) -> Vec<Op> {
    vec![
        Op::Cnot {
            control: a,
            target: b,
        },
        Op::Cnot {
            control: b,
            target: a,
        },
        Op::Cnot {
            control: a,
            target: b,
        },
    ]
}

/// Reversible NOT on a target register, controlled by a single control.
///
/// This is just a CNOT — included for readability in circuit composition.
pub fn controlled_not(control: usize, target: usize) -> Vec<Op> {
    vec![Op::Cnot { control, target }]
}

/// Identity circuit — does nothing (useful as a placeholder or for testing).
pub fn identity() -> Vec<Op> {
    vec![]
}

/// Compose multiple circuits into a single operation sequence.
pub fn compose(circuits: Vec<Vec<Op>>) -> Vec<Op> {
    circuits.into_iter().flatten().collect()
}

/// Reverse a circuit — produces the inverse operation sequence.
///
/// Since all gates in Rewind are self-inverse, reversing a circuit
/// simply reverses the order of operations.
pub fn reverse(circuit: &[Op]) -> Vec<Op> {
    circuit.iter().rev().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rewind_core::bitplane::BitPlane;
    use rewind_core::runtime::ReversibleRuntime;

    fn run_circuit(num_regs: usize, init: &[u64], ops: &[Op]) -> Vec<u64> {
        let regs = init
            .iter()
            .map(|v| BitPlane::from_words(vec![*v]))
            .collect();
        let mut rt = ReversibleRuntime::new(regs);
        rt.execute_all_tracked(ops);
        (0..num_regs).map(|i| rt.register(i).words()[0]).collect()
    }

    fn verify_reversible(num_regs: usize, init: &[u64], ops: &[Op]) {
        let regs: Vec<_> = init
            .iter()
            .map(|v| BitPlane::from_words(vec![*v]))
            .collect();
        let mut rt = ReversibleRuntime::new(regs);
        let original: Vec<_> = (0..num_regs).map(|i| rt.register(i).words()[0]).collect();
        rt.execute_all_tracked(ops);
        rt.rewind_all().unwrap();
        let restored: Vec<_> = (0..num_regs).map(|i| rt.register(i).words()[0]).collect();
        assert_eq!(original, restored, "Circuit is not reversible!");
    }

    #[test]
    fn swap_circuit_swaps_values() {
        let ops = swap(0, 1);
        let result = run_circuit(2, &[0xAA, 0xBB], &ops);
        assert_eq!(result, vec![0xBB, 0xAA]);
    }

    #[test]
    fn swap_is_reversible() {
        verify_reversible(2, &[0xAA, 0xBB], &swap(0, 1));
    }

    #[test]
    fn double_swap_is_identity() {
        let ops = compose(vec![swap(0, 1), swap(0, 1)]);
        let result = run_circuit(2, &[0xAA, 0xBB], &ops);
        assert_eq!(result, vec![0xAA, 0xBB]);
    }

    #[test]
    fn half_adder_computes_correctly() {
        // 1 + 1 = sum=0, carry=1 (in single-bit terms)
        // With u64: all bits set in both inputs
        let ops = half_adder(0, 1, 2);
        verify_reversible(3, &[0xFF, 0xFF, 0x00], &ops);
    }

    #[test]
    fn half_adder_is_reversible() {
        verify_reversible(3, &[0xAA, 0x55, 0x00], &half_adder(0, 1, 2));
    }

    #[test]
    fn full_adder_is_reversible() {
        verify_reversible(4, &[0xFF, 0xAA, 0x55, 0x00], &full_adder(0, 1, 2, 3));
    }

    #[test]
    fn reverse_circuit_is_inverse() {
        let circuit = vec![
            Op::Not(0),
            Op::Cnot {
                control: 0,
                target: 1,
            },
            Op::Toffoli {
                c1: 0,
                c2: 1,
                target: 2,
            },
        ];
        let reversed = reverse(&circuit);
        let combined = compose(vec![circuit, reversed]);

        // Forward + reverse = identity
        let result = run_circuit(3, &[0xAA, 0xBB, 0xCC], &combined);
        assert_eq!(result, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn compose_flattens_correctly() {
        let c = compose(vec![swap(0, 1), vec![Op::Not(0)], swap(0, 1)]);
        assert_eq!(c.len(), 7); // 3 + 1 + 3
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use rewind_core::bitplane::BitPlane;
    use rewind_core::runtime::ReversibleRuntime;

    proptest! {
        #[test]
        fn swap_is_always_reversible(a: u64, b: u64) {
            let regs = vec![
                BitPlane::from_words(vec![a]),
                BitPlane::from_words(vec![b]),
            ];
            let mut rt = ReversibleRuntime::new(regs);
            let original = rt.registers().to_vec();
            rt.execute_all_tracked(&swap(0, 1));
            // Verify swap happened
            prop_assert_eq!(rt.register(0).words()[0], b);
            prop_assert_eq!(rt.register(1).words()[0], a);
            // Verify rewind works
            rt.rewind_all().unwrap();
            prop_assert_eq!(rt.registers().to_vec(), original);
        }

        #[test]
        fn half_adder_is_always_reversible(a: u64, b: u64) {
            let regs = vec![
                BitPlane::from_words(vec![a]),
                BitPlane::from_words(vec![b]),
                BitPlane::from_words(vec![0]),
            ];
            let mut rt = ReversibleRuntime::new(regs);
            let original = rt.registers().to_vec();
            rt.execute_all_tracked(&half_adder(0, 1, 2));
            rt.rewind_all().unwrap();
            prop_assert_eq!(rt.registers().to_vec(), original);
        }

        #[test]
        fn full_adder_is_always_reversible(a: u64, b: u64, cin: u64) {
            let regs = vec![
                BitPlane::from_words(vec![a]),
                BitPlane::from_words(vec![b]),
                BitPlane::from_words(vec![cin]),
                BitPlane::from_words(vec![0]),
            ];
            let mut rt = ReversibleRuntime::new(regs);
            let original = rt.registers().to_vec();
            rt.execute_all_tracked(&full_adder(0, 1, 2, 3));
            rt.rewind_all().unwrap();
            prop_assert_eq!(rt.registers().to_vec(), original);
        }
    }
}
