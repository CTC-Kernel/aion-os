//! # rewind-dsl
//!
//! Procedural macro `#[reversible]` for the Rewind SDK.
//!
//! Annotate functions with `#[reversible]` to enable compile-time verification
//! and automatic generation of a companion `_reverse` function.
//!
//! ## Generated Code
//!
//! For `#[reversible] fn foo(...)`, generates both `foo` and `foo_reverse`.
//! The reverse function applies inverted operations in reverse order:
//! - `+=` becomes `-=`
//! - `-=` becomes `+=`
//! - `^=` stays `^=` (self-inverse)

use proc_macro::TokenStream;

mod codegen;
mod validate;

/// Marks a function as reversible with compile-time verification.
///
/// Validates the function body and generates a `_reverse` companion.
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
/// // Auto-generates: fn add_xor_reverse(x, y, val) { *y ^= *x; *x -= val; }
/// ```
#[proc_macro_attribute]
pub fn reversible(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    if let Err(errors) = validate::validate_reversible(&input) {
        let mut output = TokenStream::new();
        for err in errors {
            output.extend(TokenStream::from(err.to_compile_error()));
        }
        output.extend(TokenStream::from(quote::quote! { #input }));
        return output;
    }

    let reverse_fn = codegen::generate_reverse(&input);

    TokenStream::from(quote::quote! {
        #input
        #reverse_fn
    })
}
