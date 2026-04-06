//! Execution backends for the reversible virtual machine.
//!
//! The `ExecutionBackend` trait abstracts hardware targets. The default
//! `SimulatedCpu` executes operations on conventional processors.
//! Future backends can target Vaire chips, FPGAs, or RISC-V extensions.
//!
//! # Examples
//!
//! ```
//! use rewind_core::backend::{SimulatedCpu, ExecutionBackend};
//! use rewind_core::engine::Op;
//! use rewind_core::bitplane::BitPlane;
//!
//! let mut cpu = SimulatedCpu::with_registers(vec![
//!     BitPlane::from_words(vec![0xFF]),
//!     BitPlane::from_words(vec![0x00]),
//! ]);
//! cpu.execute_op(&Op::Cnot { control: 0, target: 1 });
//! assert_eq!(cpu.register(1).words()[0], 0xFF);
//! ```

use crate::bitplane::BitPlane;
use crate::engine::Op;

/// Trait abstracting the execution target for reversible programs.
pub trait ExecutionBackend {
    /// Execute a single operation on this backend.
    fn execute_op(&mut self, op: &Op);

    /// Returns the name of this backend.
    fn name(&self) -> &str;

    /// Returns the total number of operations executed.
    fn ops_executed(&self) -> u64;
}

/// Default backend: executes reversible operations on a conventional CPU.
///
/// Stores registers internally and applies gate operations directly
/// using bitwise operations on `BitPlane` data.
#[derive(Debug)]
pub struct SimulatedCpu {
    registers: Vec<BitPlane>,
    ops_count: u64,
}

impl SimulatedCpu {
    /// Creates a new backend with the given initial registers.
    pub fn with_registers(registers: Vec<BitPlane>) -> Self {
        Self {
            registers,
            ops_count: 0,
        }
    }

    /// Creates an empty backend (no registers).
    pub fn new() -> Self {
        Self {
            registers: Vec::new(),
            ops_count: 0,
        }
    }

    /// Returns a reference to a register.
    pub fn register(&self, index: usize) -> &BitPlane {
        &self.registers[index]
    }

    /// Returns all registers.
    pub fn registers(&self) -> &[BitPlane] {
        &self.registers
    }

    /// Returns the total number of operations executed.
    pub fn steps_executed(&self) -> u64 {
        self.ops_count
    }
}

impl Default for SimulatedCpu {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionBackend for SimulatedCpu {
    fn execute_op(&mut self, op: &Op) {
        match op {
            Op::Not(i) => {
                let r = self.registers[*i].not();
                self.registers[*i] = r;
            }
            Op::Cnot { control, target } => {
                let ctrl = self.registers[*control].clone();
                self.registers[*target].xor_assign(&ctrl);
            }
            Op::Toffoli { c1, c2, target } => {
                let mask = self.registers[*c1].and(&self.registers[*c2]);
                self.registers[*target].xor_assign(&mask);
            }
            Op::Fredkin { control, a, b } => {
                let diff = self.registers[*a].xor(&self.registers[*b]);
                let masked = diff.and(&self.registers[*control]);
                self.registers[*a].xor_assign(&masked);
                self.registers[*b].xor_assign(&masked);
            }
        }
        self.ops_count += 1;
    }

    fn name(&self) -> &str {
        "SimulatedCpu"
    }

    fn ops_executed(&self) -> u64 {
        self.ops_count
    }
}

/// A recording backend that logs all operations for analysis/replay.
///
/// Wraps another backend and records every operation executed.
#[derive(Debug)]
pub struct RecordingBackend<B: ExecutionBackend> {
    inner: B,
    log: Vec<Op>,
}

impl<B: ExecutionBackend> RecordingBackend<B> {
    /// Creates a new recording backend wrapping the inner backend.
    pub fn new(inner: B) -> Self {
        Self {
            inner,
            log: Vec::new(),
        }
    }

    /// Returns the operation log.
    pub fn log(&self) -> &[Op] {
        &self.log
    }

    /// Returns the number of logged operations.
    pub fn log_len(&self) -> usize {
        self.log.len()
    }

    /// Consumes self and returns the inner backend and log.
    pub fn into_parts(self) -> (B, Vec<Op>) {
        (self.inner, self.log)
    }
}

impl<B: ExecutionBackend> ExecutionBackend for RecordingBackend<B> {
    fn execute_op(&mut self, op: &Op) {
        self.log.push(op.clone());
        self.inner.execute_op(op);
    }

    fn name(&self) -> &str {
        "RecordingBackend"
    }

    fn ops_executed(&self) -> u64 {
        self.inner.ops_executed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulated_cpu_executes_not() {
        let mut cpu = SimulatedCpu::with_registers(vec![BitPlane::from_words(vec![0xFF])]);
        cpu.execute_op(&Op::Not(0));
        assert_eq!(cpu.register(0).words()[0], !0xFFu64);
        assert_eq!(cpu.ops_executed(), 1);
    }

    #[test]
    fn simulated_cpu_executes_cnot() {
        let mut cpu = SimulatedCpu::with_registers(vec![
            BitPlane::from_words(vec![0xFF]),
            BitPlane::from_words(vec![0x00]),
        ]);
        cpu.execute_op(&Op::Cnot {
            control: 0,
            target: 1,
        });
        assert_eq!(cpu.register(1).words()[0], 0xFF);
    }

    #[test]
    fn simulated_cpu_executes_toffoli() {
        let mut cpu = SimulatedCpu::with_registers(vec![
            BitPlane::from_words(vec![0xFF]),
            BitPlane::from_words(vec![0x0F]),
            BitPlane::from_words(vec![0x00]),
        ]);
        cpu.execute_op(&Op::Toffoli {
            c1: 0,
            c2: 1,
            target: 2,
        });
        assert_eq!(cpu.register(2).words()[0], 0x0F);
    }

    #[test]
    fn simulated_cpu_executes_fredkin() {
        let mut cpu = SimulatedCpu::with_registers(vec![
            BitPlane::from_words(vec![u64::MAX]), // All 1s = full swap
            BitPlane::from_words(vec![0xAA]),
            BitPlane::from_words(vec![0x55]),
        ]);
        cpu.execute_op(&Op::Fredkin {
            control: 0,
            a: 1,
            b: 2,
        });
        assert_eq!(cpu.register(1).words()[0], 0x55);
        assert_eq!(cpu.register(2).words()[0], 0xAA);
    }

    #[test]
    fn simulated_cpu_forward_backward() {
        let mut cpu = SimulatedCpu::with_registers(vec![
            BitPlane::from_words(vec![0xDEAD]),
            BitPlane::from_words(vec![0xBEEF]),
        ]);
        let original = cpu.registers().to_vec();

        let ops = vec![
            Op::Not(0),
            Op::Cnot {
                control: 0,
                target: 1,
            },
        ];

        // Forward
        for op in &ops {
            cpu.execute_op(op);
        }
        assert_ne!(cpu.registers().to_vec(), original);

        // Backward (same ops in reverse — all self-inverse)
        for op in ops.iter().rev() {
            cpu.execute_op(op);
        }
        assert_eq!(cpu.registers().to_vec(), original);
    }

    #[test]
    fn recording_backend_logs_ops() {
        let cpu = SimulatedCpu::with_registers(vec![BitPlane::from_words(vec![0])]);
        let mut recorder = RecordingBackend::new(cpu);

        recorder.execute_op(&Op::Not(0));
        recorder.execute_op(&Op::Not(0));

        assert_eq!(recorder.log_len(), 2);
        assert_eq!(recorder.ops_executed(), 2);

        let (inner, log) = recorder.into_parts();
        assert_eq!(log.len(), 2);
        assert_eq!(inner.register(0).words()[0], 0); // NOT applied twice = identity
    }

    #[test]
    fn default_cpu_has_no_registers() {
        let cpu = SimulatedCpu::default();
        assert_eq!(cpu.registers().len(), 0);
        assert_eq!(cpu.steps_executed(), 0);
    }
}
