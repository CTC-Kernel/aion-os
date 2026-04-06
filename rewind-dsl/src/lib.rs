//! # rewind-dsl
//!
//! Procedural macro `#[reversible]` for the Rewind SDK.
//!
//! Annotate functions with `#[reversible]` to enable compile-time verification
//! that all operations are reversible. The macro rejects destructive assignments,
//! `mem::forget`, I/O operations, and automatically generates inverse code.
//!
//! ## Example
//!
//! ```ignore
//! #[reversible]
//! fn add_xor(x: &mut u64, y: &mut u64, val: u64) {
//!     *x += val;
//!     *y ^= *x;
//! }
//! ```

use proc_macro::TokenStream;

/// Marks a function as reversible, enabling compile-time verification.
///
/// Currently a pass-through placeholder — full validation and inverse
/// code generation will be implemented in Story 4.1-4.3.
#[proc_macro_attribute]
pub fn reversible(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Pass-through for now — validation comes in Epic 4
    item
}
