use crate::identity::{
    ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision, ShaderSourceUnitIdentity,
};
use crate::profile::{ShaderCompilerRealization, ShaderFrontendProfile};
use crate::source::ShaderSourceSnapshot;

/// Opaque structural identity for one exact closed semantic compilation input.
///
/// This identity is derived from the logical package, root module, source-unit/revision binding,
/// and frontend profile. It has no independent caller-supplied scalar representation. Exact source
/// bytes remain bound by the normative `(source unit, revision)` reuse invariant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderCompilationInputIdentity {
    package: ShaderPackageIdentity,
    root_module: ShaderModuleIdentity,
    source_unit: ShaderSourceUnitIdentity,
    source_revision: ShaderSourceRevision,
    profile: ShaderFrontendProfile,
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
        }
    }

    /// Returns the logical package identity.
    pub const fn package(self) -> ShaderPackageIdentity {
        self.package
    }

    /// Returns the logical root-module identity.
    pub const fn root_module(self) -> ShaderModuleIdentity {
        self.root_module
    }

    /// Returns the logical source-unit identity.
    pub const fn source_unit(self) -> ShaderSourceUnitIdentity {
        self.source_unit
    }

    /// Returns the exact source revision.
    pub const fn source_revision(self) -> ShaderSourceRevision {
        self.source_revision
    }

    /// Returns the selected frontend profile.
    pub const fn profile(self) -> ShaderFrontendProfile {
        self.profile
    }
}

/// One exact closed semantic compilation input.
///
/// The first accepted exact-WGSL profile contains exactly one package, one root module, and one
/// complete source snapshot. It has no implicit filesystem, registry, network, include, import, or
/// generated-companion input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderCompilationInput {
    identity: ShaderCompilationInputIdentity,
    source: ShaderSourceSnapshot,
}

impl ShaderCompilationInput {
    /// Forms one closed input for the accepted exact-WGSL profile.
    pub fn exact_wgsl(
        package: ShaderPackageIdentity,
        root_module: ShaderModuleIdentity,
        source: ShaderSourceSnapshot,
    ) -> Self {
        let identity = ShaderCompilationInputIdentity::exact_wgsl(package, root_module, &source);
        Self { identity, source }
    }

    /// Returns the structurally derived compilation-input identity.
    pub const fn identity(&self) -> ShaderCompilationInputIdentity {
        self.identity
    }

    /// Returns the logical package identity.
    pub const fn package(&self) -> ShaderPackageIdentity {
        self.identity.package()
    }

    /// Returns the logical root-module identity.
    pub const fn root_module(&self) -> ShaderModuleIdentity {
        self.identity.root_module()
    }

    /// Returns the single exact source snapshot participating in this input.
    pub const fn source(&self) -> &ShaderSourceSnapshot {
        &self.source
    }

    /// Returns the selected semantic frontend profile.
    pub const fn profile(&self) -> ShaderFrontendProfile {
        self.identity.profile()
    }
}

/// One explicit compilation invocation: a closed semantic input plus one compiler realization.
///
/// Construction does not pre-classify realization coverage. If a selected realization cannot cover
/// an otherwise valid accepted profile request, the compiler boundary reports the normative
/// `Unsupported` outcome rather than a separate construction failure.
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

    fn sample_input() -> ShaderCompilationInput {
        ShaderCompilationInput::exact_wgsl(
            ShaderPackageIdentity::try_from_raw(2).unwrap(),
            ShaderModuleIdentity::try_from_raw(3).unwrap(),
            ShaderSourceSnapshot::new(
                ShaderSourceUnitIdentity::try_from_raw(4).unwrap(),
                ShaderSourceRevision::try_from_raw(5).unwrap(),
                "fn helper() {}",
            ),
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
        assert_eq!(left.identity().profile().semantic_name(), "wgsl-exact-2026-08-17");
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
