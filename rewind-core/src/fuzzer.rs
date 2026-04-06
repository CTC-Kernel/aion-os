//! Program fuzzing utilities — generate random reversible programs for testing.
//!
//! # Examples
//!
//! ```
//! use rewind_core::fuzzer::ProgramGenerator;
//! use rewind_core::runtime::ReversibleRuntime;
//! use rewind_core::bitplane::BitPlane;
//!
//! let pg = ProgramGenerator::new(4, 42); // 4 registers, seed 42
//! let ops = pg.generate(20); // 20 random operations
//!
//! let mut rt = ReversibleRuntime::new(vec![
//!     BitPlane::from_words(vec![0xAA]),
//!     BitPlane::from_words(vec![0xBB]),
//!     BitPlane::from_words(vec![0xCC]),
//!     BitPlane::from_words(vec![0xDD]),
//! ]);
//! let original = rt.registers().to_vec();
//! rt.execute_all_tracked(&ops);
//! rt.rewind_all().unwrap();
//! assert_eq!(rt.registers().to_vec(), original); // Always reversible!
//! ```

use crate::engine::Op;

/// Generates random reversible programs for testing and fuzzing.
#[derive(Debug)]
pub struct ProgramGenerator {
    num_registers: usize,
    seed: u64,
}

impl ProgramGenerator {
    /// Creates a new generator with the given number of registers and seed.
    pub fn new(num_registers: usize, seed: u64) -> Self {
        assert!(num_registers >= 2, "Need at least 2 registers");
        Self {
            num_registers,
            seed,
        }
    }

    /// Generates a program with the given number of random operations.
    pub fn generate(&self, num_ops: usize) -> Vec<Op> {
        let mut ops = Vec::with_capacity(num_ops);
        let mut rng = self.seed;

        for _ in 0..num_ops {
            rng = lcg_next(rng);
            let op = self.random_op(&mut rng);
            ops.push(op);
        }
        ops
    }

    /// Generates a single random operation.
    fn random_op(&self, rng: &mut u64) -> Op {
        *rng = lcg_next(*rng);
        let kind = *rng % 4;
        *rng = lcg_next(*rng);

        match kind {
            0 => Op::Not((*rng as usize) % self.num_registers),
            1 => {
                let ctrl = (*rng as usize) % self.num_registers;
                *rng = lcg_next(*rng);
                let tgt =
                    (ctrl + 1 + (*rng as usize) % (self.num_registers - 1)) % self.num_registers;
                Op::Cnot {
                    control: ctrl,
                    target: tgt,
                }
            }
            2 => {
                let c1 = (*rng as usize) % self.num_registers;
                *rng = lcg_next(*rng);
                let c2 = (c1 + 1 + (*rng as usize) % (self.num_registers - 1)) % self.num_registers;
                *rng = lcg_next(*rng);
                let mut tgt = (*rng as usize) % self.num_registers;
                while tgt == c1 || tgt == c2 {
                    *rng = lcg_next(*rng);
                    tgt = (*rng as usize) % self.num_registers;
                }
                Op::Toffoli {
                    c1,
                    c2,
                    target: tgt,
                }
            }
            _ => {
                let ctrl = (*rng as usize) % self.num_registers;
                *rng = lcg_next(*rng);
                let a =
                    (ctrl + 1 + (*rng as usize) % (self.num_registers - 1)) % self.num_registers;
                *rng = lcg_next(*rng);
                let mut b = (*rng as usize) % self.num_registers;
                while b == ctrl || b == a {
                    *rng = lcg_next(*rng);
                    b = (*rng as usize) % self.num_registers;
                }
                Op::Fredkin {
                    control: ctrl,
                    a,
                    b,
                }
            }
        }
    }
}

/// Simple LCG random number generator (deterministic, no dependencies).
fn lcg_next(state: u64) -> u64 {
    state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
}

/// Validates that a program's register indices are within bounds.
///
/// Returns `Ok(())` if valid, or a list of (op_index, error_message) pairs.
pub fn validate_program(ops: &[Op], num_registers: usize) -> Result<(), Vec<(usize, String)>> {
    let mut errors = Vec::new();

    for (i, op) in ops.iter().enumerate() {
        let check = |idx: usize, name: &str| -> Option<String> {
            if idx >= num_registers {
                Some(format!(
                    "{name} index {idx} out of bounds (max: {})",
                    num_registers - 1
                ))
            } else {
                None
            }
        };

        match op {
            Op::Not(r) => {
                if let Some(e) = check(*r, "register") {
                    errors.push((i, e));
                }
            }
            Op::Cnot { control, target } => {
                if let Some(e) = check(*control, "control") {
                    errors.push((i, e));
                }
                if let Some(e) = check(*target, "target") {
                    errors.push((i, e));
                }
                if control == target {
                    errors.push((i, "CNOT control == target (no-op)".to_string()));
                }
            }
            Op::Toffoli { c1, c2, target } => {
                if let Some(e) = check(*c1, "c1") {
                    errors.push((i, e));
                }
                if let Some(e) = check(*c2, "c2") {
                    errors.push((i, e));
                }
                if let Some(e) = check(*target, "target") {
                    errors.push((i, e));
                }
            }
            Op::Fredkin { control, a, b } => {
                if let Some(e) = check(*control, "control") {
                    errors.push((i, e));
                }
                if let Some(e) = check(*a, "a") {
                    errors.push((i, e));
                }
                if let Some(e) = check(*b, "b") {
                    errors.push((i, e));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitplane::BitPlane;
    use crate::runtime::ReversibleRuntime;

    #[test]
    fn generator_produces_correct_count() {
        let pg = ProgramGenerator::new(4, 42);
        let ops = pg.generate(10);
        assert_eq!(ops.len(), 10);
    }

    #[test]
    fn generator_produces_valid_indices() {
        let pg = ProgramGenerator::new(3, 12345);
        let ops = pg.generate(100);
        assert!(validate_program(&ops, 3).is_ok());
    }

    #[test]
    fn generated_programs_are_always_reversible() {
        for seed in 0..20 {
            let pg = ProgramGenerator::new(4, seed);
            let ops = pg.generate(30);

            let mut rt = ReversibleRuntime::new(vec![
                BitPlane::from_words(vec![0xAA + seed]),
                BitPlane::from_words(vec![0xBB + seed]),
                BitPlane::from_words(vec![0xCC + seed]),
                BitPlane::from_words(vec![0xDD + seed]),
            ]);
            let original = rt.registers().to_vec();

            rt.execute_all_tracked(&ops);
            rt.rewind_all().unwrap();
            assert_eq!(rt.registers().to_vec(), original, "Failed for seed {seed}");
        }
    }

    #[test]
    fn validate_catches_out_of_bounds() {
        let ops = vec![Op::Not(5)]; // Register 5 doesn't exist with 3 registers
        let result = validate_program(&ops, 3);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 1);
    }

    #[test]
    fn validate_catches_cnot_self_loop() {
        let ops = vec![Op::Cnot {
            control: 0,
            target: 0,
        }];
        let result = validate_program(&ops, 3);
        assert!(result.is_err());
    }

    #[test]
    fn validate_passes_valid_program() {
        let ops = vec![
            Op::Not(0),
            Op::Cnot {
                control: 0,
                target: 1,
            },
            Op::Toffoli {
                c1: 0,
                c2: 1,
                target: 2,
            },
            Op::Fredkin {
                control: 0,
                a: 1,
                b: 2,
            },
        ];
        assert!(validate_program(&ops, 3).is_ok());
    }

    #[test]
    fn different_seeds_produce_different_programs() {
        let pg1 = ProgramGenerator::new(4, 1);
        let pg2 = ProgramGenerator::new(4, 2);
        let ops1 = pg1.generate(10);
        let ops2 = pg2.generate(10);
        // Very unlikely to be identical
        let s1 = ops1
            .iter()
            .map(|o| format!("{o}"))
            .collect::<Vec<_>>()
            .join(",");
        let s2 = ops2
            .iter()
            .map(|o| format!("{o}"))
            .collect::<Vec<_>>()
            .join(",");
        assert_ne!(s1, s2);
    }
}
