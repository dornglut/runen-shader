use core::fmt;
use core::num::NonZeroU64;

macro_rules! define_scalar_identity {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(NonZeroU64);

        impl $name {
            /// Constructs an identity from a nonzero diagnostic representation.
            ///
            /// The raw value is not a persistence, wire, ABI, ordering, freshness,
            /// filesystem, or content-hash contract.
            pub const fn try_from_raw(raw: u64) -> Option<Self> {
                match NonZeroU64::new(raw) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            /// Returns the current opaque representation for diagnostics only.
            pub const fn diagnostic_raw(self) -> u64 {
                self.0.get()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

define_scalar_identity!(
    ShaderPackageIdentity,
    "Opaque logical identity for one RunenShader package namespace."
);
define_scalar_identity!(
    ShaderModuleIdentity,
    "Opaque logical identity for one shader module within a package namespace."
);
define_scalar_identity!(
    ShaderSourceUnitIdentity,
    "Opaque logical identity for one shader source unit."
);
define_scalar_identity!(
    ShaderSourceRevision,
    "Opaque revision identity meaningful only together with one source-unit identity."
);
define_scalar_identity!(
    ShaderCompilationInputIdentity,
    "Opaque identity for one exact closed semantic compilation input."
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_identities_reject_zero_and_preserve_type_separation() {
        assert!(ShaderPackageIdentity::try_from_raw(0).is_none());
        assert!(ShaderModuleIdentity::try_from_raw(0).is_none());
        assert!(ShaderSourceUnitIdentity::try_from_raw(0).is_none());
        assert!(ShaderSourceRevision::try_from_raw(0).is_none());
        assert!(ShaderCompilationInputIdentity::try_from_raw(0).is_none());

        let package = ShaderPackageIdentity::try_from_raw(7).unwrap();
        let module = ShaderModuleIdentity::try_from_raw(7).unwrap();
        assert_eq!(package.diagnostic_raw(), module.diagnostic_raw());
    }

    #[test]
    fn source_revision_raw_value_does_not_create_ordering_api() {
        let revision = ShaderSourceRevision::try_from_raw(42).unwrap();
        assert_eq!(revision.diagnostic_raw(), 42);
    }
}
