//! BitPlane — Structure of Arrays (SoA) bit storage for SIMD-friendly operations.
//!
//! Stores bits in contiguous `Vec<u64>` arrays, enabling efficient SIMD
//! parallelization of Toffoli, CNOT, and Pauli-X gate operations.
