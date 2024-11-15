use crate::tok::tokbuf::{TokBuf, TokCursor};
use crate::tok::class::{TokClass, TokRef, delims, ItemDeclarator};
use crate::parse::ast::{self, Ast, AstRef, calc_ast_size_upperbound, AST_ALIGN};
use crate::util::bump_allocator::BumpAllocator;

// -- Support ------------------------------------------------------------------------------------

struct TokStream<'a> { cursor: TokCursor<'a> }

impl<'a> TokStream<'a> {
    fn new(tokbuf: &'a TokBuf<'a>) -> Self {
        Self { cursor: TokCursor::new(tokbuf) }
    }
    
    fn capture<C: TokClass>(&mut self) -> Option<TokRef<C>> {
        todo!()
    }

    fn assert<C: TokClass>(&mut self) -> TokRef<C> {
        todo!()
    }
}


struct ParseContext<'a, 'b, 'c> {
    stream: &'a mut TokStream<'b>,
    ast_mem: &'c mut BumpAllocator
}

impl<'a, 'b, 'c> ParseContext<'a, 'b, 'c> {
    fn new(stream: &'a mut TokStream<'b>, ast_mem: &'c mut BumpAllocator) -> Self {
        Self { stream, ast_mem  }
    }
}

// -- Parser -------------------------------------------------------------------------------------

pub fn parse(tokbuf: &TokBuf) -> Ast {
    let mut stream = TokStream::new(tokbuf);
    let mut mem = BumpAllocator::new(calc_ast_size_upperbound(tokbuf.len()), AST_ALIGN);
    
    let mut ll_head: Option<AstRef<ast::TopLevelItemNode>> = None;
    let mut next: &mut Option<AstRef<ast::TopLevelItemNode>> = &mut ll_head;
    
    while stream.cursor.has_next() {
        parse_tl_item(&mut ParseContext::new(&mut stream, &mut mem));
    }

    return Ast { mem, root: ast::Root { ll_head } };
}

/// Parses the next top-level item (proc, struct, namespace, etc.).
fn parse_tl_item(ctx: &mut ParseContext) -> AstRef<ast::TopLevelItemNode> {
    if let Some(declarator) = ctx.stream.capture::<ItemDeclarator>() {

    }
    todo!()
}

fn parse_proc_def(ctx: &mut ParseContext) -> ast::ProcDefinition {
    todo!()
}
