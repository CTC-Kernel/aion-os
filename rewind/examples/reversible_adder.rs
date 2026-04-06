//! Reversible Adder — real computation using Toffoli gates.
//!
//! Demonstrates a half-adder and swap circuit built from reversible gates.
//! Every operation can be perfectly undone.
//!
//! Run with: `cargo run -p rewind --example reversible_adder`

use rewind::prelude::*;
use rewind_gates::circuits;

fn main() {
    println!("=== Rewind: Reversible Adder Circuit ===\n");

    // --- SWAP demo ---
    println!("--- SWAP Circuit (3 CNOTs) ---");
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0xAAAA]),
        BitPlane::from_words(vec![0xBBBB]),
    ]);

    println!(
        "Before swap: R0=0x{:04X}, R1=0x{:04X}",
        rt.register(0).words()[0],
        rt.register(1).words()[0]
    );

    rt.execute_all_tracked(&circuits::swap(0, 1));
    println!(
        "After swap:  R0=0x{:04X}, R1=0x{:04X}",
        rt.register(0).words()[0],
        rt.register(1).words()[0]
    );

    rt.rewind_all().unwrap();
    println!(
        "After undo:  R0=0x{:04X}, R1=0x{:04X}",
        rt.register(0).words()[0],
        rt.register(1).words()[0]
    );
    assert!(rt.is_garbage_free());

    // --- Half-adder demo ---
    println!("\n--- Half-Adder Circuit (CNOT + Toffoli) ---");
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0b1010]), // Input A
        BitPlane::from_words(vec![0b1100]), // Input B
        BitPlane::from_words(vec![0b0000]), // Carry (starts at 0)
    ]);

    println!(
        "Inputs:  A=0b{:04b}, B=0b{:04b}, Carry=0b{:04b}",
        rt.register(0).words()[0],
        rt.register(1).words()[0],
        rt.register(2).words()[0]
    );

    rt.execute_all_tracked(&circuits::half_adder(0, 1, 2));

    println!(
        "Results: A=0b{:04b}, Sum=0b{:04b}, Carry=0b{:04b}",
        rt.register(0).words()[0],
        rt.register(1).words()[0],
        rt.register(2).words()[0]
    );

    // Rewind
    rt.rewind_all().unwrap();
    println!(
        "Rewound: A=0b{:04b}, B=0b{:04b}, Carry=0b{:04b}",
        rt.register(0).words()[0],
        rt.register(1).words()[0],
        rt.register(2).words()[0]
    );
    assert!(rt.is_garbage_free());

    // --- Circuit composition ---
    println!("\n--- Composed Circuit: Swap + NOT + Swap ---");
    let composed = circuits::compose(vec![
        circuits::swap(0, 1),
        vec![Op::Not(0)],
        circuits::swap(0, 1),
    ]);
    println!("Circuit has {} operations (all reversible)", composed.len());

    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0xFF]),
        BitPlane::from_words(vec![0x00]),
    ]);
    let original = rt.registers().to_vec();
    rt.execute_all_tracked(&composed);
    rt.rewind_all().unwrap();
    assert_eq!(rt.registers().to_vec(), original);
    println!("Composed circuit: forward + rewind = identity ✓");

    println!("\n✅ Real reversible computation — every bit accounted for!");
}
