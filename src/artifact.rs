use core::fmt;
use std::sync::Arc;

use crate::identity::{
    ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision, ShaderSourceUnitIdentity,
};
use crate::input::{ShaderCompilationInputIdentity, ShaderCompilationInvocation};
use crate::profile::{ShaderCompilerRealization, ShaderFrontendProfile};
use crate::source::ShaderSourceSnapshot;

/// Deterministic structural identity for one RunenShader artifact invocation.
///
/// This identity deliberately selects no digest or serialized representation. Equality is defined
/// structurally by the exact closed compilation-input identity and compiler realization identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderArtifactIdentity {
    compilation_input: ShaderCompilationInputIdentity,
    realization: ShaderCompilerRealization,
}

impl ShaderArtifactIdentity {
    /// Derives artifact identity deterministically from one complete semantic invocation.
    pub fn for_invocation(invocation: &ShaderCompilationInvocation) -> Self {
        Self {
            compilation_input: invocation.input().identity(),
            realization: invocation.realization(),
        }
    }

    /// Returns the compilation-input identity participating in this artifact identity.
    pub const fn compilation_input(self) -> ShaderCompilationInputIdentity {
        self.compilation_input
    }

    /// Returns the frontend-profile identity participating in this artifact identity.
    pub const fn profile(self) -> ShaderFrontendProfile {
        self.compilation_input.profile()
    }

    /// Returns the compiler-realization identity participating in this artifact identity.
    pub const fn realization(self) -> ShaderCompilerRealization {
        self.realization
    }
}

/// RunenShader-owned provenance for the exact source and invocation that formed an artifact.
///
/// The closed compilation-input identity is the single authority for package/module/source/profile
/// participation; provenance does not duplicate those fields into independently mutable state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderArtifactProvenance {
    compilation_input: ShaderCompilationInputIdentity,
    realization: ShaderCompilerRealization,
}

impl ShaderArtifactProvenance {
    /// Derives provenance from the exact closed invocation.
    pub fn for_invocation(invocation: &ShaderCompilationInvocation) -> Self {
        Self {
            compilation_input: invocation.input().identity(),
            realization: invocation.realization(),
        }
    }

    /// Returns the exact closed compilation-input identity.
    pub const fn compilation_input(self) -> ShaderCompilationInputIdentity {
        self.compilation_input
    }

    /// Returns the logical package identity.
    pub const fn package(self) -> ShaderPackageIdentity {
        self.compilation_input.package()
    }

    /// Returns the logical root-module identity.
    pub const fn root_module(self) -> ShaderModuleIdentity {
        self.compilation_input.root_module()
    }

    /// Returns the logical source-unit identity.
    pub const fn source_unit(self) -> ShaderSourceUnitIdentity {
        self.compilation_input.source_unit()
    }

    /// Returns the exact source revision.
    pub const fn source_revision(self) -> ShaderSourceRevision {
        self.compilation_input.source_revision()
    }

    /// Returns the frontend profile.
    pub const fn profile(self) -> ShaderFrontendProfile {
        self.compilation_input.profile()
    }

    /// Returns the compiler realization.
    pub const fn realization(self) -> ShaderCompilerRealization {
        self.realization
    }
}

/// Zero-based half-open byte range `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderByteRange {
    start: usize,
    end: usize,
}

impl ShaderByteRange {
    /// Constructs a non-reversed byte range.
    pub const fn new(start: usize, end: usize) -> Option<Self> {
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    /// Returns the inclusive start byte offset.
    pub const fn start(self) -> usize {
        self.start
    }

    /// Returns the exclusive end byte offset.
    pub const fn end(self) -> usize {
        self.end
    }

    /// Returns the byte length of the range.
    pub const fn byte_len(self) -> usize {
        self.end - self.start
    }

    /// Reports whether this is an empty in-place range.
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// Error returned when a byte range lies outside one exact source/artifact mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShaderRangeError {
    range: ShaderByteRange,
    byte_len: usize,
}

impl ShaderRangeError {
    /// Returns the requested range.
    pub const fn range(self) -> ShaderByteRange {
        self.range
    }

    /// Returns the mapped source/artifact byte length.
    pub const fn source_byte_len(self) -> usize {
        self.byte_len
    }
}

impl fmt::Display for ShaderRangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "shader byte range [{}, {}) exceeds source length {}",
            self.range.start, self.range.end, self.byte_len
        )
    }
}

impl std::error::Error for ShaderRangeError {}

/// Source-side result of mapping an exact canonical-WGSL artifact range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderMappedSourceRange {
    source_unit: ShaderSourceUnitIdentity,
    revision: ShaderSourceRevision,
    range: ShaderByteRange,
}

impl ShaderMappedSourceRange {
    /// Returns the logical source-unit identity.
    pub const fn source_unit(self) -> ShaderSourceUnitIdentity {
        self.source_unit
    }

    /// Returns the exact source revision.
    pub const fn revision(self) -> ShaderSourceRevision {
        self.revision
    }

    /// Returns the mapped half-open UTF-8 byte range.
    pub const fn range(self) -> ShaderByteRange {
        self.range
    }
}

/// Compact total identity mapping for the exact-WGSL profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExactWgslSourceMap {
    source_unit: ShaderSourceUnitIdentity,
    revision: ShaderSourceRevision,
    byte_len: usize,
}

impl ExactWgslSourceMap {
    /// Forms the total identity mapping for one exact source snapshot.
    pub fn for_source(source: &ShaderSourceSnapshot) -> Self {
        Self {
            source_unit: source.source_unit(),
            revision: source.revision(),
            byte_len: source.byte_len(),
        }
    }

    /// Returns the exact source/artifact byte length covered by this map.
    pub const fn byte_len(self) -> usize {
        self.byte_len
    }

    /// Maps an artifact byte range to the identical source byte range.
    pub fn map_artifact_range(
        self,
        range: ShaderByteRange,
    ) -> Result<ShaderMappedSourceRange, ShaderRangeError> {
        if range.end() > self.byte_len {
            return Err(ShaderRangeError {
                range,
                byte_len: self.byte_len,
            });
        }

        Ok(ShaderMappedSourceRange {
            source_unit: self.source_unit,
            revision: self.revision,
            range,
        })
    }
}

/// Accepted canonical shader artifact data.
///
/// Ordinary callers cannot construct this type directly. A later accepted compiler realization
/// forms it only after successful profile validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderArtifact {
    pub(crate) identity: ShaderArtifactIdentity,
    pub(crate) canonical_wgsl: Arc<str>,
    pub(crate) provenance: ShaderArtifactProvenance,
    pub(crate) source_map: ExactWgslSourceMap,
}

impl ShaderArtifact {
    /// Returns the deterministic artifact identity.
    pub const fn identity(&self) -> ShaderArtifactIdentity {
        self.identity
    }

    /// Returns exact canonical WGSL bytes as UTF-8 text.
    pub fn canonical_wgsl(&self) -> &str {
        &self.canonical_wgsl
    }

    /// Returns RunenShader-owned artifact provenance.
    pub const fn provenance(&self) -> &ShaderArtifactProvenance {
        &self.provenance
    }

    /// Returns the exact-WGSL identity source map.
    pub const fn source_map(&self) -> ExactWgslSourceMap {
        self.source_map
    }
}

#[cfg(test)]
mod tests {
    use crate::identity::{
        ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision, ShaderSourceUnitIdentity,
    };
    use crate::input::ShaderCompilationInput;
    use crate::profile::ShaderCompilerRealization;

    use super::*;

    fn sample_invocation(source_text: &str) -> ShaderCompilationInvocation {
        let input = ShaderCompilationInput::exact_wgsl(
            ShaderPackageIdentity::try_from_raw(12).unwrap(),
            ShaderModuleIdentity::try_from_raw(13).unwrap(),
            ShaderSourceSnapshot::new(
                ShaderSourceUnitIdentity::try_from_raw(14).unwrap(),
                ShaderSourceRevision::try_from_raw(15).unwrap(),
                source_text,
            ),
        );
        ShaderCompilationInvocation::new(input, ShaderCompilerRealization::Naga3001ExactWgslGateV1)
    }

    #[test]
    fn artifact_identity_and_provenance_are_deterministic_from_invocation() {
        let left = sample_invocation("// exact\r\nfn helper() {}\n");
        let right = sample_invocation("// exact\r\nfn helper() {}\n");

        assert_eq!(
            ShaderArtifactIdentity::for_invocation(&left),
            ShaderArtifactIdentity::for_invocation(&right)
        );
        assert_eq!(
            ShaderArtifactProvenance::for_invocation(&left),
            ShaderArtifactProvenance::for_invocation(&right)
        );
    }

    #[test]
    fn exact_source_map_is_total_identity_mapping_with_bounds_checks() {
        let invocation = sample_invocation("abc\u{00e9}");
        let source = invocation.input().source();
        let map = ExactWgslSourceMap::for_source(source);
        let full = ShaderByteRange::new(0, source.byte_len()).unwrap();
        let mapped = map.map_artifact_range(full).unwrap();

        assert_eq!(mapped.source_unit(), source.source_unit());
        assert_eq!(mapped.revision(), source.revision());
        assert_eq!(mapped.range(), full);
        assert_eq!(map.byte_len(), source.byte_len());

        let empty_at_end = ShaderByteRange::new(source.byte_len(), source.byte_len()).unwrap();
        assert!(map.map_artifact_range(empty_at_end).is_ok());
        assert!(ShaderByteRange::new(3, 2).is_none());
        let out_of_bounds = ShaderByteRange::new(0, source.byte_len() + 1).unwrap();
        assert!(map.map_artifact_range(out_of_bounds).is_err());
    }

    #[test]
    fn artifact_data_preserves_the_exact_source_bytes() {
        let invocation = sample_invocation("\u{feff}// exact\r\nfn helper() { }\n");
        let source = invocation.input().source();
        let artifact = ShaderArtifact {
            identity: ShaderArtifactIdentity::for_invocation(&invocation),
            canonical_wgsl: Arc::clone(&source.source),
            provenance: ShaderArtifactProvenance::for_invocation(&invocation),
            source_map: ExactWgslSourceMap::for_source(source),
        };

        assert_eq!(
            artifact.canonical_wgsl().as_bytes(),
            source.text().as_bytes()
        );
        assert_eq!(artifact.source_map().byte_len(), source.byte_len());
    }
}
