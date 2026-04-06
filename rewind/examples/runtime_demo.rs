//! ReversibleRuntime — the unified API for reversible computation.
//!
//! Run with: `cargo run -p rewind --example runtime_demo`

use rewind::prelude::*;

fn main() {
    println!("=== Rewind Runtime — Unified Reversible Computation ===\n");

    // Create a runtime with 3 registers
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0xAA]),
        BitPlane::from_words(vec![0xBB]),
        BitPlane::from_words(vec![0xCC]),
    ]);

    println!("Initial state:");
    print_regs(&rt);

    // Execute operations with automatic tracking
    println!("\n--- Executing 4 operations (tracked) ---");
    rt.execute_tracked(Op::Not(0));
    rt.execute_tracked(Op::Cnot {
        control: 0,
        target: 1,
    });
    rt.execute_tracked(Op::Toffoli {
        c1: 0,
        c2: 1,
        target: 2,
    });
    rt.execute_tracked(Op::Not(2));

    println!("After 4 operations ({} in history):", rt.history_len());
    print_regs(&rt);

    // Partial rewind — undo just the last 2
    println!("\n--- Rewinding 2 operations ---");
    rt.rewind(2).unwrap();
    println!("After partial rewind ({} in history):", rt.history_len());
    print_regs(&rt);

    // Rewind everything
    println!("\n--- Rewinding all remaining ---");
    rt.rewind_all().unwrap();
    println!("After full rewind:");
    print_regs(&rt);

    assert_eq!(rt.register(0).words()[0], 0xAA);
    assert_eq!(rt.register(1).words()[0], 0xBB);
    assert_eq!(rt.register(2).words()[0], 0xCC);
    assert!(rt.is_garbage_free());

    println!(
        "\n✅ Garbage-free: {} | History: {}",
        rt.is_garbage_free(),
        rt.history_len()
    );
    println!("\nInformation is Sacred. Every operation rewound perfectly.");
}

fn print_regs(rt: &ReversibleRuntime) {
    for i in 0..rt.num_registers() {
        println!("  R{i}: 0x{:016X}", rt.register(i).words()[0]);
    }
}
