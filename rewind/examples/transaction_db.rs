//! Reversible Transaction Database — rollback without WAL.
//!
//! Demonstrates a simple key-value store where every operation is
//! reversible. Instead of a Write-Ahead Log, the computation itself
//! IS the log — rollback is just rewind.
//!
//! Run with: `cargo run -p rewind --example transaction_db`

use rewind::prelude::*;

fn main() {
    println!("=== Rewind: Base de Donnees Transactionnelle ===\n");
    println!("Chaque operation est reversible — pas besoin de WAL.\n");

    // Registers represent "database fields":
    // R0 = account balance A
    // R1 = account balance B
    // R2 = transaction counter
    // R3 = audit log (XOR accumulator)
    let mut db = ReversibleRuntime::new(vec![
        BitPlane::from(1000u64), // Account A: 1000
        BitPlane::from(500u64),  // Account B: 500
        BitPlane::from(0u64),    // Transaction counter
        BitPlane::from(0u64),    // Audit XOR accumulator
    ]);

    println!("Etat initial:");
    print_db(&db);

    // Transaction 1: Transfer A -> B (using XOR as simplified transfer)
    println!("\n--- Transaction 1: Transfert A -> B ---");
    let tx1 = ProgramBuilder::new()
        .cnot(0, 1) // B ^= A (simplified transfer)
        .not(2) // Increment counter (simplified)
        .cnot(0, 3) // Audit: log ^= A
        .build_ops();
    db.execute_all_tracked(&tx1);
    print_db(&db);

    // Checkpoint after TX1
    let _ckpt1 = db.checkpoint();
    println!("  Checkpoint sauvegarde apres TX1");

    // Transaction 2: Another operation
    println!("\n--- Transaction 2: Operation sur B ---");
    let tx2 = ProgramBuilder::new()
        .cnot(1, 0) // A ^= B
        .not(2) // Counter
        .cnot(1, 3) // Audit
        .build_ops();
    db.execute_all_tracked(&tx2);
    print_db(&db);

    // Transaction 3: Bad transaction (will be rolled back)
    println!("\n--- Transaction 3: Transaction defectueuse ---");
    let tx3 = ProgramBuilder::new()
        .not(0) // Corrupt A!
        .not(1) // Corrupt B!
        .not(2) // Counter
        .build_ops();
    db.execute_all_tracked(&tx3);
    print_db(&db);
    println!("  ERREUR detectee! Rollback...");

    // Rollback TX3 by rewinding 3 operations
    db.rewind(tx3.len()).unwrap();
    println!("\n--- Apres rollback de TX3 ---");
    print_db(&db);

    // Rollback TX2 by rewinding
    println!("\n--- Rollback TX2 ---");
    db.rewind(tx2.len()).unwrap();
    print_db(&db);

    // Rollback TX1 to get back to initial state
    println!("\n--- Rollback TX1 → etat initial ---");
    db.rewind(tx1.len()).unwrap();
    print_db(&db);

    assert_eq!(db.register(0).words()[0], 1000);
    assert_eq!(db.register(1).words()[0], 500);
    assert_eq!(db.register(2).words()[0], 0);
    assert!(db.is_garbage_free());

    println!("\n✅ Base de donnees transactionnelle reversible.");
    println!("   Pas de WAL, pas de journal — le calcul EST le journal.");
    println!("   Rollback par transaction: rewind(tx.len()) — instantane.");
    println!("   {}", db.stats());
}

fn print_db(db: &ReversibleRuntime) {
    println!(
        "  Account A: {} | Account B: {} | TxCount: {} | Audit: 0x{:X}",
        db.register(0).words()[0],
        db.register(1).words()[0],
        db.register(2).words()[0],
        db.register(3).words()[0],
    );
}
