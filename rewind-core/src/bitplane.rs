//! BitPlane — Structure of Arrays (SoA) bit storage for SIMD-friendly operations.
//!
//! Stores bits in contiguous `Vec<u64>` arrays, enabling efficient SIMD
//! parallelization of Toffoli, CNOT, and Pauli-X gate operations.
//!
//! # Examples
//!
//! ```
//! use rewind_core::bitplane::BitPlane;
//!
//! let a = BitPlane::from_words(vec![0xFF00FF00]);
//! let b = BitPlane::from_words(vec![0x0F0F0F0F]);
//! let result = a.xor(&b);
//! assert_eq!(result.words()[0], 0xFF00FF00 ^ 0x0F0F0F0F);
//! ```

/// A contiguous array of bits stored in `u64` words, optimized for bulk
/// bitwise operations (XOR, AND, NOT) that map directly to SIMD instructions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BitPlane {
    words: Vec<u64>,
}

impl BitPlane {
    /// Creates a new BitPlane with the given number of 64-bit words, all zeroed.
    pub fn zeroed(num_words: usize) -> Self {
        Self {
            words: vec![0u64; num_words],
        }
    }

    /// Creates a BitPlane from existing word data.
    pub fn from_words(words: Vec<u64>) -> Self {
        Self { words }
    }

    /// Returns a reference to the underlying word array.
    pub fn words(&self) -> &[u64] {
        &self.words
    }

    /// Returns a mutable reference to the underlying word array.
    pub fn words_mut(&mut self) -> &mut [u64] {
        &mut self.words
    }

    /// Returns the number of 64-bit words in this plane.
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Returns true if this plane has no words.
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Bitwise XOR with another BitPlane. Both must have the same length.
    ///
    /// This is the fundamental operation for CNOT and Toffoli gates.
    pub fn xor(&self, other: &BitPlane) -> BitPlane {
        assert_eq!(
            self.words.len(),
            other.words.len(),
            "BitPlane length mismatch"
        );
        let words = self
            .words
            .iter()
            .zip(other.words.iter())
            .map(|(a, b)| a ^ b)
            .collect();
        BitPlane { words }
    }

    /// Bitwise AND with another BitPlane. Both must have the same length.
    ///
    /// Used in Toffoli gate: `target ^= (control1 & control2)`.
    pub fn and(&self, other: &BitPlane) -> BitPlane {
        assert_eq!(
            self.words.len(),
            other.words.len(),
            "BitPlane length mismatch"
        );
        let words = self
            .words
            .iter()
            .zip(other.words.iter())
            .map(|(a, b)| a & b)
            .collect();
        BitPlane { words }
    }

    /// Bitwise NOT (invert all bits). This is the Pauli-X gate.
    pub fn not(&self) -> BitPlane {
        let words = self.words.iter().map(|w| !w).collect();
        BitPlane { words }
    }

    /// In-place XOR: `self ^= other`.
    pub fn xor_assign(&mut self, other: &BitPlane) {
        assert_eq!(
            self.words.len(),
            other.words.len(),
            "BitPlane length mismatch"
        );
        for (a, b) in self.words.iter_mut().zip(other.words.iter()) {
            *a ^= b;
        }
    }

    /// Checks if all bits are zero (useful for garbage-free verification).
    pub fn is_zero(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    /// Returns the total number of bits (words * 64).
    pub fn bit_count(&self) -> usize {
        self.words.len() * 64
    }

    /// Counts the number of set (1) bits (population count).
    pub fn popcount(&self) -> u32 {
        self.words.iter().map(|w| w.count_ones()).sum()
    }
}

impl std::fmt::Display for BitPlane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.words.is_empty() {
            return write!(f, "[]");
        }
        if self.words.len() == 1 {
            return write!(f, "0x{:016X}", self.words[0]);
        }
        write!(
            f,
            "[{} words: 0x{:016X}..0x{:016X}]",
            self.words.len(),
            self.words[0],
            self.words[self.words.len() - 1]
        )
    }
}

// Convenient conversions

impl From<u64> for BitPlane {
    /// Creates a single-word BitPlane from a u64.
    fn from(value: u64) -> Self {
        Self::from_words(vec![value])
    }
}

impl From<Vec<u64>> for BitPlane {
    /// Creates a BitPlane from a vector of u64 words.
    fn from(words: Vec<u64>) -> Self {
        Self::from_words(words)
    }
}

impl From<&[u64]> for BitPlane {
    /// Creates a BitPlane from a slice of u64 words.
    fn from(words: &[u64]) -> Self {
        Self::from_words(words.to_vec())
    }
}

// Standard trait implementations

impl std::ops::BitXorAssign<&BitPlane> for BitPlane {
    /// In-place XOR: `self ^= other`. Alias for `xor_assign`.
    fn bitxor_assign(&mut self, rhs: &BitPlane) {
        self.xor_assign(rhs);
    }
}

impl std::ops::BitXor<&BitPlane> for &BitPlane {
    type Output = BitPlane;
    /// XOR: `self ^ other`. Alias for `xor`.
    fn bitxor(self, rhs: &BitPlane) -> BitPlane {
        self.xor(rhs)
    }
}

impl std::ops::BitAnd<&BitPlane> for &BitPlane {
    type Output = BitPlane;
    /// AND: `self & other`. Alias for `and`.
    fn bitand(self, rhs: &BitPlane) -> BitPlane {
        self.and(rhs)
    }
}

impl std::ops::Not for &BitPlane {
    type Output = BitPlane;
    /// NOT: `!self`. Alias for `not`.
    fn not(self) -> BitPlane {
        BitPlane::not(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeroed_creates_all_zeros() {
        let bp = BitPlane::zeroed(4);
        assert_eq!(bp.len(), 4);
        assert!(bp.is_zero());
    }

    #[test]
    fn xor_works() {
        let a = BitPlane::from_words(vec![0xFF, 0x00]);
        let b = BitPlane::from_words(vec![0x0F, 0xF0]);
        let c = a.xor(&b);
        assert_eq!(c.words(), &[0xFF ^ 0x0F, 0x00 ^ 0xF0]);
    }

    #[test]
    fn and_works() {
        let a = BitPlane::from_words(vec![0xFF, 0x0F]);
        let b = BitPlane::from_words(vec![0x0F, 0xFF]);
        let c = a.and(&b);
        assert_eq!(c.words(), &[0x0F, 0x0F]);
    }

    #[test]
    fn not_works() {
        let a = BitPlane::from_words(vec![0u64]);
        let b = a.not();
        assert_eq!(b.words(), &[u64::MAX]);
    }

    #[test]
    fn xor_is_self_inverse() {
        let a = BitPlane::from_words(vec![0xDEADBEEF, 0xCAFEBABE]);
        let b = BitPlane::from_words(vec![0x12345678, 0x9ABCDEF0]);
        let c = a.xor(&b);
        let restored = c.xor(&b);
        assert_eq!(a, restored);
    }

    #[test]
    fn xor_assign_works() {
        let mut a = BitPlane::from_words(vec![0xFF]);
        let b = BitPlane::from_words(vec![0x0F]);
        a.xor_assign(&b);
        assert_eq!(a.words(), &[0xFF ^ 0x0F]);
    }

    #[test]
    fn not_is_self_inverse() {
        let a = BitPlane::from_words(vec![0xDEADBEEF]);
        assert_eq!(a.not().not(), a);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn xor_self_inverse(a_val: u64, b_val: u64) {
            let a = BitPlane::from_words(vec![a_val]);
            let b = BitPlane::from_words(vec![b_val]);
            prop_assert_eq!(a.xor(&b).xor(&b), a);
        }

        #[test]
        fn not_self_inverse(val: u64) {
            let a = BitPlane::from_words(vec![val]);
            prop_assert_eq!(a.not().not(), a);
        }

        #[test]
        fn xor_commutative(a_val: u64, b_val: u64) {
            let a = BitPlane::from_words(vec![a_val]);
            let b = BitPlane::from_words(vec![b_val]);
            prop_assert_eq!(a.xor(&b), b.xor(&a));
        }
    }
}
