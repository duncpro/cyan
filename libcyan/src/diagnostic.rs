use crate::source_unit::SourceUnitId;
use crate::util::inline_vec::InlineVec;
use crate::tok::tokbuf;

pub struct Diagnostic {
    elements: InlineVec<DiagnosticElement, 3>,
    severity: DiagnosticSeverity,
    title: &'static [u8],
}

pub enum DiagnosticElement {
    SourceQuote(SourceQuote),
    StaticMessage(&'static [u8])
}

pub struct SourceQuote {
    source_unit: SourceUnitId,
    indicated_toks: InlineVec<tokbuf::Key, 3>
}

#[repr(u8)]
pub enum DiagnosticSeverity { Err, Warn }
