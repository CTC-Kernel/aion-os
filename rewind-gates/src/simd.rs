//! SIMD-optimized reversible gates using `std::simd` (nightly).
//!
//! Parallelizes Toffoli/CNOT/Pauli-X across 128-512 bits per instruction
//! depending on the target architecture (SSE, AVX2, AVX-512, NEON).
