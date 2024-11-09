use std::sync::LazyLock;
use crate::util::prefix_tree::PrefixTree;
use crate::util::ascii;
use crate::util::string_interner::StringInterner;
use crate::tok::tok::{LineComment, DecIntLiteral, Linebreaks, StaticTok, Tok, StrLiteral, Unexpected};
use crate::tok::tokbuf::TokBuf;
use crate::tok::ident::{Ident, iter_ident_prefix_chs, is_ident_ch, is_ident_str};
use crate::tok::tok::Spaces;

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

enum Prefix {
    DoubleQuote,
    Digit,
    Static(StaticTok),
    Linebreak,
    Space,
    DoubleForwardSlash,
    IdentPrefixCh
}

static PREFIX_TREE: LazyLock<PrefixTree<u8, Prefix>> = LazyLock::new(|| {
    let mut tree: PrefixTree<u8, Prefix> = PrefixTree::default();
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


pub fn lex<'a>(source_text: &[u8], string_interner: &'a StringInterner) -> TokBuf<'a> {
    let mut ctx = LexContext { 
        tokbuf: TokBuf::new(string_interner),
        stream: ByteStream::new(source_text)
    };
    
    while ctx.stream.rem().len() > 0 {
        match PREFIX_TREE.get(ctx.stream.rem().iter()) {
            Some(Prefix::DoubleQuote) => lex_double_quote(&mut ctx),
            Some(Prefix::Digit) => lex_digit(&mut ctx),
            Some(Prefix::Static(stok)) => lex_stok(&mut ctx, *stok),
            Some(Prefix::Linebreak) => lex_linebreak(&mut ctx),
            Some(Prefix::Space) => lex_space(&mut ctx),
            Some(Prefix::DoubleForwardSlash) => lex_double_forward_slash(&mut ctx), 
            Some(Prefix::IdentPrefixCh) => lex_ident_prefix_ch(&mut ctx), 
            None => lex_other(&mut ctx)
        }
    }
    
    return ctx.tokbuf;
}

struct LexContext<'a, 'b> {
    tokbuf: TokBuf<'a>,
    stream: ByteStream<'b>
}

fn lex_double_quote(ctx: &mut LexContext) {
    let begin = ctx.stream.pos;
    ctx.stream.advance_n(1); // Advance past opening double quote.
    ctx.stream.advance_while(|ch| ch != ascii::DOUBLE_QUOTE);
    ctx.stream.advance_if(|ch| ch == ascii::DOUBLE_QUOTE); // Advance past closing double quote.
    let end = ctx.stream.pos;
    let source_text = &ctx.stream.bytes[begin..end];
    ctx.tokbuf.push(&Tok::StrLiteral(StrLiteral::new(source_text)));
    
}

fn lex_digit(ctx: &mut LexContext) {
    let digits = ctx.stream.advance_while(|ch| ascii::is_numeric_ch(ch));
    ctx.tokbuf.push(&Tok::DecIntLiteral(DecIntLiteral::new(digits)));
}

fn lex_stok(ctx: &mut LexContext, stok: StaticTok) {
    // If the static token we matched is also a valid identifier-prefix, *and* it is consecutive
    // with a valid identifier character, then it should be considered an identifier. For instance
    // `enum` is a keyword token, but `enumaaa` is an identifier.
    if is_ident_str(stok.source_text()) {
        if let Some(next) = ctx.stream.rem().get(stok.source_text().len()) {
            if is_ident_ch(*next) {
                lex_ident_prefix_ch(ctx);
                return;
            }
        }
    }
    // If the static token we matched is not consecutive with a valid identifier character,
    // then it is indeed a static token and not an identifier.
    ctx.stream.advance_n(stok.source_text().len());
    ctx.tokbuf.push(&Tok::Static(stok));
}

fn lex_linebreak(ctx: &mut LexContext) {
    let linebreaks = ctx.stream.advance_while(|ch| ch == ascii::LINEBREAK);
    let count = u32::try_from(linebreaks.len()).unwrap();
    ctx.tokbuf.push(&Tok::Linebreaks(Linebreaks::new(count)));
}

fn lex_space(ctx: &mut LexContext) {
    let spaces = ctx.stream.advance_while(|ch| ch == ascii::SPACE);
    let count = u32::try_from(spaces.len()).unwrap();
    ctx.tokbuf.push(&Tok::Spaces(Spaces::new(count)));
}

fn lex_double_forward_slash(ctx: &mut LexContext) {
    ctx.stream.advance_n(2);
    let content = ctx.stream.advance_while(|ch| ch != ascii::LINEBREAK);
    ctx.tokbuf.push(&Tok::LineComment(LineComment::new(content)));
}

fn lex_ident_prefix_ch(ctx: &mut LexContext) {
    let source_text = ctx.stream.advance_while(|ch| is_ident_ch(ch));
    ctx.tokbuf.push(&Tok::Ident(Ident::new(source_text)));
}

fn lex_other(ctx: &mut LexContext) {
    let ch = ctx.stream.advance();
    ctx.tokbuf.push(&Tok::Unexpected(Unexpected { ch }));
}
