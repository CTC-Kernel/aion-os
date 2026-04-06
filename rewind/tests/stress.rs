//! Stress tests — verify reversibility under extreme conditions.

use proptest::prelude::*;
use rewind::prelude::*;
use rewind_gates::{algorithms, circuits};

proptest! {
    #[test]
    fn large_program_is_reversible(seed: u64) {
        // Generate a program of 50+ random operations
        let mut ops = Vec::new();
        let mut rng = seed;
        for _ in 0..50 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let op = match rng % 5 {
                0 => Op::Not((rng as usize / 5) % 4),
                1 => {
                    let c = (rng as usize / 5) % 4;
                    Op::Cnot { control: c, target: (c + 1) % 4 }
                }
                2 => {
                    let c1 = (rng as usize / 5) % 4;
                    Op::Toffoli { c1, c2: (c1 + 1) % 4, target: (c1 + 2) % 4 }
                }
                3 => {
                    // Swap circuit (3 ops)
                    let a = (rng as usize / 5) % 4;
                    let b = (a + 1) % 4;
                    ops.extend(circuits::swap(a, b));
                    continue;
                }
                _ => {
                    // Fan-out
                    let src = (rng as usize / 5) % 4;
                    let tgt = (src + 2) % 4;
                    Op::Cnot { control: src, target: tgt }
                }
            };
            ops.push(op);
        }

        let regs = vec![
            BitPlane::from_words(vec![seed]),
            BitPlane::from_words(vec![seed.wrapping_mul(7)]),
            BitPlane::from_words(vec![seed.wrapping_mul(13)]),
            BitPlane::from_words(vec![seed.wrapping_mul(31)]),
        ];
        let mut rt = ReversibleRuntime::new(regs);
        let original = rt.registers().to_vec();

        rt.execute_all_tracked(&ops);
        rt.rewind_all().unwrap();
        prop_assert_eq!(rt.registers().to_vec(), original);
        prop_assert!(rt.is_garbage_free());
    }

    #[test]
    fn compose_then_reverse_is_identity(a: u64, b: u64, c: u64) {
        let circuit = circuits::compose(vec![
            vec![Op::Not(0)],
            circuits::swap(0, 1),
            vec![Op::Cnot { control: 0, target: 1 }],
            vec![Op::Toffoli { c1: 0, c2: 1, target: 2 }],
        ]);
        let inverse = circuits::reverse(&circuit);
        let roundtrip = circuits::compose(vec![circuit, inverse]);

        let mut rt = ReversibleRuntime::new(vec![
            BitPlane::from_words(vec![a]),
            BitPlane::from_words(vec![b]),
            BitPlane::from_words(vec![c]),
        ]);
        let original = rt.registers().to_vec();
        rt.execute_all_tracked(&roundtrip);
        prop_assert_eq!(rt.registers().to_vec(), original);
    }

    #[test]
    fn checkpoint_restore_preserves_state(a: u64, b: u64) {
        let mut rt = ReversibleRuntime::new(vec![
            BitPlane::from_words(vec![a]),
            BitPlane::from_words(vec![b]),
        ]);
        let ckpt = rt.checkpoint();

        // Do random stuff
        rt.execute_tracked(Op::Not(0));
        rt.execute_tracked(Op::Cnot { control: 0, target: 1 });
        rt.execute_tracked(Op::Not(1));

        // Restore
        rt.restore(ckpt).unwrap();
        prop_assert_eq!(rt.register(0).words()[0], a);
        prop_assert_eq!(rt.register(1).words()[0], b);
    }

    #[test]
    fn algorithms_always_reversible(a: u64, b: u64, c: u64, d: u64) {
        let regs = vec![
            BitPlane::from_words(vec![a]),
            BitPlane::from_words(vec![b]),
            BitPlane::from_words(vec![c]),
            BitPlane::from_words(vec![d]),
        ];
        let mut rt = ReversibleRuntime::new(regs);
        let original = rt.registers().to_vec();

        // Chain multiple algorithms
        let ops = circuits::compose(vec![
            algorithms::rotate(&[0, 1, 2, 3]),
            algorithms::xor_accumulate(&[0, 1, 2], 3),
            algorithms::fan_out(0, &[1, 2]),
            circuits::swap(0, 3),
        ]);

        rt.execute_all_tracked(&ops);
        rt.rewind_all().unwrap();
        prop_assert_eq!(rt.registers().to_vec(), original);
        prop_assert!(rt.is_garbage_free());
    }

    #[test]
    fn serialize_deserialize_roundtrip(seed: u64) {
        let ops = vec![
            Op::Not((seed as usize) % 10),
            Op::Cnot { control: 0, target: 1 },
            Op::Toffoli { c1: 2, c2: 3, target: 4 },
        ];
        let text = rewind_core::program::serialize_text(&ops);
        let parsed = rewind_core::program::deserialize_text(&text).unwrap();
        let text2 = rewind_core::program::serialize_text(&parsed);
        prop_assert_eq!(text, text2);
    }
}
