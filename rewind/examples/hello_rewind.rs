//! Hello Rewind — your first reversible program.
//!
//! Run with: `cargo run --example hello_rewind`

use rewind_core::bitplane::BitPlane;
use rewind_core::engine::{ExecutionEngine, Op, ReversibleProgram};

fn main() {
    println!("=== Rewind: Information is Sacred ===\n");

    // Create 3 registers with initial values
    let registers = vec![
        BitPlane::from_words(vec![0xFF]),  // Register 0: 0xFF
        BitPlane::from_words(vec![0x0F]),  // Register 1: 0x0F
        BitPlane::from_words(vec![0x00]),  // Register 2: 0x00
    ];

    let mut engine = ExecutionEngine::new(registers);
    println!("Initial state:");
    print_registers(&engine);

    // Build a reversible program
    let program = ReversibleProgram::new(vec![
        Op::Not(0),                                    // Invert register 0
        Op::Cnot { control: 0, target: 1 },           // XOR reg1 with reg0
        Op::Toffoli { c1: 0, c2: 1, target: 2 },     // reg2 ^= (reg0 AND reg1)
    ]);

    // Execute forward
    program.forward(&mut engine);
    println!("\nAfter forward execution:");
    print_registers(&engine);

    // Execute backward — should restore original state
    program.backward(&mut engine);
    println!("\nAfter backward execution (restored!):");
    print_registers(&engine);

    // Verify restoration
    assert_eq!(engine.registers()[0].words()[0], 0xFF);
    assert_eq!(engine.registers()[1].words()[0], 0x0F);
    assert_eq!(engine.registers()[2].words()[0], 0x00);

    println!("\n✅ Forward → Backward = Original state. Information preserved!");
}

fn print_registers(engine: &ExecutionEngine) {
    for (i, reg) in engine.registers().iter().enumerate() {
        println!("  Register {i}: 0x{:016X}", reg.words()[0]);
    }
}
