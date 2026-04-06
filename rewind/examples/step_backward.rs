//! Step Backward — demonstrating time-travel debugging with checkpoints.
//!
//! Run with: `cargo run --example step_backward`

use rewind_core::bitplane::BitPlane;
use rewind_core::engine::{ExecutionEngine, Op, ReversibleProgram};

fn main() {
    println!("=== Rewind: Step Backward Debugging ===\n");

    let registers = vec![
        BitPlane::from_words(vec![42]),
        BitPlane::from_words(vec![0]),
    ];
    let mut engine = ExecutionEngine::new(registers);

    // Save checkpoint before computation
    let checkpoint = engine.checkpoint();
    println!(
        "Checkpoint saved. Register 0 = {}",
        engine.registers()[0].words()[0]
    );

    // Execute several operations
    let program = ReversibleProgram::new(vec![
        Op::Not(0),
        Op::Cnot {
            control: 0,
            target: 1,
        },
        Op::Not(1),
    ]);

    program.forward(&mut engine);
    println!(
        "After computation: Register 0 = {}, Register 1 = {}",
        engine.registers()[0].words()[0],
        engine.registers()[1].words()[0],
    );

    // Something went wrong? Step backward!
    program.backward(&mut engine);
    println!(
        "After step backward: Register 0 = {}, Register 1 = {}",
        engine.registers()[0].words()[0],
        engine.registers()[1].words()[0],
    );

    // Or restore from checkpoint
    engine.restore(checkpoint).unwrap();
    println!(
        "After checkpoint restore: Register 0 = {}",
        engine.registers()[0].words()[0]
    );

    assert_eq!(engine.registers()[0].words()[0], 42);
    println!("\n✅ Time-travel debugging works. The past is always accessible.");
}
