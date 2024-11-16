use std::marker::PhantomData;
use crate::diagnostic::{self, AnyDiagnostic};
use crate::source_unit::SourceUnitId;
use crate::tok;
use crate::tok::tokbuf::{TokBuf, TokCursor};
use crate::tok::class::{delims, TokClass, TokRef};
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
        self.discard::<tok::class::Formatting>();
        let tokref = self.cursor.match_ref()?;
        self.cursor.advance();
        return Some(tokref);
    }

    /// Consumes and discards all formatting tokens (whitespace, linebreaks, etc). Then, checks if
    /// the next token in the stream is in the token-class `C`. If so, returns it, but does not
    /// consume it. Otherwise, returns `None` and doesnt consume it.
    fn peek<C: TokClass>(&mut self) -> Option<C::View<'a>> {
        self.discard::<tok::class::Formatting>();
        return self.cursor.r#match::<C>();
    }

    /// Consumes the next token in the stream (which is asserted to be in class `C`). Then, returns
    /// a reference to it. If the next token is not in class `C` or the buffer is empty, panics.
    /// Unlike many other matchers, this procedure **does not** ignore formatting tokens like 
    /// whitespace and linebreaks.
    ///
    /// # Purpose: Dispatch-Then-Claim
    /// This procedure is intended to facilitate the Dispatch-Then-Claim pattern. A dispatcher
    /// procedure maintains a jump table whose key is computed by peeking the next token in the
    /// stream. Then the delegate procedure consumes the token, placing it in the AST.
    fn assert_ref<C: TokClass>(&mut self) -> TokRef<C> {
        let Some(tokref) = self.cursor.match_ref() else {
            panic!("Expected token of class {} but next token does not qualify.",
                std::any::type_name::<C>());
        };
        self.cursor.advance();
        return tokref;
    }

    /// Consumes and discards all tokens up to but not including the next occurrence of `C`.
    fn sync<C: TokClass>(&mut self) {
        while let Some(next) = self.cursor.read_tok() {
            if C::r#match(&next).is_some() {
                return;
            }
            self.cursor.advance();
        }
    }

    /// Consumes and discards the sequence of consecutive tokens matching the class `C`.
    fn discard<C: TokClass>(&mut self) {
        while self.cursor.r#match::<C>().is_some() {
            self.cursor.advance();
        }
    }
}

// -- Support ------------------------------------------------------------------------------------

type AstAllocator = BumpAllocator<AST_ALIGN>;

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
    ast_mem: &'a mut AstAllocator,
    source_unit: SourceUnitId,
    diagnostics: &'a mut Vec<AnyDiagnostic>,
}

impl<'a, 'b> ParseContext<'a, 'b> {
    fn new(stream: &'a mut TokStream<'b>, ast_mem: &'a mut AstAllocator, source_unit: SourceUnitId,
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
    let mut mem = AstAllocator::new(calc_ast_size_upperbound(tokbuf.len()));
    let root = parse_root(&mut ParseContext::new(&mut stream, &mut mem, source_unit, diagnostics));
    mem.shrink_to_fit();
    return Ast { mem, root };
}

fn parse_root(ctx: &mut ParseContext) -> ast::Root {
    let mut ll_head: Option<AstRef<ast::TopLevelItemNode>> = None;
    let mut next: &mut Option<AstRef<ast::TopLevelItemNode>> = &mut ll_head;
    
    while ctx.stream.cursor.has_next() {
        /// A source unit is a list of top level items.
        /// Every top level item begins with an `ItemDeclarator`.
        let Some(declarator) = ctx.stream.peek::<tok::class::ItemDeclarator>() else {
            let diagnostic = diagnostic::MissingTok::new(ctx.source_unit, ctx.stream.cursor.at());
            ctx.diagnostics.push(AnyDiagnostic::MissingTok(diagnostic));
            ctx.stream.sync::<tok::class::ItemDeclarator>();
            continue;
        };
        let Ok(tl_item) = parse_tl_item(ctx, declarator) else {
            // The panic occurred within parse_tl_item. It was reported there.
            ctx.stream.sync::<tok::class::ItemDeclarator>();
            continue;
        };
        extend_ll(ctx.ast_mem, &mut next, tl_item);
    }

    return ast::Root { ll_head };
}

/// Parses the next top-level item (proc, struct, namespace, etc.).
fn parse_tl_item(ctx: &mut ParseContext, declarator: tok::class::ItemDeclarator) 
-> ParseResult<ast::AnyTopLevelItem> 
{
    use tok::class::ItemDeclarator::*;
    use ast::AnyTopLevelItem;
    return Ok(match declarator {
        Proc => AnyTopLevelItem::Proc(parse_proc_def(ctx)?),
        Struct => todo!(),
        Enum => todo!(),
        LineComment => AnyTopLevelItem::LineComment(parse_line_comment(ctx)?),
    });
}

fn parse_proc_def(ctx: &mut ParseContext) -> ParseResult<ast::ProcDefinition> {
    let proc_keyword = ctx.stream.assert_ref::<delims::Proc>();
    let ident = ctx.expect_ref::<tok::class::Ident>()?;
    let parameters = parse_parameters(ctx)?;
    let return_type_separator = ctx.expect_ref::<delims::Colon>()?;
    let return_type = parse_type(ctx)?;
    let body = parse_imperative_block(ctx)?;
    return Ok(ast::ProcDefinition { proc_keyword, ident, parameters, return_type_separator,
        return_type, body });
}

fn parse_parameters(ctx: &mut ParseContext) -> ParseResult<ast::Parameters> {
    let open_paren = ctx.expect_ref::<delims::OpenParen>()?;
    let mut first: Option<AstRef<ast::ParameterNode>> = None;
    let mut ll_next = &mut first;
    loop {
        if !ctx.stream.cursor.has_next() { break; }
        if ctx.stream.peek::<delims::CloseParen>().is_some() { break; }
        let ident = ctx.expect_ref::<tok::class::Ident>()?;
        let colon = ctx.expect_ref::<delims::Colon>()?;
        let ty = parse_type(ctx)?;
        let comma = ctx.stream.consume_ref::<delims::Comma>();
        extend_ll(ctx.ast_mem, &mut ll_next, ast::Parameter { ident, colon, ty, comma });
        if comma.is_none() { break; }
    }
    let close_paren = ctx.expect_ref::<delims::CloseParen>()?;
    return Ok(ast::Parameters { open_paren, close_paren, first });
}

fn parse_type(ctx: &mut ParseContext) -> ParseResult<ast::Type> {
    let ident = ctx.expect_ref::<tok::class::Ident>()?;
    let mut arguments: Option<ast::TypeArguments> = None;
    if ctx.stream.peek::<delims::LessThan>().is_some() {
        arguments = Some(parse_type_arguments(ctx)?);
    }
    return Ok(ast::Type::NamedType(ast::NamedType { ident, arguments }));
}

fn parse_type_arguments(ctx: &mut ParseContext) -> ParseResult<ast::TypeArguments> {
    let open_angle = ctx.stream.assert_ref::<delims::LessThan>();
    let mut first: Option<AstRef<ast::TypeArgumentNode>> = None;
    let mut ll_next = &mut first;
    loop {
        if !ctx.stream.cursor.has_next() { break; }
        if ctx.stream.peek::<delims::GreaterThan>().is_some() { break; }
        let ty = parse_type(ctx)?;
        let comma = ctx.stream.consume_ref::<delims::Comma>();
        extend_ll(ctx.ast_mem, &mut ll_next, ast::TypeArgument { ty, comma });
        if comma.is_none() { break; }
    }
    let close_angle = ctx.expect_ref::<delims::GreaterThan>()?;
    return Ok(ast::TypeArguments { open_angle, first, close_angle });
}

fn parse_line_comment(ctx: &mut ParseContext) -> ParseResult<ast::LineComment> {
    let tok = ctx.stream.assert_ref::<tok::class::LineComment>();
    return Ok(ast::LineComment { tok })
}

fn parse_imperative_block(ctx: &mut ParseContext) -> ParseResult<ast::ImperativeBlock> {
    let open_curly = ctx.expect_ref::<delims::OpenCurly>()?;
    let mut first: Option<AstRef<ast::StatementNode>> = None;
    let mut ll_next = &mut first;
    loop {
        if !ctx.stream.cursor.has_next() { break; }
        if ctx.stream.peek::<delims::CloseCurly>().is_some() { break; }
        // TODO: parse statement
    }
    let close_curly = ctx.expect_ref::<delims::CloseCurly>()?;
    return Ok(ast::ImperativeBlock { open_curly, first, close_curly });
}

fn parse_statement(ctx: &mut ParseContext) -> ParseResult<ast::AnyStatement> {
    todo!()
}

// -- Tests --------------------------------------------------------------------------------------

#[cfg(test)]
mod test_parser {
    use crate::diagnostic::AnyDiagnostic;
    use crate::tok::lex::lex;
    use crate::util::str_interner::StrInterner;
    use super::parse;
    
    #[test]
    fn smoke_test() {
        const SOURCE_TEXT: &'static str = "\
            proc main(): int {    \n\
                \n\
            }\
        ";

        let string_interner = StrInterner::default();
        let tokbuf = lex(SOURCE_TEXT.as_bytes(), &string_interner);
        let mut diagnostics: Vec<AnyDiagnostic> = Vec::new();
        let ast = parse(&tokbuf, 0, &mut diagnostics);
        assert!(diagnostics.is_empty());
    }
}
