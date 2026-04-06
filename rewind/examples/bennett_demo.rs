//! Bennett's Algorithm Demo — automatic reversible compilation.
//!
//! Shows how Bennett's algorithm transforms a sequence of operations
//! into a reversible computation with uncomputation of intermediates.
//!
//! Run with: `cargo run -p rewind --example bennett_demo`

use rewind::prelude::*;
use rewind_bennett::{BennettConfig, BennettExecutor, ComputationGraph};

fn main() {
    println!("=== Rewind: Algorithme de Bennett ===\n");

    // A computation with 5 steps
    let graph = ComputationGraph::linear_chain(&[
        "load_data",
        "transform_1",
        "transform_2",
        "combine",
        "output",
    ]);

    let executor = BennettExecutor::new(BennettConfig::default());
    let plan = executor.plan(&graph);

    println!("Computation: 5 etapes lineaires");
    println!(
        "Bennett plan: {} forward, {} backward, {} checkpoints",
        plan.forward_steps.len(),
        plan.backward_steps.len(),
        plan.strategy.checkpoint_count()
    );
    println!("Time overhead: {:.2}x", executor.estimated_time_overhead(5));
    println!(
        "Space overhead: {:.2}x",
        executor.estimated_space_overhead(5)
    );

    // Execute with real operations
    println!("\n--- Execution reelle avec Bennett ---");
    let ops = vec![
        Op::Not(0),
        Op::Cnot {
            control: 0,
            target: 1,
        },
        Op::Toffoli {
            c1: 0,
            c2: 1,
            target: 2,
        },
        Op::Cnot {
            control: 2,
            target: 3,
        },
        Op::Not(3),
    ];

    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0xAA]),
        BitPlane::from_words(vec![0x00]),
        BitPlane::from_words(vec![0x00]),
        BitPlane::from_words(vec![0x00]),
    ]);

    println!("Initial: {:X?}", rt.snapshot());

    let result = executor.execute_with_ops(&ops, &mut rt);

    println!("Apres Bennett: {:X?}", rt.snapshot());
    println!(
        "Forward: {} ops, Backward: {} ops, Checkpoints: {}",
        result.forward_ops, result.backward_ops, result.checkpoints_used
    );
    println!("Stats: {}", rt.stats());

    // Compare different epsilon values
    println!("\n--- Comparaison des parametres epsilon ---");
    for eps in [0.1, 0.3, 0.5, 0.7, 0.9] {
        let config = BennettConfig::new(eps);
        let exec = BennettExecutor::new(config);
        let plan = exec.plan(&ComputationGraph::linear_chain(&[
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j",
        ]));
        println!(
            "  eps={:.1}: {} checkpoints, time overhead {:.2}x",
            eps,
            plan.strategy.checkpoint_count(),
            exec.estimated_time_overhead(10)
        );
    }

    println!("\n=== Bennett: tout calcul irreversible peut devenir reversible. ===");
}
