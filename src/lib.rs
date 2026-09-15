//! RunenShader defines reusable shader-source and shader-compilation semantics.
//!
//! The normative contract is repository-owned under `spec/`. The public Rust surface implements
//! the exact-WGSL semantic data boundary and its pinned Naga 30.0.1 compiler realization, plus the
//! accepted closed WESL composition profile and its first pinned wesl-rs 0.5.0 realization.
//! Compiler-private IR, reflection, resolver handles, and upstream source-map identities remain
//! non-authoritative implementation details.

mod artifact;
mod compiler;
mod identity;
mod input;
mod outcome;
mod profile;
mod source;
mod wesl_realization;
mod wgsl_gate;
mod wgsl_validation;

pub use artifact::{
    ExactWgslSourceMap, ShaderArtifact, ShaderArtifactIdentity, ShaderArtifactProvenance,
    ShaderArtifactRangeMapping, ShaderArtifactSourceMap, ShaderByteRange, ShaderMappedSourceRange,
    ShaderRangeError,
};
pub use compiler::ShaderCompiler;
pub use identity::{
    ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision, ShaderSourceUnitIdentity,
};
pub use input::{
    ShaderCompilationInput, ShaderCompilationInputIdentity, ShaderCompilationInvocation,
    ShaderWeslFeatureValue, ShaderWeslModuleBinding, ShaderWeslModuleIdentity,
    ShaderWeslModulePath,
};
pub use outcome::{
    ShaderCompilationOutcome, ShaderCompilationResult, ShaderDiagnostic, ShaderInvariantError,
    ShaderSourceSubject,
};
pub use profile::{ShaderCompilerRealization, ShaderFrontendProfile};
pub use source::ShaderSourceSnapshot;
