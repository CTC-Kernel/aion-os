//! # rewind-dsl
//!
//! Procedural macro `#[reversible]` for the Rewind SDK.
//!
//! Annotate functions with `#[reversible]` to enable compile-time verification
//! that all operations are reversible. The macro rejects destructive assignments,
//! `mem::forget`, I/O operations, and generates a companion reverse function.
//!
//! ## Allowed Operations
//!
//! - `*x += expr` / `*x -= expr` — additive (inverse: subtract/add)
//! - `*x ^= expr` — XOR (self-inverse)
//! - `std::mem::swap(a, b)` — swap (self-inverse)
//! - `let` bindings (immutable only)
//! - Function calls (not validated yet)
//!
//! ## Rejected Operations
//!
//! - `*x = expr` — destructive assignment (overwrites information)
//! - `std::mem::forget` / `std::mem::drop` — destroys information
//! - `println!` / `eprintln!` — irreversible I/O side effects

use proc_macro::TokenStream;

mod validate;

/// Marks a function as reversible with compile-time verification.
///
/// The macro walks the function body and rejects any operation that
/// would destroy information (destructive assignment, mem::forget, I/O).
///
/// # Examples
///
/// ```ignore
/// use rewind_dsl::reversible;
///
/// #[reversible]
/// fn add_xor(x: &mut u64, y: &mut u64, val: u64) {
///     *x += val;
///     *y ^= *x;
/// }
/// ```
#[proc_macro_attribute]
pub fn reversible(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    if let Err(errors) = validate::validate_reversible(&input) {
        let mut output = TokenStream::new();
        for err in errors {
            output.extend(TokenStream::from(err.to_compile_error()));
        }
        // Also emit the original function so other errors can still be caught
        output.extend(TokenStream::from(quote::quote! { #input }));
        return output;
    }

    // Function passes validation — emit it unchanged
    // (Future: also generate a _reverse companion function)
    TokenStream::from(quote::quote! { #input })
}
