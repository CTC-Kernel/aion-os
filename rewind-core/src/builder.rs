//! Fluent program builder for constructing reversible programs.
//!
//! # Examples
//!
//! ```
//! use rewind_core::builder::ProgramBuilder;
//!
//! let program = ProgramBuilder::new()
//!     .not(0)
//!     .cnot(0, 1)
//!     .toffoli(0, 1, 2)
//!     .fredkin(0, 1, 2)
//!     .not(2)
//!     .build();
//!
//! assert_eq!(program.ops.len(), 5);
//! ```

use crate::engine::{Op, ReversibleProgram};

/// Fluent builder for constructing reversible programs step by step.
#[derive(Debug, Default)]
pub struct ProgramBuilder {
    ops: Vec<Op>,
}

impl ProgramBuilder {
    /// Creates a new empty builder.
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    /// Adds a Pauli-X (NOT) gate on the given register.
    pub fn not(mut self, register: usize) -> Self {
        self.ops.push(Op::Not(register));
        self
    }

    /// Adds a CNOT gate: `target ^= control`.
    pub fn cnot(mut self, control: usize, target: usize) -> Self {
        self.ops.push(Op::Cnot { control, target });
        self
    }

    /// Adds a Toffoli (CCNOT) gate: `target ^= c1 AND c2`.
    pub fn toffoli(mut self, c1: usize, c2: usize, target: usize) -> Self {
        self.ops.push(Op::Toffoli { c1, c2, target });
        self
    }

    /// Adds a Fredkin (CSWAP) gate: conditional swap of a and b where control=1.
    pub fn fredkin(mut self, control: usize, a: usize, b: usize) -> Self {
        self.ops.push(Op::Fredkin { control, a, b });
        self
    }

    /// Appends a sequence of operations from another source.
    pub fn extend(mut self, ops: &[Op]) -> Self {
        self.ops.extend_from_slice(ops);
        self
    }

    /// Appends the operations from another program.
    pub fn append_program(mut self, program: &ReversibleProgram) -> Self {
        self.ops.extend_from_slice(&program.ops);
        self
    }

    /// Repeats the current program N times.
    pub fn repeat(mut self, times: usize) -> Self {
        let current = self.ops.clone();
        for _ in 1..times {
            self.ops.extend_from_slice(&current);
        }
        self
    }

    /// Returns the number of operations added so far.
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Returns true if no operations have been added.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Builds the final `ReversibleProgram`.
    pub fn build(self) -> ReversibleProgram {
        ReversibleProgram::new(self.ops)
    }

    /// Builds and returns just the ops vector.
    pub fn build_ops(self) -> Vec<Op> {
        self.ops
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_basic() {
        let prog = ProgramBuilder::new()
            .not(0)
            .cnot(0, 1)
            .toffoli(0, 1, 2)
            .build();
        assert_eq!(prog.ops.len(), 3);
    }

    #[test]
    fn builder_fredkin() {
        let prog = ProgramBuilder::new().fredkin(0, 1, 2).build();
        assert_eq!(prog.ops.len(), 1);
    }

    #[test]
    fn builder_repeat() {
        let prog = ProgramBuilder::new().not(0).cnot(0, 1).repeat(3).build();
        assert_eq!(prog.ops.len(), 6); // 2 * 3
    }

    #[test]
    fn builder_extend() {
        let extra = vec![Op::Not(3), Op::Not(4)];
        let prog = ProgramBuilder::new().not(0).extend(&extra).build();
        assert_eq!(prog.ops.len(), 3);
    }

    #[test]
    fn builder_empty() {
        let b = ProgramBuilder::new();
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);
    }

    #[test]
    fn builder_reversible() {
        use crate::bitplane::BitPlane;
        use crate::runtime::ReversibleRuntime;

        let ops = ProgramBuilder::new()
            .not(0)
            .cnot(0, 1)
            .toffoli(0, 1, 2)
            .fredkin(0, 1, 2)
            .build_ops();

        let mut rt = ReversibleRuntime::new(vec![
            BitPlane::from_words(vec![0xAA]),
            BitPlane::from_words(vec![0xBB]),
            BitPlane::from_words(vec![0xCC]),
        ]);
        let original = rt.registers().to_vec();

        rt.execute_all_tracked(&ops);
        rt.rewind_all().unwrap();
        assert_eq!(rt.registers().to_vec(), original);
    }
}
