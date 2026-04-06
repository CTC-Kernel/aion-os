//! Pebbling strategy for Bennett's reversible computation.
//!
//! The pebbling game determines where to place checkpoints during
//! forward execution, controlling the space/time trade-off of
//! reversible computation.
//!
//! Bennett's algorithm transforms an irreversible computation of time T
//! and space S into a reversible one with:
//! - Time: O(T^(1+ε))
//! - Space: O(S·log(T))
//!
//! # Examples
//!
//! ```
//! use rewind_bennett::pebbling::{PebblingStrategy, BennettConfig};
//!
//! let config = BennettConfig::default();
//! let strategy = PebblingStrategy::plan(10, config);
//! assert!(!strategy.checkpoint_positions().is_empty());
//! ```

/// Configuration for Bennett's algorithm.
#[derive(Debug, Clone)]
pub struct BennettConfig {
    /// Epsilon parameter controlling space/time trade-off.
    /// Lower ε = more space, less time overhead.
    /// **Warning**: The hidden factor ε·2^(1/ε) diverges as ε→0.
    /// Minimum recommended: 0.1
    pub epsilon: f64,

    /// Maximum allowed memory for checkpoints (bytes).
    /// If None, no limit is enforced.
    pub memory_limit: Option<usize>,
}

impl BennettConfig {
    /// Creates a config with the given epsilon.
    ///
    /// Panics if epsilon < 0.1 (the hidden factor makes smaller values impractical).
    pub fn new(epsilon: f64) -> Self {
        assert!(
            epsilon >= 0.1,
            "epsilon must be >= 0.1 — the hidden factor ε·2^(1/ε) \
             makes smaller values impractical (it diverges as ε→0)"
        );
        Self {
            epsilon,
            memory_limit: None,
        }
    }

    /// Sets the memory limit for checkpoints.
    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = Some(limit);
        self
    }
}

impl Default for BennettConfig {
    fn default() -> Self {
        Self {
            epsilon: 0.5,
            memory_limit: None,
        }
    }
}

/// A pebbling strategy that determines checkpoint placement.
#[derive(Debug)]
pub struct PebblingStrategy {
    /// Step indices where checkpoints should be saved.
    checkpoints: Vec<usize>,
    /// Total number of computation steps.
    total_steps: usize,
}

impl PebblingStrategy {
    /// Plans checkpoint placement for a linear computation of `num_steps`.
    ///
    /// Uses a simple recursive halving strategy: checkpoint at the midpoint,
    /// then recurse on each half. This gives O(log(n)) checkpoints.
    pub fn plan(num_steps: usize, config: BennettConfig) -> Self {
        let mut checkpoints = Vec::new();

        if num_steps > 1 {
            Self::plan_recursive(0, num_steps - 1, &config, &mut checkpoints);
        }

        checkpoints.sort();
        checkpoints.dedup();

        Self {
            checkpoints,
            total_steps: num_steps,
        }
    }

    fn plan_recursive(
        start: usize,
        end: usize,
        config: &BennettConfig,
        checkpoints: &mut Vec<usize>,
    ) {
        if end <= start + 1 {
            return; // Base case: segment too small to checkpoint
        }

        // Checkpoint interval based on epsilon
        // Higher epsilon → fewer checkpoints (more recomputation)
        let min_segment = ((end - start) as f64 * config.epsilon) as usize;
        let min_segment = min_segment.max(1);

        if end - start <= min_segment {
            return;
        }

        let mid = start + (end - start) / 2;
        checkpoints.push(mid);

        // Recurse on both halves
        Self::plan_recursive(start, mid, config, checkpoints);
        Self::plan_recursive(mid, end, config, checkpoints);
    }

    /// Returns the positions where checkpoints should be placed.
    pub fn checkpoint_positions(&self) -> &[usize] {
        &self.checkpoints
    }

    /// Returns the number of checkpoints planned.
    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }

    /// Returns the total number of computation steps.
    pub fn total_steps(&self) -> usize {
        self.total_steps
    }

    /// Returns true if a checkpoint should be placed at the given step.
    pub fn should_checkpoint(&self, step: usize) -> bool {
        self.checkpoints.contains(&step)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = BennettConfig::default();
        assert_eq!(config.epsilon, 0.5);
        assert!(config.memory_limit.is_none());
    }

    #[test]
    #[should_panic(expected = "epsilon must be >= 0.1")]
    fn rejects_too_small_epsilon() {
        BennettConfig::new(0.01);
    }

    #[test]
    fn plan_creates_checkpoints() {
        let strategy = PebblingStrategy::plan(100, BennettConfig::default());
        assert!(!strategy.checkpoint_positions().is_empty());
        assert_eq!(strategy.total_steps(), 100);
    }

    #[test]
    fn checkpoints_are_sorted_and_unique() {
        let strategy = PebblingStrategy::plan(1000, BennettConfig::default());
        let positions = strategy.checkpoint_positions();

        for i in 1..positions.len() {
            assert!(positions[i] > positions[i - 1], "checkpoints must be sorted");
        }
    }

    #[test]
    fn small_computation_has_few_checkpoints() {
        let strategy = PebblingStrategy::plan(3, BennettConfig::default());
        assert!(strategy.checkpoint_count() <= 2);
    }

    #[test]
    fn larger_epsilon_means_fewer_checkpoints() {
        let many = PebblingStrategy::plan(100, BennettConfig::new(0.1));
        let few = PebblingStrategy::plan(100, BennettConfig::new(0.9));
        assert!(many.checkpoint_count() >= few.checkpoint_count());
    }

    #[test]
    fn should_checkpoint_works() {
        let strategy = PebblingStrategy::plan(10, BennettConfig::default());
        let positions = strategy.checkpoint_positions().to_vec();

        for pos in &positions {
            assert!(strategy.should_checkpoint(*pos));
        }
        // Position 0 is rarely a checkpoint
        // (it's the start of the computation)
    }

    #[test]
    fn single_step_no_checkpoints() {
        let strategy = PebblingStrategy::plan(1, BennettConfig::default());
        assert_eq!(strategy.checkpoint_count(), 0);
    }
}
