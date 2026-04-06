//! Computation graph (DAG) for Bennett's algorithm.
//!
//! Represents a sequence of computation steps as a directed acyclic graph,
//! where each node is a computation step and edges represent data dependencies.
//!
//! # Examples
//!
//! ```
//! use rewind_bennett::graph::ComputationGraph;
//!
//! let mut graph = ComputationGraph::new();
//! let s0 = graph.add_step("input");
//! let s1 = graph.add_step("not_x");
//! let s2 = graph.add_step("xor_xy");
//! graph.add_dependency(s0, s1);
//! graph.add_dependency(s1, s2);
//! assert_eq!(graph.step_count(), 3);
//! ```

/// Identifier for a computation step in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StepId(pub usize);

/// A single step in the computation graph.
#[derive(Debug, Clone)]
pub struct ComputationStep {
    /// Human-readable label for this step.
    pub label: String,
    /// Steps that must complete before this one (data dependencies).
    pub dependencies: Vec<StepId>,
}

/// Directed acyclic graph representing a computation.
///
/// Used by Bennett's algorithm to plan checkpoint placement
/// and determine the optimal pebbling strategy.
#[derive(Debug)]
pub struct ComputationGraph {
    steps: Vec<ComputationStep>,
}

impl ComputationGraph {
    /// Creates a new empty computation graph.
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Adds a computation step and returns its ID.
    pub fn add_step(&mut self, label: &str) -> StepId {
        let id = StepId(self.steps.len());
        self.steps.push(ComputationStep {
            label: label.to_string(),
            dependencies: Vec::new(),
        });
        id
    }

    /// Adds a dependency: `to` depends on `from` (from must complete before to).
    pub fn add_dependency(&mut self, from: StepId, to: StepId) {
        self.steps[to.0].dependencies.push(from);
    }

    /// Returns the number of steps in the graph.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Returns a reference to a step by ID.
    pub fn step(&self, id: StepId) -> &ComputationStep {
        &self.steps[id.0]
    }

    /// Returns a topological ordering of steps (dependencies first).
    pub fn topological_order(&self) -> Vec<StepId> {
        // Simple topological sort via DFS for the DAG
        let n = self.steps.len();
        let mut visited = vec![false; n];
        let mut order = Vec::with_capacity(n);

        for i in 0..n {
            if !visited[i] {
                self.topo_dfs(StepId(i), &mut visited, &mut order);
            }
        }

        order
    }

    fn topo_dfs(&self, id: StepId, visited: &mut [bool], order: &mut Vec<StepId>) {
        if visited[id.0] {
            return;
        }
        visited[id.0] = true;

        for &dep in &self.steps[id.0].dependencies {
            self.topo_dfs(dep, visited, order);
        }

        order.push(id);
    }

    /// Creates a simple linear chain of steps (most common case).
    pub fn linear_chain(labels: &[&str]) -> Self {
        let mut graph = Self::new();
        let mut prev = None;
        for label in labels {
            let id = graph.add_step(label);
            if let Some(p) = prev {
                graph.add_dependency(p, id);
            }
            prev = Some(id);
        }
        graph
    }
}

impl Default for ComputationGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_chain_creates_correct_graph() {
        let graph = ComputationGraph::linear_chain(&["a", "b", "c"]);
        assert_eq!(graph.step_count(), 3);
        assert_eq!(graph.step(StepId(0)).label, "a");
        assert_eq!(graph.step(StepId(1)).dependencies, vec![StepId(0)]);
        assert_eq!(graph.step(StepId(2)).dependencies, vec![StepId(1)]);
    }

    #[test]
    fn topological_order_respects_dependencies() {
        let graph = ComputationGraph::linear_chain(&["a", "b", "c"]);
        let order = graph.topological_order();
        assert_eq!(order, vec![StepId(0), StepId(1), StepId(2)]);
    }

    #[test]
    fn diamond_dependency() {
        let mut graph = ComputationGraph::new();
        let a = graph.add_step("a");
        let b = graph.add_step("b");
        let c = graph.add_step("c");
        let d = graph.add_step("d");
        graph.add_dependency(a, b);
        graph.add_dependency(a, c);
        graph.add_dependency(b, d);
        graph.add_dependency(c, d);

        let order = graph.topological_order();
        // a must come first, d must come last
        assert_eq!(order[0], StepId(0)); // a
        assert_eq!(order[3], StepId(3)); // d
    }
}
