//! RunenShader defines reusable shader-source and shader-compilation semantics.
//!
//! The normative contract is repository-owned under `spec/`. The current public Rust surface
//! implements the dependency-free semantic data kernel: explicit logical identities, immutable
//! exact source snapshots, closed exact-WGSL inputs, profile/realization identity, deterministic
//! artifact evidence, exact source mapping, and typed outcome data.
//!
//! No shader parser/compiler realization is implemented yet. In particular, the presence of the
//! accepted Naga realization identity does not mean Naga is linked or that WGSL compilation support
//! has shipped.

mod artifact;
mod identity;
mod input;
mod outcome;
mod profile;
mod source;

pub use artifact::{
    ExactWgslSourceMap, ShaderArtifact, ShaderArtifactIdentity, ShaderArtifactProvenance,
    ShaderByteRange, ShaderMappedSourceRange, ShaderRangeError,
};
pub use identity::{
    ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision, ShaderSourceUnitIdentity,
};
pub use input::{
    ShaderCompilationInput, ShaderCompilationInputIdentity, ShaderCompilationInvocation,
};
pub use outcome::{
    ShaderCompilationOutcome, ShaderCompilationResult, ShaderDiagnostic, ShaderInvariantError,
    ShaderSourceSubject,
};
pub use profile::{ShaderCompilerRealization, ShaderFrontendProfile};
pub use source::ShaderSourceSnapshot;
