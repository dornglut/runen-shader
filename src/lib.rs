//! RunenShader defines reusable shader-source and shader-compilation semantics.
//!
//! The normative contract is repository-owned under `spec/`. The public Rust surface implements
//! the exact-WGSL semantic data boundary and its pinned Naga 30.0.1 compiler realization. Naga
//! remains a private implementation detail: accepted artifacts preserve the original source
//! bytes and contain no Naga IR or reflection data.

mod artifact;
mod compiler;
mod identity;
mod input;
mod outcome;
mod profile;
mod source;
mod wgsl_gate;

pub use artifact::{
    ExactWgslSourceMap, ShaderArtifact, ShaderArtifactIdentity, ShaderArtifactProvenance,
    ShaderByteRange, ShaderMappedSourceRange, ShaderRangeError,
};
pub use compiler::ShaderCompiler;
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
