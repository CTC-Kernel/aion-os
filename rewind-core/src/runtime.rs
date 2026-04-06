//! Unified reversible runtime — combines execution engine with garbage-free collection.
//!
//! The `ReversibleRuntime` is the main entry point for running reversible programs.
//! It manages registers, checkpoints, and ancilla tracking in a single API.
//!
//! # Examples
//!
//! ```
//! use rewind_core::runtime::ReversibleRuntime;
//! use rewind_core::engine::Op;
//! use rewind_core::bitplane::BitPlane;
//!
//! let regs = vec![
//!     BitPlane::from_words(vec![42]),
//!     BitPlane::from_words(vec![0]),
//! ];
//! let mut rt = ReversibleRuntime::new(regs);
//!
//! // Run forward with ancilla tracking
//! rt.execute_tracked(Op::Not(0));
//! rt.execute_tracked(Op::Cnot { control: 0, target: 1 });
//!
//! // Rewind everything — ancilla restored automatically
//! rt.rewind_all().unwrap();
//!
//! assert_eq!(rt.register(0).words()[0], 42); // Restored!
//! assert!(rt.is_garbage_free());
//! ```

use crate::bitplane::BitPlane;
use crate::engine::{ExecutionEngine, Op};
use crate::error::RewindError;
use crate::state::CheckpointId;

/// Unified runtime combining execution engine with ancilla tracking.
///
/// Every operation executed via `execute_tracked` records its inverse
/// on an internal stack. Calling `rewind_all` replays the inverses
/// in LIFO order, perfectly restoring the original state.
#[derive(Debug)]
pub struct ReversibleRuntime {
    engine: ExecutionEngine,
    /// Stack of operations executed (for rewind).
    history: Vec<Op>,
}

impl ReversibleRuntime {
    /// Creates a new runtime with the given initial registers.
    pub fn new(registers: Vec<BitPlane>) -> Self {
        Self {
            engine: ExecutionEngine::new(registers),
            history: Vec::new(),
        }
    }

    /// Executes a single operation with tracking (for later rewind).
    pub fn execute_tracked(&mut self, op: Op) {
        self.apply_op(&op);
        self.history.push(op);
    }

    /// Executes a sequence of operations with tracking.
    pub fn execute_all_tracked(&mut self, ops: &[Op]) {
        for op in ops {
            self.execute_tracked(op.clone());
        }
    }

    /// Rewinds the last `n` operations, restoring the previous state.
    ///
    /// Since all gates are self-inverse, rewinding applies the same
    /// operations in reverse order.
    pub fn rewind(&mut self, n: usize) -> Result<(), RewindError> {
        for _ in 0..n {
            let op = self.history.pop().ok_or(RewindError::GarbageRemaining(0))?;
            self.apply_op(&op);
        }
        Ok(())
    }

    /// Rewinds ALL operations, restoring to initial state.
    pub fn rewind_all(&mut self) -> Result<(), RewindError> {
        let n = self.history.len();
        self.rewind(n)
    }

    /// Returns true if all operations have been rewound (no history remaining).
    pub fn is_garbage_free(&self) -> bool {
        self.history.is_empty()
    }

    /// Returns the number of tracked operations in history.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Creates a checkpoint of the current state.
    pub fn checkpoint(&mut self) -> CheckpointId {
        self.engine.checkpoint()
    }

    /// Restores a previous checkpoint.
    pub fn restore(&mut self, id: CheckpointId) -> Result<(), RewindError> {
        self.engine.restore(id)
    }

    /// Returns a reference to a register.
    pub fn register(&self, index: usize) -> &BitPlane {
        &self.engine.registers()[index]
    }

    /// Returns all registers.
    pub fn registers(&self) -> &[BitPlane] {
        self.engine.registers()
    }

    /// Returns the number of registers.
    pub fn num_registers(&self) -> usize {
        self.engine.registers().len()
    }

    fn apply_op(&mut self, op: &Op) {
        match op {
            Op::Not(i) => self.engine.apply_not(*i),
            Op::Cnot { control, target } => self.engine.apply_cnot(*control, *target),
            Op::Toffoli { c1, c2, target } => self.engine.apply_toffoli(*c1, *c2, *target),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt(vals: &[u64]) -> ReversibleRuntime {
        let regs = vals
            .iter()
            .map(|v| BitPlane::from_words(vec![*v]))
            .collect();
        ReversibleRuntime::new(regs)
    }

    #[test]
    fn execute_and_rewind_all() {
        let mut runtime = rt(&[0xAA, 0xBB, 0xCC]);
        let original: Vec<_> = runtime.registers().to_vec();

        runtime.execute_tracked(Op::Not(0));
        runtime.execute_tracked(Op::Cnot {
            control: 0,
            target: 1,
        });
        runtime.execute_tracked(Op::Toffoli {
            c1: 0,
            c2: 1,
            target: 2,
        });

        assert_ne!(runtime.registers().to_vec(), original);
        assert_eq!(runtime.history_len(), 3);

        runtime.rewind_all().unwrap();
        assert_eq!(runtime.registers().to_vec(), original);
        assert!(runtime.is_garbage_free());
    }

    #[test]
    fn partial_rewind() {
        let mut runtime = rt(&[0xFF, 0x00]);

        runtime.execute_tracked(Op::Not(0));
        runtime.execute_tracked(Op::Cnot {
            control: 0,
            target: 1,
        });
        assert_eq!(runtime.history_len(), 2);

        // Rewind only the CNOT
        runtime.rewind(1).unwrap();
        assert_eq!(runtime.history_len(), 1);

        // Register 0 is still NOTed, register 1 is back to 0x00
        assert_eq!(runtime.register(1).words()[0], 0x00);
    }

    #[test]
    fn execute_all_tracked() {
        let mut runtime = rt(&[0xAA, 0xBB]);
        let original: Vec<_> = runtime.registers().to_vec();

        let ops = vec![
            Op::Not(0),
            Op::Cnot {
                control: 0,
                target: 1,
            },
            Op::Not(1),
        ];
        runtime.execute_all_tracked(&ops);
        assert_eq!(runtime.history_len(), 3);

        runtime.rewind_all().unwrap();
        assert_eq!(runtime.registers().to_vec(), original);
    }

    #[test]
    fn checkpoint_and_restore_with_runtime() {
        let mut runtime = rt(&[42, 0]);
        let ckpt = runtime.checkpoint();

        runtime.execute_tracked(Op::Not(0));
        assert_ne!(runtime.register(0).words()[0], 42);

        runtime.restore(ckpt).unwrap();
        assert_eq!(runtime.register(0).words()[0], 42);
    }

    #[test]
    fn rewind_empty_is_ok() {
        let mut runtime = rt(&[0]);
        assert!(runtime.rewind_all().is_ok());
        assert!(runtime.is_garbage_free());
    }

    #[test]
    fn complex_sequence_rewind() {
        let mut runtime = rt(&[0x12, 0x34, 0x56, 0x78]);
        let original: Vec<_> = runtime.registers().to_vec();

        // 10 operations
        for _ in 0..5 {
            runtime.execute_tracked(Op::Not(0));
            runtime.execute_tracked(Op::Toffoli {
                c1: 1,
                c2: 2,
                target: 3,
            });
        }
        assert_eq!(runtime.history_len(), 10);

        runtime.rewind_all().unwrap();
        assert_eq!(runtime.registers().to_vec(), original);
        assert!(runtime.is_garbage_free());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn any_operation_sequence_is_reversible(a: u64, b: u64, c: u64, ops_count in 1usize..20) {
            let mut runtime = ReversibleRuntime::new(vec![
                BitPlane::from_words(vec![a]),
                BitPlane::from_words(vec![b]),
                BitPlane::from_words(vec![c]),
            ]);
            let original = runtime.registers().to_vec();

            // Generate pseudo-random operations
            let mut seed = a.wrapping_mul(7).wrapping_add(b);
            for _ in 0..ops_count {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let op = match seed % 3 {
                    0 => Op::Not((seed as usize / 3) % 3),
                    1 => {
                        let ctrl = (seed as usize / 3) % 3;
                        let tgt = (ctrl + 1) % 3;
                        Op::Cnot { control: ctrl, target: tgt }
                    }
                    _ => {
                        let c1 = (seed as usize / 3) % 3;
                        let c2 = (c1 + 1) % 3;
                        let tgt = (c1 + 2) % 3;
                        Op::Toffoli { c1, c2, target: tgt }
                    }
                };
                runtime.execute_tracked(op);
            }

            runtime.rewind_all().unwrap();
            prop_assert_eq!(runtime.registers().to_vec(), original);
            prop_assert!(runtime.is_garbage_free());
        }
    }
}
