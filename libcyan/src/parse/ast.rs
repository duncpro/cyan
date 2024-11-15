///! This module defines the AST for the Cyan language.
///!
///! When implementing a new AST `Node`, the following must be true...
///! - The type **must** have `align_of` `AST_ALIGN`!
///! - A test must verify the new type has `align_of` `AST_ALIGN`!
///! - The type **must** be included in `MAX_NODE_SIZE` in `calc_ast_size_upperbound`!

use crate::tok::class::{delims, BinaryOperator, Ident, Literal, TokRef};
use crate::util::bump_allocator::{self, BumpAllocator, LLNode};
use crate::util::misc::max_of_usizes;

// -- Support ------------------------------------------------------------------------------------

pub const AST_ALIGN: usize = 4;

pub type AstRef<T> = bump_allocator::Handle<T>;

/// Calculates an upperbound for the size in bytes of the AST given the size of the token buffer.
///
/// This function is used to allocate the memory for the AST all at once, immediately prior
/// to syntactic analysis and AST construction. 
pub fn calc_ast_size_upperbound(tok_count: usize) -> usize {
    const MAX_NODE_SIZE: usize = max_of_usizes([
        size_of::<ExprNode>(),
        size_of::<ParameterNode>(),
        size_of::<BlockStatementNode>(),
        size_of::<TopLevelItemNode>(),
    ]);
    return tok_count * MAX_NODE_SIZE;
}

pub struct Ast { pub mem: BumpAllocator, pub root: Root }

// -- Root --------------------------------------------------------------------------------------

pub struct Root {
    pub ll_head: Option<AstRef<TopLevelItemNode>>
}

#[repr(u8)]
pub enum TopLevelItem {
    Proc(ProcDefinition)
}

pub struct TopLevelItemNode {
    pub item: TopLevelItem,
    pub next: Option<AstRef<Self>>
}

impl TopLevelItemNode {
    pub fn new(item: TopLevelItem) -> Self { return Self { item, next: None  }; }
}

impl LLNode for TopLevelItemNode {
    fn next_mut(&mut self) -> &mut Option<AstRef<Self>> {
        return &mut self.next;
    }
}

// -- Expressions --------------------------------------------------------------------------------

#[repr(u8)]
pub enum ExprNode {
    Ident(IdentExpr),
    Infix(InfixExpr),
    Literal(LiteralExpr)
}

pub struct IdentExpr {
    ident: TokRef<Ident>
}

pub struct InfixExpr {
    left_operand: AstRef<ExprNode>,
    operator: TokRef<BinaryOperator>,
    right_operand: AstRef<ExprNode>
}

pub struct LiteralExpr {
    tok: TokRef<Literal>
}

// -- Types -------------------------------------------------------------------------------------

#[repr(u8)]
pub enum Type {
    NamedType(NamedType)
}

pub struct NamedType { name: TokRef<Ident> }

// -- Procedure Definition ----------------------------------------------------------------------

pub struct ProcDefinition {
    proc_keyword: TokRef<delims::Proc>,
    proc_name: TokRef<Ident>,
    return_type: Option<Type>,
    parameters: Parameters,
    body: Block
}

pub struct Parameters {    
    open_paren: TokRef<delims::OpenParen>,
    close_paren: TokRef<delims::CloseParen>,
    first: Option<AstRef<ParameterNode>>
}

pub struct ParameterNode {
    ident: TokRef<Ident>,
    colon: TokRef<delims::Colon>,
    ty: Type,
    next: Option<AstRef<ParameterNode>>,
    comma: Option<TokRef<delims::Comma>>
}

// -- Procedure Invocation -----------------------------------------------------------------------


// -- Statements ---------------------------------------------------------------------------------

pub struct Block {
    open_curly: TokRef<delims::OpenCurly>,
    close_curly: TokRef<delims::CloseCurly>,
    first_statement: Option<AstRef<BlockStatementNode>>,
}

pub struct BlockStatementNode {
    next: Option<AstRef<BlockStatementNode>>
}


// -- Tests --------------------------------------------------------------------------------------

#[cfg(test)]
pub mod ast_tests {
    use super::*;
    
    #[test]
    fn verify_align() {
        assert_eq!(AST_ALIGN, align_of::<ExprNode>());
        assert_eq!(AST_ALIGN, align_of::<ParameterNode>());
        assert_eq!(AST_ALIGN, align_of::<BlockStatementNode>());
        assert_eq!(AST_ALIGN, align_of::<TopLevelItemNode>());
    }
}
