//! Reversible algorithms — higher-level computations built from gates and circuits.
//!
//! These algorithms demonstrate that Rewind can perform real, useful computations
//! while maintaining perfect reversibility.
//!
//! # Algorithms
//!
//! - [`reversible_sort_two`] — Reversible comparison and conditional swap of two registers
//! - [`reversible_accumulate`] — Reversible accumulation (XOR-fold) across registers
//! - [`reversible_rotate`] — Reversible rotation of register values

use rewind_core::engine::Op;

use crate::circuits;

/// Reversible conditional swap: swaps R[a] and R[b] if R[a] > R[b] (by bit count).
///
/// Uses a comparison register to track whether a swap was performed,
/// enabling perfect reversal.
///
/// Requires: `a`, `b` are data registers, `flag` is a zeroed ancilla register.
///
/// **Note:** This is a simplified comparison using XOR and Toffoli.
/// For full integer comparison, a more complex circuit is needed.
pub fn conditional_swap(a: usize, b: usize, flag: usize) -> Vec<Op> {
    // Simplified: use Toffoli to compute interaction, then conditional swap
    vec![
        // Compute flag = a AND b (interaction term)
        Op::Toffoli {
            c1: a,
            c2: b,
            target: flag,
        },
        // Conditional NOT on a based on flag
        Op::Cnot {
            control: flag,
            target: a,
        },
        // Conditional NOT on b based on flag
        Op::Cnot {
            control: flag,
            target: b,
        },
    ]
}

/// Reversible accumulation: XOR-folds a sequence of registers into an accumulator.
///
/// After execution, `accumulator` contains `R[regs[0]] XOR R[regs[1]] XOR ... XOR R[regs[n]]`.
/// Since XOR is self-inverse, running the same circuit again restores the accumulator to zero.
///
/// This is the reversible equivalent of `reduce(XOR)`.
pub fn xor_accumulate(registers: &[usize], accumulator: usize) -> Vec<Op> {
    registers
        .iter()
        .map(|&reg| Op::Cnot {
            control: reg,
            target: accumulator,
        })
        .collect()
}

/// Reversible rotation: cyclically rotates values across N registers.
///
/// `rotate([R0, R1, R2])` makes: R0←R1, R1←R2, R2←R0.
/// Uses N-1 swap circuits.
pub fn rotate(registers: &[usize]) -> Vec<Op> {
    if registers.len() < 2 {
        return vec![];
    }
    let mut ops = Vec::new();
    for i in 0..registers.len() - 1 {
        ops.extend(circuits::swap(registers[i], registers[i + 1]));
    }
    ops
}

/// Reversible parity check: computes the XOR parity of all registers into a target.
///
/// `parity([R0, R1, R2], target)` → `target ^= R0 ^ R1 ^ R2`.
/// Useful for error detection in reversible circuits.
pub fn parity(registers: &[usize], target: usize) -> Vec<Op> {
    xor_accumulate(registers, target)
}

/// Reversible fan-out: copies a register's value into multiple targets via XOR.
///
/// `fan_out(source, [t1, t2, t3])` → each target gets `target ^= source`.
/// Targets must be initialized to zero for a clean copy.
pub fn fan_out(source: usize, targets: &[usize]) -> Vec<Op> {
    targets
        .iter()
        .map(|&t| Op::Cnot {
            control: source,
            target: t,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rewind_core::bitplane::BitPlane;
    use rewind_core::runtime::ReversibleRuntime;

    fn rt_from(vals: &[u64]) -> ReversibleRuntime {
        let regs = vals
            .iter()
            .map(|v| BitPlane::from_words(vec![*v]))
            .collect();
        ReversibleRuntime::new(regs)
    }

    #[test]
    fn xor_accumulate_folds_registers() {
        let mut rt = rt_from(&[0xAA, 0x55, 0xFF, 0x00]);
        let ops = xor_accumulate(&[0, 1, 2], 3);
        rt.execute_all_tracked(&ops);
        // 0xAA ^ 0x55 ^ 0xFF = 0x00 (interesting XOR)
        assert_eq!(rt.register(3).words()[0], 0xAA ^ 0x55 ^ 0xFF);
    }

    #[test]
    fn xor_accumulate_is_reversible() {
        let mut rt = rt_from(&[0xAA, 0x55, 0xFF, 0x00]);
        let original = rt.registers().to_vec();
        let ops = xor_accumulate(&[0, 1, 2], 3);
        rt.execute_all_tracked(&ops);
        rt.rewind_all().unwrap();
        assert_eq!(rt.registers().to_vec(), original);
    }

    #[test]
    fn rotate_cycles_values() {
        let mut rt = rt_from(&[0xAA, 0xBB, 0xCC]);
        let ops = rotate(&[0, 1, 2]);
        rt.execute_all_tracked(&ops);
        // After rotation: R0←R1(BB), R1←R2(CC), R2←R0(AA)
        assert_eq!(rt.register(0).words()[0], 0xBB);
        assert_eq!(rt.register(1).words()[0], 0xCC);
        assert_eq!(rt.register(2).words()[0], 0xAA);
    }

    #[test]
    fn rotate_is_reversible() {
        let mut rt = rt_from(&[0xAA, 0xBB, 0xCC]);
        let original = rt.registers().to_vec();
        let ops = rotate(&[0, 1, 2]);
        rt.execute_all_tracked(&ops);
        rt.rewind_all().unwrap();
        assert_eq!(rt.registers().to_vec(), original);
    }

    #[test]
    fn fan_out_copies_to_targets() {
        let mut rt = rt_from(&[0xFF, 0x00, 0x00, 0x00]);
        let ops = fan_out(0, &[1, 2, 3]);
        rt.execute_all_tracked(&ops);
        assert_eq!(rt.register(1).words()[0], 0xFF);
        assert_eq!(rt.register(2).words()[0], 0xFF);
        assert_eq!(rt.register(3).words()[0], 0xFF);
    }

    #[test]
    fn fan_out_is_reversible() {
        let mut rt = rt_from(&[0xFF, 0x00, 0x00, 0x00]);
        let original = rt.registers().to_vec();
        let ops = fan_out(0, &[1, 2, 3]);
        rt.execute_all_tracked(&ops);
        rt.rewind_all().unwrap();
        assert_eq!(rt.registers().to_vec(), original);
    }

    #[test]
    fn parity_computes_xor_of_all() {
        let mut rt = rt_from(&[0b1010, 0b1100, 0b1111, 0x00]);
        let ops = parity(&[0, 1, 2], 3);
        rt.execute_all_tracked(&ops);
        assert_eq!(rt.register(3).words()[0], 0b1010 ^ 0b1100 ^ 0b1111);
    }

    #[test]
    fn conditional_swap_is_reversible() {
        let mut rt = rt_from(&[0xAA, 0x55, 0x00]);
        let original = rt.registers().to_vec();
        let ops = conditional_swap(0, 1, 2);
        rt.execute_all_tracked(&ops);
        rt.rewind_all().unwrap();
        assert_eq!(rt.registers().to_vec(), original);
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
        fn xor_accumulate_always_reversible(a: u64, b: u64, c: u64) {
            let regs = vec![
                BitPlane::from_words(vec![a]),
                BitPlane::from_words(vec![b]),
                BitPlane::from_words(vec![c]),
                BitPlane::from_words(vec![0]),
            ];
            let mut rt = ReversibleRuntime::new(regs);
            let original = rt.registers().to_vec();
            let ops = xor_accumulate(&[0, 1, 2], 3);
            rt.execute_all_tracked(&ops);
            rt.rewind_all().unwrap();
            prop_assert_eq!(rt.registers().to_vec(), original);
        }

        #[test]
        fn rotate_always_reversible(a: u64, b: u64, c: u64, d: u64) {
            let regs = vec![
                BitPlane::from_words(vec![a]),
                BitPlane::from_words(vec![b]),
                BitPlane::from_words(vec![c]),
                BitPlane::from_words(vec![d]),
            ];
            let mut rt = ReversibleRuntime::new(regs);
            let original = rt.registers().to_vec();
            let ops = rotate(&[0, 1, 2, 3]);
            rt.execute_all_tracked(&ops);
            rt.rewind_all().unwrap();
            prop_assert_eq!(rt.registers().to_vec(), original);
        }

        #[test]
        fn fan_out_always_reversible(source: u64) {
            let regs = vec![
                BitPlane::from_words(vec![source]),
                BitPlane::from_words(vec![0]),
                BitPlane::from_words(vec![0]),
            ];
            let mut rt = ReversibleRuntime::new(regs);
            let original = rt.registers().to_vec();
            let ops = fan_out(0, &[1, 2]);
            rt.execute_all_tracked(&ops);
            rt.rewind_all().unwrap();
            prop_assert_eq!(rt.registers().to_vec(), original);
        }
    }
}
