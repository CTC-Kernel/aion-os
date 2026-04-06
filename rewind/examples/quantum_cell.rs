//! QuantumCell — demonstrating linear types in Rust.
//!
//! Run with: `cargo run --example quantum_cell`

use rewind_core::QuantumCell;

fn main() {
    println!("=== Rewind: QuantumCell — Information is Sacred ===\n");

    // Create a QuantumCell — it MUST be consumed
    let mut cell = QuantumCell::new(42u64);
    println!("Created QuantumCell with value: {}", cell.get());

    // Modify in-place (reversible operation)
    *cell.get_mut() += 10;
    println!("After += 10: {}", cell.get());

    // Consume the cell — this is the ONLY way to extract the value
    let value = cell.consume();
    println!("Consumed value: {value}");
    assert_eq!(value, 52);

    println!("\n✅ QuantumCell consumed correctly. No information lost!");
    println!("\n(Try uncommenting the line below to see what happens when you DON'T consume)");
    println!("// let _leaked = QuantumCell::new(99); // This would panic!");
}
