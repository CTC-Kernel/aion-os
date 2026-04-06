//! Reversible Algorithms — practical computations that can be perfectly undone.
//!
//! Run with: `cargo run -p rewind --example algorithms`

use rewind::prelude::*;
use rewind_gates::algorithms;

fn main() {
    println!("=== Rewind: Algorithmes Reversibles ===\n");

    // --- XOR Accumulate ---
    println!("--- XOR Accumulate (reduce par XOR) ---");
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0b1010]),
        BitPlane::from_words(vec![0b1100]),
        BitPlane::from_words(vec![0b1111]),
        BitPlane::from_words(vec![0b0000]), // accumulateur
    ]);
    let ops = algorithms::xor_accumulate(&[0, 1, 2], 3);
    rt.execute_all_tracked(&ops);
    println!(
        "  R0=0b{:04b}, R1=0b{:04b}, R2=0b{:04b} → ACC=0b{:04b}",
        0b1010u64,
        0b1100u64,
        0b1111u64,
        rt.register(3).words()[0]
    );
    rt.rewind_all().unwrap();
    println!(
        "  Apres rewind: ACC=0b{:04b} (zero)",
        rt.register(3).words()[0]
    );

    // --- Rotate ---
    println!("\n--- Rotation Cyclique ---");
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0xAA]),
        BitPlane::from_words(vec![0xBB]),
        BitPlane::from_words(vec![0xCC]),
    ]);
    println!(
        "  Avant : R0=0x{:02X}, R1=0x{:02X}, R2=0x{:02X}",
        rt.register(0).words()[0],
        rt.register(1).words()[0],
        rt.register(2).words()[0]
    );
    rt.execute_all_tracked(&algorithms::rotate(&[0, 1, 2]));
    println!(
        "  Apres : R0=0x{:02X}, R1=0x{:02X}, R2=0x{:02X}",
        rt.register(0).words()[0],
        rt.register(1).words()[0],
        rt.register(2).words()[0]
    );
    rt.rewind_all().unwrap();
    println!(
        "  Rewind: R0=0x{:02X}, R1=0x{:02X}, R2=0x{:02X}",
        rt.register(0).words()[0],
        rt.register(1).words()[0],
        rt.register(2).words()[0]
    );

    // --- Fan-Out ---
    println!("\n--- Fan-Out (copie reversible) ---");
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0xFF]),
        BitPlane::from_words(vec![0x00]),
        BitPlane::from_words(vec![0x00]),
        BitPlane::from_words(vec![0x00]),
    ]);
    rt.execute_all_tracked(&algorithms::fan_out(0, &[1, 2, 3]));
    println!(
        "  Source=0xFF → R1=0x{:02X}, R2=0x{:02X}, R3=0x{:02X}",
        rt.register(1).words()[0],
        rt.register(2).words()[0],
        rt.register(3).words()[0]
    );
    rt.rewind_all().unwrap();
    println!(
        "  Rewind:  R1=0x{:02X}, R2=0x{:02X}, R3=0x{:02X} (zeros restaures)",
        rt.register(1).words()[0],
        rt.register(2).words()[0],
        rt.register(3).words()[0]
    );

    // --- Stats ---
    println!("\n--- Statistiques Runtime ---");
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0xDEAD]),
        BitPlane::from_words(vec![0xBEEF]),
    ]);
    for _ in 0..100 {
        rt.execute_tracked(Op::Not(0));
        rt.execute_tracked(Op::Cnot {
            control: 0,
            target: 1,
        });
    }
    println!("  {}", rt.stats());
    rt.rewind_all().unwrap();
    println!("  Apres rewind: {}", rt.stats());
    assert!(rt.is_garbage_free());

    println!("\n=== Tous les algorithmes sont parfaitement reversibles. ===");
}
