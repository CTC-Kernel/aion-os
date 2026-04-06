//! Validation logic for the `#[reversible]` macro.
//!
//! Walks the AST of a function body and rejects irreversible operations.

use syn::visit::Visit;
use syn::{Expr, ExprAssign, ExprCall, ExprMacro, ExprPath, ItemFn, Stmt, StmtMacro};

/// Validates that a function body contains only reversible operations.
///
/// Returns `Ok(())` if all operations are reversible, or a list of errors
/// pointing to the exact tokens that violate reversibility.
pub fn validate_reversible(func: &ItemFn) -> Result<(), Vec<syn::Error>> {
    let mut visitor = ReversibilityChecker { errors: Vec::new() };
    visitor.visit_item_fn(func);

    if visitor.errors.is_empty() {
        Ok(())
    } else {
        Err(visitor.errors)
    }
}

struct ReversibilityChecker {
    errors: Vec<syn::Error>,
}

impl ReversibilityChecker {
    fn check_macro<T: quote::ToTokens>(&mut self, mac: &syn::Macro, span_source: &T) {
        let macro_name = mac.path.segments.last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        match macro_name.as_str() {
            "println" | "eprintln" | "print" | "eprint" => {
                self.errors.push(syn::Error::new_spanned(
                    span_source,
                    format!(
                        "{}! is forbidden in #[reversible] — \
                         I/O is an irreversible side effect. \
                         Move I/O outside the reversible block",
                        macro_name
                    ),
                ));
            }
            "dbg" => {
                self.errors.push(syn::Error::new_spanned(
                    span_source,
                    "dbg! is forbidden in #[reversible] — \
                     it produces I/O side effects. \
                     Use borrow/get for inspection instead",
                ));
            }
            _ => {}
        }
    }
}

impl<'ast> Visit<'ast> for ReversibilityChecker {
    fn visit_expr_assign(&mut self, node: &'ast ExprAssign) {
        // Direct assignment `x = expr` is destructive — it overwrites information.
        // Only compound assignments (+=, -=, ^=) are allowed.
        self.errors.push(syn::Error::new_spanned(
            node,
            "destructive assignment in #[reversible] function — \
             this overwrites information. Use `+=`, `-=`, or `^=` instead",
        ));

        // Still visit children to catch nested issues
        syn::visit::visit_expr_assign(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        // Check for forbidden function calls: mem::forget, mem::drop
        if let Expr::Path(ExprPath { path, .. }) = &*node.func {
            let path_str = path_to_string(path);
            if path_str.contains("mem::forget") || path_str.ends_with("forget") {
                self.errors.push(syn::Error::new_spanned(
                    node,
                    "mem::forget is forbidden in #[reversible] — \
                     it bypasses Drop and destroys information",
                ));
            }
            if path_str.contains("mem::drop") || path_str == "drop" {
                self.errors.push(syn::Error::new_spanned(
                    node,
                    "explicit drop is forbidden in #[reversible] — \
                     it destroys information. Use consume() instead",
                ));
            }
        }

        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_macro(&mut self, node: &'ast ExprMacro) {
        self.check_macro(&node.mac, node);
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_stmt_macro(&mut self, node: &'ast StmtMacro) {
        // Check statement-position macros (println!(...); etc.)
        self.check_macro(&node.mac, node);
        syn::visit::visit_stmt_macro(self, node);
    }

    fn visit_stmt(&mut self, node: &'ast Stmt) {
        syn::visit::visit_stmt(self, node);
    }
}

/// Converts a syn::Path to a string for pattern matching.
fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn allows_compound_assignments() {
        let func: ItemFn = parse_quote! {
            fn add(x: &mut u64, val: u64) {
                *x += val;
                *x -= 1;
                *x ^= 0xFF;
            }
        };
        assert!(validate_reversible(&func).is_ok());
    }

    #[test]
    fn rejects_destructive_assignment() {
        let func: ItemFn = parse_quote! {
            fn bad(x: &mut u64) {
                *x = 42;
            }
        };
        let errors = validate_reversible(&func).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("destructive assignment"));
    }

    #[test]
    fn rejects_mem_forget() {
        let func: ItemFn = parse_quote! {
            fn bad(x: u64) {
                std::mem::forget(x);
            }
        };
        let errors = validate_reversible(&func).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("mem::forget"));
    }

    #[test]
    fn rejects_println() {
        let func: ItemFn = parse_quote! {
            fn bad(x: u64) {
                println!("value: {}", x);
            }
        };
        let errors = validate_reversible(&func).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("I/O"));
    }

    #[test]
    fn rejects_dbg() {
        let func: ItemFn = parse_quote! {
            fn bad(x: u64) {
                dbg!(x);
            }
        };
        let errors = validate_reversible(&func).unwrap_err();
        assert!(errors[0].to_string().contains("dbg!"));
    }

    #[test]
    fn reports_multiple_errors() {
        let func: ItemFn = parse_quote! {
            fn bad(x: &mut u64) {
                *x = 42;
                println!("oops");
            }
        };
        let errors = validate_reversible(&func).unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn allows_let_bindings() {
        let func: ItemFn = parse_quote! {
            fn ok(x: &mut u64) {
                let val = 42u64;
                *x += val;
            }
        };
        assert!(validate_reversible(&func).is_ok());
    }

    #[test]
    fn allows_function_calls() {
        let func: ItemFn = parse_quote! {
            fn ok(x: &mut u64) {
                some_reversible_fn(x);
            }
        };
        assert!(validate_reversible(&func).is_ok());
    }

    #[test]
    fn allows_empty_function() {
        let func: ItemFn = parse_quote! {
            fn noop() {}
        };
        assert!(validate_reversible(&func).is_ok());
    }

    #[test]
    fn rejects_drop_call() {
        let func: ItemFn = parse_quote! {
            fn bad(x: u64) {
                drop(x);
            }
        };
        let errors = validate_reversible(&func).unwrap_err();
        assert!(errors[0].to_string().contains("drop"));
    }
}
