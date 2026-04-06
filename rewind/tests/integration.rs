//! End-to-end integration tests for the Rewind SDK.
//!
//! These tests verify that all components work together correctly:
//! QuantumCell + Gates + Runtime + GC + Circuits.

use rewind::prelude::*;
use rewind_gates::circuits;
use rewind_gc::{GarbageFreeCollector, MemoryBudget};

#[test]
fn full_pipeline_forward_backward() {
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0xDEAD]),
        BitPlane::from_words(vec![0xBEEF]),
        BitPlane::from_words(vec![0xCAFE]),
    ]);
    let original = rt.registers().to_vec();

    // Complex program: 10 operations
    let program = circuits::compose(vec![
        vec![Op::Not(0)],
        circuits::swap(0, 1),
        vec![Op::Cnot {
            control: 1,
            target: 2,
        }],
        vec![Op::Toffoli {
            c1: 0,
            c2: 2,
            target: 1,
        }],
        circuits::swap(1, 2),
        vec![Op::Not(2)],
    ]);

    rt.execute_all_tracked(&program);
    assert_ne!(rt.registers().to_vec(), original);
    assert!(!rt.is_garbage_free());

    rt.rewind_all().unwrap();
    assert_eq!(rt.registers().to_vec(), original);
    assert!(rt.is_garbage_free());
}

#[test]
fn quantum_cell_lifecycle() {
    // Create, modify, consume — the full linear type lifecycle
    let mut cell = QuantumCell::new(vec![1u64, 2, 3]);
    cell.get_mut().push(4);
    assert_eq!(cell.get().len(), 4);
    let value = cell.consume();
    assert_eq!(value, vec![1, 2, 3, 4]);
}

#[test]
#[should_panic(expected = "information lost")]
fn quantum_cell_panic_on_drop() {
    let _cell = QuantumCell::new(42u64);
    // Dropped without consume → panic
}

#[test]
fn gates_are_reversible() {
    // Verify all gate types with the assert_reversible helper
    assert_reversible(
        &rewind_gates::scalar::PauliX,
        BitPlane::from_words(vec![0xFFFF]),
    );
    assert_reversible(
        &rewind_gates::scalar::Cnot,
        rewind_gates::scalar::CnotState {
            control: BitPlane::from_words(vec![0xAA]),
            target: BitPlane::from_words(vec![0x55]),
        },
    );
    assert_reversible(
        &rewind_gates::scalar::Toffoli,
        rewind_gates::scalar::ToffoliState {
            control1: BitPlane::from_words(vec![0xAA]),
            control2: BitPlane::from_words(vec![0x55]),
            target: BitPlane::from_words(vec![0xFF]),
        },
    );
}

#[test]
fn garbage_free_collector_with_runtime() {
    let mut gc = GarbageFreeCollector::new(MemoryBudget::new(1024));

    // Simulate forward execution checkpoints
    gc.checkpoint_ancilla(BitPlane::from_words(vec![0xAA]))
        .unwrap();
    gc.checkpoint_ancilla(BitPlane::from_words(vec![0xBB]))
        .unwrap();
    gc.checkpoint_ancilla(BitPlane::from_words(vec![0xCC]))
        .unwrap();

    assert_eq!(gc.ancilla_count(), 3);
    assert!(!gc.is_garbage_free());

    // Uncompute in LIFO order
    assert_eq!(gc.uncompute().unwrap().words()[0], 0xCC);
    assert_eq!(gc.uncompute().unwrap().words()[0], 0xBB);
    assert_eq!(gc.uncompute().unwrap().words()[0], 0xAA);

    assert!(gc.is_garbage_free());
    assert!(gc.verify().is_ok());
}

#[test]
fn checkpoint_restore_workflow() {
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![100]),
        BitPlane::from_words(vec![200]),
    ]);

    let ckpt = rt.checkpoint();

    // Do some computation
    rt.execute_tracked(Op::Not(0));
    rt.execute_tracked(Op::Cnot {
        control: 0,
        target: 1,
    });

    // Restore to checkpoint
    rt.restore(ckpt).unwrap();
    assert_eq!(rt.register(0).words()[0], 100);
    assert_eq!(rt.register(1).words()[0], 200);
}

#[test]
fn circuits_compose_and_reverse() {
    let circuit = circuits::compose(vec![
        circuits::swap(0, 1),
        vec![Op::Not(0)],
        vec![Op::Cnot {
            control: 0,
            target: 1,
        }],
    ]);

    let inverse = circuits::reverse(&circuit);
    let roundtrip = circuits::compose(vec![circuit, inverse]);

    // Forward + inverse = identity
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0xAA]),
        BitPlane::from_words(vec![0xBB]),
    ]);
    let original = rt.registers().to_vec();

    rt.execute_all_tracked(&roundtrip);
    assert_eq!(rt.registers().to_vec(), original);
}

#[test]
fn xor_cipher_encrypt_decrypt() {
    let message: u64 = 0x48454C4C4F;
    let key: u64 = 0xDEADBEEFCAFE;

    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![message]),
        BitPlane::from_words(vec![key]),
    ]);

    // Encrypt
    rt.execute_tracked(Op::Cnot {
        control: 1,
        target: 0,
    });
    let encrypted = rt.register(0).words()[0];
    assert_ne!(encrypted, message);

    // Decrypt via rewind
    rt.rewind_all().unwrap();
    assert_eq!(rt.register(0).words()[0], message);
    assert!(rt.is_garbage_free());
}

#[test]
fn traced_execution_collects_snapshots() {
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0]),
        BitPlane::from_words(vec![0]),
    ]);

    let ops = vec![
        Op::Not(0),
        Op::Cnot {
            control: 0,
            target: 1,
        },
    ];

    let mut snapshots = Vec::new();
    rt.execute_traced(&ops, |_step, _op, regs| {
        snapshots.push(regs.to_vec());
    });

    assert_eq!(snapshots.len(), 2);
    // After NOT R0: R0 should be all 1s
    assert_ne!(snapshots[0][0], 0);
    // After CNOT: R1 should equal R0
    assert_eq!(snapshots[1][0], snapshots[1][1]);
}

#[test]
fn bennett_plans_computation() {
    use rewind_bennett::{BennettConfig, BennettExecutor, ComputationGraph};

    let graph = ComputationGraph::linear_chain(&["load", "compute", "transform", "output"]);
    let executor = BennettExecutor::new(BennettConfig::default());
    let plan = executor.plan(&graph);

    assert_eq!(plan.forward_steps.len(), 4);
    assert_eq!(plan.backward_steps.len(), 3); // Skip final result
    assert!(plan.strategy.checkpoint_count() > 0);
}
