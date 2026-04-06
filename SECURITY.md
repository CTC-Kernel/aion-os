# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in Rewind, please report it responsibly:

1. **Do NOT open a public issue**
2. Email: [Create a private security advisory](https://github.com/CTC-Kernel/aion-os/security/advisories/new)
3. Include: description, steps to reproduce, potential impact

We will acknowledge receipt within 48 hours and provide a timeline for a fix.

## Security Considerations

Rewind is a reversible computing SDK. Key security aspects:

- **`unsafe` code** is minimized and confined to `QuantumCell` internals (`ManuallyDrop`)
- **`mem::forget`** bypasses `QuantumCell`'s Drop check — mitigated by `#[reversible]` macro rejection
- **Dependencies** are audited via `cargo audit` in CI
- **Supply chain** uses minimal dependencies (syn, quote, proptest, criterion)
