//! Fluent API Demo — build reversible programs elegantly.
//!
//! Run with: `cargo run -p rewind --example fluent_api`

use rewind::prelude::*;

fn main() {
    println!("=== Rewind: API Fluente ===\n");

    // --- ProgramBuilder ---
    println!("--- ProgramBuilder: construction fluente ---");
    let program = ProgramBuilder::new()
        .not(0)
        .cnot(0, 1)
        .toffoli(0, 1, 2)
        .fredkin(0, 1, 2)
        .not(2)
        .build();

    println!("Programme: {} operations", program.ops.len());
    for (i, op) in program.ops.iter().enumerate() {
        println!("  [{i}] {op}");
    }

    // Execute and rewind
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from(0xAAu64),
        BitPlane::from(0xBBu64),
        BitPlane::from(0xCCu64),
    ]);
    let original = rt.snapshot();

    rt.execute_all_tracked(&program.ops);
    println!("\nApres forward: {:X?}", rt.snapshot());

    rt.rewind_all().unwrap();
    assert_eq!(rt.snapshot(), original);
    println!("Apres rewind: {:X?} (restaure!)", rt.snapshot());

    // --- QuantumCell operators ---
    println!("\n--- QuantumCell: operateurs +=, -=, ^= ---");
    let mut cell = QuantumCell::new(100u64);
    cell += 50;
    cell -= 20;
    cell ^= 0xFF;
    println!("  100 + 50 - 20 ^ 0xFF = {}", cell.consume());

    // --- BitPlane conversions ---
    println!("\n--- BitPlane: conversions From ---");
    let a = BitPlane::from(0xFFu64);
    let b = BitPlane::from(0x0Fu64);
    let c = &a ^ &b;
    let d = &a & &b;
    let e = !&a;
    println!("  0xFF ^ 0x0F = 0x{:X}", c.words()[0]);
    println!("  0xFF & 0x0F = 0x{:X}", d.words()[0]);
    println!("  !0xFF = 0x{:X}", e.words()[0]);

    // --- Repeat ---
    println!("\n--- ProgramBuilder: repeat ---");
    let repeated = ProgramBuilder::new().not(0).cnot(0, 1).repeat(5).build();
    println!("  2 ops x 5 = {} ops total", repeated.ops.len());

    println!("\n=== API fluente, ergonomique, reversible. ===");
}
