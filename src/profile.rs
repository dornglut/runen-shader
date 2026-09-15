/// RunenShader-owned identity for an accepted shader frontend profile.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderFrontendProfile {
    /// Exact WGSL using the W3C Candidate Recommendation Draft dated 2026-08-17.
    WgslExact20260817,
}

impl ShaderFrontendProfile {
    /// Returns the stable semantic profile name defined by the normative specification.
    pub const fn semantic_name(self) -> &'static str {
        match self {
            Self::WgslExact20260817 => "wgsl-exact-2026-08-17",
        }
    }
}

/// RunenShader-owned identity for one accepted concrete compiler realization contract.
///
/// This value identifies a realization contract. It does not expose or wrap a compiler-private
/// object, and its presence does not imply that the realization is implemented by this crate yet.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderCompilerRealization {
    /// Naga 30.0.1 with the exact-WGSL realization configuration accepted by the spec.
    Naga3001ExactWgsl,
}

impl ShaderCompilerRealization {
    /// Returns the RunenShader semantic realization name.
    pub const fn semantic_name(self) -> &'static str {
        match self {
            Self::Naga3001ExactWgsl => "naga-30.0.1-wgsl-exact",
        }
    }

    /// Reports whether this realization contract is accepted for the given frontend profile.
    pub const fn supports(self, profile: ShaderFrontendProfile) -> bool {
        matches!(
            (self, profile),
            (
                Self::Naga3001ExactWgsl,
                ShaderFrontendProfile::WgslExact20260817
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_and_realization_are_distinct_semantic_values() {
        let profile = ShaderFrontendProfile::WgslExact20260817;
        let realization = ShaderCompilerRealization::Naga3001ExactWgsl;

        assert_eq!(profile.semantic_name(), "wgsl-exact-2026-08-17");
        assert_eq!(realization.semantic_name(), "naga-30.0.1-wgsl-exact");
        assert!(realization.supports(profile));
    }
}
