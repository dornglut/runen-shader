/// RunenShader-owned identity for an accepted shader frontend profile.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderFrontendProfile {
    /// Exact WGSL using the W3C Candidate Recommendation Draft dated 2026-08-17.
    WgslExact20260817,
    /// Closed WESL composition pinned to the accepted 2026-08-22 WESL semantic revision.
    WeslComposition20260822,
}

impl ShaderFrontendProfile {
    /// Returns the stable semantic profile name defined by the normative specification.
    pub const fn semantic_name(self) -> &'static str {
        match self {
            Self::WgslExact20260817 => "wgsl-exact-2026-08-17",
            Self::WeslComposition20260822 => "wesl-composition-2026-08-22",
        }
    }
}

/// RunenShader-owned identity for one accepted concrete compiler realization contract.
///
/// This value identifies realization evidence. It does not expose or wrap a compiler-private
/// object; the selected realization is implemented by this crate behind the public compiler
/// authority.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderCompilerRealization {
    /// Naga 30.0.1 exact-WGSL realization with profile-gate revision V1.
    ///
    /// The identity fixes the accepted realization configuration: Naga 30.0.1,
    /// `default-features = false`, `wgsl-in` only, all validation flags/capabilities, all subgroup
    /// stages/operations for validator purposes, and RunenShader exact-WGSL profile gate V1.
    Naga3001ExactWgslGateV1,
}

impl ShaderCompilerRealization {
    /// Returns a diagnostic label for this exact realization identity.
    ///
    /// The label is not a persistence, wire, or ABI encoding of the identity.
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::Naga3001ExactWgslGateV1 => "naga-30.0.1-wgsl-exact-gate-v1",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_frontend_profiles_have_stable_distinct_semantic_names() {
        assert_eq!(
            ShaderFrontendProfile::WgslExact20260817.semantic_name(),
            "wgsl-exact-2026-08-17"
        );
        assert_eq!(
            ShaderFrontendProfile::WeslComposition20260822.semantic_name(),
            "wesl-composition-2026-08-22"
        );
    }

    #[test]
    fn profile_and_realization_are_distinct_semantic_values() {
        let profile = ShaderFrontendProfile::WgslExact20260817;
        let realization = ShaderCompilerRealization::Naga3001ExactWgslGateV1;

        assert_eq!(profile.semantic_name(), "wgsl-exact-2026-08-17");
        assert_eq!(
            realization.diagnostic_label(),
            "naga-30.0.1-wgsl-exact-gate-v1"
        );
    }
}
