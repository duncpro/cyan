use std::num::NonZeroU8;
use crate::util::string_interner::StringInterner;
use crate::util::bits::Truncate;
use crate::util::ascii;
use crate::tok::ident::Ident;
use crate::tok::tok::{Tok, DecIntLiteral, StaticTok, StrLiteral, LineComment, Spaces, Linebreaks,
    Unexpected};

pub struct TokBuf<'a> {
    string_interner: &'a StringInterner,
    str_table: Vec<u8>,
    buf: Vec<TokBufEntry>,
    lines: Vec<u32>
}

impl<'a> TokBuf<'a> {
    pub fn new(string_interner: &'a StringInterner) -> TokBuf<'a> {
        return Self { 
            string_interner,
            str_table: Vec::new(),
            buf: Vec::new(),
            lines: Vec::new()
        };
    }
    
    pub fn push(&mut self, tok: &Tok) {
        let tok_addr = u32::try_from(self.buf.len()).unwrap();
        
        let kind = match tok {
            Tok::Static(_) => TokType::Static,
            Tok::StrLiteral(_) => TokType::StrLiteral,
            Tok::DecIntLiteral(_) => TokType::DecIntLiteral,
            Tok::Ident(_) => TokType::Ident,
            Tok::Linebreaks(_) => TokType::Linebreaks,
            Tok::Spaces(_) => TokType::Spaces,
            Tok::LineComment(_) => TokType::LineComment,
            Tok::Unexpected(_) => TokType::Unexpected,
        };

        let etc = match tok {
            Tok::Static(stok) => u32::from(stok.id().get()),
            Tok::StrLiteral(lit) => self.insert_str_table_entry(lit.source_text()),
            Tok::DecIntLiteral(lit) => self.insert_str_table_entry(lit.digits()),
            Tok::Ident(ident) => self.string_interner.intern(ident.source_text()).get(),
            Tok::Linebreaks(lbs) => lbs.count(),
            Tok::Spaces(spaces) => spaces.count(),
            Tok::LineComment(lc) => self.insert_str_table_entry(lc.content()),
            Tok::Unexpected(unexpected) => u32::from(unexpected.ch),
        };

        self.buf.push(TokBufEntry::new(kind, etc));   

        if let Tok::Linebreaks(lbs) = tok {
            for _ in 0..(lbs.count()) {
                self.lines.push(tok_addr);
            }
        }

        if let Tok::StrLiteral(lit) = tok {
            for ch in lit.source_text() {
                if *ch == ascii::LINEBREAK {
                    self.lines.push(tok_addr);
                }
            }
        }
    }

    pub fn get(&'a self, idx: usize) -> Option<Tok<'a>> {
        let tbe = self.buf.get(idx)?;
        match tbe.kind() {
            TokType::Static => {
                let stok_id = u8::try_from(tbe.etc()).unwrap();
                let stok = StaticTok::from_id(stok_id).unwrap();
                return Some(Tok::Static(stok));
            },
            TokType::StrLiteral => {
                let source_text = self.lookup_str_table_entry(tbe.etc());
                return Some(Tok::StrLiteral(StrLiteral::new(source_text)));
            },
            TokType::DecIntLiteral => {
                let digits = self.lookup_str_table_entry(tbe.etc());
                return Some(Tok::DecIntLiteral(DecIntLiteral::new(digits)));
            },
            TokType::Ident => {
                let source_text = self.string_interner.lookup_str(tbe.etc());
                return Some(Tok::Ident(Ident::new(source_text)));
            },
            TokType::Linebreaks => {
                let count = tbe.etc();
                return Some(Tok::Linebreaks(Linebreaks::new(count)));
            },
            TokType::Spaces => {
                let count = tbe.etc();
                return Some(Tok::Spaces(Spaces::new(count)));
            },
            TokType::LineComment => {
                let content = self.lookup_str_table_entry(tbe.etc());
                return Some(Tok::LineComment(LineComment::new(content)));            
            },
            TokType::Unexpected => {
                let ch: u8 = tbe.etc().truncate();
                return Some(Tok::Unexpected(Unexpected { ch }));
            }
        }
    }

    fn insert_str_table_entry(&mut self, entry: &[u8]) -> Etc {
        assert!(!entry.contains(&0));
        let str_table_key = u32::try_from(self.str_table.len()).unwrap();
        self.str_table.extend_from_slice(entry);
        self.str_table.push(0);
        return str_table_key;
    }

    fn lookup_str_table_entry(&self, key: Etc) -> &[u8] {
        let idx = usize::try_from(key).unwrap();
        let len = self.str_table[idx..].iter().copied()
            .take_while(|ch| *ch != 0).count();
        return &self.str_table[idx..(idx + len)];
    }

    /// Returns the 0-based index of the line where the token begins.
    fn get_line_no(&self, tok_addr: u32) -> usize {
        return self.lines.partition_point(|lb_addr| *lb_addr < tok_addr);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TokType {
    Static = 1,
    StrLiteral = 2,
    DecIntLiteral = 3,
    Ident = 4,
    Linebreaks = 5,
    Spaces = 6,
    LineComment = 7,
    Unexpected = 8
}

impl TokType {
    fn variants() -> &'static [Self] {
        return &[
            Self::Static,
            Self::StrLiteral,
            Self::DecIntLiteral,
            Self::Ident,
            Self::Linebreaks,
            Self::Spaces,
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
mod test_tok_type {
    use super::TokType;

    #[test]
    fn test_id_conversion() {
        for variant in TokType::variants() {
            let converted = TokType::from_id(variant.id().get());
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
    fn kind(self) -> TokType {
        // The leftmost 8 bits in `data` are `kind_id`.
        let kind_id: u8 = (self.data >> (3 * 8)).truncate();
        return TokType::from_id(kind_id).unwrap();
    }
    
    fn etc(self) -> Etc { return self.data >> 8; }

    fn new(kind: TokType, etc: Etc) -> Self {
        // Make sure the leftmost 8 bits in `etc` are unused,
        // since they are reserved for `kind`.
        let kind_mask = u32::from(u8::MAX) << (3 * 8);
        assert!(etc == (etc & kind_mask));

        let kind_bits = u32::from(kind.id().get()) << (3 * 8);
        let data = kind_bits | etc;
        return Self { data };
    }
}

