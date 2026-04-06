//! Reversible XOR Cipher — encryption that can be perfectly undone.
//!
//! Demonstrates a practical use case: encrypting data with XOR (CNOT gates),
//! then decrypting by rewinding the computation. Since XOR is self-inverse,
//! the same circuit encrypts AND decrypts.
//!
//! Run with: `cargo run -p rewind --example xor_cipher`

use rewind::prelude::*;

fn main() {
    println!("=== Rewind: Chiffrement XOR Reversible ===\n");

    // Message secret (en u64 pour simplifier)
    let message: u64 = 0x48454C4C4F; // "HELLO" en hex
    let key: u64 = 0xDEADBEEFCAFE;

    println!("Message original : 0x{message:012X}");
    println!("Cle de chiffrement : 0x{key:012X}");

    // Registres : [message, cle]
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![message]),
        BitPlane::from_words(vec![key]),
    ]);

    // Chiffrement = CNOT(cle -> message) : message ^= cle
    println!("\n--- Chiffrement (CNOT : message ^= cle) ---");
    rt.execute_tracked(Op::Cnot {
        control: 1,
        target: 0,
    });

    let encrypted = rt.register(0).words()[0];
    println!("Message chiffre : 0x{encrypted:012X}");
    assert_ne!(encrypted, message);

    // Dechiffrement = rewind !
    println!("\n--- Dechiffrement (rewind) ---");
    rt.rewind_all().unwrap();

    let decrypted = rt.register(0).words()[0];
    println!("Message dechiffre : 0x{decrypted:012X}");
    assert_eq!(decrypted, message);
    assert!(rt.is_garbage_free());

    // Deuxieme approche : chiffrement multi-couche
    println!("\n--- Chiffrement multi-couche (3 passes) ---");
    let key2: u64 = 0x1234567890AB;
    let key3: u64 = 0xFEDCBA987654;

    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![message]),
        BitPlane::from_words(vec![key]),
        BitPlane::from_words(vec![key2]),
        BitPlane::from_words(vec![key3]),
    ]);

    // 3 couches de chiffrement
    rt.execute_tracked(Op::Cnot {
        control: 1,
        target: 0,
    }); // couche 1
    rt.execute_tracked(Op::Cnot {
        control: 2,
        target: 0,
    }); // couche 2
    rt.execute_tracked(Op::Cnot {
        control: 3,
        target: 0,
    }); // couche 3

    let triple_encrypted = rt.register(0).words()[0];
    println!("Apres 3 couches : 0x{triple_encrypted:012X}");

    // Dechiffrement complet en un seul rewind
    rt.rewind_all().unwrap();
    assert_eq!(rt.register(0).words()[0], message);
    println!(
        "Apres rewind complet : 0x{:012X}",
        rt.register(0).words()[0]
    );
    println!("Garbage-free : {}", rt.is_garbage_free());

    println!("\n=== Le chiffrement reversible preserve toute l'information. ===");
}
