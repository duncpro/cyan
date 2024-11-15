use std::num::NonZeroU8;
use crate::util::ascii;
use crate::tok::ident::Ident;
use crate::util::str_list::StrRef;

#[derive(Clone, Copy, Debug)]
pub enum Tok<'a> {
    Static(StaticTok), 
    StrLiteral(StrLiteral<'a>),
    DecIntLiteral(DecIntLiteral<'a>),
    Ident(Ident<'a>),   
    Linebreak,
    Align(Align),
    LineComment(LineComment<'a>),
    Unexpected(Unexpected),
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
    Ampersand = 28,
    Semicolon = 29,
    /// Represents a *single* space character in the source text.
    ///
    /// Note that most coding style guides recommend indenting of nested blocks with more than one
    /// space. Therefore, it is extremely common that long sequences of consecutive spaces appear 
    /// in the source text. However such a sequence is **not** represented by 
    /// `Tok::StaticTok(StaticTok::Space)`. Instead, it is represented by `Tok::Align(Align)`.
    ///
    /// ```txt
    ///   std::sum(a, b);
    ///              -  <- `Tok::StaticTok(StaticTok::Space)`
    ///
    ///    if x condition_is_true {
    ///        std::println("Hello World");
    ///    ____  <- `Tok::Align(Align)`
    ///    }
    /// ```
    Space = 30,
    Comma = 31
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
            Self::Ampersand,
            Self::Semicolon,
            Self::Space,
            Self::Comma
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
            StaticTok::OpenCurly => "{",
            StaticTok::CloseCurly => "}",
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
            StaticTok::Ampersand => "&",
            StaticTok::Semicolon => ";",
            StaticTok::Space => " ",
            StaticTok::Comma => ",",
        }.as_bytes();
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StrLiteral<'a> {
    // The entirety of the source text of this string literal including the
    // leading quote and the trailing quote (if closed).
    pub str_ref: StrRef<'a>
}

#[derive(Clone, Copy, Debug)]
pub struct DecIntLiteral<'a> {
    /// The character-encoded decimal digits from the source text.
    /// This slice is guaranteed to have a length of at least one.
    pub str_ref: StrRef<'a>
}

#[derive(Clone, Copy, Debug)]
pub struct Align { pub count: u32 }

#[derive(Clone, Copy, Debug)]
pub struct LineComment<'a> {
    /// The source-text appearing after the double-slash prefix but before the
    /// terminating linebreak.
    pub str_ref: StrRef<'a>
}

#[derive(Clone, Copy, Debug)]
pub struct Unexpected { pub ch: u8 }

