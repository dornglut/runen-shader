use runen_shader::{
    ShaderCompilationInput, ShaderCompilationInvocation, ShaderCompilationOutcome, ShaderCompiler,
    ShaderCompilerRealization, ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision,
    ShaderSourceSnapshot, ShaderSourceUnitIdentity, ShaderWeslFeatureValue,
    ShaderWeslModuleBinding, ShaderWeslModulePath,
};

fn snapshot(unit: u64, revision: u64) -> ShaderSourceSnapshot {
    snapshot_with_source(unit, revision, "fn helper() {}")
}

fn snapshot_with_source(unit: u64, revision: u64, source: &str) -> ShaderSourceSnapshot {
    ShaderSourceSnapshot::new(
        ShaderSourceUnitIdentity::try_from_raw(unit).unwrap(),
        ShaderSourceRevision::try_from_raw(revision).unwrap(),
        source,
    )
}

fn module(path: &str, module: u64, unit: u64, revision: u64) -> ShaderWeslModuleBinding {
    module_with_source(path, module, unit, revision, "fn helper() {}")
}

fn module_with_source(
    path: &str,
    module: u64,
    unit: u64,
    revision: u64,
    source: &str,
) -> ShaderWeslModuleBinding {
    ShaderWeslModuleBinding::new(
        ShaderWeslModulePath::new(path),
        ShaderModuleIdentity::try_from_raw(module).unwrap(),
        snapshot_with_source(unit, revision, source),
    )
}

fn input(
    package: u64,
    root: ShaderWeslModuleBinding,
    additional: Vec<ShaderWeslModuleBinding>,
    debug: bool,
) -> ShaderCompilationInput {
    ShaderCompilationInput::wesl_composition(
        ShaderPackageIdentity::try_from_raw(package).unwrap(),
        root,
        additional,
        vec![ShaderWeslFeatureValue::new("debug", debug)],
    )
}

#[test]
fn every_accepted_wesl_identity_dimension_changes_structural_input_identity() {
    let baseline = input(
        1,
        module("package::main", 10, 20, 1),
        vec![module("package::math", 11, 21, 1)],
        false,
    );

    let changed_package = input(
        2,
        module("package::main", 10, 20, 1),
        vec![module("package::math", 11, 21, 1)],
        false,
    );
    assert_ne!(baseline.identity(), changed_package.identity());

    // Keep the identical two admitted module bindings and change only which one is the explicit
    // main/root module.
    let changed_root = input(
        1,
        module("package::math", 11, 21, 1),
        vec![module("package::main", 10, 20, 1)],
        false,
    );
    assert_ne!(baseline.identity(), changed_root.identity());

    let changed_module_identity = input(
        1,
        module("package::main", 10, 20, 1),
        vec![module("package::math", 12, 21, 1)],
        false,
    );
    assert_ne!(baseline.identity(), changed_module_identity.identity());

    let changed_source_unit = input(
        1,
        module("package::main", 10, 20, 1),
        vec![module("package::math", 11, 22, 1)],
        false,
    );
    assert_ne!(baseline.identity(), changed_source_unit.identity());

    let changed_source_revision = input(
        1,
        module("package::main", 10, 20, 1),
        vec![module("package::math", 11, 21, 2)],
        false,
    );
    assert_ne!(baseline.identity(), changed_source_revision.identity());

    let changed_module_key = input(
        1,
        module("package::main", 10, 20, 1),
        vec![module("package::math2", 11, 21, 1)],
        false,
    );
    assert_ne!(baseline.identity(), changed_module_key.identity());

    let changed_feature_value = input(
        1,
        module("package::main", 10, 20, 1),
        vec![module("package::math", 11, 21, 1)],
        true,
    );
    assert_ne!(baseline.identity(), changed_feature_value.identity());

    let changed_feature_name = ShaderCompilationInput::wesl_composition(
        ShaderPackageIdentity::try_from_raw(1).unwrap(),
        module("package::main", 10, 20, 1),
        vec![module("package::math", 11, 21, 1)],
        vec![ShaderWeslFeatureValue::new("release", false)],
    );
    assert_ne!(baseline.identity(), changed_feature_name.identity());
}

#[test]
fn uncovered_naga_realization_returns_unsupported_without_parsing_wesl_as_wgsl() {
    let input = ShaderCompilationInput::wesl_composition(
        ShaderPackageIdentity::try_from_raw(100).unwrap(),
        module_with_source(
            "package::main",
            101,
            201,
            1,
            "import package::math::helper;\nfn main() {}",
        ),
        vec![module_with_source(
            "package::math",
            102,
            202,
            1,
            "fn helper() {}",
        )],
        vec![],
    );
    let invocation =
        ShaderCompilationInvocation::new(input, ShaderCompilerRealization::Naga3001ExactWgslGateV1);

    let result = ShaderCompiler::new().compile(&invocation).unwrap();
    let ShaderCompilationOutcome::Unsupported(diagnostics) = result else {
        panic!("expected uncovered WESL realization to be unsupported");
    };
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0]
            .subject()
            .unwrap()
            .source_unit()
            .diagnostic_raw(),
        201
    );
}
