//! Reversible execution engine — forward/backward execution with checkpoints.
//!
//! The engine executes sequences of reversible operations bidirectionally,
//! maintaining an ancilla stack for state restoration during backward execution.
//!
//! # Examples
//!
//! ```
//! use rewind_core::engine::{ExecutionEngine, ReversibleProgram, Op};
//! use rewind_core::bitplane::BitPlane;
//!
//! let regs = vec![BitPlane::from_words(vec![0xAA]), BitPlane::from_words(vec![0xBB])];
//! let mut engine = ExecutionEngine::new(regs);
//! let original = engine.registers().to_vec();
//!
//! let prog = ReversibleProgram::new(vec![Op::Not(0), Op::Cnot { control: 0, target: 1 }]);
//! prog.forward(&mut engine);
//! prog.backward(&mut engine);
//! assert_eq!(engine.registers().to_vec(), original);
//! ```

use crate::bitplane::BitPlane;
use crate::error::RewindError;
use crate::state::CheckpointId;
use std::collections::HashMap;

/// A snapshot of the execution state that can be restored.
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// The register states at checkpoint time.
    pub registers: Vec<BitPlane>,
}

/// Manages checkpoints and state restoration for reversible execution.
#[derive(Debug)]
pub struct ExecutionEngine {
    /// Current register file.
    registers: Vec<BitPlane>,
    /// Saved checkpoints indexed by ID.
    checkpoints: HashMap<u32, Snapshot>,
    /// Next checkpoint ID to assign.
    next_checkpoint_id: u32,
}

impl ExecutionEngine {
    /// Creates a new engine with the given initial register states.
    pub fn new(registers: Vec<BitPlane>) -> Self {
        Self {
            registers,
            checkpoints: HashMap::new(),
            next_checkpoint_id: 0,
        }
    }

    /// Returns a reference to the current register file.
    pub fn registers(&self) -> &[BitPlane] {
        &self.registers
    }

    /// Returns a mutable reference to a specific register.
    pub fn register_mut(&mut self, index: usize) -> &mut BitPlane {
        &mut self.registers[index]
    }

    /// Creates a checkpoint of the current state.
    ///
    /// Returns a `CheckpointId` that can be used with [`restore`](Self::restore).
    pub fn checkpoint(&mut self) -> CheckpointId {
        let id = self.next_checkpoint_id;
        self.next_checkpoint_id += 1;
        let snapshot = Snapshot {
            registers: self.registers.clone(),
        };
        self.checkpoints.insert(id, snapshot);
        CheckpointId(id)
    }

    /// Restores state from a previously saved checkpoint.
    ///
    /// Returns `Err(RewindError::CheckpointNotFound)` if the ID is invalid.
    pub fn restore(&mut self, id: CheckpointId) -> Result<(), RewindError> {
        let snapshot = self
            .checkpoints
            .get(&id.0)
            .ok_or(RewindError::CheckpointNotFound(id))?;
        self.registers = snapshot.registers.clone();
        Ok(())
    }

    /// Returns the number of saved checkpoints.
    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }

    /// Removes a checkpoint, freeing its memory.
    pub fn discard_checkpoint(&mut self, id: CheckpointId) -> Result<(), RewindError> {
        self.checkpoints
            .remove(&id.0)
            .ok_or(RewindError::CheckpointNotFound(id))?;
        Ok(())
    }

    /// Applies a Pauli-X (NOT) to the register at `index`.
    pub fn apply_not(&mut self, index: usize) {
        let reg = &self.registers[index];
        self.registers[index] = reg.not();
    }

    /// Applies CNOT: `registers[target] ^= registers[control]`.
    pub fn apply_cnot(&mut self, control: usize, target: usize) {
        let ctrl = self.registers[control].clone();
        self.registers[target].xor_assign(&ctrl);
    }

    /// Applies Toffoli: `registers[target] ^= registers[c1] AND registers[c2]`.
    pub fn apply_toffoli(&mut self, c1: usize, c2: usize, target: usize) {
        let mask = self.registers[c1].and(&self.registers[c2]);
        self.registers[target].xor_assign(&mask);
    }
}

/// A reversible program: a sequence of operations that can be run forward and backward.
#[derive(Debug, Clone)]
pub struct ReversibleProgram {
    /// The operations in forward order.
    pub ops: Vec<Op>,
}

/// A single reversible operation in a program.
#[derive(Debug, Clone)]
pub enum Op {
    /// Pauli-X (NOT) on register index.
    Not(usize),
    /// CNOT: control, target register indices.
    Cnot { control: usize, target: usize },
    /// Toffoli: control1, control2, target register indices.
    Toffoli { c1: usize, c2: usize, target: usize },
}

impl ReversibleProgram {
    /// Creates a new program from a list of operations.
    pub fn new(ops: Vec<Op>) -> Self {
        Self { ops }
    }

    /// Executes the program forward on the engine.
    pub fn forward(&self, engine: &mut ExecutionEngine) {
        for op in &self.ops {
            match op {
                Op::Not(i) => engine.apply_not(*i),
                Op::Cnot { control, target } => engine.apply_cnot(*control, *target),
                Op::Toffoli { c1, c2, target } => engine.apply_toffoli(*c1, *c2, *target),
            }
        }
    }

    /// Executes the program backward (in reverse order) on the engine.
    ///
    /// Since all gates are self-inverse, backward execution applies the same
    /// operations in reverse order.
    pub fn backward(&self, engine: &mut ExecutionEngine) {
        for op in self.ops.iter().rev() {
            match op {
                Op::Not(i) => engine.apply_not(*i),
                Op::Cnot { control, target } => engine.apply_cnot(*control, *target),
                Op::Toffoli { c1, c2, target } => engine.apply_toffoli(*c1, *c2, *target),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine(vals: &[u64]) -> ExecutionEngine {
        let regs: Vec<BitPlane> = vals.iter().map(|v| BitPlane::from_words(vec![*v])).collect();
        ExecutionEngine::new(regs)
    }

    #[test]
    fn not_forward_backward_restores_state() {
        let mut engine = make_engine(&[0xDEADBEEF]);
        let original = engine.registers()[0].clone();

        let prog = ReversibleProgram::new(vec![Op::Not(0)]);
        prog.forward(&mut engine);
        assert_ne!(engine.registers()[0], original);
        prog.backward(&mut engine);
        assert_eq!(engine.registers()[0], original);
    }

    #[test]
    fn cnot_forward_backward_restores_state() {
        let mut engine = make_engine(&[0xFF, 0x0F]);
        let original: Vec<_> = engine.registers().to_vec();

        let prog = ReversibleProgram::new(vec![Op::Cnot { control: 0, target: 1 }]);
        prog.forward(&mut engine);
        prog.backward(&mut engine);
        assert_eq!(engine.registers().to_vec(), original);
    }

    #[test]
    fn toffoli_forward_backward_restores_state() {
        let mut engine = make_engine(&[0xFF, 0x0F, 0xAB]);
        let original: Vec<_> = engine.registers().to_vec();

        let prog = ReversibleProgram::new(vec![Op::Toffoli { c1: 0, c2: 1, target: 2 }]);
        prog.forward(&mut engine);
        prog.backward(&mut engine);
        assert_eq!(engine.registers().to_vec(), original);
    }

    #[test]
    fn complex_program_forward_backward() {
        let mut engine = make_engine(&[0xAA, 0xBB, 0xCC]);
        let original: Vec<_> = engine.registers().to_vec();

        let prog = ReversibleProgram::new(vec![
            Op::Not(0),
            Op::Cnot { control: 0, target: 1 },
            Op::Toffoli { c1: 0, c2: 1, target: 2 },
            Op::Not(2),
            Op::Cnot { control: 2, target: 0 },
        ]);

        prog.forward(&mut engine);
        assert_ne!(engine.registers().to_vec(), original);
        prog.backward(&mut engine);
        assert_eq!(engine.registers().to_vec(), original);
    }

    #[test]
    fn checkpoint_and_restore() {
        let mut engine = make_engine(&[0xAA, 0xBB]);
        let ckpt = engine.checkpoint();

        engine.apply_not(0);
        engine.apply_cnot(0, 1);
        assert_ne!(engine.registers()[0].words()[0], 0xAA);

        engine.restore(ckpt).unwrap();
        assert_eq!(engine.registers()[0].words()[0], 0xAA);
        assert_eq!(engine.registers()[1].words()[0], 0xBB);
    }

    #[test]
    fn restore_invalid_checkpoint_errors() {
        let mut engine = make_engine(&[0x00]);
        let result = engine.restore(CheckpointId(999));
        assert!(matches!(result, Err(RewindError::CheckpointNotFound(_))));
    }

    #[test]
    fn multiple_checkpoints() {
        let mut engine = make_engine(&[0x00]);

        let ckpt1 = engine.checkpoint();
        engine.apply_not(0);
        let ckpt2 = engine.checkpoint();
        engine.apply_not(0);

        // State is back to 0x00 (NOT applied twice)
        assert_eq!(engine.registers()[0].words()[0], 0x00);

        // Restore to ckpt2 (after first NOT)
        engine.restore(ckpt2).unwrap();
        assert_eq!(engine.registers()[0].words()[0], !0x00u64);

        // Restore to ckpt1 (original)
        engine.restore(ckpt1).unwrap();
        assert_eq!(engine.registers()[0].words()[0], 0x00);
    }

    #[test]
    fn discard_checkpoint() {
        let mut engine = make_engine(&[0x00]);
        let ckpt = engine.checkpoint();
        assert_eq!(engine.checkpoint_count(), 1);
        engine.discard_checkpoint(ckpt).unwrap();
        assert_eq!(engine.checkpoint_count(), 0);
        assert!(engine.restore(ckpt).is_err());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn any_single_gate_program_is_reversible(val: u64, gate_type in 0u8..3) {
            let mut engine = make_engine_prop(val);
            let original = engine.registers().to_vec();

            let prog = match gate_type {
                0 => ReversibleProgram::new(vec![Op::Not(0)]),
                1 => ReversibleProgram::new(vec![Op::Cnot { control: 0, target: 1 }]),
                _ => ReversibleProgram::new(vec![Op::Toffoli { c1: 0, c2: 1, target: 2 }]),
            };

            prog.forward(&mut engine);
            prog.backward(&mut engine);
            prop_assert_eq!(engine.registers().to_vec(), original);
        }

        #[test]
        fn multi_gate_program_is_reversible(a: u64, b: u64, c: u64) {
            let regs = vec![
                BitPlane::from_words(vec![a]),
                BitPlane::from_words(vec![b]),
                BitPlane::from_words(vec![c]),
            ];
            let mut engine = ExecutionEngine::new(regs);
            let original = engine.registers().to_vec();

            let prog = ReversibleProgram::new(vec![
                Op::Not(0),
                Op::Cnot { control: 0, target: 1 },
                Op::Toffoli { c1: 0, c2: 1, target: 2 },
                Op::Not(2),
            ]);

            prog.forward(&mut engine);
            prog.backward(&mut engine);
            prop_assert_eq!(engine.registers().to_vec(), original);
        }
    }

    fn make_engine_prop(val: u64) -> ExecutionEngine {
        let regs = vec![
            BitPlane::from_words(vec![val]),
            BitPlane::from_words(vec![val.wrapping_mul(7)]),
            BitPlane::from_words(vec![val.wrapping_mul(13)]),
        ];
        ExecutionEngine::new(regs)
    }
}
