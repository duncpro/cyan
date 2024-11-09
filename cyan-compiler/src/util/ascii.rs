use std::ops::Range;

pub const fn is_alphabetic_ch(ch: u8) -> bool {
    return 65 <= ch && ch < 123;
}

pub const fn is_numeric_ch(ch: u8) -> bool {
    return 48 <= ch && ch < 58;
}

pub const fn is_alphanumeric_ch(ch: u8) -> bool {
    return is_alphabetic_ch(ch) || is_numeric_ch(ch);
}


pub const UNDERSCORE: u8 = 95;
pub const DOUBLE_QUOTE: u8 = 34;
pub const BACKSLASH: u8 = 92;
pub const FORWARDSLASH: u8 = 47;
pub const SPACE: u8 = 32;
pub const LINEBREAK: u8 = 10;

// Ranges
pub const DIGITS: Range<u8> = 48..58;
pub const ALPHABET: Range<u8> = 65..123;
