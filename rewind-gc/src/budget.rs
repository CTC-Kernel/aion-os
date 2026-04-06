//! Memory budget configuration for the ancilla stack.
//!
//! Prevents unbounded memory growth by enforcing configurable limits
//! on the total ancilla storage, triggering errors when exceeded.
//!
//! # Examples
//!
//! ```
//! use rewind_gc::budget::MemoryBudget;
//!
//! let budget = MemoryBudget::new(1024); // 1 KB limit
//! assert!(budget.check(512).is_ok());   // Under budget
//! assert!(budget.check(2048).is_err()); // Over budget
//! ```

use rewind_core::error::RewindError;

/// Configurable memory budget for ancilla storage.
///
/// When the ancilla stack exceeds the budget, operations return
/// `RewindError::MemoryBudgetExceeded` instead of silently growing.
#[derive(Debug, Clone)]
pub struct MemoryBudget {
    /// Maximum allowed bytes for ancilla storage.
    limit_bytes: usize,
}

impl MemoryBudget {
    /// Creates a new budget with the given byte limit.
    pub fn new(limit_bytes: usize) -> Self {
        Self { limit_bytes }
    }

    /// Creates an unlimited budget (no enforcement).
    pub fn unlimited() -> Self {
        Self {
            limit_bytes: usize::MAX,
        }
    }

    /// Returns the configured limit in bytes.
    pub fn limit(&self) -> usize {
        self.limit_bytes
    }

    /// Checks if the given usage is within budget.
    ///
    /// Returns `Ok(())` if within limits, or `Err(RewindError::MemoryBudgetExceeded)`.
    pub fn check(&self, current_usage: usize) -> Result<(), RewindError> {
        if current_usage > self.limit_bytes {
            Err(RewindError::MemoryBudgetExceeded {
                used: current_usage,
                limit: self.limit_bytes,
            })
        } else {
            Ok(())
        }
    }

    /// Returns the remaining available bytes.
    pub fn remaining(&self, current_usage: usize) -> usize {
        self.limit_bytes.saturating_sub(current_usage)
    }
}

impl Default for MemoryBudget {
    fn default() -> Self {
        // Default: 64 MB — generous for most computations
        Self::new(64 * 1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn within_budget() {
        let budget = MemoryBudget::new(1024);
        assert!(budget.check(512).is_ok());
        assert!(budget.check(1024).is_ok());
    }

    #[test]
    fn over_budget() {
        let budget = MemoryBudget::new(1024);
        let err = budget.check(2048).unwrap_err();
        assert!(err.to_string().contains("2048 > 1024"));
    }

    #[test]
    fn unlimited_budget() {
        let budget = MemoryBudget::unlimited();
        assert!(budget.check(usize::MAX - 1).is_ok());
    }

    #[test]
    fn remaining_calculation() {
        let budget = MemoryBudget::new(1000);
        assert_eq!(budget.remaining(400), 600);
        assert_eq!(budget.remaining(1000), 0);
        assert_eq!(budget.remaining(1500), 0); // Saturates at 0
    }

    #[test]
    fn default_is_64mb() {
        let budget = MemoryBudget::default();
        assert_eq!(budget.limit(), 64 * 1024 * 1024);
    }
}
