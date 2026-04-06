//! Code generation for the `#[reversible]` macro.
//!
//! Generates a companion `_reverse` function that applies the inverse
//! of each operation in reverse order.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ExprAssign, ItemFn, Stmt};

/// Generates a reverse companion function for a validated reversible function.
///
/// For a function `fn foo(...)`, generates `fn foo_reverse(...)` that applies
/// the inverse operations in reverse order.
pub fn generate_reverse(func: &ItemFn) -> TokenStream {
    let vis = &func.vis;
    let original_name = &func.sig.ident;
    let reverse_name = syn::Ident::new(&format!("{}_reverse", original_name), original_name.span());
    let inputs = &func.sig.inputs;
    let output = &func.sig.output;

    let reversed_stmts = reverse_statements(&func.block.stmts);

    quote! {
        /// Auto-generated reverse function — applies inverse operations in reverse order.
        #vis fn #reverse_name(#inputs) #output {
            #(#reversed_stmts)*
        }
    }
}

/// Reverses statements and inverts compound assignments.
///
/// - `*x += expr` becomes `*x -= expr`
/// - `*x -= expr` becomes `*x += expr`
/// - `*x ^= expr` stays `*x ^= expr` (XOR is self-inverse)
/// - Other statements are kept as-is but in reverse order
fn reverse_statements(stmts: &[Stmt]) -> Vec<Stmt> {
    stmts
        .iter()
        .rev()
        .map(invert_statement)
        .collect()
}

fn invert_statement(stmt: &Stmt) -> Stmt {
    match stmt {
        Stmt::Expr(expr, semi) => Stmt::Expr(invert_expr(expr), *semi),
        other => other.clone(),
    }
}

fn invert_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Assign(ExprAssign { left, right, .. }) => {
            // This shouldn't happen (validation rejects it), but just in case
            Expr::Assign(ExprAssign {
                attrs: vec![],
                left: left.clone(),
                eq_token: syn::token::Eq::default(),
                right: right.clone(),
            })
        }
        Expr::Binary(bin) => {
            use syn::BinOp;
            let new_op = match &bin.op {
                BinOp::AddAssign(_) => BinOp::SubAssign(syn::token::MinusEq::default()),
                BinOp::SubAssign(_) => BinOp::AddAssign(syn::token::PlusEq::default()),
                BinOp::BitXorAssign(_) => BinOp::BitXorAssign(syn::token::CaretEq::default()),
                other => *other, // Keep as-is for unknown ops
            };
            Expr::Binary(syn::ExprBinary {
                attrs: bin.attrs.clone(),
                left: bin.left.clone(),
                op: new_op,
                right: bin.right.clone(),
            })
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn generates_reverse_function_name() {
        let func: ItemFn = parse_quote! {
            fn add_stuff(x: &mut u64) {
                *x += 5;
            }
        };
        let output = generate_reverse(&func);
        let output_str = output.to_string();
        assert!(output_str.contains("add_stuff_reverse"));
    }

    #[test]
    fn reverse_has_same_signature() {
        let func: ItemFn = parse_quote! {
            fn compute(x: &mut u64, y: &mut u64) {
                *x += 1;
                *y ^= *x;
            }
        };
        let output = generate_reverse(&func);
        let output_str = output.to_string();
        assert!(output_str.contains("x : & mut u64"));
        assert!(output_str.contains("y : & mut u64"));
    }

    #[test]
    fn reverses_statement_order() {
        let func: ItemFn = parse_quote! {
            fn two_ops(x: &mut u64, y: &mut u64) {
                *x += 5;
                *y ^= *x;
            }
        };
        let output = generate_reverse(&func);
        let output_str = output.to_string();
        // In the reverse function, y ^= x should come BEFORE x -= 5
        let xor_pos = output_str.find("^=").unwrap();
        let sub_pos = output_str.find("-=").unwrap();
        assert!(
            xor_pos < sub_pos,
            "XOR should come before subtraction in reverse"
        );
    }

    #[test]
    fn inverts_add_to_sub() {
        let func: ItemFn = parse_quote! {
            fn add(x: &mut u64) {
                *x += 42;
            }
        };
        let output = generate_reverse(&func);
        let output_str = output.to_string();
        assert!(output_str.contains("-="), "add should become subtract");
        assert!(
            !output_str.contains("+="),
            "add should not remain in reverse"
        );
    }

    #[test]
    fn xor_stays_xor() {
        let func: ItemFn = parse_quote! {
            fn xor_op(x: &mut u64) {
                *x ^= 0xFF;
            }
        };
        let output = generate_reverse(&func);
        let output_str = output.to_string();
        assert!(
            output_str.contains("^="),
            "XOR should remain XOR (self-inverse)"
        );
    }

    #[test]
    fn inverts_sub_to_add() {
        let func: ItemFn = parse_quote! {
            fn sub(x: &mut u64) {
                *x -= 10;
            }
        };
        let output = generate_reverse(&func);
        let output_str = output.to_string();
        assert!(output_str.contains("+="), "sub should become add");
    }
}
