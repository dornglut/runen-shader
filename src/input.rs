use core::fmt;

use crate::identity::{
    ShaderCompilationInputIdentity, ShaderModuleIdentity, ShaderPackageIdentity,
};
use crate::profile::{ShaderCompilerRealization, ShaderFrontendProfile};
use crate::source::ShaderSourceSnapshot;

/// One exact closed semantic compilation input.
///
/// The first accepted exact-WGSL profile contains exactly one package, one root module, and one
/// complete source snapshot. It has no implicit filesystem, registry, network, include, import, or
/// generated-companion input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderCompilationInput {
    identity: ShaderCompilationInputIdentity,
    package: ShaderPackageIdentity,
    root_module: ShaderModuleIdentity,
    source: ShaderSourceSnapshot,
    profile: ShaderFrontendProfile,
}

impl ShaderCompilationInput {
    /// Forms one closed input for the accepted exact-WGSL profile.
    pub fn exact_wgsl(
        identity: ShaderCompilationInputIdentity,
        package: ShaderPackageIdentity,
        root_module: ShaderModuleIdentity,
        source: ShaderSourceSnapshot,
    ) -> Self {
        Self {
            identity,
            package,
            root_module,
            source,
            profile: ShaderFrontendProfile::WgslExact20260817,
        }
    }

    /// Returns the exact compilation-input identity supplied by the caller.
    pub const fn identity(&self) -> ShaderCompilationInputIdentity {
        self.identity
    }

    /// Returns the logical package identity.
    pub const fn package(&self) -> ShaderPackageIdentity {
        self.package
    }

    /// Returns the logical root-module identity.
    pub const fn root_module(&self) -> ShaderModuleIdentity {
        self.root_module
    }

    /// Returns the single exact source snapshot participating in this input.
    pub const fn source(&self) -> &ShaderSourceSnapshot {
        &self.source
    }

    /// Returns the selected semantic frontend profile.
    pub const fn profile(&self) -> ShaderFrontendProfile {
        self.profile
    }
}

/// Error returned when an explicit compiler realization cannot realize a compilation input's
/// frontend profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShaderInvocationError {
    profile: ShaderFrontendProfile,
    realization: ShaderCompilerRealization,
}

impl ShaderInvocationError {
    /// Returns the frontend profile that could not be realized.
    pub const fn profile(&self) -> ShaderFrontendProfile {
        self.profile
    }

    /// Returns the incompatible realization identity.
    pub const fn realization(&self) -> ShaderCompilerRealization {
        self.realization
    }
}

impl fmt::Display for ShaderInvocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "compiler realization {} does not realize frontend profile {}",
            self.realization.semantic_name(),
            self.profile.semantic_name()
        )
    }
}

impl std::error::Error for ShaderInvocationError {}

/// One explicit compilation invocation: a closed semantic input plus one compiler realization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderCompilationInvocation {
    input: ShaderCompilationInput,
    realization: ShaderCompilerRealization,
}

impl ShaderCompilationInvocation {
    /// Combines a closed input with an explicitly selected compatible realization.
    pub fn new(
        input: ShaderCompilationInput,
        realization: ShaderCompilerRealization,
    ) -> Result<Self, ShaderInvocationError> {
        if !realization.supports(input.profile()) {
            return Err(ShaderInvocationError {
                profile: input.profile(),
                realization,
            });
        }

        Ok(Self { input, realization })
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
        ShaderCompilationInputIdentity, ShaderModuleIdentity, ShaderPackageIdentity,
        ShaderSourceRevision, ShaderSourceUnitIdentity,
    };
    use crate::profile::ShaderCompilerRealization;

    use super::*;

    fn sample_input() -> ShaderCompilationInput {
        ShaderCompilationInput::exact_wgsl(
            ShaderCompilationInputIdentity::try_from_raw(1).unwrap(),
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
    fn exact_wgsl_input_is_closed_and_explicit() {
        let input = sample_input();

        assert_eq!(input.identity().diagnostic_raw(), 1);
        assert_eq!(input.package().diagnostic_raw(), 2);
        assert_eq!(input.root_module().diagnostic_raw(), 3);
        assert_eq!(input.source().source_unit().diagnostic_raw(), 4);
        assert_eq!(input.source().revision().diagnostic_raw(), 5);
        assert_eq!(input.profile().semantic_name(), "wgsl-exact-2026-08-17");
    }

    #[test]
    fn invocation_keeps_profile_and_realization_distinct() {
        let invocation = ShaderCompilationInvocation::new(
            sample_input(),
            ShaderCompilerRealization::Naga3001ExactWgsl,
        )
        .unwrap();

        assert_eq!(
            invocation.input().profile().semantic_name(),
            "wgsl-exact-2026-08-17"
        );
        assert_eq!(
            invocation.realization().semantic_name(),
            "naga-30.0.1-wgsl-exact"
        );
    }
}
