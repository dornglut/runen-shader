//! Compose two explicitly supplied WESL modules into generated canonical WGSL.

use runen_shader::{
    ShaderArtifactSourceMap, ShaderByteRange, ShaderCompilationInput, ShaderCompilationInvocation,
    ShaderCompilationOutcome, ShaderCompiler, ShaderCompilerRealization, ShaderDiagnostic,
    ShaderFrontendProfile, ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision,
    ShaderSourceSnapshot, ShaderSourceUnitIdentity, ShaderWeslModuleBinding,
    ShaderWeslModulePath,
};

const MAIN: &str = "import package::math::add;\n@compute @workgroup_size(1) fn main() { let value = add(1u, 2u); }\n";
const MATH: &str = "public fn add(a: u32, b: u32) -> u32 { return a + b; }\n";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logical identity is explicit; WESL module paths are resolution keys, not identities.
    let package = ShaderPackageIdentity::try_from_raw(1).expect("nonzero package identity");
    let main_module = ShaderModuleIdentity::try_from_raw(2).expect("nonzero main module identity");
    let math_module = ShaderModuleIdentity::try_from_raw(3).expect("nonzero math module identity");
    let main_unit = ShaderSourceUnitIdentity::try_from_raw(4).expect("nonzero main source identity");
    let math_unit = ShaderSourceUnitIdentity::try_from_raw(5).expect("nonzero math source identity");
    let revision = ShaderSourceRevision::try_from_raw(1).expect("nonzero source revision");

    let root = ShaderWeslModuleBinding::new(
        ShaderWeslModulePath::new("package::main"),
        main_module,
        ShaderSourceSnapshot::new(main_unit, revision, MAIN),
    );
    let math = ShaderWeslModuleBinding::new(
        ShaderWeslModulePath::new("package::math"),
        math_module,
        ShaderSourceSnapshot::new(math_unit, revision, MATH),
    );
    let input = ShaderCompilationInput::wesl_composition(package, root, vec![math], vec![]);
    let invocation = ShaderCompilationInvocation::new(
        input,
        ShaderCompilerRealization::Wesl050Composition20260822V1,
    );
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
        Err(invariant) => return Err(format!("RunenShader invariant violation: {invariant}").into()),
    };

    let wgsl = artifact.canonical_wgsl();
    if !wgsl.contains("fn main") || wgsl.contains("import ") {
        return Err("WESL composition did not produce the expected import-free WGSL entry point".into());
    }
    let provenance = artifact.provenance();
    let modules = provenance
        .wesl_modules()
        .ok_or("accepted WESL artifact has no module provenance")?;
    if provenance.package() != package
        || provenance.root_module() != main_module
        || provenance.profile() != ShaderFrontendProfile::WeslComposition20260822
        || modules.len() != 2
        || !modules.iter().any(|module| {
            module.module() == main_module
                && module.source_unit() == main_unit
                && module.source_revision() == revision
        })
        || !modules.iter().any(|module| {
            module.module() == math_module
                && module.source_unit() == math_unit
                && module.source_revision() == revision
        })
    {
        return Err("WESL artifact provenance does not cover both explicit modules".into());
    }

    // V1 does not claim precise transformed-byte attribution, even though provenance is complete.
    let map = artifact.source_map();
    if !matches!(map, ShaderArtifactSourceMap::Unattributable { byte_len } if byte_len == wgsl.len()) {
        return Err("WESL artifact did not report total unattributable mapping".into());
    }
    let full_range = ShaderByteRange::new(0, wgsl.len()).expect("non-reversed artifact range");
    if map.map_artifact_range(full_range)?.exact_source().is_some() {
        return Err("WESL V1 incorrectly claimed an exact transformed-byte mapping".into());
    }

    println!("Generated canonical WGSL:\n{wgsl}");
    println!("Participating logical modules: {}", modules.len());
    println!("Source mapping: Unattributable ({} bytes)", map.byte_len());
    Ok(())
}

fn unexpected_outcome(kind: &str, diagnostics: &[ShaderDiagnostic]) -> Box<dyn std::error::Error> {
    for diagnostic in diagnostics {
        eprintln!("{kind}: {}", diagnostic.summary());
    }
    format!("unexpected compilation outcome: {kind}").into()
}
