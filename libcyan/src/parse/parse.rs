use std::marker::PhantomData;
use crate::diagnostic::{self, AnyDiagnostic};
use crate::source_unit::SourceUnitId;
use crate::tok::tokbuf::{TokBuf, TokCursor};
use crate::tok::class::{delims, Ident, ItemDeclarator, TokClass, TokRef};
use crate::parse::ast::{self, Ast, AstRef, calc_ast_size_upperbound, AST_ALIGN};
use crate::util::bump_allocator::{BumpAllocator, extend_ll};

// -- TokStream ----------------------------------------------------------------------------------

struct TokStream<'a> { cursor: TokCursor<'a> }

impl<'a> TokStream<'a> {
    fn new(tokbuf: &'a TokBuf<'a>) -> Self {
        Self { cursor: TokCursor::new(tokbuf) }
    }

    /// Consumes and discards all formatting tokens (whitespace, linebreaks, etc). Then, checks if
    /// the next token in the stream is in the token-class `C`. If so, consumes it and returns
    /// a reference to it. Otherwise, returns `None` and doesnt consume it.
    fn consume_ref<C: TokClass>(&mut self) -> Option<TokRef<C>> {
        self.discard_formatting();
        let tokref = self.cursor.try_make_ref()?;
        self.cursor.advance();
        return Some(tokref);
    }

    /// Consumes and discards all formatting tokens (whitespace, linebreaks, etc). Then, checks if
    /// the next token in the stream is in the token-class `C`. If so, returns it, but does not
    /// consume it. Otherwise, returns `None` and doesnt consume it.
    fn peek<C: TokClass>(&mut self) -> Option<C::View<'a>> {
        self.discard_formatting();
        return self.cursor.try_read_class::<C>();
    }

    /// Consumes the next token in the stream (which is asserted to be in class `C`). Then, returns
    /// a reference to it. 
    ///
    /// If the next token is not in class `C` or the buffer is empty, panics.
    ///
    /// This procedure **does not** skip formatting tokens like whitespace and linebreaks.
    ///
    /// # Purpose: Dispatch-Then-Claim
    /// This procedure is intended to facilitate the Dispatch-Then-Claim pattern. A dispatcher
    /// procedure maintains a jump table whose key is computed by peeking the next token in the
    /// stream. Then the delegate procedure consumes the token, placing it in the AST.
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

    fn discard_formatting(&mut self) {
        todo!()
    }
}

// -- Support ------------------------------------------------------------------------------------

/// A `ParsePanic` is thrown when an unexpected token sequence is encountered.
/// Important: The code that constructs a `ParsePanic` is **also responsible for** pushing
/// a `Diagnostic`. 
///
/// Oppositely, when a caller encounters a `ParsePanic`, they can rest-assured the error
/// has already been reported within a diagnostic. 
struct ParsePanic;

type ParseResult<T> = Result<T, ParsePanic>;

struct ParseContext<'a, 'b> {
    stream: &'a mut TokStream<'b>,
    ast_mem: &'a mut BumpAllocator,
    source_unit: SourceUnitId,
    diagnostics: &'a mut Vec<AnyDiagnostic>,
}

impl<'a, 'b> ParseContext<'a, 'b> {
    fn new(stream: &'a mut TokStream<'b>, ast_mem: &'a mut BumpAllocator, source_unit: SourceUnitId,
        diagnostics: &'a mut Vec<AnyDiagnostic>) -> Self 
    {
        Self { stream, ast_mem, source_unit, diagnostics }
    }

    fn expect_ref<C: TokClass>(&mut self) -> ParseResult<TokRef<C>> {    
        if let Some(tokref) = self.stream.consume_ref::<C>() { return Ok(tokref); };
        let diagnostic = diagnostic::MissingTok::new(self.source_unit, self.stream.cursor.at());
        self.diagnostics.push(AnyDiagnostic::MissingTok(diagnostic));
        return Err(ParsePanic);
    }
}

// -- Parser -------------------------------------------------------------------------------------

pub fn parse(tokbuf: &TokBuf, source_unit: SourceUnitId, diagnostics: &mut Vec<AnyDiagnostic>)  
-> Ast 
{
    let mut stream = TokStream::new(tokbuf);
    let mut mem = BumpAllocator::new(calc_ast_size_upperbound(tokbuf.len()), AST_ALIGN);
    let root = parse_root(&mut ParseContext::new(&mut stream, &mut mem, source_unit, diagnostics));
    return Ast { mem, root };
}

fn parse_root(ctx: &mut ParseContext) -> ast::Root {
    let mut ll_head: Option<AstRef<ast::TopLevelItemNode>> = None;
    let mut next: &mut Option<AstRef<ast::TopLevelItemNode>> = &mut ll_head;
    
    while ctx.stream.cursor.has_next() {
        /// A source unit is a list of top level items.
        /// Every top level item begins with an `ItemDeclarator`.
        let Some(declarator) = ctx.stream.peek::<ItemDeclarator>() else {
            let diagnostic = diagnostic::MissingTok::new(ctx.source_unit, ctx.stream.cursor.at());
            ctx.diagnostics.push(AnyDiagnostic::MissingTok(diagnostic));
            ctx.stream.sync::<ItemDeclarator>();
            continue;
        };
        let Ok(tl_item) = parse_tl_item(ctx, declarator) else {
            // The panic occurred within parse_tl_item. It was reported there.
            ctx.stream.sync::<ItemDeclarator>();
            continue;
        };
        extend_ll(ctx.ast_mem, &mut next, ast::TopLevelItemNode::new(tl_item));
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
    let ident = ctx.expect_ref::<Ident>()?;
    let parameters = parse_parameters(ctx);
    todo!()
}

fn parse_parameters(ctx: &mut ParseContext) -> ParseResult<ast::Parameters> {
    todo!()
}

fn parse_type(ctx: &mut ParseContext) -> ParseResult<ast::Type> {
    todo!();
}
