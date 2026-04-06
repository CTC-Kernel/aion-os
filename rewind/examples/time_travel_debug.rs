//! Time-Travel Debugging — step through execution forward and backward.
//!
//! This is Rewind's killer feature: see every state change as it happens,
//! then rewind step by step to find exactly where things went wrong.
//!
//! Run with: `cargo run -p rewind --example time_travel_debug`

use rewind::prelude::*;
use rewind_gates::circuits;

fn main() {
    println!("=== Rewind: Time-Travel Debugging ===\n");
    println!("Watch every state change, then rewind step by step.\n");

    // A complex circuit: swap + adder + more gates
    let program = circuits::compose(vec![
        vec![Op::Not(0)],
        circuits::swap(0, 1),
        vec![Op::Cnot {
            control: 1,
            target: 2,
        }],
        vec![Op::Toffoli {
            c1: 0,
            c2: 2,
            target: 3,
        }],
        vec![Op::Not(3)],
    ]);

    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0x0A]), // R0 = 10
        BitPlane::from_words(vec![0x14]), // R1 = 20
        BitPlane::from_words(vec![0x00]), // R2 = 0
        BitPlane::from_words(vec![0x00]), // R3 = 0
    ]);

    // === FORWARD with tracing ===
    println!(">>> FORWARD EXECUTION (step by step)");
    println!(
        "    Step {:>2}  {:>20}  R0={:02X} R1={:02X} R2={:02X} R3={:02X}",
        "-",
        "(initial)",
        rt.register(0).words()[0],
        rt.register(1).words()[0],
        rt.register(2).words()[0],
        rt.register(3).words()[0],
    );

    rt.execute_traced(&program, |step, op, regs| {
        let op_name = format_op(op);
        println!(
            "    Step {:>2}  {:>20}  R0={:02X} R1={:02X} R2={:02X} R3={:02X}",
            step, op_name, regs[0], regs[1], regs[2], regs[3],
        );
    });

    println!("\n    History: {} operations tracked\n", rt.history_len());

    // === BACKWARD with tracing ===
    println!("<<< BACKWARD EXECUTION (rewinding step by step)");
    let n = rt.history_len();
    rt.rewind_traced(n, |step, op, regs| {
        let op_name = format!("undo {}", format_op(op));
        println!(
            "    Step {:>2}  {:>20}  R0={:02X} R1={:02X} R2={:02X} R3={:02X}",
            step, op_name, regs[0], regs[1], regs[2], regs[3],
        );
    })
    .unwrap();

    println!();
    assert_eq!(rt.register(0).words()[0], 0x0A);
    assert_eq!(rt.register(1).words()[0], 0x14);
    assert_eq!(rt.register(2).words()[0], 0x00);
    assert_eq!(rt.register(3).words()[0], 0x00);
    assert!(rt.is_garbage_free());

    println!("✅ Every step traced. Every step reversed. Zero information lost.");
    println!("   This is what time-travel debugging looks like.");
}

fn format_op(op: &Op) -> String {
    match op {
        Op::Not(i) => format!("NOT R{i}"),
        Op::Cnot { control, target } => format!("CNOT R{control}→R{target}"),
        Op::Toffoli { c1, c2, target } => format!("TOFF R{c1},R{c2}→R{target}"),
    }
}
