use core::fmt;

use crate::artifact::{ShaderArtifact, ShaderByteRange};
use crate::identity::{ShaderSourceRevision, ShaderSourceUnitIdentity};

/// Logical source subject attached to one RunenShader diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderSourceSubject {
    source_unit: ShaderSourceUnitIdentity,
    revision: ShaderSourceRevision,
}

impl ShaderSourceSubject {
    /// Forms a diagnostic subject from one exact logical source revision.
    pub const fn new(
        source_unit: ShaderSourceUnitIdentity,
        revision: ShaderSourceRevision,
    ) -> Self {
        Self {
            source_unit,
            revision,
        }
    }

    /// Returns the logical source-unit identity.
    pub const fn source_unit(self) -> ShaderSourceUnitIdentity {
        self.source_unit
    }

    /// Returns the logical source revision.
    pub const fn revision(self) -> ShaderSourceRevision {
        self.revision
    }
}

/// Minimal RunenShader-owned source-facing diagnostic evidence.
///
/// Stable diagnostic codes are intentionally not selected by the current specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderDiagnostic {
    summary: String,
    subject: Option<ShaderSourceSubject>,
    range: Option<ShaderByteRange>,
    realization_detail: Option<String>,
}

impl ShaderDiagnostic {
    /// Creates a diagnostic with RunenShader-owned summary text.
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            subject: None,
            range: None,
            realization_detail: None,
        }
    }

    /// Attaches exact logical source attribution and an optional truthful byte range.
    pub fn with_source(
        mut self,
        subject: ShaderSourceSubject,
        range: Option<ShaderByteRange>,
    ) -> Self {
        self.subject = Some(subject);
        self.range = range;
        self
    }

    /// Attaches clearly non-authoritative compiler-realization detail.
    pub fn with_realization_detail(mut self, detail: impl Into<String>) -> Self {
        self.realization_detail = Some(detail.into());
        self
    }

    /// Returns the RunenShader-owned diagnostic summary.
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Returns logical source attribution when known.
    pub const fn subject(&self) -> Option<ShaderSourceSubject> {
        self.subject
    }

    /// Returns the most precise truthful source byte range currently known.
    pub const fn range(&self) -> Option<ShaderByteRange> {
        self.range
    }

    /// Returns optional non-authoritative realization detail.
    pub fn realization_detail(&self) -> Option<&str> {
        self.realization_detail.as_deref()
    }
}

/// Ordinary public semantic result of one shader compilation invocation.
///
/// The dependency-free semantic kernel defines these classes but does not yet implement a compiler
/// path that produces them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShaderCompilationOutcome {
    /// One complete canonical artifact was formed.
    Accepted(ShaderArtifact),
    /// Explicit semantic input was invalid under the selected RunenShader/profile contract.
    Rejected(Vec<ShaderDiagnostic>),
    /// The request was semantically valid but outside the selected realization's accepted coverage.
    Unsupported(Vec<ShaderDiagnostic>),
    /// Operational realization failure prevented a semantic result.
    Failed(Vec<ShaderDiagnostic>),
}

/// RunenShader implementation-defect boundary, separate from ordinary compilation outcomes.
///
/// This type is intentionally not publicly constructible. A later compiler realization may return
/// it when an internal invariant is violated; it must never be relabeled as user rejection,
/// unsupported coverage, or ordinary tool failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderInvariantError {
    pub(crate) summary: &'static str,
}

impl ShaderInvariantError {
    /// Returns the internal invariant summary.
    pub const fn summary(&self) -> &'static str {
        self.summary
    }
}

impl fmt::Display for ShaderInvariantError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.summary)
    }
}

impl std::error::Error for ShaderInvariantError {}

/// Full public result shape for a future RunenShader compilation entry point.
pub type ShaderCompilationResult = Result<ShaderCompilationOutcome, ShaderInvariantError>;

#[cfg(test)]
mod tests {
    use crate::identity::{ShaderSourceRevision, ShaderSourceUnitIdentity};

    use super::*;

    #[test]
    fn ordinary_outcome_classes_remain_distinct() {
        let diagnostic = ShaderDiagnostic::new("invalid source").with_source(
            ShaderSourceSubject::new(
                ShaderSourceUnitIdentity::try_from_raw(1).unwrap(),
                ShaderSourceRevision::try_from_raw(2).unwrap(),
            ),
            ShaderByteRange::new(0, 1),
        );

        let rejected = ShaderCompilationOutcome::Rejected(vec![diagnostic.clone()]);
        let unsupported = ShaderCompilationOutcome::Unsupported(vec![diagnostic.clone()]);
        let failed = ShaderCompilationOutcome::Failed(vec![diagnostic]);

        assert!(matches!(rejected, ShaderCompilationOutcome::Rejected(_)));
        assert!(matches!(
            unsupported,
            ShaderCompilationOutcome::Unsupported(_)
        ));
        assert!(matches!(failed, ShaderCompilationOutcome::Failed(_)));
    }

    #[test]
    fn diagnostic_source_attribution_uses_logical_identity() {
        let subject = ShaderSourceSubject::new(
            ShaderSourceUnitIdentity::try_from_raw(7).unwrap(),
            ShaderSourceRevision::try_from_raw(8).unwrap(),
        );
        let range = ShaderByteRange::new(3, 5).unwrap();
        let diagnostic = ShaderDiagnostic::new("parse error")
            .with_source(subject, Some(range))
            .with_realization_detail("private compiler detail");

        assert_eq!(diagnostic.subject(), Some(subject));
        assert_eq!(diagnostic.range(), Some(range));
        assert_eq!(diagnostic.realization_detail(), Some("private compiler detail"));
    }
}
