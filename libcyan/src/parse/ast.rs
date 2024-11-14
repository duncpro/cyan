///! This module defines the AST for the Cyan language.
///!
///! When implementing a new AST node, the following must be true...
///! - The type **must** have `align_of` `AST_ALIGN`!
///! - A test must verify the new type has `align_of` `AST_ALIGN`!
///! - The type **must** be included in `MAX_NODE_SIZE` in `calc_ast_size_upperbound`!

use crate::tok::class::{BinaryOperator, Ident, TokRef};
use crate::util::bump_allocator;
use crate::util::misc::max_of_usizes;

pub const AST_ALIGN: usize = 4;

pub type AstRef<T> = bump_allocator::Handle<T>;

/// Calculates an upperbound for the size in bytes of the AST given the size of the token buffer.
///
/// This function is used to allocate the memory for the AST all at once, immediately prior
/// to syntactic analysis and AST construction. 
pub fn calc_ast_size_upperbound(tok_count: usize) -> usize {
    const MAX_NODE_SIZE: usize = max_of_usizes([
        size_of::<Expr>()
    ]);
    return tok_count * MAX_NODE_SIZE;
}

// -- Expressions --------------------------------------------------------------------------------

/// All expressions in the AST are represented by this type, [`Expr`].
/// Naked expressions such as `IdentExpr`, `InfixExpr`, and all others are **never** 
/// placed directly into the AST. They are **always** wrapped in an [`Expr`].
#[repr(u8)]
pub enum Expr {
    Ident(IdentExpr),
    Infix(InfixExpr)
}

pub struct IdentExpr {
    ident: TokRef<Ident>
}

pub struct InfixExpr {
    left_operand: AstRef<Expr>,
    operator: TokRef<BinaryOperator>,
    right_operand: AstRef<Expr>
}

// -- Tests --------------------------------------------------------------------------------------

#[cfg(test)]
pub mod ast_tests {
    use super::*;
    
    #[test]
    fn very_align_of_4() {
        assert_eq!(4, align_of::<Expr>());
    }
}
