use crate::util::str_list::StrRef;
use crate::util::ascii;

#[derive(Clone, Copy, Debug)]
pub struct Ident<'a> {
    pub source_text: StrRef<'a>
}

impl<'a> Ident<'a> {
     pub fn new(slice: &'a [u8]) -> Self {
        let source_text = StrRef::Slice(slice);
        return Self { source_text };
    }
}

// Character classifiers

pub const fn is_ident_ch(ch: u8) -> bool {
    return ascii::is_alphanumeric_ch(ch) || ch == ascii::UNDERSCORE;
}

pub const fn is_ident_chs(s: &[u8]) -> bool {
    let mut i: usize = 0;
    while i < s.len() {
        if !is_ident_ch(s[i]) {
            return false;
        }
        i += 1;
    }
    return true;
}

pub const fn is_ident_prefix_ch(ch: u8) -> bool {
    return ascii::is_alphabetic_ch(ch) || ch == ascii::UNDERSCORE;
}

pub const fn is_ident_str(s: &[u8]) -> bool {
    let Some(first_ch) = s.first() else { return false; };
    if !is_ident_prefix_ch(*first_ch) { return false; }
    return is_ident_chs(&s);
}

pub fn iter_ident_prefix_chs() -> impl Iterator<Item = u8> {
    let underscore = std::iter::once(ascii::UNDERSCORE);
    let alphabet = ascii::ALPHABET.into_iter();
    return underscore.chain(alphabet);
}
