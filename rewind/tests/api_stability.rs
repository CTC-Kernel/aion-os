//! API stability tests — verify every public API works as documented.
//!
//! These tests lock down the v1.0 API contract.

use rewind::prelude::*;
use rewind_gates::{algorithms, circuits};

// === QuantumCell API ===

#[test]
fn quantum_cell_new_get_consume() {
    let cell = QuantumCell::new(42u64);
    assert_eq!(*cell.get(), 42);
    assert_eq!(cell.consume(), 42);
}

#[test]
fn quantum_cell_get_mut() {
    let mut cell = QuantumCell::new(10u64);
    *cell.get_mut() = 20;
    assert_eq!(cell.consume(), 20);
}

#[test]
fn quantum_cell_map_in_place() {
    let mut cell = QuantumCell::new(10u64);
    cell.map_in_place(|v| *v *= 3);
    assert_eq!(cell.consume(), 30);
}

#[test]
fn quantum_cell_add_assign() {
    let mut cell = QuantumCell::new(10u64);
    cell += 5u64;
    assert_eq!(cell.consume(), 15);
}

#[test]
fn quantum_cell_sub_assign() {
    let mut cell = QuantumCell::new(10u64);
    cell -= 3u64;
    assert_eq!(cell.consume(), 7);
}

#[test]
fn quantum_cell_xor_assign() {
    let mut cell = QuantumCell::new(0xFFu64);
    cell ^= 0x0Fu64;
    assert_eq!(cell.consume(), 0xF0);
}

#[test]
#[should_panic(expected = "information lost")]
fn quantum_cell_drop_panics() {
    let _cell = QuantumCell::new(42u64);
}

// === BitPlane API ===

#[test]
fn bitplane_from_conversions() {
    let a = BitPlane::from(42u64);
    assert_eq!(a.words()[0], 42);

    let b = BitPlane::from(vec![1u64, 2, 3]);
    assert_eq!(b.len(), 3);

    let c = BitPlane::zeroed(5);
    assert!(c.is_zero());
    assert_eq!(c.bit_count(), 320);
}

#[test]
fn bitplane_operators() {
    let a = BitPlane::from(0xFFu64);
    let b = BitPlane::from(0x0Fu64);

    assert_eq!((&a ^ &b).words()[0], 0xF0);
    assert_eq!((&a & &b).words()[0], 0x0F);
    assert_eq!((!&a).words()[0], !0xFFu64);
}

#[test]
fn bitplane_popcount() {
    let bp = BitPlane::from(0b11110000u64);
    assert_eq!(bp.popcount(), 4);
}

// === Op API ===

#[test]
fn op_display() {
    assert_eq!(Op::Not(0).to_string(), "NOT R0");
    assert_eq!(
        Op::Cnot {
            control: 1,
            target: 2
        }
        .to_string(),
        "CNOT R1->R2"
    );
    assert_eq!(
        Op::Toffoli {
            c1: 0,
            c2: 1,
            target: 2
        }
        .to_string(),
        "TOFF R0,R1->R2"
    );
    assert_eq!(
        Op::Fredkin {
            control: 0,
            a: 1,
            b: 2
        }
        .to_string(),
        "FRED R0:R1<->R2"
    );
}

#[test]
fn op_equality() {
    assert_eq!(Op::Not(0), Op::Not(0));
    assert_ne!(Op::Not(0), Op::Not(1));
}

// === ProgramBuilder API ===

#[test]
fn program_builder_chain() {
    let prog = ProgramBuilder::new()
        .not(0)
        .cnot(0, 1)
        .toffoli(0, 1, 2)
        .fredkin(0, 1, 2)
        .build();
    assert_eq!(prog.len(), 4);
    assert!(!prog.is_empty());
}

#[test]
fn program_reversed() {
    let prog = ProgramBuilder::new().not(0).cnot(0, 1).build();
    let rev = prog.reversed();
    assert_eq!(
        rev.ops[0],
        Op::Cnot {
            control: 0,
            target: 1
        }
    );
    assert_eq!(rev.ops[1], Op::Not(0));
}

#[test]
fn program_concat() {
    let a = ProgramBuilder::new().not(0).build();
    let b = ProgramBuilder::new().not(1).build();
    let c = a.concat(&b);
    assert_eq!(c.len(), 2);
}

// === ReversibleRuntime API ===

#[test]
fn runtime_full_lifecycle() {
    let mut rt = ReversibleRuntime::new(vec![BitPlane::from(0xAAu64), BitPlane::from(0xBBu64)]);
    let original = rt.registers().to_vec();

    // Execute
    rt.execute_tracked(Op::Not(0));
    rt.execute_tracked(Op::Cnot {
        control: 0,
        target: 1,
    });
    assert_eq!(rt.history_len(), 2);
    assert!(!rt.is_garbage_free());

    // Stats
    let stats = rt.stats();
    assert_eq!(stats.total_ops, 2);
    assert_eq!(stats.history_depth, 2);
    assert_eq!(stats.num_registers, 2);

    // Rewind
    rt.rewind_all().unwrap();
    assert_eq!(rt.registers().to_vec(), original);
    assert!(rt.is_garbage_free());
}

#[test]
fn runtime_run_forward() {
    let program = ProgramBuilder::new().not(0).cnot(0, 1).build();
    let mut rt = ReversibleRuntime::new(vec![BitPlane::from(0xAAu64), BitPlane::from(0x00u64)]);
    let original = rt.registers().to_vec();

    rt.run_forward(&program);
    rt.rewind_all().unwrap();
    assert_eq!(rt.registers().to_vec(), original);
}

#[test]
fn runtime_checkpoint() {
    let mut rt = ReversibleRuntime::new(vec![BitPlane::from(42u64)]);
    let ckpt = rt.checkpoint();
    rt.execute_tracked(Op::Not(0));
    rt.restore(ckpt).unwrap();
    assert_eq!(rt.register(0).words()[0], 42);
}

#[test]
fn runtime_iter_registers() {
    let rt = ReversibleRuntime::new(vec![
        BitPlane::from(1u64),
        BitPlane::from(2u64),
        BitPlane::from(3u64),
    ]);
    let vals: Vec<u64> = rt.iter_registers().map(|r| r.words()[0]).collect();
    assert_eq!(vals, vec![1, 2, 3]);
}

#[test]
fn runtime_history() {
    let mut rt = ReversibleRuntime::new(vec![BitPlane::from(0u64)]);
    rt.execute_tracked(Op::Not(0));
    assert_eq!(rt.history().len(), 1);
    assert_eq!(rt.history()[0], Op::Not(0));
}

// === Circuits API ===

#[test]
fn circuits_all_reversible() {
    let test_cases: Vec<(&str, Vec<Op>)> = vec![
        ("swap", circuits::swap(0, 1)),
        ("half_adder", circuits::half_adder(0, 1, 2)),
        ("full_adder", circuits::full_adder(0, 1, 2, 3)),
        ("xor_accumulate", algorithms::xor_accumulate(&[0, 1, 2], 3)),
        ("rotate", algorithms::rotate(&[0, 1, 2, 3])),
        ("fan_out", algorithms::fan_out(0, &[1, 2, 3])),
        ("parity", algorithms::parity(&[0, 1, 2], 3)),
    ];

    for (name, ops) in test_cases {
        let mut rt = ReversibleRuntime::new(vec![
            BitPlane::from(0xAAu64),
            BitPlane::from(0xBBu64),
            BitPlane::from(0xCCu64),
            BitPlane::from(0xDDu64),
        ]);
        let original = rt.registers().to_vec();

        rt.execute_all_tracked(&ops);
        rt.rewind_all().unwrap();
        assert_eq!(
            rt.registers().to_vec(),
            original,
            "Circuit '{name}' is not reversible"
        );
    }
}

// === Gates API ===

#[test]
fn all_gates_satisfy_reversibility_contract() {
    assert_reversible(&PauliX, BitPlane::from(0xDEADBEEFu64));

    assert_reversible(
        &Cnot,
        CnotState {
            control: BitPlane::from(0xAAu64),
            target: BitPlane::from(0x55u64),
        },
    );

    assert_reversible(
        &Toffoli,
        ToffoliState {
            control1: BitPlane::from(0xAAu64),
            control2: BitPlane::from(0x55u64),
            target: BitPlane::from(0xFFu64),
        },
    );

    assert_reversible(
        &Fredkin,
        FredkinState {
            control: BitPlane::from(0xFFu64),
            target_a: BitPlane::from(0xAAu64),
            target_b: BitPlane::from(0x55u64),
        },
    );
}

// === Program Serialization API ===

#[test]
fn program_text_roundtrip() {
    let ops = vec![
        Op::Not(0),
        Op::Cnot {
            control: 1,
            target: 2,
        },
        Op::Toffoli {
            c1: 0,
            c2: 1,
            target: 2,
        },
        Op::Fredkin {
            control: 0,
            a: 1,
            b: 2,
        },
    ];
    let text = rewind_core::program::serialize_text(&ops);
    let parsed = rewind_core::program::deserialize_text(&text).unwrap();
    assert_eq!(ops, parsed);
}

#[test]
fn program_info_analysis() {
    let ops = ProgramBuilder::new()
        .not(0)
        .cnot(0, 1)
        .toffoli(0, 1, 2)
        .fredkin(0, 1, 2)
        .build_ops();

    let info = rewind_core::program::ProgramInfo::analyze(&ops);
    assert_eq!(info.total_ops, 4);
    assert_eq!(info.min_registers(), 3);
    assert!(info.depth > 0);
}
