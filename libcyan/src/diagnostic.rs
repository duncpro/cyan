use crate::source_unit::SourceUnitId;
use crate::util::inline_vec::InlineVec;
use crate::tok::tokbuf;

pub enum DiagnosticElement {
    SourceQuote(SourceQuote),
    StaticMessage(StaticMessage)
}

pub struct SourceQuote {
    source_unit: SourceUnitId,
    indicated_toks: InlineVec<tokbuf::Key, 3>
}

pub struct StaticMessage { message: &'static [u8] }
