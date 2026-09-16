//! Run a closed, exact-WGSL compilation through RunenShader's public API.

use runen_shader::{
    ShaderArtifactSourceMap, ShaderByteRange, ShaderCompilationInput, ShaderCompilationInvocation,
    ShaderCompilationOutcome, ShaderCompiler, ShaderCompilerRealization, ShaderDiagnostic,
    ShaderFrontendProfile, ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision,
    ShaderSourceSnapshot, ShaderSourceUnitIdentity,
};

const WGSL: &str = "@compute @workgroup_size(1)\nfn main() {}\n";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // These nonzero values are example-local logical identities, not paths, hashes, or timestamps.
    let package = ShaderPackageIdentity::try_from_raw(1).expect("nonzero package identity");
    let module = ShaderModuleIdentity::try_from_raw(2).expect("nonzero module identity");
    let source_unit = ShaderSourceUnitIdentity::try_from_raw(3).expect("nonzero source identity");
    let revision = ShaderSourceRevision::try_from_raw(1).expect("nonzero source revision");

    let source = ShaderSourceSnapshot::new(source_unit, revision, WGSL);
    let input = ShaderCompilationInput::exact_wgsl(package, module, source);
    let invocation =
        ShaderCompilationInvocation::new(input, ShaderCompilerRealization::Naga3001ExactWgslGateV1);
    let mut compiler = ShaderCompiler::new();
    let artifact = match compiler.compile(&invocation) {
        Ok(ShaderCompilationOutcome::Accepted(artifact)) => artifact,
        Ok(ShaderCompilationOutcome::Rejected(diagnostics)) => {
            return Err(unexpected_outcome("Rejected", &diagnostics));
        }
        Ok(ShaderCompilationOutcome::Unsupported(diagnostics)) => {
            return Err(unexpected_outcome("Unsupported", &diagnostics));
        }
        Ok(ShaderCompilationOutcome::Failed(diagnostics)) => {
            return Err(unexpected_outcome("Failed", &diagnostics));
        }
        Err(invariant) => {
            return Err(format!("RunenShader invariant violation: {invariant}").into());
        }
    };

    if artifact.canonical_wgsl() != WGSL {
        return Err("exact-WGSL compilation did not preserve source bytes".into());
    }
    let provenance = artifact.provenance();
    if provenance.package() != package
        || provenance.root_module() != module
        || provenance.source_unit() != source_unit
        || provenance.source_revision() != revision
        || provenance.profile() != ShaderFrontendProfile::WgslExact20260817
    {
        return Err("exact-WGSL artifact provenance disagrees with the explicit input".into());
    }

    let map = artifact.source_map();
    if !matches!(map, ShaderArtifactSourceMap::ExactIdentity(_)) {
        return Err("exact-WGSL artifact did not provide an identity source map".into());
    }
    let full_range = ShaderByteRange::new(0, WGSL.len()).expect("non-reversed source range");
    let mapped = map
        .map_artifact_range(full_range)?
        .exact_source()
        .ok_or("exact-WGSL range was not mapped to its logical source")?;
    if mapped.source_unit() != source_unit
        || mapped.revision() != revision
        || mapped.range() != full_range
    {
        return Err("exact-WGSL source mapping disagrees with the explicit source".into());
    }

    println!("Exact canonical WGSL:\n{}", artifact.canonical_wgsl());
    println!(
        "Exact mapping: {} bytes to source {source_unit} revision {revision}",
        WGSL.len()
    );
    Ok(())
}

fn unexpected_outcome(kind: &str, diagnostics: &[ShaderDiagnostic]) -> Box<dyn std::error::Error> {
    for diagnostic in diagnostics {
        eprintln!("{kind}: {}", diagnostic.summary());
    }
    format!("unexpected compilation outcome: {kind}").into()
}
