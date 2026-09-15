use std::sync::Arc;

use crate::{ShaderSourceRevision, ShaderSourceUnitIdentity};

/// Immutable exact UTF-8 source snapshot bound to one logical source-unit revision.
///
/// Reusing the same `(source unit, revision)` pair for different bytes violates the normative
/// RunenShader identity contract. This value deliberately performs no filesystem discovery,
/// normalization, hashing, or process-global registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderSourceSnapshot {
    source_unit: ShaderSourceUnitIdentity,
    revision: ShaderSourceRevision,
    pub(crate) source: Arc<str>,
}

impl ShaderSourceSnapshot {
    /// Captures exact UTF-8 source text under an explicit logical identity and revision.
    pub fn new(
        source_unit: ShaderSourceUnitIdentity,
        revision: ShaderSourceRevision,
        source: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            source_unit,
            revision,
            source: source.into(),
        }
    }

    /// Returns the logical source-unit identity.
    pub const fn source_unit(&self) -> ShaderSourceUnitIdentity {
        self.source_unit
    }

    /// Returns the logical source revision.
    pub const fn revision(&self) -> ShaderSourceRevision {
        self.revision
    }

    /// Returns the exact source text without normalization or re-emission.
    pub fn text(&self) -> &str {
        &self.source
    }

    /// Returns the exact UTF-8 byte length of the source snapshot.
    pub fn byte_len(&self) -> usize {
        self.source.len()
    }

    /// Reports whether the exact source snapshot is empty.
    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_preserves_exact_utf8_bytes() {
        let unit = ShaderSourceUnitIdentity::try_from_raw(1).unwrap();
        let revision = ShaderSourceRevision::try_from_raw(9).unwrap();
        let source = "\u{feff}// comment\r\nfn helper() {  }\n";
        let snapshot = ShaderSourceSnapshot::new(unit, revision, source);

        assert_eq!(snapshot.source_unit(), unit);
        assert_eq!(snapshot.revision(), revision);
        assert_eq!(snapshot.text().as_bytes(), source.as_bytes());
        assert_eq!(snapshot.byte_len(), source.len());
        assert!(!snapshot.is_empty());
    }

    #[test]
    fn equal_bytes_do_not_merge_distinct_logical_snapshots() {
        let first = ShaderSourceSnapshot::new(
            ShaderSourceUnitIdentity::try_from_raw(1).unwrap(),
            ShaderSourceRevision::try_from_raw(1).unwrap(),
            "fn helper() {}",
        );
        let second = ShaderSourceSnapshot::new(
            ShaderSourceUnitIdentity::try_from_raw(2).unwrap(),
            ShaderSourceRevision::try_from_raw(1).unwrap(),
            "fn helper() {}",
        );

        assert_ne!(first, second);
        assert_eq!(first.text(), second.text());
    }
}
