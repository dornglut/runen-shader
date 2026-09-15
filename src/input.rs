use std::cmp::Ordering;
use std::sync::Arc;

use crate::identity::{
    ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision, ShaderSourceUnitIdentity,
};
use crate::profile::{ShaderCompilerRealization, ShaderFrontendProfile};
use crate::source::ShaderSourceSnapshot;

/// RunenShader-owned spelling of one WESL module-resolution key.
///
/// Construction deliberately preserves the supplied spelling without validating profile admission.
/// The future WESL realization is responsible for rejecting non-canonical, malformed, external-
/// package, or otherwise out-of-profile keys. This value is a resolution fact, not logical module
/// or source identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderWeslModulePath(Arc<str>);

impl ShaderWeslModulePath {
    /// Captures one WESL module-resolution-key spelling without filesystem interpretation.
    pub fn new(path: impl Into<Arc<str>>) -> Self {
        Self(path.into())
    }

    /// Returns the preserved resolution-key spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One explicit WESL conditional-translation feature value.
///
/// Feature spelling is preserved exactly. Invalid or duplicate feature entries remain representable
/// so the future WESL realization can classify them under the accepted profile instead of this data
/// layer silently repairing caller input.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderWeslFeatureValue {
    name: Arc<str>,
    enabled: bool,
}

impl ShaderWeslFeatureValue {
    /// Captures one explicit feature name/value pair.
    pub fn new(name: impl Into<Arc<str>>, enabled: bool) -> Self {
        Self {
            name: name.into(),
            enabled,
        }
    }

    /// Returns the preserved feature name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the explicit boolean feature value.
    pub const fn enabled(&self) -> bool {
        self.enabled
    }
}

/// One admitted logical WESL module and its exact source snapshot.
///
/// The resolution path is not identity. The logical module identity and source-unit/revision remain
/// explicit RunenShader authority even when multiple inputs use the same WESL path spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderWeslModuleBinding {
    resolution_path: ShaderWeslModulePath,
    module: ShaderModuleIdentity,
    source: ShaderSourceSnapshot,
}

impl ShaderWeslModuleBinding {
    /// Forms one WESL resolution-path to logical-module/source binding.
    pub fn new(
        resolution_path: ShaderWeslModulePath,
        module: ShaderModuleIdentity,
        source: ShaderSourceSnapshot,
    ) -> Self {
        Self {
            resolution_path,
            module,
            source,
        }
    }

    /// Returns the WESL resolution-key spelling.
    pub const fn resolution_path(&self) -> &ShaderWeslModulePath {
        &self.resolution_path
    }

    /// Returns the logical RunenShader module identity.
    pub const fn module(&self) -> ShaderModuleIdentity {
        self.module
    }

    /// Returns the exact immutable source snapshot bound to this module.
    pub const fn source(&self) -> &ShaderSourceSnapshot {
        &self.source
    }
}

/// Structural identity evidence for one WESL module binding.
///
/// Exact source bytes are intentionally absent. They remain governed by the normative
/// `(source unit, revision) -> exact bytes` binding invariant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderWeslModuleIdentity {
    resolution_path: ShaderWeslModulePath,
    module: ShaderModuleIdentity,
    source_unit: ShaderSourceUnitIdentity,
    source_revision: ShaderSourceRevision,
}

impl ShaderWeslModuleIdentity {
    fn for_binding(binding: &ShaderWeslModuleBinding) -> Self {
        Self {
            resolution_path: binding.resolution_path.clone(),
            module: binding.module,
            source_unit: binding.source.source_unit(),
            source_revision: binding.source.revision(),
        }
    }

    /// Returns the WESL resolution-key spelling participating in the closed input.
    pub const fn resolution_path(&self) -> &ShaderWeslModulePath {
        &self.resolution_path
    }

    /// Returns the logical module identity.
    pub const fn module(&self) -> ShaderModuleIdentity {
        self.module
    }

    /// Returns the exact logical source-unit identity.
    pub const fn source_unit(&self) -> ShaderSourceUnitIdentity {
        self.source_unit
    }

    /// Returns the exact source revision.
    pub const fn source_revision(&self) -> ShaderSourceRevision {
        self.source_revision
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ShaderCompilationInputIdentityKind {
    ExactWgsl,
    WeslComposition {
        root_resolution_path: ShaderWeslModulePath,
        modules: Arc<[ShaderWeslModuleIdentity]>,
        features: Arc<[ShaderWeslFeatureValue]>,
    },
}

/// Opaque structural identity for one exact closed semantic compilation input.
///
/// Package, root/main module, root/main source-unit/revision, frontend profile, and all
/// profile-specific closed-input evidence participate structurally. Exact source bytes remain bound
/// separately by the normative `(source unit, revision)` reuse invariant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderCompilationInputIdentity {
    package: ShaderPackageIdentity,
    root_module: ShaderModuleIdentity,
    source_unit: ShaderSourceUnitIdentity,
    source_revision: ShaderSourceRevision,
    profile: ShaderFrontendProfile,
    kind: ShaderCompilationInputIdentityKind,
}

impl ShaderCompilationInputIdentity {
    fn exact_wgsl(
        package: ShaderPackageIdentity,
        root_module: ShaderModuleIdentity,
        source: &ShaderSourceSnapshot,
    ) -> Self {
        Self {
            package,
            root_module,
            source_unit: source.source_unit(),
            source_revision: source.revision(),
            profile: ShaderFrontendProfile::WgslExact20260817,
            kind: ShaderCompilationInputIdentityKind::ExactWgsl,
        }
    }

    fn wesl_composition(
        package: ShaderPackageIdentity,
        root: &ShaderWeslModuleBinding,
        modules: &[ShaderWeslModuleBinding],
        features: &[ShaderWeslFeatureValue],
    ) -> Self {
        let module_identities = modules
            .iter()
            .map(ShaderWeslModuleIdentity::for_binding)
            .collect::<Vec<_>>()
            .into();
        let features = Arc::<[ShaderWeslFeatureValue]>::from(features.to_vec());

        Self {
            package,
            root_module: root.module(),
            source_unit: root.source().source_unit(),
            source_revision: root.source().revision(),
            profile: ShaderFrontendProfile::WeslComposition20260822,
            kind: ShaderCompilationInputIdentityKind::WeslComposition {
                root_resolution_path: root.resolution_path().clone(),
                modules: module_identities,
                features,
            },
        }
    }

    /// Returns the logical package identity.
    pub const fn package(&self) -> ShaderPackageIdentity {
        self.package
    }

    /// Returns the logical root/main module identity.
    pub const fn root_module(&self) -> ShaderModuleIdentity {
        self.root_module
    }

    /// Returns the logical root/main source-unit identity.
    ///
    /// For exact WGSL this is the only source. For WESL this is the explicitly selected main
    /// module source; use [`Self::wesl_modules`] for complete participating-source evidence.
    pub const fn source_unit(&self) -> ShaderSourceUnitIdentity {
        self.source_unit
    }

    /// Returns the exact root/main source revision.
    ///
    /// For exact WGSL this is the only revision. For WESL use [`Self::wesl_modules`] for all
    /// participating revisions.
    pub const fn source_revision(&self) -> ShaderSourceRevision {
        self.source_revision
    }

    /// Returns the selected frontend profile.
    pub const fn profile(&self) -> ShaderFrontendProfile {
        self.profile
    }

    /// Returns the explicit WESL main-module resolution key when this is a WESL input.
    pub fn wesl_root_resolution_path(&self) -> Option<&ShaderWeslModulePath> {
        match &self.kind {
            ShaderCompilationInputIdentityKind::ExactWgsl => None,
            ShaderCompilationInputIdentityKind::WeslComposition {
                root_resolution_path,
                ..
            } => Some(root_resolution_path),
        }
    }

    /// Returns normalized structural evidence for every admitted WESL module, including the root.
    pub fn wesl_modules(&self) -> Option<&[ShaderWeslModuleIdentity]> {
        match &self.kind {
            ShaderCompilationInputIdentityKind::ExactWgsl => None,
            ShaderCompilationInputIdentityKind::WeslComposition { modules, .. } => Some(modules),
        }
    }

    /// Returns normalized explicit WESL conditional-feature evidence.
    pub fn wesl_features(&self) -> Option<&[ShaderWeslFeatureValue]> {
        match &self.kind {
            ShaderCompilationInputIdentityKind::ExactWgsl => None,
            ShaderCompilationInputIdentityKind::WeslComposition { features, .. } => Some(features),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShaderWeslCompositionInput {
    root: ShaderWeslModuleBinding,
    modules: Arc<[ShaderWeslModuleBinding]>,
    features: Arc<[ShaderWeslFeatureValue]>,
}

/// One exact closed semantic compilation input.
///
/// Exact WGSL contains one package, one root module, and one complete source snapshot. The accepted
/// WESL profile instead carries one explicit main-module binding plus a finite closed set of module
/// snapshots and conditional features. Neither form performs ambient filesystem, registry, network,
/// package-manager, or process-state discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderCompilationInput {
    identity: ShaderCompilationInputIdentity,
    /// Root/main source retained here so the accepted exact-WGSL compiler path remains unchanged.
    source: ShaderSourceSnapshot,
    wesl: Option<ShaderWeslCompositionInput>,
}

impl ShaderCompilationInput {
    /// Forms one closed input for the accepted exact-WGSL profile.
    pub fn exact_wgsl(
        package: ShaderPackageIdentity,
        root_module: ShaderModuleIdentity,
        source: ShaderSourceSnapshot,
    ) -> Self {
        let identity = ShaderCompilationInputIdentity::exact_wgsl(package, root_module, &source);
        Self {
            identity,
            source,
            wesl: None,
        }
    }

    /// Forms one closed semantic input for the accepted WESL composition profile.
    ///
    /// The root/main binding is semantically distinguished. Additional-module and feature ordering
    /// is not semantic; storage is normalized without removing duplicates or contradictory entries.
    pub fn wesl_composition(
        package: ShaderPackageIdentity,
        root: ShaderWeslModuleBinding,
        mut additional_modules: Vec<ShaderWeslModuleBinding>,
        mut features: Vec<ShaderWeslFeatureValue>,
    ) -> Self {
        let source = root.source().clone();
        let mut modules = Vec::with_capacity(additional_modules.len() + 1);
        modules.push(root.clone());
        modules.append(&mut additional_modules);
        modules.sort_by(compare_wesl_module_bindings);
        features.sort_by(compare_wesl_features);

        let identity =
            ShaderCompilationInputIdentity::wesl_composition(package, &root, &modules, &features);
        let wesl = ShaderWeslCompositionInput {
            root,
            modules: modules.into(),
            features: features.into(),
        };

        Self {
            identity,
            source,
            wesl: Some(wesl),
        }
    }

    /// Returns the structurally derived compilation-input identity.
    pub fn identity(&self) -> ShaderCompilationInputIdentity {
        self.identity.clone()
    }

    /// Returns the logical package identity.
    pub const fn package(&self) -> ShaderPackageIdentity {
        self.identity.package
    }

    /// Returns the logical root/main module identity.
    pub const fn root_module(&self) -> ShaderModuleIdentity {
        self.identity.root_module
    }

    /// Returns the explicit root/main source snapshot.
    ///
    /// For exact WGSL this is the sole source snapshot. For WESL it is only the main-module source;
    /// callers needing complete WESL evidence must use [`Self::wesl_modules`].
    pub const fn source(&self) -> &ShaderSourceSnapshot {
        &self.source
    }

    /// Returns the selected semantic frontend profile.
    pub const fn profile(&self) -> ShaderFrontendProfile {
        self.identity.profile
    }

    /// Returns the explicit WESL main-module binding when this is a WESL input.
    pub fn wesl_root(&self) -> Option<&ShaderWeslModuleBinding> {
        self.wesl.as_ref().map(|wesl| &wesl.root)
    }

    /// Returns normalized WESL module bindings, including the root/main binding.
    pub fn wesl_modules(&self) -> Option<&[ShaderWeslModuleBinding]> {
        self.wesl.as_ref().map(|wesl| wesl.modules.as_ref())
    }

    /// Returns normalized explicit WESL conditional feature values.
    pub fn wesl_features(&self) -> Option<&[ShaderWeslFeatureValue]> {
        self.wesl.as_ref().map(|wesl| wesl.features.as_ref())
    }
}

fn compare_wesl_module_bindings(
    left: &ShaderWeslModuleBinding,
    right: &ShaderWeslModuleBinding,
) -> Ordering {
    left.resolution_path()
        .as_str()
        .cmp(right.resolution_path().as_str())
        .then_with(|| {
            left.module()
                .diagnostic_raw()
                .cmp(&right.module().diagnostic_raw())
        })
        .then_with(|| {
            left.source()
                .source_unit()
                .diagnostic_raw()
                .cmp(&right.source().source_unit().diagnostic_raw())
        })
        .then_with(|| {
            left.source()
                .revision()
                .diagnostic_raw()
                .cmp(&right.source().revision().diagnostic_raw())
        })
        // Bytes are only a final storage-order tie breaker for conflicting snapshots with identical
        // semantic identity. They never enter ShaderCompilationInputIdentity.
        .then_with(|| {
            left.source()
                .text()
                .as_bytes()
                .cmp(right.source().text().as_bytes())
        })
}

fn compare_wesl_features(
    left: &ShaderWeslFeatureValue,
    right: &ShaderWeslFeatureValue,
) -> Ordering {
    left.name()
        .cmp(right.name())
        .then_with(|| left.enabled().cmp(&right.enabled()))
}

/// One explicit compilation invocation: a closed semantic input plus one compiler realization.
///
/// Construction does not pre-classify realization coverage. If a selected realization cannot cover
/// an otherwise valid accepted profile request, the compiler boundary reports the normative
/// `Unsupported` outcome rather than a separate construction failure. Profile/realization pairings
/// for which RunenShader has no accepted realization remain ordinary unsupported coverage until a
/// conforming realization is accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderCompilationInvocation {
    input: ShaderCompilationInput,
    realization: ShaderCompilerRealization,
}

impl ShaderCompilationInvocation {
    /// Combines a closed input with an explicitly selected compiler realization.
    pub const fn new(
        input: ShaderCompilationInput,
        realization: ShaderCompilerRealization,
    ) -> Self {
        Self { input, realization }
    }

    /// Returns the complete closed semantic input.
    pub const fn input(&self) -> &ShaderCompilationInput {
        &self.input
    }

    /// Returns the selected compiler realization identity.
    pub const fn realization(&self) -> ShaderCompilerRealization {
        self.realization
    }
}

#[cfg(test)]
mod tests {
    use crate::identity::{
        ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision, ShaderSourceUnitIdentity,
    };
    use crate::profile::ShaderCompilerRealization;

    use super::*;

    fn snapshot(unit: u64, revision: u64, source: &str) -> ShaderSourceSnapshot {
        ShaderSourceSnapshot::new(
            ShaderSourceUnitIdentity::try_from_raw(unit).unwrap(),
            ShaderSourceRevision::try_from_raw(revision).unwrap(),
            source,
        )
    }

    fn sample_input() -> ShaderCompilationInput {
        ShaderCompilationInput::exact_wgsl(
            ShaderPackageIdentity::try_from_raw(2).unwrap(),
            ShaderModuleIdentity::try_from_raw(3).unwrap(),
            snapshot(4, 5, "fn helper() {}"),
        )
    }

    fn module(
        path: &str,
        module: u64,
        unit: u64,
        revision: u64,
        source: &str,
    ) -> ShaderWeslModuleBinding {
        ShaderWeslModuleBinding::new(
            ShaderWeslModulePath::new(path),
            ShaderModuleIdentity::try_from_raw(module).unwrap(),
            snapshot(unit, revision, source),
        )
    }

    fn wesl_input(reverse: bool) -> ShaderCompilationInput {
        let root = module("package::main", 10, 20, 1, "import package::math::add;\n");
        let math = module("package::math", 11, 21, 2, "fn add() {}\n");
        let util = module("package::util", 12, 22, 3, "fn helper() {}\n");
        let mut additional = vec![math, util];
        let mut features = vec![
            ShaderWeslFeatureValue::new("debug", false),
            ShaderWeslFeatureValue::new("textured", true),
        ];
        if reverse {
            additional.reverse();
            features.reverse();
        }
        ShaderCompilationInput::wesl_composition(
            ShaderPackageIdentity::try_from_raw(9).unwrap(),
            root,
            additional,
            features,
        )
    }

    #[test]
    fn exact_wgsl_input_identity_is_structurally_derived() {
        let left = sample_input();
        let right = sample_input();

        assert_eq!(left.identity(), right.identity());
        assert_eq!(left.identity().package().diagnostic_raw(), 2);
        assert_eq!(left.identity().root_module().diagnostic_raw(), 3);
        assert_eq!(left.identity().source_unit().diagnostic_raw(), 4);
        assert_eq!(left.identity().source_revision().diagnostic_raw(), 5);
        assert_eq!(
            left.identity().profile().semantic_name(),
            "wgsl-exact-2026-08-17"
        );
        assert!(left.identity().wesl_modules().is_none());
    }

    #[test]
    fn wesl_input_identity_and_evidence_are_order_independent() {
        let left = wesl_input(false);
        let right = wesl_input(true);

        assert_eq!(left.identity(), right.identity());
        assert_eq!(left, right);
        assert_eq!(
            left.profile(),
            ShaderFrontendProfile::WeslComposition20260822
        );
        assert_eq!(left.root_module().diagnostic_raw(), 10);
        assert_eq!(left.source().source_unit().diagnostic_raw(), 20);
        assert_eq!(
            left.wesl_root().unwrap().resolution_path().as_str(),
            "package::main"
        );

        let modules = left.wesl_modules().unwrap();
        assert_eq!(modules.len(), 3);
        assert_eq!(modules[0].resolution_path().as_str(), "package::main");
        assert_eq!(modules[1].resolution_path().as_str(), "package::math");
        assert_eq!(modules[2].resolution_path().as_str(), "package::util");

        let identity_modules = left.identity().wesl_modules().unwrap().to_vec();
        assert_eq!(identity_modules.len(), 3);
        assert_eq!(identity_modules[0].module().diagnostic_raw(), 10);
        assert_eq!(identity_modules[1].source_unit().diagnostic_raw(), 21);
        assert_eq!(identity_modules[2].source_revision().diagnostic_raw(), 3);

        let features = left.wesl_features().unwrap();
        assert_eq!(features[0].name(), "debug");
        assert!(!features[0].enabled());
        assert_eq!(features[1].name(), "textured");
        assert!(features[1].enabled());
    }

    #[test]
    fn wesl_structural_identity_changes_with_semantic_evidence() {
        let baseline = wesl_input(false);

        let changed_feature = ShaderCompilationInput::wesl_composition(
            ShaderPackageIdentity::try_from_raw(9).unwrap(),
            module("package::main", 10, 20, 1, "root bytes ignored by identity"),
            vec![
                module("package::math", 11, 21, 2, "math"),
                module("package::util", 12, 22, 3, "util"),
            ],
            vec![
                ShaderWeslFeatureValue::new("debug", true),
                ShaderWeslFeatureValue::new("textured", true),
            ],
        );
        assert_ne!(baseline.identity(), changed_feature.identity());

        let changed_path = ShaderCompilationInput::wesl_composition(
            ShaderPackageIdentity::try_from_raw(9).unwrap(),
            module("package::main", 10, 20, 1, "different root bytes"),
            vec![
                module("package::math2", 11, 21, 2, "math"),
                module("package::util", 12, 22, 3, "util"),
            ],
            vec![
                ShaderWeslFeatureValue::new("debug", false),
                ShaderWeslFeatureValue::new("textured", true),
            ],
        );
        assert_ne!(baseline.identity(), changed_path.identity());
    }

    #[test]
    fn exact_source_bytes_do_not_enter_wesl_compilation_input_identity() {
        let first = ShaderCompilationInput::wesl_composition(
            ShaderPackageIdentity::try_from_raw(9).unwrap(),
            module("package::main", 10, 20, 1, "first root bytes"),
            vec![module("package::math", 11, 21, 2, "first math bytes")],
            vec![],
        );
        let second = ShaderCompilationInput::wesl_composition(
            ShaderPackageIdentity::try_from_raw(9).unwrap(),
            module("package::main", 10, 20, 1, "different root bytes"),
            vec![module("package::math", 11, 21, 2, "different math bytes")],
            vec![],
        );

        assert_eq!(first.identity(), second.identity());
        assert_ne!(first, second);
    }

    #[test]
    fn duplicate_and_contradictory_wesl_entries_are_preserved() {
        let input = ShaderCompilationInput::wesl_composition(
            ShaderPackageIdentity::try_from_raw(9).unwrap(),
            module("package::main", 10, 20, 1, "root"),
            vec![
                module("package::dup", 11, 21, 1, "a"),
                module("package::dup", 12, 22, 1, "b"),
            ],
            vec![
                ShaderWeslFeatureValue::new("flag", false),
                ShaderWeslFeatureValue::new("flag", true),
            ],
        );

        let modules = input.wesl_modules().unwrap();
        assert_eq!(modules.len(), 3);
        assert_eq!(
            modules
                .iter()
                .filter(|module| module.resolution_path().as_str() == "package::dup")
                .count(),
            2
        );
        let features = input.wesl_features().unwrap();
        assert_eq!(features.len(), 2);
        assert_eq!(features[0].name(), "flag");
        assert_eq!(features[1].name(), "flag");
        assert_ne!(features[0].enabled(), features[1].enabled());
    }

    #[test]
    fn invocation_keeps_profile_and_realization_distinct() {
        let invocation = ShaderCompilationInvocation::new(
            sample_input(),
            ShaderCompilerRealization::Naga3001ExactWgslGateV1,
        );

        assert_eq!(
            invocation.input().profile().semantic_name(),
            "wgsl-exact-2026-08-17"
        );
        assert_eq!(
            invocation.realization().diagnostic_label(),
            "naga-30.0.1-wgsl-exact-gate-v1"
        );
    }
}
