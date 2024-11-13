///! The token representation is modeled after Google's Carbon Language compiler as described by
///! Chandler Carruth in his talk "Modernizing Compiler Design for Carbon Toolchain" at CppNow 2023.
///! See https://www.youtube.com/watch?v=ZI198eFghJk&t=2817s.

use std::num::{NonZeroU8, NonZeroUsize};
use crate::util::str_interner::StrInterner;
use crate::util::str_list::{StrRef, StrList, StrListKey, StrListRef};
use crate::util::bits::Truncate;
use crate::util::ascii;
use crate::tok::ident::Ident;
use crate::tok::tok::{Tok, DecIntLiteral, StaticTok, StrLiteral, LineComment, Align, Unexpected};

#[derive(Clone, Copy, Default)]
pub struct Key { data: u32 }

impl Key {
    const ADDR_MAX: u32 = u32::MAX >> 8;
    fn addr(self) -> u32 { return self.data >> 8; }
    fn pack_idx(self) -> u8 { return self.data.truncate(); }
    fn new(addr: u32, pack_idx: u8) -> Self {
        assert!(addr <= Self::ADDR_MAX);
        let mut data: u32 = 0;
        data |= u32::from(pack_idx);
        data |= (addr << 8);
        return Self { data };
    }
}

/// A dense representation of a source file.
///
/// Unlike the character-encoded source file, the [`TokBuf`] arranges the information in the 
/// source-text so that data relevant to the parser is contiguous. For instance, the parser does
/// not care about the content of a string literal, integer literal, or ident.
/// It cares completely about the type of the token, and not at all about its state. 
/// The token buffer preserves this extra state but puts it off to the side, out of the
/// way of the parser. As a consequence, when the parser iterates over the token buffer, the cache
/// is saturated in relevant information. Irrelevant data like literal and ident content is out of 
/// the way, never placed in the cache, and therefore never getting in the way of actually useful
/// information. 
pub struct TokBuf<'a> {
    string_interner: &'a StrInterner,
    str_table: StrList, 

    /// The densely packed token sequence, intended for consumption by the parser.
    /// Each [`TokBufEntry`] is 32 bits wide. Wide tokens like string literals,
    /// integer literals, and idents occupy an entire 32-bit [`TokBufEntry`]. 
    ///
    /// However narrow tokens (static tokens) are packed together into triplets. That is, up to 
    /// three static tokens per one [`TokBufEntry`]. 
    buf: Vec<TokBufEntry>,
    
    lines: Vec<u32>
}

impl<'a> TokBuf<'a> {
    pub fn new(string_interner: &'a StrInterner) -> TokBuf<'a> {
        return Self { 
            string_interner,
            str_table: StrList::default(),
            buf: Vec::new(),
            lines: Vec::new()
        };
    }

    fn push_static_tok(&mut self, stok: StaticTok) {
        if self.buf.last().is_none_or(|last| last.kind() != EntryType::StaticPack) {
            self.buf.push(TokBufEntry::new(EntryType::StaticPack, 0));
        }

        let last = self.buf.last_mut().unwrap();
        assert!(matches!(last.kind(), EntryType::StaticPack));

        let occupied = 4 - last.etc().leading_zeros() / 8;
        if occupied < 3 {
            let mut etc: Etc = u32::from(stok.id().get()) << (8 * occupied);
            etc |= last.etc();
            *last = TokBufEntry::new(EntryType::StaticPack, etc);
        } else {
            let etc: Etc = u32::from(stok.id().get());
            self.buf.push(TokBufEntry::new(EntryType::StaticPack, etc));
        }
    }

    fn push_str_literal(&mut self, lit: StrLiteral) {
        let etc = self.insert_str_table_entry(lit.str_ref.get());
        let entry = TokBufEntry::new(EntryType::StrLiteral, etc);
        let tok_addr = u32::try_from(self.buf.len()).unwrap();
        self.buf.push(entry);
        for ch in lit.str_ref.get() {
            if *ch == ascii::LINEBREAK {
                self.lines.push(tok_addr);
            }
        }
    }

    fn push_dec_int_literal(&mut self, lit: DecIntLiteral) {
        let etc = self.insert_str_table_entry(lit.str_ref.get());
        let entry = TokBufEntry::new(EntryType::DecIntLiteral, etc);
        self.buf.push(entry);
    }

    fn push_ident(&mut self, ident: Ident) {
        let intern_key = self.string_interner.intern(ident.source_text.get());
        let etc = u32::try_from(intern_key.get()).unwrap();
        let entry = TokBufEntry::new(EntryType::Ident, etc);
        self.buf.push(entry);
    }

    fn push_linebreak(&mut self) {
        let entry = TokBufEntry::new(EntryType::Linebreak, 0);
        let tok_addr = u32::try_from(self.buf.len()).unwrap();
        self.buf.push(entry);
        self.lines.push(tok_addr);
    }

    fn push_align(&mut self, align: Align) {
        let entry = TokBufEntry::new(EntryType::Align, align.count);
        self.buf.push(entry);
    }

    fn push_line_comment(&mut self, lc: LineComment) {
        let etc = self.insert_str_table_entry(lc.str_ref.get());
        let entry = TokBufEntry::new(EntryType::LineComment, etc);
        self.buf.push(entry);
    }

    fn push_unexpected(&mut self, unexpected: Unexpected) {
        let etc = u32::from(unexpected.ch);
        let entry = TokBufEntry::new(EntryType::Unexpected, etc);
        self.buf.push(entry);
    }
    
    pub fn push(&mut self, tok: Tok) {
        match tok {
            Tok::Static(stok) => self.push_static_tok(stok),
            Tok::StrLiteral(lit) => self.push_str_literal(lit),
            Tok::DecIntLiteral(lit) => self.push_dec_int_literal(lit),
            Tok::Ident(ident) => self.push_ident(ident),
            Tok::Linebreak => self.push_linebreak(),
            Tok::Align(indent) => self.push_align(indent),
            Tok::LineComment(lc) => self.push_line_comment(lc),
            Tok::Unexpected(unexpected) => self.push_unexpected(unexpected),
        }
    }

    pub fn iter(&'a self) -> impl Iterator<Item = Tok<'a>> + 'a {
        return TokCursor::new(&self);
    }

    pub fn get(&'a self, key: Key) -> Option<Tok<'a>> {
        let idx = usize::try_from(key.addr()).ok()?;
        let tbe = self.buf.get(idx)?;

        if tbe.kind() != EntryType::StaticPack && key.pack_idx() != 0 { 
            return None; 
        }
        
        match tbe.kind() {
            EntryType::StaticPack => {
                if key.pack_idx() > 2 { return None; }
                let pack_offset = key.pack_idx() * 8;
                let stok_id = (tbe.etc() >> pack_offset).truncate();
                if stok_id == 0 { return None; }
                let stok = StaticTok::from_id(stok_id).unwrap();
                return Some(Tok::Static(stok));
            },
            EntryType::StrLiteral => {
                let str_ref = self.make_str_table_ref(tbe.etc());
                return Some(Tok::StrLiteral(StrLiteral { str_ref }));
            },
            EntryType::DecIntLiteral => {
                let str_ref = self.make_str_table_ref(tbe.etc());
                return Some(Tok::DecIntLiteral(DecIntLiteral { str_ref }));
            },
            EntryType::Ident => {
                let str_table_key = NonZeroUsize::new(
                    usize::try_from(tbe.etc()).unwrap()).unwrap();
                let str_ref = StrRef::List(StrListRef::new(
                    self.string_interner.str_list(), str_table_key));
                return Some(Tok::Ident(Ident { source_text: str_ref }));
            },
            EntryType::Linebreak => return Some(Tok::Linebreak),
            EntryType::Align => {
                let count = tbe.etc();
                return Some(Tok::Align(Align { count }));
            },
            EntryType::LineComment => {
                let content = self.make_str_table_ref(tbe.etc());
                return Some(Tok::LineComment(LineComment { str_ref: content }));            
            },
            EntryType::Unexpected => {
                let ch: u8 = tbe.etc().truncate();
                return Some(Tok::Unexpected(Unexpected { ch }));
            }
        }
    }

    fn insert_str_table_entry(&mut self, entry: &[u8]) -> Etc {
        let str_table_key = self.str_table.push(entry);
        let etc = u32::try_from(str_table_key.get()).unwrap();
        return etc;
    }

    fn make_str_table_ref<'b>(&'b self, etc: Etc) -> StrRef<'b> {
        let key = NonZeroUsize::new(usize::try_from(etc).unwrap()).unwrap();
        return StrRef::List(StrListRef::new(&self.str_table, key));
    }

    /// Returns the 0-based index of the line where the token begins.
    fn get_line_no(&self, tok_addr: u32) -> usize {
        return self.lines.partition_point(|lb_addr| *lb_addr < tok_addr);
    }
}

#[cfg(test)]
mod test_tok_buf {
    use crate::tok::ident::Ident;
    use crate::util::str_interner::StrInterner;
    use crate::tok::tok::{StaticTok, StrLiteral, Tok};
    use crate::util::str_list::StrRef;
    use super::TokBuf;

    #[test]
    fn test_static_pack() {
        let interner = StrInterner::default();
        let mut tokbuf = TokBuf::new(&interner);
        tokbuf.push(Tok::Static(StaticTok::If));
        tokbuf.push(Tok::Static(StaticTok::Let));
        tokbuf.push(Tok::Static(StaticTok::Ampersand));
        tokbuf.push(Tok::Static(StaticTok::ColonColon));
        let toks: Vec<Tok> = tokbuf.iter().collect();
        assert!(matches!(toks[0], Tok::Static(StaticTok::If)));
        assert!(matches!(toks[1], Tok::Static(StaticTok::Let)));
        assert!(matches!(toks[2], Tok::Static(StaticTok::Ampersand)));
        assert!(matches!(toks[3], Tok::Static(StaticTok::ColonColon)));
    }

    #[test]
    fn test_str_literal() {
        let interner = StrInterner::default();
        let mut tokbuf = TokBuf::new(&interner);
        const SOURCE_TEXT: &'static [u8] = "\"Hello World\"".as_bytes();
        tokbuf.push(Tok::StrLiteral(StrLiteral { str_ref: StrRef::Slice(SOURCE_TEXT) }));
        let toks: Vec<Tok> = tokbuf.iter().collect();
        let Tok::StrLiteral(lit) = toks[0] else { panic!(); };
        assert_eq!(lit.str_ref.get(), SOURCE_TEXT);
    }

    #[test]
    fn test_ident() {
        let interner = StrInterner::default();
        let mut tokbuf = TokBuf::new(&interner);
        const SOURCE_TEXT: &'static [u8] = "main".as_bytes();
        tokbuf.push(Tok::Ident(Ident::new(SOURCE_TEXT)));
        let toks: Vec<Tok> = tokbuf.iter().collect();
        let Tok::Ident(ident) = toks[0] else { panic!(); };
        assert_eq!(ident.source_text.get(), SOURCE_TEXT);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EntryType {
    StaticPack = 1,
    StrLiteral = 2,
    DecIntLiteral = 3,
    Ident = 4,
    Linebreak = 5,
    Align = 6,
    LineComment = 7,
    Unexpected = 8
}

impl EntryType {
    fn variants() -> &'static [Self] {
        return &[
            Self::StaticPack,
            Self::StrLiteral,
            Self::DecIntLiteral,
            Self::Ident,
            Self::Linebreak,
            Self::Align,
            Self::LineComment,
            Self::Unexpected
        ];
    }
    
    fn id(self) -> NonZeroU8 {
        return NonZeroU8::try_from(self as u8).unwrap();
    }

    fn from_id(id: u8) -> Option<Self> {
        if id == 0 { return None; }
        return Self::variants().get(usize::from(id - 1)).copied();
    }
}

#[cfg(test)]
mod test_entry_type {
    use super::EntryType;

    #[test]
    fn test_id_conversion() {
        for variant in EntryType::variants() {
            let converted = EntryType::from_id(variant.id().get());
            assert_eq!(converted, Some(*variant));
        }
    }
}


/// A 24-bit bitstring dedicated to information beyond `kind` in a [`TokBufEntry`].
/// The etc-space usually holds an index into another table where the remainder
/// of the token's state is stored.
type Etc = u32;

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
struct TokBufEntry { data: u32 }

impl TokBufEntry {
    fn kind(self) -> EntryType {
        // The leftmost 8 bits in `data` are `kind_id`.
        let kind_id: u8 = (self.data >> (3 * 8)).truncate();
        return EntryType::from_id(kind_id).unwrap();
    }
    
    fn etc(self) -> Etc { 
        let kind_mask = u32::from(u8::MAX) << (3 * 8);
        return self.data & !kind_mask; 
    }

    fn new(kind: EntryType, etc: Etc) -> Self {
        // Make sure the leftmost 8 bits in `etc` are unused since they are reserved for `kind`.
        let kind_mask = u32::from(u8::MAX) << (3 * 8);
        assert_eq!(etc, etc & !kind_mask);

        let kind_bits = u32::from(kind.id().get()) << (3 * 8);
        let data = kind_bits | etc;
        return Self { data };
    }
}

#[derive(Clone, Copy)]
pub struct TokCursor<'a> {
    pos: Key,
    tokbuf: &'a TokBuf<'a>
}

impl<'a> TokCursor<'a> {
    pub fn new(tokbuf: &'a TokBuf<'a>) -> Self {
        Self { pos: Key::default(), tokbuf }
    }

    /// Returns the [`Tok`] at the cursor's position, or None if the cursor is at the
    /// end of the buffer.
    pub fn read(&self) -> Option<Tok<'a>> { return self.tokbuf.get(self.pos); }

    /// Returns a [`Key`] of the next token in the buffer. 
    /// Or, if no tokens remain, the key points to a nonexistent token immediately past
    /// the last real token in the buffer.
    pub fn at(&self) -> Key { return self.pos; }

    /// Advances the cursor past the next token in the buffer. 
    /// If no tokens remain, this is a no-op.
    pub fn forward(&mut self)  {
        if self.read().is_none() { return; }

        let next_pack_key = Key::new(self.pos.addr(), self.pos.pack_idx() + 1);
        if self.tokbuf.get(next_pack_key).is_some() {
            self.pos = next_pack_key;
            return;
        }

        let next_addr_key = Key::new(self.pos.addr() + 1, 0);
        self.pos = next_addr_key;
    }  
}

impl<'a> Iterator for TokCursor<'a> {
    type Item = Tok<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let elapsed = self.read();
        self.forward();
        return elapsed;
    }
}
