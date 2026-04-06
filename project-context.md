# Project Context — Rewind (Aion-OS)

## Identity

**Rewind** is the first natively reversible computing SDK for Rust. Every operation is structurally invertible, verified at compile-time, enabling bidirectional execution and time-travel debugging.

**Principle:** "Information is Sacred — it must never be destroyed."

## Architecture

Cargo workspace with 7 crates:

| Crate | Purpose |
|-------|---------|
| `rewind-core` | QuantumCell, ReversibleOp trait, BitPlane (SoA), ExecutionEngine, ExecutionBackend |
| `rewind-gates` | Toffoli, CNOT, Pauli-X gates (scalar, SIMD behind feature flags) |
| `rewind-gc` | Garbage-Free Collector — AncillaStack, MemoryBudget |
| `rewind-dsl` | `#[reversible]` proc-macro — compile-time reversibility validation |
| `rewind-bennett` | Bennett's algorithm — ComputationGraph, PebblingStrategy, BennettExecutor |
| `rewind` | Facade crate — re-exports everything |
| `rewind-playground` | WASM playground (future) |

## Key Decisions (ADRs)

- **ADR-01**: Workspace multi-crate (not monolithic)
- **ADR-02**: Arena allocator with typed indices (not Pin<Box<T>>)
- **ADR-03**: Match dispatch for VM opcodes (not computed goto)
- **ADR-04**: Structure of Arrays (SoA) BitPlane layout for SIMD
- **ADR-05**: Linear types via Drop+panic+ManuallyDrop (not native — Rust has affine, not linear)
- **ADR-06**: Extensive feature flags (simd, stable-simd, bennett)

## Coding Conventions

- Crate names: `rewind-{module}` (kebab-case)
- All public items have rustdoc with examples
- Every `ReversibleOp` implementation MUST have a proptest verifying `undo(execute(x)) == x`
- No `unsafe` in user-facing code without justification
- `#[should_panic]` for expected panics (e.g., QuantumCell dropped without consume)
- Typed newtype indices: `RegisterId(u32)`, `AncillaId(u32)`, `CheckpointId(u32)`

## Testing

- `proptest` for property-based testing (reversibility verification)
- `criterion` for benchmarks (rewind-gates/benches/)
- All tests run with `cargo test`, benchmarks with `cargo bench -p rewind-gates`

## Known Limitations

- `mem::forget` can bypass QuantumCell's Drop — mitigated by `#[reversible]` macro rejection
- SIMD gates not yet implemented (feature flags exist, implementations are placeholders)
- Bennett executor plans checkpoints but doesn't yet execute full reversible programs automatically
- `#[reversible]` validates but doesn't generate inverse code yet

## License

Dual Apache 2.0 / MIT
