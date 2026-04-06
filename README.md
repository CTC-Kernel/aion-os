# Rewind

**The first natively reversible computing SDK for Rust.**

*Information is Sacred — it must never be destroyed.*

---

Rewind guarantees that every operation is structurally invertible, verified at compile-time. Unlike record-replay debuggers (rr, UndoDB) that record traces with overhead, Rewind makes computation itself reversible — enabling bidirectional execution, zero-cost rollbacks, and native time-travel debugging.

## Quick Start

```toml
[dependencies]
rewind = "0.1"
```

```rust
use rewind::prelude::*;

#[reversible]
fn example(x: &mut u64, y: &mut u64) {
    *x += 42;
    *y ^= *x;
}
// Automatically generates inverse: *y ^= *x; *x -= 42;
```

> Full quickstart guide coming with v0.1 release.

## Crates

| Crate | Description |
|-------|-------------|
| `rewind` | Facade crate — re-exports everything |
| `rewind-core` | QuantumCell, ReversibleOp trait, ExecutionBackend |
| `rewind-gates` | Toffoli, CNOT, Pauli-X gates (scalar + SIMD) |
| `rewind-gc` | Garbage-Free Collector (ancilla mirror stack) |
| `rewind-dsl` | `#[reversible]` proc-macro |
| `rewind-bennett` | Bennett's algorithm (automatic reversible compilation) |
| `rewind-playground` | WASM interactive playground |

## Features

- `simd` — `std::simd` gate optimizations (nightly Rust)
- `stable-simd` — `pulp`-based SIMD (stable Rust)
- `bennett` — Bennett's automatic reversible compilation

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
