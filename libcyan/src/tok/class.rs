use std::marker::PhantomData;
use crate::tok::tok::{StaticTok, Tok, StrLiteral, DecIntLiteral};
use crate::tok::tokbuf::Key;

/// A 32-bit pointer to a token inside of the token buffer.
/// In an abstract sense, `TokRef`s are the leaves of the AST.
#[repr(transparent)]
#[derive(Debug)]
pub struct TokRef<C: TokClass> {
    pd: PhantomData<C>,
    key: Key
}

/// Represents a class of tokens. For example "binary operators",
/// "keywords", "boolean literals".
pub trait TokClass {
    type View<'a>;

    /// If `tok` is a member of this class, returns `Some(Self::View)`,
    /// otherwise returns `None`.
    fn classify<'a>(tok: &'a Tok<'a>) -> Option<Self::View<'a>>;
}

impl<C: TokClass> Clone for TokRef<C> {
    fn clone(&self) -> Self { *self }
}

impl<C: TokClass> Copy for TokRef<C> {}

// -- Binary Operators ---------------------------------------------------------------------------

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum BinaryOperator {
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    EqEq,
    NotEq,
    Eq,
}

impl TokClass for BinaryOperator {
    type View<'a> = Self;

    fn classify<'a>(tok: &'a Tok<'a>) -> Option<Self::View<'a>> {
        match tok {
            Tok::Static(StaticTok::LessThan) => Some(Self::LessThan),
            Tok::Static(StaticTok::LessThanEq) => Some(Self::LessThanEq),
            Tok::Static(StaticTok::GreaterThan) => Some(Self::GreaterThan),
            Tok::Static(StaticTok::GreaterThanEq) => Some(Self::GreaterThanEq),
            Tok::Static(StaticTok::EqEq) => Some(Self::EqEq),
            Tok::Static(StaticTok::NotEq) => Some(Self::NotEq),
            Tok::Static(StaticTok::Eq) => Some(Self::Eq),
            _ => None
        }
    }
}

// -- Ident --------------------------------------------------------------------------------------

pub struct Ident;

impl TokClass for Ident {
    type View<'a> = &'a crate::tok::ident::Ident<'a>;

    fn classify<'a>(tok: &'a Tok<'a>) -> Option<Self::View<'a>> {
        match tok {
            Tok::Ident(ident) => Some(ident),
            _ => None
        }
    }
}

// Literals

#[derive(Clone, Copy, Debug)]
pub enum AnyLiteral<'a> {
    Str(&'a StrLiteral<'a>),
    DecInt(&'a DecIntLiteral<'a>)
}

pub struct Literal;

impl TokClass for Literal {
    type View<'a> = AnyLiteral<'a>;
    fn classify<'a>(tok: &'a Tok<'a>) -> Option<Self::View<'a>> {
        match tok {
            Tok::StrLiteral(lit) => Some(AnyLiteral::Str(lit)),
            Tok::DecIntLiteral(lit) => Some(AnyLiteral::DecInt(lit)),
            _ => None
        }
    }
}

// -- Delimiters --------------------------------------------------------------------------------

pub mod delims {
    use crate::tok::tok::{StaticTok, Tok};
    use super::TokClass;
    
    macro_rules! make_delim_class {
        ($id:ident) => {
            pub struct $id;
    
            impl TokClass for $id {
                type View<'a> = Self;
                
                fn classify<'a>(tok: &'a Tok<'a>) -> Option<Self::View<'a>> {
                    match tok {
                        Tok::Static(StaticTok::$id) => Some(Self),
                        _ => None
                    }
                }
            }
        };
    }
    
    make_delim_class!(OpenCurly);
    make_delim_class!(CloseCurly);
    make_delim_class!(OpenParen);
    make_delim_class!(CloseParen);
    make_delim_class!(Proc);
    make_delim_class!(Comma);
    make_delim_class!(Colon);
}


// -- TL Item Declarators -----------------------------------------------------------------------

pub enum ItemDeclarator {
    Proc,
    Struct,
    Enum
}

impl TokClass for ItemDeclarator {
    type View<'a> = Self;

    fn classify<'a>(tok: &'a Tok<'a>) -> Option<Self::View<'a>> {
        match tok {
            Tok::Static(StaticTok::Proc) => Some(Self::Proc),
            Tok::Static(StaticTok::Struct) => Some(Self::Struct),
            Tok::Static(StaticTok::Enum) => Some(Self::Enum),
            _ => None
        }
    }
    
}
