# Changelog

All notable changes to Rewind are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-06

### Added

#### Core (`rewind-core`)
- `QuantumCell<T>` — linear type that must be consumed exactly once (panics on drop if not consumed)
- `ReversibleOp` trait with `execute()`/`undo()` and `assert_reversible()`/`check_reversible()` helpers
- `BitPlane` — Structure of Arrays bit storage for SIMD-friendly gate operations
- `ExecutionEngine` — register file with forward/backward execution and checkpoint/restore
- `ReversibleRuntime` — unified API combining engine + operation tracking + rewind
- `ReversibleProgram` with `Op` enum (Not, Cnot, Toffoli) and `Display` implementation
- Typed newtype indices: `RegisterId`, `AncillaId`, `CheckpointId`
- `RewindError` enum with descriptive messages
- Execution tracing: `execute_traced()` and `rewind_traced()` with step-by-step callbacks

#### Gates (`rewind-gates`)
- Pauli-X (NOT), CNOT, Toffoli (CCNOT) — scalar implementations
- All gates proven reversible via proptest (`undo(execute(x)) == x`)
- Pre-built circuits: `half_adder`, `full_adder`, `swap`, `compose`, `reverse`
- Criterion benchmark suite (93B bit-ops/sec Toffoli on 1024 words)
- SIMD feature flags (`simd`, `stable-simd`) — placeholder for v0.2

#### Garbage-Free Collector (`rewind-gc`)
- `AncillaStack` — LIFO mirror stack for intermediate computation states
- `MemoryBudget` — configurable memory limit with enforcement
- `GarbageFreeCollector` — combined stack + budget for zero-leak computation

#### DSL (`rewind-dsl`)
- `#[reversible]` proc-macro attribute with compile-time validation
- Rejects: destructive assignment (`=`), `mem::forget`, `mem::drop`, I/O macros
- Allows: compound assignments (`+=`, `-=`, `^=`), `let` bindings, function calls
- Auto-generates `_reverse` companion function (inverts `+=`/`-=`, preserves `^=`, reverses order)

#### Bennett (`rewind-bennett`)
- `ComputationGraph` — DAG with topological sort for computation steps
- `PebblingStrategy` — recursive halving checkpoint placement with configurable epsilon
- `BennettExecutor` — plans forward/backward execution with time/space overhead estimates
- Guard against epsilon < 0.1 (hidden factor divergence warning)

#### Facade (`rewind`)
- Re-exports all public types from core, gates, and DSL
- `prelude` module for convenient `use rewind::prelude::*;`

#### Examples
- `hello_rewind` — forward/backward with NOT, CNOT, Toffoli
- `step_backward` — checkpoint and restore debugging
- `quantum_cell` — linear type lifecycle
- `runtime_demo` — unified ReversibleRuntime API
- `reversible_adder` — half-adder, swap, circuit composition
- `time_travel_debug` — step-by-step traced execution and rewind
- `xor_cipher` — reversible XOR encryption/decryption

#### Infrastructure
- Cargo workspace with 7 crates
- GitHub Actions CI (test, clippy, docs, fmt, security audit)
- Dual license Apache 2.0 / MIT
- 150 tests (unit + proptest + integration + doc-tests)
- Criterion benchmarks
- README complet en francais avec diagrammes Mermaid
