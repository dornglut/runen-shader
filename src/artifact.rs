use core::fmt;
use std::sync::Arc;

use crate::identity::{
    ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision, ShaderSourceUnitIdentity,
};
use crate::input::{
    ShaderCompilationInputIdentity, ShaderCompilationInvocation, ShaderWeslFeatureValue,
    ShaderWeslModuleIdentity, ShaderWeslModulePath,
};
use crate::profile::{ShaderCompilerRealization, ShaderFrontendProfile};
use crate::source::ShaderSourceSnapshot;

/// Deterministic structural identity for one RunenShader artifact invocation.
///
/// This identity deliberately selects no digest or serialized representation. Equality is defined
/// structurally by the exact closed compilation-input identity and compiler realization identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderArtifactIdentity {
    compilation_input: Arc<ShaderCompilationInputIdentity>,
    profile: ShaderFrontendProfile,
    realization: ShaderCompilerRealization,
}

impl ShaderArtifactIdentity {
    /// Derives artifact identity deterministically from one complete semantic invocation.
    pub fn for_invocation(invocation: &ShaderCompilationInvocation) -> Self {
        let compilation_input = invocation.input().identity();
        Self {
            profile: compilation_input.profile(),
            compilation_input: Arc::new(compilation_input),
            realization: invocation.realization(),
        }
    }

    /// Returns the compilation-input identity participating in this artifact identity.
    pub fn compilation_input(&self) -> ShaderCompilationInputIdentity {
        self.compilation_input.as_ref().clone()
    }

    /// Returns the frontend-profile identity participating in this artifact identity.
    pub const fn profile(&self) -> ShaderFrontendProfile {
        self.profile
    }

    /// Returns the compiler-realization identity participating in this artifact identity.
    pub const fn realization(&self) -> ShaderCompilerRealization {
        self.realization
    }
}

/// RunenShader-owned provenance for the exact closed input and invocation that formed an artifact.
///
/// The closed compilation-input identity is the single authority for package/module/source/profile
/// participation; provenance does not duplicate those fields into independently mutable state.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    pub fn compilation_input(&self) -> ShaderCompilationInputIdentity {
        self.compilation_input.clone()
    }

    /// Returns the logical package identity.
    pub const fn package(&self) -> ShaderPackageIdentity {
        self.compilation_input.package()
    }

    /// Returns the logical root/main module identity.
    pub const fn root_module(&self) -> ShaderModuleIdentity {
        self.compilation_input.root_module()
    }

    /// Returns the logical root/main source-unit identity.
    ///
    /// For exact WGSL this is the sole source. For WESL this identifies the explicit main-module
    /// source; use [`Self::wesl_modules`] for complete participating-source provenance.
    pub const fn source_unit(&self) -> ShaderSourceUnitIdentity {
        self.compilation_input.source_unit()
    }

    /// Returns the exact root/main source revision.
    ///
    /// For exact WGSL this is the sole revision. For WESL use [`Self::wesl_modules`] for complete
    /// participating-source provenance.
    pub const fn source_revision(&self) -> ShaderSourceRevision {
        self.compilation_input.source_revision()
    }

    /// Returns the frontend profile.
    pub const fn profile(&self) -> ShaderFrontendProfile {
        self.compilation_input.profile()
    }

    /// Returns the compiler realization.
    pub const fn realization(&self) -> ShaderCompilerRealization {
        self.realization
    }

    /// Returns the explicit WESL main-module resolution key when this provenance is WESL-shaped.
    pub fn wesl_root_resolution_path(&self) -> Option<&ShaderWeslModulePath> {
        self.compilation_input.wesl_root_resolution_path()
    }

    /// Returns deterministic structural evidence for every admitted WESL module, including root.
    pub fn wesl_modules(&self) -> Option<&[ShaderWeslModuleIdentity]> {
        self.compilation_input.wesl_modules()
    }

    /// Returns deterministic explicit WESL conditional-feature evidence.
    pub fn wesl_features(&self) -> Option<&[ShaderWeslFeatureValue]> {
        self.compilation_input.wesl_features()
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

/// Error returned when a byte range lies outside one artifact mapping.
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

    /// Returns the covered source/artifact byte length.
    pub const fn source_byte_len(self) -> usize {
        self.byte_len
    }

    /// Returns the covered artifact byte length.
    pub const fn artifact_byte_len(self) -> usize {
        self.byte_len
    }
}

impl fmt::Display for ShaderRangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "shader byte range [{}, {}) exceeds artifact length {}",
            self.range.start, self.range.end, self.byte_len
        )
    }
}

impl std::error::Error for ShaderRangeError {}

/// Source-side result of mapping a canonical-artifact range.
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

    /// Returns the logical source revision.
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
        check_range(range, self.byte_len)?;
        Ok(ShaderMappedSourceRange {
            source_unit: self.source_unit,
            revision: self.revision,
            range,
        })
    }
}

/// Truthful mapping classification for one requested canonical-artifact byte range.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderArtifactRangeMapping {
    /// The artifact range is byte-identical to one logical source range.
    ExactSource(ShaderMappedSourceRange),
    /// The artifact range is proven to be generated rather than copied from authored source bytes.
    Generated(ShaderByteRange),
    /// The realization cannot truthfully classify the artifact range more precisely.
    Unattributable(ShaderByteRange),
}

impl ShaderArtifactRangeMapping {
    /// Returns the requested artifact range represented by this mapping result.
    pub const fn artifact_range(self) -> ShaderByteRange {
        match self {
            Self::ExactSource(mapped) => mapped.range(),
            Self::Generated(range) | Self::Unattributable(range) => range,
        }
    }

    /// Returns an exact logical source mapping when one is proven.
    pub const fn exact_source(self) -> Option<ShaderMappedSourceRange> {
        match self {
            Self::ExactSource(mapped) => Some(mapped),
            Self::Generated(_) | Self::Unattributable(_) => None,
        }
    }
}

/// Closed mapping representation for accepted canonical artifacts.
///
/// Exact WGSL has a total byte-identity mapping. Transformed realizations can classify total
/// generated output when that fact is proven, or conservatively retain total unattributable
/// coverage when no stronger byte relation has been established.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderArtifactSourceMap {
    /// Total byte-identity mapping for an exact-WGSL artifact.
    ExactIdentity(ExactWgslSourceMap),
    /// Total artifact coverage proven to be generated.
    Generated { byte_len: usize },
    /// Total artifact coverage with no proven byte-level attribution classification.
    Unattributable { byte_len: usize },
}

impl ShaderArtifactSourceMap {
    /// Forms the exact identity mapping used by the exact-WGSL realization.
    pub fn exact(source: &ShaderSourceSnapshot) -> Self {
        Self::ExactIdentity(ExactWgslSourceMap::for_source(source))
    }

    /// Forms total generated coverage for one transformed artifact.
    pub const fn generated(byte_len: usize) -> Self {
        Self::Generated { byte_len }
    }

    /// Forms total unattributable coverage for one transformed artifact.
    pub const fn unattributable(byte_len: usize) -> Self {
        Self::Unattributable { byte_len }
    }

    /// Returns the canonical artifact byte length covered by this map.
    pub const fn byte_len(self) -> usize {
        match self {
            Self::ExactIdentity(map) => map.byte_len(),
            Self::Generated { byte_len } | Self::Unattributable { byte_len } => byte_len,
        }
    }

    /// Maps one in-bounds artifact range using the strongest truthful evidence available.
    pub fn map_artifact_range(
        self,
        range: ShaderByteRange,
    ) -> Result<ShaderArtifactRangeMapping, ShaderRangeError> {
        check_range(range, self.byte_len())?;
        Ok(match self {
            Self::ExactIdentity(map) => {
                ShaderArtifactRangeMapping::ExactSource(map.map_artifact_range(range)?)
            }
            Self::Generated { .. } => ShaderArtifactRangeMapping::Generated(range),
            Self::Unattributable { .. } => ShaderArtifactRangeMapping::Unattributable(range),
        })
    }
}

fn check_range(range: ShaderByteRange, byte_len: usize) -> Result<(), ShaderRangeError> {
    if range.end() > byte_len {
        Err(ShaderRangeError { range, byte_len })
    } else {
        Ok(())
    }
}

/// Accepted canonical shader artifact data.
///
/// Ordinary callers cannot construct this type directly. Accepted realizations form it only after
/// frontend/profile processing and independent canonical-WGSL admission have completed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderArtifact {
    pub(crate) identity: ShaderArtifactIdentity,
    pub(crate) canonical_wgsl: Arc<str>,
    pub(crate) provenance: ShaderArtifactProvenance,
    pub(crate) source_map: ShaderArtifactSourceMap,
}

impl ShaderArtifact {
    /// Returns the deterministic artifact identity.
    pub fn identity(&self) -> ShaderArtifactIdentity {
        self.identity.clone()
    }

    /// Returns exact canonical WGSL bytes as UTF-8 text.
    pub fn canonical_wgsl(&self) -> &str {
        &self.canonical_wgsl
    }

    /// Returns RunenShader-owned artifact provenance.
    pub const fn provenance(&self) -> &ShaderArtifactProvenance {
        &self.provenance
    }

    /// Returns total mapping coverage for this canonical artifact.
    pub const fn source_map(&self) -> ShaderArtifactSourceMap {
        self.source_map
    }
}

#[cfg(test)]
mod tests {
    use crate::identity::{
        ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision, ShaderSourceUnitIdentity,
    };
    use crate::input::{
        ShaderCompilationInput, ShaderWeslFeatureValue, ShaderWeslModuleBinding,
        ShaderWeslModulePath,
    };
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

    fn wesl_binding(path: &str, module: u64, unit: u64) -> ShaderWeslModuleBinding {
        ShaderWeslModuleBinding::new(
            ShaderWeslModulePath::new(path),
            ShaderModuleIdentity::try_from_raw(module).unwrap(),
            ShaderSourceSnapshot::new(
                ShaderSourceUnitIdentity::try_from_raw(unit).unwrap(),
                ShaderSourceRevision::try_from_raw(1).unwrap(),
                "fn helper() {}",
            ),
        )
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
    fn provenance_can_expose_complete_wesl_input_evidence_without_a_realization() {
        let input = ShaderCompilationInput::wesl_composition(
            ShaderPackageIdentity::try_from_raw(40).unwrap(),
            wesl_binding("package::main", 41, 51),
            vec![
                wesl_binding("package::math", 42, 52),
                wesl_binding("package::util", 43, 53),
            ],
            vec![ShaderWeslFeatureValue::new("debug", true)],
        );
        let invocation = ShaderCompilationInvocation::new(
            input,
            ShaderCompilerRealization::Naga3001ExactWgslGateV1,
        );
        let provenance = ShaderArtifactProvenance::for_invocation(&invocation);

        assert_eq!(
            provenance.profile(),
            ShaderFrontendProfile::WeslComposition20260822
        );
        assert_eq!(provenance.root_module().diagnostic_raw(), 41);
        assert_eq!(provenance.source_unit().diagnostic_raw(), 51);
        assert_eq!(
            provenance.wesl_root_resolution_path().unwrap().as_str(),
            "package::main"
        );
        let modules = provenance.wesl_modules().unwrap();
        assert_eq!(modules.len(), 3);
        assert_eq!(modules[0].source_unit().diagnostic_raw(), 51);
        assert_eq!(modules[1].source_unit().diagnostic_raw(), 52);
        assert_eq!(modules[2].source_unit().diagnostic_raw(), 53);
        assert_eq!(provenance.wesl_features().unwrap()[0].name(), "debug");
    }

    #[test]
    fn exact_source_map_is_total_identity_mapping_with_bounds_checks() {
        let invocation = sample_invocation("abc\u{00e9}");
        let source = invocation.input().source();
        let map = ShaderArtifactSourceMap::exact(source);
        let full = ShaderByteRange::new(0, source.byte_len()).unwrap();
        let mapped = map.map_artifact_range(full).unwrap();
        let exact = mapped.exact_source().unwrap();

        assert_eq!(exact.source_unit(), source.source_unit());
        assert_eq!(exact.revision(), source.revision());
        assert_eq!(exact.range(), full);
        assert_eq!(map.byte_len(), source.byte_len());

        let empty_at_end = ShaderByteRange::new(source.byte_len(), source.byte_len()).unwrap();
        assert!(map.map_artifact_range(empty_at_end).is_ok());
        assert!(ShaderByteRange::new(3, 2).is_none());
        let out_of_bounds = ShaderByteRange::new(0, source.byte_len() + 1).unwrap();
        assert!(map.map_artifact_range(out_of_bounds).is_err());
    }

    #[test]
    fn generated_and_unattributable_maps_are_total_without_inventing_source_identity() {
        let range = ShaderByteRange::new(2, 7).unwrap();
        let generated = ShaderArtifactSourceMap::generated(9);
        assert_eq!(
            generated.map_artifact_range(range).unwrap(),
            ShaderArtifactRangeMapping::Generated(range)
        );

        let unattributable = ShaderArtifactSourceMap::unattributable(9);
        assert_eq!(
            unattributable.map_artifact_range(range).unwrap(),
            ShaderArtifactRangeMapping::Unattributable(range)
        );
        assert!(
            unattributable
                .map_artifact_range(ShaderByteRange::new(0, 10).unwrap())
                .is_err()
        );
    }

    #[test]
    fn artifact_data_preserves_the_exact_source_bytes() {
        let invocation = sample_invocation("\u{feff}// exact\r\nfn helper() { }\n");
        let source = invocation.input().source();
        let artifact = ShaderArtifact {
            identity: ShaderArtifactIdentity::for_invocation(&invocation),
            canonical_wgsl: Arc::clone(&source.source),
            provenance: ShaderArtifactProvenance::for_invocation(&invocation),
            source_map: ShaderArtifactSourceMap::exact(source),
        };

        assert_eq!(
            artifact.canonical_wgsl().as_bytes(),
            source.text().as_bytes()
        );
        assert_eq!(artifact.source_map().byte_len(), source.byte_len());
        assert!(matches!(
            artifact.source_map(),
            ShaderArtifactSourceMap::ExactIdentity(_)
        ));
    }
}
