//! Reversible Physics Simulation — time-symmetric particle simulation.
//!
//! Simulates particles bouncing in a 1D space using only reversible operations.
//! The simulation can run forward AND backward in time perfectly.
//!
//! This demonstrates that Rewind can implement real physical simulations
//! that respect T-symmetry (time-reversal symmetry) of fundamental physics.
//!
//! Run with: `cargo run -p rewind --example physics_sim`

use rewind::prelude::*;

fn main() {
    println!("=== Rewind: Simulation Physique Reversible ===\n");
    println!("Simulation de 2 particules dans un espace 1D.");
    println!("Les operations sont purement reversibles — T-symetrie respectee.\n");

    // Registers:
    // R0 = position particule A (encoded as bits)
    // R1 = position particule B
    // R2 = velocity/momentum A (encoded as bits)
    // R3 = velocity/momentum B
    // R4 = interaction scratch (ancilla, starts at 0)
    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from(0x10u64), // Particle A at position 0x10
        BitPlane::from(0x30u64), // Particle B at position 0x30
        BitPlane::from(0x01u64), // Velocity A = 0x01
        BitPlane::from(0x02u64), // Velocity B = 0x02
        BitPlane::from(0x00u64), // Interaction scratch
    ]);

    println!("Etat initial:");
    print_state(&rt);

    // One "timestep" = a reversible circuit:
    // 1. Update positions (pos ^= vel — simplified reversible dynamics)
    // 2. Compute interaction (scratch = A AND B interaction term)
    // 3. Update velocities based on interaction
    let timestep = ProgramBuilder::new()
        // Update positions: pos ^= velocity
        .cnot(2, 0) // posA ^= velA
        .cnot(3, 1) // posB ^= velB
        // Compute interaction force
        .toffoli(0, 1, 4) // scratch ^= posA AND posB
        // Update velocities based on interaction
        .cnot(4, 2) // velA ^= interaction
        .cnot(4, 3) // velB ^= interaction
        // Clean up interaction scratch (uncompute)
        .toffoli(0, 1, 4) // undo: scratch ^= posA AND posB
        .build_ops();

    // Run 5 timesteps forward
    println!("\n>>> FORWARD (5 pas de temps)");
    for t in 0..5 {
        rt.execute_all_tracked(&timestep);
        print!("  t={}: ", t + 1);
        print_state(&rt);
    }

    let mid_state = rt.snapshot();
    println!("\nEtat au milieu: {:X?}", mid_state);
    println!("Operations tracees: {}", rt.history_len());

    // Run 5 timesteps backward — T-symmetry!
    println!("\n<<< BACKWARD (5 pas de temps — rembobinage)");
    for t in (0..5).rev() {
        rt.rewind(timestep.len()).unwrap();
        print!("  t={}: ", t);
        print_state(&rt);
    }

    // Verify perfect restoration
    assert_eq!(rt.register(0).words()[0], 0x10);
    assert_eq!(rt.register(1).words()[0], 0x30);
    assert_eq!(rt.register(2).words()[0], 0x01);
    assert_eq!(rt.register(3).words()[0], 0x02);
    assert_eq!(rt.register(4).words()[0], 0x00); // Scratch propre
    assert!(rt.is_garbage_free());

    println!("\n✅ Simulation parfaitement reversible.");
    println!("   T-symetrie respectee: forward(5) + backward(5) = etat initial.");
    println!("   Ancilla scratch = 0 (garbage-free).");
    println!("   {}", rt.stats());
}

fn print_state(rt: &ReversibleRuntime) {
    println!(
        "posA=0x{:02X} posB=0x{:02X} velA=0x{:02X} velB=0x{:02X} scratch=0x{:02X}",
        rt.register(0).words()[0],
        rt.register(1).words()[0],
        rt.register(2).words()[0],
        rt.register(3).words()[0],
        rt.register(4).words()[0],
    );
}
