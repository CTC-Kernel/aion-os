//! Execution backends for the reversible virtual machine.
//!
//! The `ExecutionBackend` trait abstracts hardware targets, allowing the same
//! reversible programs to run on simulated CPU today and on reversible chips tomorrow.
//!
//! # Examples
//!
//! ```
//! use rewind_core::backend::SimulatedCpu;
//!
//! let backend = SimulatedCpu::new();
//! // Backend will be used by the execution engine in future stories
//! ```

/// Trait abstracting the execution target for reversible programs.
///
/// Implementors provide the ability to execute sequences of reversible
/// opcodes on a specific hardware target. The default implementation
/// is [`SimulatedCpu`], which runs on conventional processors.
pub trait ExecutionBackend {
    /// Execute a single forward step on the backend.
    fn step_forward(&mut self);

    /// Execute a single backward step on the backend.
    fn step_backward(&mut self);

    /// Returns the name of this backend (for logging/display).
    fn name(&self) -> &str;
}

/// Default backend: simulates reversible execution on a conventional CPU
/// using match dispatch over opcodes.
///
/// This is the only backend available in v0.1. Future backends will target
/// Vaire chips, FPGAs, or RISC-V reversible extensions.
#[derive(Debug)]
pub struct SimulatedCpu {
    steps_executed: u64,
}

impl SimulatedCpu {
    /// Creates a new simulated CPU backend.
    pub fn new() -> Self {
        Self { steps_executed: 0 }
    }

    /// Returns the total number of steps executed (forward + backward).
    pub fn steps_executed(&self) -> u64 {
        self.steps_executed
    }
}

impl Default for SimulatedCpu {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionBackend for SimulatedCpu {
    fn step_forward(&mut self) {
        self.steps_executed += 1;
    }

    fn step_backward(&mut self) {
        self.steps_executed += 1;
    }

    fn name(&self) -> &str {
        "SimulatedCpu"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulated_cpu_tracks_steps() {
        let mut cpu = SimulatedCpu::new();
        assert_eq!(cpu.steps_executed(), 0);
        cpu.step_forward();
        cpu.step_forward();
        cpu.step_backward();
        assert_eq!(cpu.steps_executed(), 3);
    }

    #[test]
    fn simulated_cpu_name() {
        let cpu = SimulatedCpu::new();
        assert_eq!(cpu.name(), "SimulatedCpu");
    }

    #[test]
    fn default_works() {
        let cpu = SimulatedCpu::default();
        assert_eq!(cpu.steps_executed(), 0);
    }
}
