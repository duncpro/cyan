use crate::tok::tokbuf::{TokBuf, TokCursor};
use crate::tok::class::{TokClass, TokRef, delims, ItemDeclarator};
use crate::parse::ast::{self, Ast, AstRef, calc_ast_size_upperbound, AST_ALIGN};
use crate::util::bump_allocator::BumpAllocator;

// -- TokStream ----------------------------------------------------------------------------------

struct TokStream<'a> { cursor: TokCursor<'a> }

impl<'a> TokStream<'a> {
    fn new(tokbuf: &'a TokBuf<'a>) -> Self {
        Self { cursor: TokCursor::new(tokbuf) }
    }
    
    fn expect<C: TokClass>(&mut self) -> Option<TokRef<C>> {
        todo!()
    }

    fn peek<C: TokClass>(&self) -> Option<C> {
        todo!()
    }

    fn assert<C: TokClass>(&mut self) -> TokRef<C> {
        todo!()
    }

    fn sync<C: TokClass>(&mut self) {
        todo!()
    }
}

// -- Support ------------------------------------------------------------------------------------

struct ParseContext<'a, 'b, 'c> {
    stream: &'a mut TokStream<'b>,
    ast_mem: &'c mut BumpAllocator
}

impl<'a, 'b, 'c> ParseContext<'a, 'b, 'c> {
    fn new(stream: &'a mut TokStream<'b>, ast_mem: &'c mut BumpAllocator) -> Self {
        Self { stream, ast_mem  }
    }
}

struct ParsePanic;

type ParseResult<T> = Result<T, ParsePanic>;

// -- Parser -------------------------------------------------------------------------------------

pub fn parse(tokbuf: &TokBuf) -> Ast {
    let mut stream = TokStream::new(tokbuf);
    let mut mem = BumpAllocator::new(calc_ast_size_upperbound(tokbuf.len()), AST_ALIGN);
    let root = parse_root(&mut ParseContext::new(&mut stream, &mut mem));
    return Ast { mem, root };
}

fn parse_root(ctx: &mut ParseContext) -> ast::Root {
    let mut ll_head: Option<AstRef<ast::TopLevelItemNode>> = None;
    let mut next: &mut Option<AstRef<ast::TopLevelItemNode>> = &mut ll_head;
    
    while ctx.stream.cursor.has_next() {
        let Some(declarator) = ctx.stream.peek::<ItemDeclarator>() else {
            // TODO Error: Expected Top Level Item Declarator
            ctx.stream.sync::<ItemDeclarator>();
            continue;
        };
        let Ok(tl_item) = parse_tl_item(ctx, declarator) else {
            // The unrecoverable error occurred within parse_tl_item. It was reported there.
            ctx.stream.sync::<ItemDeclarator>();
            continue;
        };
        let node_handle = ctx.ast_mem.bump(ast::TopLevelItemNode::new(tl_item));
        *next = Some(node_handle);
        next = unsafe { &mut (*ctx.ast_mem.get_mut(node_handle)).next };
    }

    return ast::Root { ll_head };
}

/// Parses the next top-level item (proc, struct, namespace, etc.).
fn parse_tl_item(ctx: &mut ParseContext, declarator: ItemDeclarator) 
-> ParseResult<ast::TopLevelItem> 
{
    return Ok(match declarator {
        ItemDeclarator::Proc => ast::TopLevelItem::Proc(parse_proc_def(ctx)?),
        ItemDeclarator::Struct => todo!(),
        ItemDeclarator::Enum => todo!(),
        // TODO: We also accepts comments at the top level.
    });
}

fn parse_proc_def(ctx: &mut ParseContext) -> ParseResult<ast::ProcDefinition> {
    let proc_keyword = ctx.stream.assert::<delims::Proc>();
    todo!()
}

