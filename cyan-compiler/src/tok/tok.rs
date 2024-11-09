use std::num::NonZeroU8;
use crate::util::ascii;
use crate::tok::ident::Ident;

#[derive(Clone, Copy, Debug)]
pub enum Tok<'a> {
    Static(StaticTok), 
    StrLiteral(StrLiteral<'a>),
    DecIntLiteral(DecIntLiteral<'a>),
    Ident(Ident<'a>),
    Linebreaks(Linebreaks),
    Spaces(Spaces),
    LineComment(LineComment<'a>),
    Unexpected(Unexpected)
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum StaticTok {
    If = 1,
    For = 2,
    Let = 3,
    Struct = 4,
    Enum = 5,
    Namespace = 6,
    Import = 7,
    Break = 8,
    Continue = 9,
    Proc = 10,
    OpenParen = 11,
    CloseParen = 12,
    OpenCurly = 13,
    CloseCurly = 14,
    OpenSquare = 15,
    CloseSquare = 16,
    LessThan = 17,
    LessThanEq = 18,
    GreaterThan = 19,
    GreaterThanEq = 20,
    EqEq = 21,
    NotEq = 22,
    Eq = 23,
    Colon = 24,
    ColonColon = 25,
    Percent = 26,
    Exclamation = 27,
}

impl StaticTok {
    pub fn variants() -> &'static [Self] {
        return &[
            Self::If,
            Self::For,
            Self::Let,
            Self::Struct,
            Self::Enum,
            Self::Namespace,
            Self::Import,
            Self::Break,
            Self::Continue,
            Self::Proc,
            Self::OpenParen,
            Self::CloseParen,
            Self::OpenCurly,
            Self::CloseCurly,
            Self::OpenSquare,
            Self::CloseSquare,
            Self::LessThan,
            Self::LessThanEq,
            Self::GreaterThan,
            Self::GreaterThanEq,
            Self::EqEq,
            Self::NotEq,
            Self::Eq,
            Self::Colon,
            Self::ColonColon,
            Self::Percent,
            Self::Exclamation,
        ];
    }
    
    pub fn id(self) -> NonZeroU8 {
        return NonZeroU8::try_from(self as u8).unwrap();
    }

    pub fn from_id(id: u8) -> Option<Self> {
        if id == 0 { return None; }
        return Self::variants().get(usize::from(id - 1)).copied();
    }

    pub fn source_text(self) -> &'static [u8] {
        return match self {
            StaticTok::If => "if",
            StaticTok::For => "for",
            StaticTok::Let => "let",
            StaticTok::Struct => "struct",
            StaticTok::Enum => "enum",
            StaticTok::Namespace => "namespace",
            StaticTok::Import => "import",
            StaticTok::Break => "break",
            StaticTok::Continue => "continue",
            StaticTok::Proc => "proc",
            StaticTok::OpenParen => "(",
            StaticTok::CloseParen => ")",
            StaticTok::OpenCurly => "{{",
            StaticTok::CloseCurly => "}}",
            StaticTok::OpenSquare => "[",
            StaticTok::CloseSquare => "]",
            StaticTok::LessThan => "<",
            StaticTok::LessThanEq => "<=",
            StaticTok::GreaterThan => ">",
            StaticTok::GreaterThanEq => ">=",
            StaticTok::EqEq => "==",
            StaticTok::NotEq => "!=",
            StaticTok::Eq => "=",
            StaticTok::Colon => ":",
            StaticTok::ColonColon => "::",
            StaticTok::Percent => "%",
            StaticTok::Exclamation => "!",
        }.as_bytes();
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StrLiteral<'a> {
    // The entirety of the source text of this string literal including the
    // leading quote and the trailing quote (if closed).
    source_text: &'a [u8]
}

impl<'a> StrLiteral<'a> {
    pub fn source_text(&self) -> &'a [u8] { return self.source_text; }
    pub fn new(source_text: &'a [u8]) -> Self {
        assert!(source_text.first().copied() == Some(ascii::DOUBLE_QUOTE));
        // Do not assert the existence of a closing double quote. The literal
        // could be unclosed.
        return Self { source_text };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DecIntLiteral<'a> {
    /// The character-encoded decimal digits from the source text.
    /// This slice is guaranteed to have a length of at least one.
    digits: &'a [u8]
}

impl<'a> DecIntLiteral<'a> {
    pub fn digits(&self) -> &'a [u8] { return self.digits; }
    pub fn new(digits: &'a [u8]) -> Self {
        assert!(digits.iter().all(|ch| ascii::is_numeric_ch(*ch)));
        assert!(digits.len() > 0);
        return Self { digits }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Linebreaks { count: u32 }

impl Linebreaks {
    pub fn count(&self) -> u32 { return self.count; }
    pub fn new(count: u32) -> Self {
        assert!(count > 0);
        return Self { count };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Spaces { count: u32 }

impl Spaces {
    pub fn count(&self) -> u32 { return self.count; }
    pub fn new(count: u32) -> Self {
        assert!(count > 0);
        return Self { count };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LineComment<'a> {
    /// The source-text appearing after the double-slash prefix but before the
    /// terminating linebreak.
    content: &'a [u8]
}

impl<'a> LineComment<'a> {
    pub fn content(&self) -> &'a [u8] { return self.content; }
    pub fn new(content: &'a [u8]) -> Self {
        assert!(!content.contains(&ascii::LINEBREAK));
        return Self { content };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Unexpected { pub ch: u8 }
