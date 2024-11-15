use std::marker::PhantomData;
use crate::tok::tokbuf::{TokBuf, TokCursor};
use crate::tok::class::{delims, Ident, ItemDeclarator, TokClass, TokRef};
use crate::parse::ast::{self, Ast, AstRef, calc_ast_size_upperbound, AST_ALIGN};
use crate::util::bump_allocator::BumpAllocator;

// -- TokStream ----------------------------------------------------------------------------------

struct TokStream<'a> { cursor: TokCursor<'a> }

impl<'a> TokStream<'a> {
    fn new(tokbuf: &'a TokBuf<'a>) -> Self {
        Self { cursor: TokCursor::new(tokbuf) }
    }
    
    fn consume_ref<C: TokClass>(&mut self) -> Option<TokRef<C>> {
        let tokref = self.cursor.try_make_ref()?;
        self.cursor.advance();
        return Some(tokref);
    }

    fn peek<C: TokClass>(&self) -> Option<C::View<'a>> {
        return self.cursor.try_read_class::<C>();
    }

    /// Consumes the next token in the stream, which is expected to be in class `C`,
    /// and returns a reference to it. If the next token is not in class `C` or the buffer is
    /// empty, panics.
    ///
    /// # Purpose: Dispatch-Then-Claim
    /// This procedure is intended to facilitate the DIspatch-Then-Claim pattern.
    /// A dispatcher procedure maintains a jump table whose key is computed by peeking the next
    /// token in the stream. Then the delegate procedure consumes the token, placing it in the AST.
    fn assert_ref<C: TokClass>(&mut self) -> TokRef<C> {
        let Some(tokref) = self.cursor.try_make_ref() else {
            panic!("Expected token of class {} but next token does not qualify.",
                std::any::type_name::<C>());
        };
        return tokref;
    }

    /// Consumes and discards all tokens up to but not including the next occurence of `C`.
    fn sync<C: TokClass>(&mut self) {
        while let Some(next) = self.cursor.read_tok() {
            if C::classify(&next).is_some() {
                return;
            }
            self.cursor.advance();
        }
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
    let proc_keyword = ctx.stream.assert_ref::<delims::Proc>();
    let Some(name) = ctx.stream.consume_ref::<Ident>() else {
        // TODO: We have to report an error here. Everytime we ParsePanic we must report an error.
        return Err(ParsePanic);
    };
    let parameters = parse_parameters(ctx);
    todo!()
}

fn parse_parameters(ctx: &mut ParseContext) -> ParseResult<ast::Parameters> {
    todo!()
}

fn parse_type(ctx: &mut ParseContext) -> ParseResult<ast::Type> {
    todo!();
}
