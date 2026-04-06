# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Aion-OS is a **Reversible Virtual Machine (RVM)** written in Rust. Every computation `f(x) = y` has an inverse `f⁻¹(y) = x`, preserving information and theoretically achieving zero dissipation (Landauer limit). The system is built on Toffoli gates and linear types.

## Build & Run Commands

```bash
cargo build          # Build the project
cargo test           # Run all 26 unit tests + doctests
cargo run            # Run the demo (all 5 subsystem demonstrations)
cargo test -- --test-output  # Run tests with stdout visible
cargo test cell::    # Run tests for a specific module (cell, gates, entropy, rvm, dsl, bennett)
```

## Architecture

The crate is structured as a layered stack, each layer building on the one below:

### Layer 1: Linear Types (`src/cell.rs`)
- **`QuantumCell<N>`** — a register of N bits with linear-type discipline. Cannot be cloned. Panics on drop if bits are non-zero (enforces no information destruction). The only valid disposal is `release()` after returning to |0⟩.
- **`ReversibleOp` trait** — every operation must produce an `Ancilla` (inversion proof) alongside its output, and implement `undo()`.
- All mutations are via `xor()` and `swap_bits()` — both self-inverse by construction.

### Layer 2: Reversible Gates (`src/gates.rs`)
- **PauliX** (NOT), **CNot**, **Toffoli** (CCNOT), **Fredkin** (CSWAP) — the universal gate set.
- Each gate has `apply()`/`unapply()` for in-place register operations and a `ReversibleOp` impl for value-level computation.
- **`Gate` enum** — type-erased gate for dynamic dispatch in circuits. All gates are self-inverse (involutions).
- `batch_toffoli_u64` / `batch_toffoli_u64_undo` — packed u64 batch operations for performance-critical paths.

### Layer 3: Entropy Manager (`src/entropy.rs`)
- **`MirrorStack`** — records every gate applied during forward computation. Uncomputation replays in LIFO order to return registers to |0⟩.
- **`EntropyManager<N>`** — manages named ancilla registers with their mirror stacks. Workflow: `alloc()` → `apply()` → `uncompute_and_release()`.

### Layer 4: Bennett's Algorithm (`src/bennett.rs`)
- **`Circuit`** — a sequence of gates with `inverse()` generation.
- **`BennettCircuit`** — three-phase transform (forward → copy → reverse) that computes any function garbage-free. The copy phase uses CNOTs to extract results before uncomputing intermediate state.

### Layer 5: RVM Execution Engine (`src/rvm.rs`)
- **`RVM<N>`** — bidirectional program counter over a gate sequence. Supports `step_forward()`, `step_backward()`, `run_roundtrip()`, checkpointing, and full execution trace.

### Layer 6: DSL (`src/dsl.rs`)
- **`aion_block!` macro** — compiles a declarative block of reversible operations into `(forward, backward)` gate vectors. Only accepts `not`, `cnot`, `toffoli`, `fredkin` — anything else is a compile error.
- **`CircuitBuilder`** — fluent API alternative to the macro for dynamic circuit construction.

## Key Invariants

- **Reversibility**: for any gate sequence applied forward, applying the same sequence in reverse restores the original state. Every test verifies this roundtrip property.
- **Linear ownership**: `QuantumCell` enforces that information is never silently destroyed. Debug builds panic on non-zero drop. `release()` asserts |0⟩.
- **Pinned memory**: `QuantumCell` uses `Pin<Box<[u8; N]>>` to prevent the runtime from relocating register data, preserving address stability for the entropy manager.

## Extending the System

When adding new gates: implement both `apply()`/`unapply()` on `QuantumCell` and add a variant to the `Gate` enum. All gates in the current basis are self-inverse; if you add a non-involutory gate, `unapply` must differ from `apply`.

When adding new operations to `aion_block!`: add a new match arm in the `aion_gate!` macro (`src/dsl.rs`).
