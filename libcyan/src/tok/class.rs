use std::marker::PhantomData;
use crate::tok::tok::{self, StaticTok, Tok, StrLiteral, DecIntLiteral};
use crate::tok::tokbuf::{TokCursor, Key};

/// A 32-bit pointer to a token inside of the token buffer. In an abstract sense, `TokRef`s are
/// the leaves of the AST.
#[repr(transparent)]
#[derive(Debug)]
pub struct TokRef<C: TokClass> {
    pd: PhantomData<C>,
    key: Key
}

/// Represents a class of tokens. For example "binary operators", "keywords", "boolean literals".
pub trait TokClass {
    type View<'a>;

    /// If `tok` is a member of this class, returns `Some(Self::View)`, otherwise returns `None`.
    fn r#match<'a>(tok: &Tok<'a>) -> Option<Self::View<'a>>;
}

impl<C: TokClass> Clone for TokRef<C> {
    fn clone(&self) -> Self { *self }
}

impl<C: TokClass> Copy for TokRef<C> {}

impl<'a> TokCursor<'a> {
    pub fn match_ref<C: TokClass>(&self) -> Option<TokRef<C>> {
        let next = self.read_tok()?;
        if C::r#match(&next).is_none() { return None; }
        return Some(TokRef { pd: PhantomData, key: self.at() });
    }

    pub fn r#match<C: TokClass>(&self) -> Option<C::View<'a>> {
        let next = self.read_tok()?;
        return C::r#match(&next);
    }
}

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

    fn r#match<'a>(tok: &'a Tok<'a>) -> Option<Self::View<'a>> {
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
    type View<'a> = crate::tok::ident::Ident<'a>;

    fn r#match<'a>(tok: &Tok<'a>) -> Option<Self::View<'a>> {
        match tok {
            Tok::Ident(ident) => Some(*ident),
            _ => None
        }
    }
}

// -- Literal -----------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum AnyLiteral<'a> {
    Str(StrLiteral<'a>),
    DecInt(DecIntLiteral<'a>)
}

pub struct Literal;

impl TokClass for Literal {
    type View<'a> = AnyLiteral<'a>;
    fn r#match<'a>(tok: &Tok<'a>) -> Option<Self::View<'a>> {
        match tok {
            Tok::StrLiteral(lit) => Some(AnyLiteral::Str(*lit)),
            Tok::DecIntLiteral(lit) => Some(AnyLiteral::DecInt(*lit)),
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
                
                fn r#match<'a>(tok: &Tok<'a>) -> Option<Self::View<'a>> {
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
    make_delim_class!(LessThan);
    make_delim_class!(GreaterThan);
    make_delim_class!(Proc);
    make_delim_class!(Comma);
    make_delim_class!(Colon);
}


// -- TL Item Declarators -----------------------------------------------------------------------

pub enum ItemDeclarator {
    Proc,
    Struct,
    Enum,
    LineComment
}

impl TokClass for ItemDeclarator {
    type View<'a> = Self;

    fn r#match<'a>(tok: &Tok<'a>) -> Option<Self::View<'a>> {
        match tok {
            Tok::Static(StaticTok::Proc) => Some(Self::Proc),
            Tok::Static(StaticTok::Struct) => Some(Self::Struct),
            Tok::Static(StaticTok::Enum) => Some(Self::Enum),
            Tok::LineComment(_) => Some(Self::LineComment),
            _ => None
        }
    }
    
}

// -- Formatting ---------------------------------------------------------------------------------

pub struct Formatting;

impl TokClass for Formatting {
    type View<'a> = Self;

    fn r#match<'a>(tok: &Tok<'a>) -> Option<Self::View<'a>> {
        match tok {
            Tok::Static(StaticTok::Space) => Some(Self),
            Tok::Linebreak => Some(Self),
            Tok::Align(_) => Some(Self),
            _ => None
        }
    }
}

// -- LineComment -------------------------------------------------------------------------------

pub struct LineComment;

pub struct LineCommentView<'a> { pub value: tok::LineComment<'a> }

impl TokClass for LineComment {
    type View<'a> = LineCommentView<'a>;

    fn r#match<'a>(tok: &Tok<'a>) -> Option<Self::View<'a>> {
        match *tok {
            Tok::LineComment(value) => Some(LineCommentView { value }),
            _ => None
        }
    }
}
