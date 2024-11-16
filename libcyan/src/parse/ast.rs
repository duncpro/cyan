///! This module defines the AST for the Cyan language.
///!
///! When implementing a new AST `Node`, the following must be true...
///! - The type **must** have `align_of` `AST_ALIGN`!
///! - The type **must** be included in `MAX_NODE_SIZE` in `calc_ast_size_upperbound`!

use crate::tok;
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
        size_of::<StatementNode>(),
        size_of::<TopLevelItemNode>(),
        size_of::<TypeArgumentNode>()
    ]);
    return tok_count * MAX_NODE_SIZE;
}

pub struct Ast { pub mem: BumpAllocator<AST_ALIGN>, pub root: Root }

// -- Root --------------------------------------------------------------------------------------

pub struct Root {
    pub ll_head: Option<AstRef<LLNode<AnyTopLevelItem>>>
}

#[repr(u8)]
pub enum AnyTopLevelItem {
    Proc(ProcDefinition),
    LineComment(LineComment)
}

pub type TopLevelItemNode = LLNode<AnyTopLevelItem>;

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

pub struct NamedType { 
    pub ident: TokRef<Ident>,
    pub arguments: Option<TypeArguments>
}

pub struct TypeArguments {
    pub open_angle: TokRef<delims::LessThan>,
    pub first: Option<AstRef<TypeArgumentNode>>,
    pub close_angle: TokRef<delims::GreaterThan>,
}

pub struct TypeArgument { pub ty: Type, pub comma: Option<TokRef<delims::Comma>> }

pub type TypeArgumentNode = LLNode<TypeArgument>;

// -- Procedure Definition ----------------------------------------------------------------------

pub struct ProcDefinition {
    pub proc_keyword: TokRef<delims::Proc>,
    pub ident: TokRef<Ident>,
    pub parameters: Parameters,
    pub return_type_separator: TokRef<delims::Colon>,
    pub return_type: Type,
    pub body: ImperativeBlock
}

pub struct Parameters {    
    pub open_paren: TokRef<delims::OpenParen>,
    pub close_paren: TokRef<delims::CloseParen>,
    pub first: Option<AstRef<ParameterNode>>
}

pub struct Parameter {
    pub ident: TokRef<Ident>,
    pub colon: TokRef<delims::Colon>,
    pub ty: Type,
    pub comma: Option<TokRef<delims::Comma>>,    
}

pub type ParameterNode = LLNode<Parameter>;

// -- Procedure Invocation -----------------------------------------------------------------------


// -- Statements ---------------------------------------------------------------------------------

pub struct ImperativeBlock {
    pub open_curly: TokRef<delims::OpenCurly>,
    pub close_curly: TokRef<delims::CloseCurly>,
    pub first: Option<AstRef<StatementNode>>,
}

#[repr(u8)]
pub enum AnyStatement {
    LineComment
}

pub type StatementNode = LLNode<AnyStatement>;

// -- Line Comment -------------------------------------------------------------------------------

pub struct LineComment {
    pub tok: TokRef<tok::class::LineComment>
}

