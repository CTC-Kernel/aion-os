//! Program serialization and analysis utilities.
//!
//! Provides tools for saving, loading, analyzing, and optimizing
//! reversible programs.
//!
//! # Examples
//!
//! ```
//! use rewind_core::engine::Op;
//! use rewind_core::program::ProgramInfo;
//!
//! let ops = vec![
//!     Op::Not(0),
//!     Op::Cnot { control: 0, target: 1 },
//!     Op::Toffoli { c1: 0, c2: 1, target: 2 },
//! ];
//! let info = ProgramInfo::analyze(&ops);
//! assert_eq!(info.total_ops, 3);
//! assert_eq!(info.gate_counts.not_count, 1);
//! assert_eq!(info.gate_counts.cnot_count, 1);
//! assert_eq!(info.gate_counts.toffoli_count, 1);
//! ```

use crate::engine::Op;
use std::collections::HashSet;

/// Gate count breakdown for a program.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GateCounts {
    pub not_count: usize,
    pub cnot_count: usize,
    pub toffoli_count: usize,
}

impl GateCounts {
    /// Total number of gates.
    pub fn total(&self) -> usize {
        self.not_count + self.cnot_count + self.toffoli_count
    }
}

impl std::fmt::Display for GateCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NOT: {}, CNOT: {}, Toffoli: {}",
            self.not_count, self.cnot_count, self.toffoli_count
        )
    }
}

/// Analysis information about a reversible program.
#[derive(Debug, Clone)]
pub struct ProgramInfo {
    /// Total number of operations.
    pub total_ops: usize,
    /// Gate count breakdown.
    pub gate_counts: GateCounts,
    /// Set of register indices used.
    pub registers_used: HashSet<usize>,
    /// Maximum register index referenced.
    pub max_register: usize,
    /// Estimated circuit depth (sequential operations on same register).
    pub depth: usize,
}

impl ProgramInfo {
    /// Analyzes a sequence of operations and returns program information.
    pub fn analyze(ops: &[Op]) -> Self {
        let mut gate_counts = GateCounts::default();
        let mut registers_used = HashSet::new();
        let mut max_register = 0usize;

        for op in ops {
            match op {
                Op::Not(i) => {
                    gate_counts.not_count += 1;
                    registers_used.insert(*i);
                    max_register = max_register.max(*i);
                }
                Op::Cnot { control, target } => {
                    gate_counts.cnot_count += 1;
                    registers_used.insert(*control);
                    registers_used.insert(*target);
                    max_register = max_register.max(*control).max(*target);
                }
                Op::Toffoli { c1, c2, target } => {
                    gate_counts.toffoli_count += 1;
                    registers_used.insert(*c1);
                    registers_used.insert(*c2);
                    registers_used.insert(*target);
                    max_register = max_register.max(*c1).max(*c2).max(*target);
                }
            }
        }

        // Simple depth estimation: count max ops on any single register
        let depth = if registers_used.is_empty() {
            0
        } else {
            let mut per_reg = vec![0usize; max_register + 1];
            for op in ops {
                match op {
                    Op::Not(i) => per_reg[*i] += 1,
                    Op::Cnot { control, target } => {
                        per_reg[*control] += 1;
                        per_reg[*target] += 1;
                    }
                    Op::Toffoli { c1, c2, target } => {
                        per_reg[*c1] += 1;
                        per_reg[*c2] += 1;
                        per_reg[*target] += 1;
                    }
                }
            }
            per_reg.into_iter().max().unwrap_or(0)
        };

        Self {
            total_ops: ops.len(),
            gate_counts,
            registers_used,
            max_register,
            depth,
        }
    }

    /// Returns the minimum number of registers required.
    pub fn min_registers(&self) -> usize {
        if self.registers_used.is_empty() {
            0
        } else {
            self.max_register + 1
        }
    }
}

impl std::fmt::Display for ProgramInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Program: {} ops ({}), {} registers, depth {}",
            self.total_ops,
            self.gate_counts,
            self.registers_used.len(),
            self.depth
        )
    }
}

/// Serializes a program to a human-readable text format.
///
/// Format: one operation per line.
/// ```text
/// NOT 0
/// CNOT 0 1
/// TOFF 0 1 2
/// ```
pub fn serialize_text(ops: &[Op]) -> String {
    ops.iter()
        .map(|op| match op {
            Op::Not(i) => format!("NOT {i}"),
            Op::Cnot { control, target } => format!("CNOT {control} {target}"),
            Op::Toffoli { c1, c2, target } => format!("TOFF {c1} {c2} {target}"),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Deserializes a program from the text format.
///
/// Returns `Err` with the line number and content if parsing fails.
pub fn deserialize_text(text: &str) -> Result<Vec<Op>, (usize, String)> {
    let mut ops = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue; // Skip empty lines and comments
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let op = match parts.first() {
            Some(&"NOT") if parts.len() == 2 => {
                let idx = parts[1]
                    .parse::<usize>()
                    .map_err(|_| (i, line.to_string()))?;
                Op::Not(idx)
            }
            Some(&"CNOT") if parts.len() == 3 => {
                let control = parts[1]
                    .parse::<usize>()
                    .map_err(|_| (i, line.to_string()))?;
                let target = parts[2]
                    .parse::<usize>()
                    .map_err(|_| (i, line.to_string()))?;
                Op::Cnot { control, target }
            }
            Some(&"TOFF") if parts.len() == 4 => {
                let c1 = parts[1]
                    .parse::<usize>()
                    .map_err(|_| (i, line.to_string()))?;
                let c2 = parts[2]
                    .parse::<usize>()
                    .map_err(|_| (i, line.to_string()))?;
                let target = parts[3]
                    .parse::<usize>()
                    .map_err(|_| (i, line.to_string()))?;
                Op::Toffoli { c1, c2, target }
            }
            _ => return Err((i, line.to_string())),
        };
        ops.push(op);
    }
    Ok(ops)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_empty_program() {
        let info = ProgramInfo::analyze(&[]);
        assert_eq!(info.total_ops, 0);
        assert_eq!(info.gate_counts.total(), 0);
        assert_eq!(info.depth, 0);
    }

    #[test]
    fn analyze_mixed_program() {
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
            Op::Not(2),
        ];
        let info = ProgramInfo::analyze(&ops);
        assert_eq!(info.total_ops, 4);
        assert_eq!(info.gate_counts.not_count, 2);
        assert_eq!(info.gate_counts.cnot_count, 1);
        assert_eq!(info.gate_counts.toffoli_count, 1);
        assert_eq!(info.min_registers(), 3);
    }

    #[test]
    fn serialize_roundtrip() {
        let ops = vec![
            Op::Not(0),
            Op::Cnot {
                control: 1,
                target: 2,
            },
            Op::Toffoli {
                c1: 0,
                c2: 1,
                target: 3,
            },
        ];
        let text = serialize_text(&ops);
        let parsed = deserialize_text(&text).unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(serialize_text(&parsed), text);
    }

    #[test]
    fn deserialize_with_comments_and_blanks() {
        let text = "# This is a comment\nNOT 0\n\nCNOT 0 1\n# Another comment\nTOFF 0 1 2";
        let ops = deserialize_text(text).unwrap();
        assert_eq!(ops.len(), 3);
    }

    #[test]
    fn deserialize_invalid_line() {
        let result = deserialize_text("INVALID 1 2 3");
        assert!(result.is_err());
    }

    #[test]
    fn program_info_display() {
        let ops = vec![Op::Not(0), Op::Not(1)];
        let info = ProgramInfo::analyze(&ops);
        let display = format!("{info}");
        assert!(display.contains("2 ops"));
        assert!(display.contains("NOT: 2"));
    }

    #[test]
    fn serialize_text_format() {
        let ops = vec![
            Op::Not(5),
            Op::Cnot {
                control: 3,
                target: 7,
            },
        ];
        let text = serialize_text(&ops);
        assert_eq!(text, "NOT 5\nCNOT 3 7");
    }
}
