//! Bennett's reversible executor — transforms irreversible computations into reversible ones.
//!
//! The executor follows Bennett's three-phase approach:
//! 1. **Forward**: Execute computation with checkpoint saving
//! 2. **Copy**: Save the final result
//! 3. **Backward**: Uncompute to restore initial state (except result)
//!
//! # Examples
//!
//! ```
//! use rewind_bennett::executor::BennettExecutor;
//! use rewind_bennett::pebbling::BennettConfig;
//!
//! let executor = BennettExecutor::new(BennettConfig::default());
//! // Used by the execution engine to automatically reversibilize computations
//! ```

use crate::graph::{ComputationGraph, StepId};
use crate::pebbling::{BennettConfig, PebblingStrategy};

/// Executes computations reversibly using Bennett's algorithm.
///
/// Given an irreversible computation represented as a `ComputationGraph`,
/// the executor plans checkpoints via `PebblingStrategy` and manages
/// the forward-copy-backward execution cycle.
#[derive(Debug)]
pub struct BennettExecutor {
    config: BennettConfig,
}

/// The execution plan produced by Bennett's algorithm.
#[derive(Debug)]
pub struct BennettPlan {
    /// The pebbling strategy (checkpoint positions).
    pub strategy: PebblingStrategy,
    /// Steps to execute in forward phase.
    pub forward_steps: Vec<StepId>,
    /// Steps to uncompute in backward phase (reverse order).
    pub backward_steps: Vec<StepId>,
}

impl BennettExecutor {
    /// Creates a new executor with the given configuration.
    pub fn new(config: BennettConfig) -> Self {
        Self { config }
    }

    /// Plans a reversible execution for the given computation graph.
    ///
    /// Returns a `BennettPlan` with forward steps, checkpoint positions,
    /// and backward steps for uncomputation.
    pub fn plan(&self, graph: &ComputationGraph) -> BennettPlan {
        let topo_order = graph.topological_order();
        let strategy = PebblingStrategy::plan(topo_order.len(), self.config.clone());

        // Forward: execute in topological order
        let forward_steps = topo_order.clone();

        // Backward: uncompute in reverse topological order
        // (skip the last step — that's the result we want to keep)
        let mut backward_steps: Vec<StepId> = topo_order;
        backward_steps.pop(); // Keep the final result
        backward_steps.reverse();

        BennettPlan {
            strategy,
            forward_steps,
            backward_steps,
        }
    }

    /// Returns the estimated time overhead factor for this configuration.
    ///
    /// Bennett's algorithm has time O(T^(1+ε)), so the overhead is T^ε.
    pub fn estimated_time_overhead(&self, num_steps: usize) -> f64 {
        (num_steps as f64).powf(self.config.epsilon)
    }

    /// Returns the estimated space overhead factor.
    ///
    /// Bennett's algorithm uses O(S·log(T)) space for checkpoints.
    pub fn estimated_space_overhead(&self, num_steps: usize) -> f64 {
        (num_steps as f64).ln()
    }
}

impl Default for BennettExecutor {
    fn default() -> Self {
        Self::new(BennettConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_linear_computation() {
        let graph = ComputationGraph::linear_chain(&["step1", "step2", "step3", "step4"]);
        let executor = BennettExecutor::default();
        let plan = executor.plan(&graph);

        // Forward should include all steps
        assert_eq!(plan.forward_steps.len(), 4);

        // Backward should skip the last (result) step
        assert_eq!(plan.backward_steps.len(), 3);

        // Backward should be in reverse order
        assert_eq!(plan.backward_steps[0], StepId(2));
        assert_eq!(plan.backward_steps[1], StepId(1));
        assert_eq!(plan.backward_steps[2], StepId(0));
    }

    #[test]
    fn plan_has_checkpoints() {
        let graph = ComputationGraph::linear_chain(
            &["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]
        );
        let executor = BennettExecutor::default();
        let plan = executor.plan(&graph);

        assert!(plan.strategy.checkpoint_count() > 0);
    }

    #[test]
    fn time_overhead_scales_with_epsilon() {
        let exec_low = BennettExecutor::new(BennettConfig::new(0.1));
        let exec_high = BennettExecutor::new(BennettConfig::new(0.9));

        let overhead_low = exec_low.estimated_time_overhead(1000);
        let overhead_high = exec_high.estimated_time_overhead(1000);

        // Higher epsilon = more time overhead
        assert!(overhead_high > overhead_low);
    }

    #[test]
    fn space_overhead_is_logarithmic() {
        let executor = BennettExecutor::default();

        let space_10 = executor.estimated_space_overhead(10);
        let space_1000 = executor.estimated_space_overhead(1000);

        // log(1000) / log(10) ≈ 3 — sublinear growth
        assert!(space_1000 / space_10 < 4.0);
    }

    #[test]
    fn empty_graph() {
        let graph = ComputationGraph::new();
        let executor = BennettExecutor::default();
        let plan = executor.plan(&graph);
        assert!(plan.forward_steps.is_empty());
        assert!(plan.backward_steps.is_empty());
    }
}
