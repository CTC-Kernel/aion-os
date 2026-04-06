//! # QuantumCell — a linear type that must be consumed exactly once
//!
//! Unlike standard Rust types, a `QuantumCell` cannot be duplicated (`Clone` is forbidden)
//! or silently destroyed (dropping without consumption triggers a panic).
//! This enforces the "Information is Sacred" principle at the type level.
//!
//! # Examples
//!
//! ```
//! use rewind_core::QuantumCell;
//!
//! let cell = QuantumCell::new(42u64);
//! assert_eq!(*cell.get(), 42);

//! let value = cell.consume();
//! assert_eq!(value, 42);
//! ```
//!
//! Dropping without consuming panics:
//!
//! ```should_panic
//! use rewind_core::QuantumCell;
//!
//! let _cell = QuantumCell::new(42u64);
//! // _cell is dropped here without consume() → panic!
//! ```

use std::fmt;
use std::mem::ManuallyDrop;

/// A linear type that must be consumed exactly once.
///
/// `QuantumCell<T>` wraps a value and enforces that it is explicitly consumed
/// via [`consume`](Self::consume). Dropping a `QuantumCell` without consuming it
/// panics with an "information lost" message.
///
/// This mirrors the resource semantics of linear logic: every value is used
/// exactly once — never duplicated, never silently discarded.
///
/// # Examples
///
/// ```
/// use rewind_core::QuantumCell;
///
/// let mut cell = QuantumCell::new(10u64);
/// *cell.get_mut() += 5;
/// assert_eq!(cell.consume(), 15);
/// ```
pub struct QuantumCell<T> {
    value: ManuallyDrop<T>,
    consumed: bool,
}

impl<T> QuantumCell<T> {
    /// Creates a new `QuantumCell` wrapping the given value.
    ///
    /// The cell must eventually be consumed via [`consume`](Self::consume)
    /// or the program will panic when it is dropped.
    pub fn new(value: T) -> Self {
        Self {
            value: ManuallyDrop::new(value),
            consumed: false,
        }
    }

    /// Consumes the cell, returning the inner value.
    ///
    /// This is the only safe way to extract the value. After calling `consume`,
    /// the cell's `Drop` implementation will not panic.
    pub fn consume(mut self) -> T {
        self.consumed = true;
        // SAFETY: We set consumed = true above, so Drop will not attempt to
        // access the value. ManuallyDrop::take moves the value out, and since
        // self will be dropped (with consumed = true), there is no double-free.
        unsafe { ManuallyDrop::take(&mut self.value) }
    }

    /// Returns a reference to the inner value.
    ///
    /// This does not consume the cell — it must still be consumed later.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Returns a mutable reference to the inner value for in-place modification.
    ///
    /// This does not consume the cell — it must still be consumed later.
    /// Use this for reversible in-place operations (e.g., `+=`, `^=`).
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

/// Apply a function to the inner value in-place, without consuming the cell.
///
/// This is the core building block for reversible operations on QuantumCell.
impl<T> QuantumCell<T> {
    /// Maps a function over the inner value in-place.
    ///
    /// ```
    /// use rewind_core::QuantumCell;
    /// let mut cell = QuantumCell::new(10u64);
    /// cell.map_in_place(|v| *v += 5);
    /// assert_eq!(cell.consume(), 15);
    /// ```
    pub fn map_in_place<F: FnOnce(&mut T)>(&mut self, f: F) {
        f(&mut self.value);
    }
}

// Reversible operator implementations for QuantumCell<T>

impl<T: std::ops::AddAssign<U>, U> std::ops::AddAssign<U> for QuantumCell<T> {
    /// Reversible addition: `cell += value`. Inverse: `cell -= value`.
    fn add_assign(&mut self, rhs: U) {
        *self.get_mut() += rhs;
    }
}

impl<T: std::ops::SubAssign<U>, U> std::ops::SubAssign<U> for QuantumCell<T> {
    /// Reversible subtraction: `cell -= value`. Inverse: `cell += value`.
    fn sub_assign(&mut self, rhs: U) {
        *self.get_mut() -= rhs;
    }
}

impl<T: std::ops::BitXorAssign<U>, U> std::ops::BitXorAssign<U> for QuantumCell<T> {
    /// Reversible XOR: `cell ^= value`. Self-inverse.
    fn bitxor_assign(&mut self, rhs: U) {
        *self.get_mut() ^= rhs;
    }
}

impl<T> Drop for QuantumCell<T> {
    fn drop(&mut self) {
        if !self.consumed {
            panic!("QuantumCell dropped without being consumed — information lost");
        }
        // If consumed, value was already taken via ManuallyDrop::take — nothing to drop.
    }
}

impl<T> fmt::Debug for QuantumCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QuantumCell")
            .field("consumed", &self.consumed)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_consume_returns_value() {
        let cell = QuantumCell::new(42u64);
        assert_eq!(cell.consume(), 42);
    }

    #[test]
    #[should_panic(expected = "information lost")]
    fn drop_without_consume_panics() {
        let _cell = QuantumCell::new(42u64);
        // _cell dropped here → panic
    }

    #[test]
    fn get_does_not_consume() {
        let cell = QuantumCell::new(99u64);
        assert_eq!(*cell.get(), 99);
        // Still need to consume
        assert_eq!(cell.consume(), 99);
    }

    #[test]
    fn get_mut_modifies_value() {
        let mut cell = QuantumCell::new(10u64);
        *cell.get_mut() += 5;
        assert_eq!(cell.consume(), 15);
    }

    #[test]
    fn debug_format() {
        let cell = QuantumCell::new(42u64);
        let debug = format!("{:?}", cell);
        assert!(debug.contains("QuantumCell"));
        assert!(debug.contains("consumed: false"));
        cell.consume();
    }

    #[test]
    fn works_with_string() {
        let cell = QuantumCell::new(String::from("hello"));
        assert_eq!(cell.consume(), "hello");
    }

    #[test]
    fn works_with_vec() {
        let cell = QuantumCell::new(vec![1, 2, 3]);
        assert_eq!(cell.consume(), vec![1, 2, 3]);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn consume_returns_original_value_u64(x: u64) {
            let cell = QuantumCell::new(x);
            prop_assert_eq!(cell.consume(), x);
        }

        #[test]
        fn consume_returns_original_value_i32(x: i32) {
            let cell = QuantumCell::new(x);
            prop_assert_eq!(cell.consume(), x);
        }

        #[test]
        fn get_then_consume_preserves_value(x: u64) {
            let cell = QuantumCell::new(x);
            let borrowed = *cell.get();
            let consumed = cell.consume();
            prop_assert_eq!(borrowed, consumed);
        }

        #[test]
        fn get_mut_add_then_consume(x: u64, delta in 0u64..1000) {
            let mut cell = QuantumCell::new(x);
            *cell.get_mut() = x.wrapping_add(delta);
            prop_assert_eq!(cell.consume(), x.wrapping_add(delta));
        }
    }
}
