use std::sync::LazyLock;
use crate::util::prefix_tree::PrefixTree;
use crate::util::ascii;
use crate::util::str_interner::StrInterner;
use crate::tok::tok::{LineComment, DecIntLiteral, StaticTok, Tok, StrLiteral, Unexpected};
use crate::tok::tokbuf::TokBuf;
use crate::tok::ident::{Ident, iter_ident_prefix_chs, is_ident_prefix_ch, is_ident_ch, is_ident_str};
use crate::tok::tok::Align;
use crate::util::str_list::StrRef;

// -- ByteStream ---------------------------------------------------------------------------------

struct ByteStream<'a> { bytes: &'a [u8], pos: usize }

impl<'a> ByteStream<'a> {
    fn new(bytes: &'a [u8]) -> Self { return ByteStream { bytes, pos: 0 }; }
    
    fn rem(&self) -> &'a [u8] { return &self.bytes[self.pos..]; }
    
    fn advance_n(&mut self, n: usize) {
        assert!(self.pos + n <= self.bytes.len());
        self.pos += n;
    }
    
    fn advance_while(&mut self, pred: fn(u8) -> bool) -> &'a [u8] {
        let begin = self.pos;
        while self.bytes.get(self.pos).copied().is_some_and(pred) {
            self.pos += 1;
        }
        return &self.bytes[begin..self.pos];
    }
    
    fn advance_if(&mut self, pred: fn(u8) -> bool) {
        if self.bytes.get(self.pos).copied().is_some_and(pred) {
            self.pos += 1;
        }
    } 
    
    fn advance(&mut self) -> u8 {
        let next = *self.rem().first().unwrap();
        self.advance_n(1);
        return next;
    }
}

// -- Support ------------------------------------------------------------------------------------

enum Prefix {
    DoubleQuote,
    Digit,
    Static(StaticTok),
    Linebreak,
    Space,
    DoubleForwardSlash,
    IdentPrefixCh
}

static PREFIX_TREE: LazyLock<PrefixTree<Prefix>> = LazyLock::new(|| {
    let mut tree: PrefixTree<Prefix> = PrefixTree::default();
    tree.insert_seq(&[ascii::DOUBLE_QUOTE], Prefix::DoubleQuote);
    for digit in ascii::DIGITS {
        tree.insert_seq(&[digit], Prefix::Digit);
    }
    for stok in StaticTok::variants() {
        tree.insert_seq(stok.source_text(), Prefix::Static(*stok));
    }
    tree.insert_seq(&[ascii::LINEBREAK], Prefix::Linebreak);
    tree.insert_seq(&[ascii::SPACE], Prefix::Space);
    tree.insert_seq(&[ascii::FORWARDSLASH, ascii::FORWARDSLASH], Prefix::DoubleForwardSlash);
    for ident_prefix_ch in iter_ident_prefix_chs() {
        tree.insert_seq(&[ident_prefix_ch], Prefix::IdentPrefixCh);
    }
    return tree;
});

struct LexContext<'a, 'b, 'c> {
    tokbuf: &'c mut TokBuf<'a>,
    stream: &'c mut ByteStream<'b>
}

impl<'a, 'b, 'c> LexContext<'a, 'b, 'c> {
    fn new(tokbuf: &'c mut TokBuf<'a>, stream: &'c mut ByteStream<'b>) -> Self {
        return Self { tokbuf, stream };
    }
}

// -- Lexer --------------------------------------------------------------------------------------

pub fn lex<'a>(source_text: &[u8], interner: &'a StrInterner) -> TokBuf<'a> {
    let mut tokbuf = TokBuf::new(interner, source_text.len());
    let mut stream = ByteStream::new(source_text);
    lex_loop(&mut LexContext::new(&mut tokbuf, &mut stream));
    tokbuf.shrink_to_fit();
    return tokbuf;
}

fn lex_loop<'a>(ctx: &mut LexContext) {
    while ctx.stream.rem().len() > 0 {
        match PREFIX_TREE.get(ctx.stream.rem().iter().copied()) {
            Some(Prefix::DoubleQuote) => lex_double_quote(ctx),
            Some(Prefix::Digit) => lex_digit(ctx),
            Some(Prefix::Static(stok)) => lex_stok(ctx, *stok),
            Some(Prefix::Linebreak) => lex_linebreak(ctx),
            Some(Prefix::Space) => lex_space(ctx),
            Some(Prefix::DoubleForwardSlash) => lex_double_forward_slash(ctx), 
            Some(Prefix::IdentPrefixCh) => lex_ident_prefix_ch(ctx, 0), 
            None => lex_other(ctx)
        }
    }    
}

fn lex_double_quote(ctx: &mut LexContext) {
    let begin = ctx.stream.pos;
    ctx.stream.advance_n(1); // Advance past opening double quote.
    ctx.stream.advance_while(|ch| ch != ascii::DOUBLE_QUOTE);
    ctx.stream.advance_if(|ch| ch == ascii::DOUBLE_QUOTE); // Advance past closing double quote.
    let end = ctx.stream.pos;
    let source_text = &ctx.stream.bytes[begin..end];
    let str_ref = StrRef::Slice(source_text);
    ctx.tokbuf.push(Tok::StrLiteral(StrLiteral { str_ref }));
    
}

fn lex_digit(ctx: &mut LexContext) {
    let digits = ctx.stream.advance_while(|ch| ascii::is_numeric_ch(ch));
    let str_ref = StrRef::Slice(digits);
    ctx.tokbuf.push(Tok::DecIntLiteral(DecIntLiteral { str_ref }));
}

fn lex_stok(ctx: &mut LexContext, stok: StaticTok) {
    // If the static token we matched is also a valid identifier-prefix, *and* it is consecutive
    // with a valid identifier character, then it should be considered an identifier. For instance
    // `enum` is a keyword token, but `enumaaa` is an identifier.
    if is_ident_str(stok.source_text()) {
        if let Some(next) = ctx.stream.rem().get(stok.source_text().len()) {
            if is_ident_ch(*next) {
                lex_ident_prefix_ch(ctx, stok.source_text().len());
                return;
            }
        }
    }
    // If the static token we matched is not consecutive with a valid identifier character,
    // then it is indeed a static token and not an identifier.
    ctx.stream.advance_n(stok.source_text().len());
    ctx.tokbuf.push(Tok::Static(stok));
}

fn lex_linebreak(ctx: &mut LexContext) {
    assert_eq!(ctx.stream.advance(), ascii::LINEBREAK);
    ctx.tokbuf.push(Tok::Linebreak);
}

fn lex_space(ctx: &mut LexContext) {
    let spaces = ctx.stream.advance_while(|ch| ch == ascii::SPACE);
    let count = u32::try_from(spaces.len()).unwrap();

    assert!(count > 0);
    if count > 1 {
        ctx.tokbuf.push(Tok::Align(Align { count }));
    } else {
        ctx.tokbuf.push(Tok::Static(StaticTok::Space));
    }
  }

fn lex_double_forward_slash(ctx: &mut LexContext) {
    ctx.stream.advance_n(2);
    let content = ctx.stream.advance_while(|ch| ch != ascii::LINEBREAK);
    let str_ref = StrRef::Slice(content);
    ctx.tokbuf.push(Tok::LineComment(LineComment { str_ref }));
}

fn lex_ident_prefix_ch(ctx: &mut LexContext, assume_n: usize) {
    let begin = ctx.stream.pos;
    assert!(is_ident_prefix_ch(ctx.stream.advance()));
    ctx.stream.advance_n(assume_n);
    ctx.stream.advance_while(|ch| is_ident_ch(ch));
    let source_text = &ctx.stream.bytes[begin..ctx.stream.pos];
    ctx.tokbuf.push(Tok::Ident(Ident::new(source_text)));
}

fn lex_other(ctx: &mut LexContext) {
    let ch = ctx.stream.advance();
    ctx.tokbuf.push(Tok::Unexpected(Unexpected { ch }));
}

#[cfg(test)]
mod test_lex {
    use crate::tok::tok::{Tok, StaticTok};
    use crate::tok::tokbuf::TokBuf;
    use crate::util::str_interner::StrInterner;
    use crate::util::misc::assert_matches;
    use super::lex;

    #[test]
    fn smoke_test() {
        let source_text = "\
            proc main() {\n    \
                std::println(\"Hello World\");\n\
            }\n\
        ".as_bytes();
        
        let string_interner = StrInterner::default();
        let mut tokbuf = lex(source_text, &string_interner);
        let toks: Vec<Tok> = tokbuf.iter().collect();
        
        assert_matches!(toks[0], Tok::Static(StaticTok::Proc));
        assert_matches!(toks[1], Tok::Static(StaticTok::Space));
        assert_matches!(toks[2], Tok::Ident(main_ident));
        assert_eq!(main_ident.source_text.get(), "main".as_bytes());
        assert_matches!(toks[3], Tok::Static(StaticTok::OpenParen));
        assert_matches!(toks[4], Tok::Static(StaticTok::CloseParen));
        assert_matches!(toks[5], Tok::Static(StaticTok::Space));
        assert_matches!(toks[6], Tok::Static(StaticTok::OpenCurly));
        assert_matches!(toks[7], Tok::Linebreak);
        assert_matches!(toks[8], Tok::Align(_));
        assert_matches!(toks[9], Tok::Ident(std_ident));
        assert_eq!(std_ident.source_text.get(), "std".as_bytes());
        assert_matches!(toks[10], Tok::Static(StaticTok::ColonColon));
        assert_matches!(toks[11], Tok::Ident(println_ident));
        assert_eq!(println_ident.source_text.get(), "println".as_bytes());
        assert_matches!(toks[12], Tok::Static(StaticTok::OpenParen));
        assert_matches!(toks[13], Tok::StrLiteral(_));
        assert_matches!(toks[14], Tok::Static(StaticTok::CloseParen));
        assert_matches!(toks[15], Tok::Static(StaticTok::Semicolon));
        assert_matches!(toks[16], Tok::Linebreak);
        assert_matches!(toks[17], Tok::Static(StaticTok::CloseCurly));
        assert_matches!(toks[18], Tok::Linebreak);
    }

    #[test]
    fn test_keyword_identifer_disambiguation() {
        // This is an identifier even though it begins with a keyword.
        let source_text = "procaaaa".as_bytes();     

        let string_interner = StrInterner::default();
        let mut tokbuf = lex(source_text, &string_interner);
        let toks: Vec<Tok> = tokbuf.iter().collect();

        assert_eq!(toks.len(), 1);
        assert_matches!(toks[0], Tok::Ident(ident));
        assert_eq!(ident.source_text.get(), source_text);
    }
}
