//! Ancilla mirror stack — LIFO storage for intermediate computation states.
//!
//! During forward execution, ancilla bits are pushed onto the stack.
//! During backward execution (uncomputation), they are popped and verified,
//! ensuring zero information leakage.
//!
//! # Examples
//!
//! ```
//! use rewind_gc::stack::AncillaStack;
//! use rewind_core::bitplane::BitPlane;
//!
//! let mut stack = AncillaStack::new();
//! let state = BitPlane::from_words(vec![0xDEAD]);
//!
//! stack.push(state.clone());
//! assert_eq!(stack.len(), 1);
//!
//! let restored = stack.pop().unwrap();
//! assert_eq!(restored, state);
//! assert!(stack.is_empty());
//! ```

use rewind_core::bitplane::BitPlane;

/// LIFO stack storing ancilla BitPlanes during forward execution.
///
/// Each forward operation pushes its ancilla data. Each backward operation
/// pops and uses it to restore the previous state. After a complete
/// forward-backward cycle, the stack must be empty (garbage-free).
#[derive(Debug)]
pub struct AncillaStack {
    entries: Vec<BitPlane>,
}

impl AncillaStack {
    /// Creates a new empty ancilla stack.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Creates a stack with a pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
        }
    }

    /// Pushes an ancilla state onto the stack (during forward execution).
    pub fn push(&mut self, state: BitPlane) {
        self.entries.push(state);
    }

    /// Pops an ancilla state from the stack (during backward execution).
    ///
    /// Returns `None` if the stack is empty.
    pub fn pop(&mut self) -> Option<BitPlane> {
        self.entries.pop()
    }

    /// Returns the number of ancilla states currently stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the stack is empty (garbage-free state).
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the total memory used by the stack in bytes (approximate).
    pub fn memory_usage(&self) -> usize {
        self.entries
            .iter()
            .map(|bp| bp.len() * std::mem::size_of::<u64>())
            .sum()
    }

    /// Verifies the stack is empty (garbage-free).
    ///
    /// Returns `Ok(())` if empty, or the number of remaining entries.
    pub fn verify_garbage_free(&self) -> Result<(), usize> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self.len())
        }
    }

    /// Clears all entries, discarding ancilla data.
    ///
    /// **Warning:** This destroys information! Only use for cleanup
    /// after verification, or when explicitly discarding a computation.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for AncillaStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_lifo_order() {
        let mut stack = AncillaStack::new();
        stack.push(BitPlane::from_words(vec![1]));
        stack.push(BitPlane::from_words(vec![2]));
        stack.push(BitPlane::from_words(vec![3]));

        assert_eq!(stack.pop().unwrap().words()[0], 3);
        assert_eq!(stack.pop().unwrap().words()[0], 2);
        assert_eq!(stack.pop().unwrap().words()[0], 1);
        assert!(stack.pop().is_none());
    }

    #[test]
    fn verify_garbage_free_empty() {
        let stack = AncillaStack::new();
        assert!(stack.verify_garbage_free().is_ok());
    }

    #[test]
    fn verify_garbage_free_non_empty() {
        let mut stack = AncillaStack::new();
        stack.push(BitPlane::from_words(vec![42]));
        assert_eq!(stack.verify_garbage_free(), Err(1));
    }

    #[test]
    fn memory_usage_calculation() {
        let mut stack = AncillaStack::new();
        // Each u64 word = 8 bytes
        stack.push(BitPlane::from_words(vec![0; 10])); // 10 words = 80 bytes
        stack.push(BitPlane::from_words(vec![0; 5])); // 5 words = 40 bytes
        assert_eq!(stack.memory_usage(), 120);
    }

    #[test]
    fn with_capacity_works() {
        let stack = AncillaStack::with_capacity(100);
        assert!(stack.is_empty());
    }

    #[test]
    fn clear_empties_stack() {
        let mut stack = AncillaStack::new();
        stack.push(BitPlane::from_words(vec![1]));
        stack.push(BitPlane::from_words(vec![2]));
        stack.clear();
        assert!(stack.is_empty());
        assert!(stack.verify_garbage_free().is_ok());
    }
}
