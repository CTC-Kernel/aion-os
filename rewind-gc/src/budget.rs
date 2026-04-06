//! Memory budget configuration for the ancilla stack.
//!
//! Prevents unbounded memory growth by enforcing configurable limits
//! on the total ancilla storage, triggering forced uncomputation when needed.
