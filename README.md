# Rewind

[![CI](https://github.com/CTC-Kernel/aion-os/actions/workflows/ci.yml/badge.svg)](https://github.com/CTC-Kernel/aion-os/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

**The first natively reversible computing SDK for Rust.**

*Information is Sacred — it must never be destroyed.*

---

Rewind guarantees that every operation is structurally invertible, verified at compile-time. Unlike record-replay debuggers (rr, UndoDB) that record traces with overhead, Rewind makes computation itself reversible — enabling bidirectional execution, zero-cost rollbacks, and native time-travel debugging.

## Quick Start

```bash
git clone https://github.com/CTC-Kernel/aion-os.git
cd aion-os
cargo run -p rewind --example hello_rewind
```

### QuantumCell — Linear types in Rust

```rust
use rewind_core::QuantumCell;

// QuantumCell MUST be consumed — dropping it panics
let mut cell = QuantumCell::new(42u64);
*cell.get_mut() += 10;
let value = cell.consume(); // Only way to extract
assert_eq!(value, 52);
// Dropping without consume() → panic!("information lost")
```

### Reversible Gates — Toffoli, CNOT, Pauli-X

```rust
use rewind_core::{BitPlane, ReversibleOp, assert_reversible};
use rewind_gates::scalar::{Toffoli, ToffoliState};

let state = ToffoliState {
    control1: BitPlane::from_words(vec![0xFF]),
    control2: BitPlane::from_words(vec![0x0F]),
    target: BitPlane::from_words(vec![0x00]),
};

// Execute forward
let (output, ancilla) = Toffoli.execute(state.clone());
// Undo — perfectly restores original state
let restored = Toffoli.undo(output, ancilla);
assert_eq!(state, restored); // Information preserved!
```

### Forward/Backward Execution

```rust
use rewind_core::bitplane::BitPlane;
use rewind_core::engine::{ExecutionEngine, Op, ReversibleProgram};

let regs = vec![
    BitPlane::from_words(vec![0xAA]),
    BitPlane::from_words(vec![0xBB]),
    BitPlane::from_words(vec![0xCC]),
];
let mut engine = ExecutionEngine::new(regs);
let original = engine.registers().to_vec();

let program = ReversibleProgram::new(vec![
    Op::Not(0),
    Op::Cnot { control: 0, target: 1 },
    Op::Toffoli { c1: 0, c2: 1, target: 2 },
]);

program.forward(&mut engine);  // Execute
program.backward(&mut engine); // Rewind — back to original
assert_eq!(engine.registers().to_vec(), original);
```

### Compile-Time Reversibility Verification

```rust
use rewind_dsl::reversible;

#[reversible]
fn safe_computation(x: &mut u64, y: &mut u64) {
    *x += 42;      // OK — reversible (inverse: -= 42)
    *y ^= *x;      // OK — XOR is self-inverse
    // *x = 0;      // COMPILE ERROR: destructive assignment
    // println!();   // COMPILE ERROR: I/O is irreversible
}
```

## Examples

```bash
cargo run -p rewind --example hello_rewind    # Forward/backward execution
cargo run -p rewind --example step_backward   # Checkpoint/restore debugging
cargo run -p rewind --example quantum_cell    # Linear type demonstration
```

## Crates

| Crate | Description |
|-------|-------------|
| [`rewind`](rewind/) | Facade crate — re-exports everything |
| [`rewind-core`](rewind-core/) | QuantumCell, ReversibleOp, BitPlane, ExecutionEngine |
| [`rewind-gates`](rewind-gates/) | Toffoli, CNOT, Pauli-X gates (scalar + SIMD flags) |
| [`rewind-gc`](rewind-gc/) | Garbage-Free Collector (ancilla mirror stack + memory budget) |
| [`rewind-dsl`](rewind-dsl/) | `#[reversible]` proc-macro — compile-time validation |
| [`rewind-bennett`](rewind-bennett/) | Bennett's algorithm (computation graph + pebbling strategy) |
| [`rewind-playground`](rewind-playground/) | WASM interactive playground (planned) |

## Features

- `simd` — `std::simd` gate optimizations (nightly Rust)
- `stable-simd` — `pulp`-based SIMD (stable Rust)
- `bennett` — Bennett's automatic reversible compilation

## Performance

Scalar Toffoli gate throughput on Apple Silicon:

| Words | Bits | Time | Throughput |
|-------|------|------|-----------|
| 1 | 64 | 74 ns | ~860M gates/sec |
| 1024 | 65,536 | 700 ns | ~93B bit-ops/sec |

Run benchmarks: `cargo bench -p rewind-gates`

## Architecture

```
rewind (facade) ──► rewind-core (QuantumCell, ReversibleOp, Engine)
                ──► rewind-gates (Toffoli, CNOT, Pauli-X) ──► rewind-core
                ──► rewind-dsl (#[reversible] macro)
rewind-gc ──────────► rewind-core
rewind-bennett ─────► rewind-core
```

Key decisions:
- **Arena allocator** with typed indices (not `Pin<Box<T>>`)
- **Match dispatch** for VM opcodes (quasi-optimal on modern CPUs)
- **SoA BitPlane** layout for SIMD-friendly gate operations
- **Linear types** via `Drop` + panic + `ManuallyDrop`

See [project-context.md](project-context.md) for full architectural details.

## Theoretical Foundation

Rewind is built on proven physics and computer science:

- **Landauer's Principle** (1961) — Erasing a bit dissipates kT·ln(2) energy
- **Toffoli Gate** (1980) — Universal reversible gate (CCNOT)
- **Bennett's Algorithm** (1973) — Any computation can be made reversible
- **Girard's Linear Logic** (1987) — Each resource used exactly once

Rust's ownership system (affine types) is a native implementation of linear logic — making it the ideal language for reversible computing.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Every `ReversibleOp` implementation **must** have a proptest verifying `undo(execute(x)) == x`.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

*Built with the conviction that information should never be destroyed.*
