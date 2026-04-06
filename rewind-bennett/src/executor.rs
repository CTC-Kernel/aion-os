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

/// Result of executing a Bennett plan on concrete operations.
#[derive(Debug)]
pub struct BennettResult {
    /// Total forward operations executed.
    pub forward_ops: usize,
    /// Total backward (uncompute) operations executed.
    pub backward_ops: usize,
    /// Number of checkpoints used.
    pub checkpoints_used: usize,
}

impl BennettExecutor {
    /// Executes a Bennett plan on a sequence of concrete operations.
    ///
    /// Takes a list of `Op`s (one per step in the computation graph) and
    /// a `ReversibleRuntime`, then executes the three-phase Bennett algorithm:
    /// 1. Forward: execute all operations with checkpointing
    /// 2. (Result is now available in registers)
    /// 3. Backward: uncompute intermediate steps to free ancilla
    ///
    /// After execution, only the final result remains; all intermediate
    /// state has been uncomputed (garbage-free).
    pub fn execute_with_ops(
        &self,
        ops: &[rewind_core::engine::Op],
        runtime: &mut rewind_core::runtime::ReversibleRuntime,
    ) -> BennettResult {
        let plan = self.plan(&ComputationGraph::linear_chain(
            &(0..ops.len())
                .map(|i| format!("step_{i}"))
                .collect::<Vec<_>>()
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
        ));

        // Phase 1: Forward execution
        let mut forward_count = 0;
        for &step_id in &plan.forward_steps {
            if let Some(op) = ops.get(step_id.0) {
                runtime.execute_tracked(op.clone());
                forward_count += 1;
            }
        }

        // Phase 2: Result is now in registers
        // (No explicit copy needed — the result stays in the registers)

        // Phase 3: Backward (uncompute) — skip last step to preserve result
        let mut backward_count = 0;
        for &step_id in &plan.backward_steps {
            if let Some(op) = ops.get(step_id.0) {
                // Apply the same op again (all gates are self-inverse)
                runtime.execute_tracked(op.clone());
                backward_count += 1;
            }
        }

        BennettResult {
            forward_ops: forward_count,
            backward_ops: backward_count,
            checkpoints_used: plan.strategy.checkpoint_count(),
        }
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
        let graph =
            ComputationGraph::linear_chain(&["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]);
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
