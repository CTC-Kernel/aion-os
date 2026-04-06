//! Ancilla mirror stack — LIFO storage for intermediate computation states.
//!
//! During forward execution, ancilla bits are pushed onto the stack.
//! During backward execution (uncomputation), they are popped and verified.
