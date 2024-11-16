use crate::source_unit::SourceUnitId;
use crate::util::inline_vec::InlineVec;
use crate::tok::tokbuf;

// -- Diagnostic ---------------------------------------------------------------------------------

pub trait Diagnostic {
    fn view(&self) -> DiagnosticView;
}

// -- DiagnosticView -----------------------------------------------------------------------------

pub struct DiagnosticView {
    severity: DiagnosticSeverity,
    title: &'static str,
    elements: InlineVec<DiagnosticViewElement, 3>,
}

pub enum DiagnosticViewElement {
    SourceQuote(SourceQuote),
    StaticMessage(&'static str)
}

pub struct SourceQuote {
    source_unit: SourceUnitId,
    indicated_toks: InlineVec<tokbuf::Key, 3>
}

#[repr(u8)]
pub enum DiagnosticSeverity { Err, Warn }

// -- AnyDiagnostic ------------------------------------------------------------------------------

pub enum AnyDiagnostic {
    MissingTok(MissingTok)
}

impl AnyDiagnostic {
    pub fn view(&self) -> DiagnosticView {
        match self {
            AnyDiagnostic::MissingTok(diag) => diag.view(),
        }
    }
}

// -- MissingTok ---------------------------------------------------------------------------------

pub struct MissingTok {
    source_unit: SourceUnitId,

    // TODO: `expected_tok: ?,`

    // The key of the next token in the source buffer.
    // The parser expected `expected_tok` to be at this key but it was not there.
    // This might point to a token, or it might point to the end of the buffer.
    at: tokbuf::Key,
}

impl Diagnostic for MissingTok {
    fn view(&self) -> DiagnosticView {
        DiagnosticView { 
            severity: DiagnosticSeverity::Err, 
            title: "Missing token", 
            elements: InlineVec::from_array([
                DiagnosticViewElement::SourceQuote(SourceQuote { 
                    source_unit: self.source_unit, 
                    indicated_toks: InlineVec::from_array([self.at]),
                })
            ]),
        }
    }
}

impl MissingTok {
    pub fn new(source_unit: SourceUnitId, tok: tokbuf::Key) -> Self {
        return Self { source_unit, at: tok };
    }
}

