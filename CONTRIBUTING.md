# Contributing to Rewind

Thank you for your interest in contributing to Rewind!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/aion-os.git`
3. Create a branch: `git checkout -b my-feature`
4. Make your changes
5. Run tests: `cargo test`
6. Run clippy: `cargo clippy`
7. Submit a pull request

## Development

```bash
# Build all crates
cargo build

# Run all tests
cargo test

# Run clippy
cargo clippy

# Generate docs
cargo doc --no-deps --open
```

## Guidelines

- Every `ReversibleOp` implementation **must** have a proptest verifying `undo(execute(x)) == x`
- Follow Rust API guidelines and standard formatting (`cargo fmt`)
- All public items must have doc comments with examples
- No `unsafe` in user-facing code without justification

## License

By contributing, you agree that your contributions will be licensed under both MIT and Apache-2.0.
