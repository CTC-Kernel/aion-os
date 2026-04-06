//! Rewind CLI — execute reversible programs from the terminal.
//!
//! Usage:
//!   rewind run <file.rev>              Execute a program forward then backward
//!   rewind analyze <file.rev>          Analyze program without executing
//!   rewind example                     Run a built-in demo
//!
//! Program format (.rev):
//!   NOT <register>
//!   CNOT <control> <target>
//!   TOFF <c1> <c2> <target>
//!   # Comments start with #

use rewind_core::bitplane::BitPlane;
use rewind_core::engine::Op;
use rewind_core::program::{self, ProgramInfo};
use rewind_core::runtime::ReversibleRuntime;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("run") => {
            if let Some(path) = args.get(2) {
                run_file(path);
            } else {
                eprintln!("Usage: rewind run <file.rev>");
                std::process::exit(1);
            }
        }
        Some("analyze") => {
            if let Some(path) = args.get(2) {
                analyze_file(path);
            } else {
                eprintln!("Usage: rewind analyze <file.rev>");
                std::process::exit(1);
            }
        }
        Some("example") => run_example(),
        Some("--help" | "-h") | None => print_help(),
        Some(cmd) => {
            eprintln!("Unknown command: {cmd}");
            print_help();
            std::process::exit(1);
        }
    }
}

fn run_file(path: &str) {
    let content = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading {path}: {e}");
        std::process::exit(1);
    });

    let ops = program::deserialize_text(&content).unwrap_or_else(|(line, text)| {
        eprintln!("Parse error at line {line}: {text}");
        std::process::exit(1);
    });

    let info = ProgramInfo::analyze(&ops);
    let num_regs = info.min_registers();

    println!("=== Rewind: {path} ===");
    println!("{info}\n");

    let regs: Vec<BitPlane> = (0..num_regs)
        .map(|_| BitPlane::from_words(vec![0]))
        .collect();
    let mut rt = ReversibleRuntime::new(regs);

    // Forward with tracing
    println!(">>> FORWARD");
    rt.execute_traced(&ops, |step, op, regs| {
        let reg_str: Vec<String> = regs.iter().map(|r| format!("{r:X}")).collect();
        println!("  [{step:>3}] {op:<20} | {}", reg_str.join(" "));
    });

    println!("\n<<< BACKWARD");
    let n = rt.history_len();
    rt.rewind_traced(n, |step, op, regs| {
        let reg_str: Vec<String> = regs.iter().map(|r| format!("{r:X}")).collect();
        println!("  [{step:>3}] undo {op:<15} | {}", reg_str.join(" "));
    })
    .unwrap();

    println!("\nGarbage-free: {}", rt.is_garbage_free());
    println!("{}", rt.stats());
}

fn analyze_file(path: &str) {
    let content = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading {path}: {e}");
        std::process::exit(1);
    });

    let ops = program::deserialize_text(&content).unwrap_or_else(|(line, text)| {
        eprintln!("Parse error at line {line}: {text}");
        std::process::exit(1);
    });

    let info = ProgramInfo::analyze(&ops);
    println!("=== Rewind: Program Analysis ===");
    println!("File: {path}");
    println!("{info}");
    println!("Registers needed: {}", info.min_registers());
    println!("\nProgram listing:");
    for (i, op) in ops.iter().enumerate() {
        println!("  {i:>3}: {op}");
    }
}

fn run_example() {
    println!("=== Rewind: Built-in Demo ===\n");

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
        Op::Not(2),
        Op::Cnot {
            control: 2,
            target: 0,
        },
    ];

    let info = ProgramInfo::analyze(&ops);
    println!("{info}\n");

    let mut rt = ReversibleRuntime::new(vec![
        BitPlane::from_words(vec![0xAA]),
        BitPlane::from_words(vec![0x55]),
        BitPlane::from_words(vec![0x00]),
    ]);

    let original = rt.snapshot();
    println!("Initial: {:X?}", original);

    rt.execute_traced(&ops, |step, op, regs| {
        println!("  [{step}] {op} → {:X?}", regs);
    });

    println!("\nRewinding...");
    let n = rt.history_len();
    rt.rewind_traced(n, |step, op, regs| {
        println!("  [{step}] undo {op} → {:X?}", regs);
    })
    .unwrap();

    assert_eq!(rt.snapshot(), original);
    println!("\nRestored: {:X?}", rt.snapshot());
    println!("Garbage-free: {}", rt.is_garbage_free());
    println!("\nInformation is Sacred.");
}

fn print_help() {
    println!(
        r#"Rewind — the first natively reversible computing SDK for Rust.

USAGE:
    rewind <command> [args]

COMMANDS:
    run <file.rev>       Execute a reversible program (forward + backward)
    analyze <file.rev>   Analyze a program without executing
    example              Run a built-in demonstration
    --help, -h           Show this help

PROGRAM FORMAT (.rev):
    NOT <register>           Pauli-X (NOT) gate
    CNOT <control> <target>  Controlled-NOT gate
    TOFF <c1> <c2> <target>  Toffoli (CCNOT) gate
    # Comment                Lines starting with # are ignored

EXAMPLE:
    echo "NOT 0\nCNOT 0 1\nTOFF 0 1 2" > demo.rev
    rewind run demo.rev

Information is Sacred."#
    );
}
