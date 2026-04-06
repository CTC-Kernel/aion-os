//! Garbage-Free Collector — manages ancilla storage with budget enforcement.
//!
//! Combines the ancilla mirror stack with memory budgeting to provide
//! a complete garbage-free computation manager.
//!
//! # Examples
//!
//! ```
//! use rewind_gc::collector::GarbageFreeCollector;
//! use rewind_gc::budget::MemoryBudget;
//! use rewind_core::bitplane::BitPlane;
//!
//! let mut gc = GarbageFreeCollector::new(MemoryBudget::default());
//!
//! // Forward: save intermediate state
//! gc.checkpoint_ancilla(BitPlane::from_words(vec![0xAA])).unwrap();
//!
//! // Backward: restore and verify
//! let restored = gc.uncompute().unwrap();
//! assert_eq!(restored.words()[0], 0xAA);
//!
//! // Verify garbage-free
//! assert!(gc.is_garbage_free());
//! ```

use crate::budget::MemoryBudget;
use crate::stack::AncillaStack;
use rewind_core::bitplane::BitPlane;
use rewind_core::error::RewindError;

/// Manages ancilla bit storage with memory budget enforcement.
///
/// During forward execution, call [`checkpoint_ancilla`](Self::checkpoint_ancilla)
/// to save intermediate states. During backward execution, call
/// [`uncompute`](Self::uncompute) to restore them in LIFO order.
/// After a complete cycle, [`is_garbage_free`](Self::is_garbage_free)
/// should return `true`.
#[derive(Debug)]
pub struct GarbageFreeCollector {
    stack: AncillaStack,
    budget: MemoryBudget,
}

impl GarbageFreeCollector {
    /// Creates a new collector with the given memory budget.
    pub fn new(budget: MemoryBudget) -> Self {
        Self {
            stack: AncillaStack::new(),
            budget,
        }
    }

    /// Creates a collector with unlimited memory budget.
    pub fn unlimited() -> Self {
        Self::new(MemoryBudget::unlimited())
    }

    /// Saves an ancilla state during forward execution.
    ///
    /// Returns `Err(RewindError::MemoryBudgetExceeded)` if the budget would be exceeded.
    pub fn checkpoint_ancilla(&mut self, state: BitPlane) -> Result<(), RewindError> {
        let new_usage = self.stack.memory_usage() + state.len() * std::mem::size_of::<u64>();
        self.budget.check(new_usage)?;
        self.stack.push(state);
        Ok(())
    }

    /// Restores the most recent ancilla state during backward execution.
    ///
    /// Returns `Err(RewindError::GarbageRemaining(0))` if the stack is empty
    /// (nothing to uncompute).
    pub fn uncompute(&mut self) -> Result<BitPlane, RewindError> {
        self.stack.pop().ok_or(RewindError::GarbageRemaining(0))
    }

    /// Returns `true` if the ancilla stack is empty (garbage-free state).
    pub fn is_garbage_free(&self) -> bool {
        self.stack.is_empty()
    }

    /// Verifies garbage-free state, returning an error with the count of remaining entries.
    pub fn verify(&self) -> Result<(), RewindError> {
        self.stack
            .verify_garbage_free()
            .map_err(RewindError::GarbageRemaining)
    }

    /// Returns the number of ancilla entries currently stored.
    pub fn ancilla_count(&self) -> usize {
        self.stack.len()
    }

    /// Returns the current memory usage in bytes.
    pub fn memory_usage(&self) -> usize {
        self.stack.memory_usage()
    }

    /// Returns the remaining memory budget in bytes.
    pub fn memory_remaining(&self) -> usize {
        self.budget.remaining(self.stack.memory_usage())
    }
}

impl Default for GarbageFreeCollector {
    fn default() -> Self {
        Self::new(MemoryBudget::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_backward_cycle_is_garbage_free() {
        let mut gc = GarbageFreeCollector::unlimited();

        // Forward: save 3 states
        gc.checkpoint_ancilla(BitPlane::from_words(vec![1]))
            .unwrap();
        gc.checkpoint_ancilla(BitPlane::from_words(vec![2]))
            .unwrap();
        gc.checkpoint_ancilla(BitPlane::from_words(vec![3]))
            .unwrap();
        assert_eq!(gc.ancilla_count(), 3);
        assert!(!gc.is_garbage_free());

        // Backward: restore in LIFO order
        assert_eq!(gc.uncompute().unwrap().words()[0], 3);
        assert_eq!(gc.uncompute().unwrap().words()[0], 2);
        assert_eq!(gc.uncompute().unwrap().words()[0], 1);

        // Garbage-free!
        assert!(gc.is_garbage_free());
        assert!(gc.verify().is_ok());
    }

    #[test]
    fn budget_enforcement() {
        // Budget of 16 bytes = 2 u64 words
        let mut gc = GarbageFreeCollector::new(MemoryBudget::new(16));

        // First push: 8 bytes — OK
        gc.checkpoint_ancilla(BitPlane::from_words(vec![1]))
            .unwrap();
        // Second push: 16 bytes total — OK
        gc.checkpoint_ancilla(BitPlane::from_words(vec![2]))
            .unwrap();
        // Third push: 24 bytes — over budget!
        let result = gc.checkpoint_ancilla(BitPlane::from_words(vec![3]));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("budget"));
    }

    #[test]
    fn verify_fails_when_not_garbage_free() {
        let mut gc = GarbageFreeCollector::unlimited();
        gc.checkpoint_ancilla(BitPlane::from_words(vec![42]))
            .unwrap();
        let err = gc.verify().unwrap_err();
        assert!(err.to_string().contains("1 bits remain"));
    }

    #[test]
    fn uncompute_empty_stack_errors() {
        let mut gc = GarbageFreeCollector::unlimited();
        assert!(gc.uncompute().is_err());
    }

    #[test]
    fn memory_remaining_tracks_correctly() {
        let mut gc = GarbageFreeCollector::new(MemoryBudget::new(100));
        assert_eq!(gc.memory_remaining(), 100);
        gc.checkpoint_ancilla(BitPlane::from_words(vec![0; 5]))
            .unwrap(); // 40 bytes
        assert_eq!(gc.memory_remaining(), 60);
    }

    #[test]
    fn default_has_64mb_budget() {
        let gc = GarbageFreeCollector::default();
        assert_eq!(gc.memory_remaining(), 64 * 1024 * 1024);
    }
}
