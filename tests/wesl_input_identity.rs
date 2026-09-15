use runen_shader::{
    ShaderCompilationInput, ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision,
    ShaderSourceSnapshot, ShaderSourceUnitIdentity, ShaderWeslFeatureValue,
    ShaderWeslModuleBinding, ShaderWeslModulePath,
};

fn snapshot(unit: u64, revision: u64) -> ShaderSourceSnapshot {
    ShaderSourceSnapshot::new(
        ShaderSourceUnitIdentity::try_from_raw(unit).unwrap(),
        ShaderSourceRevision::try_from_raw(revision).unwrap(),
        "fn helper() {}",
    )
}

fn module(path: &str, module: u64, unit: u64, revision: u64) -> ShaderWeslModuleBinding {
    ShaderWeslModuleBinding::new(
        ShaderWeslModulePath::new(path),
        ShaderModuleIdentity::try_from_raw(module).unwrap(),
        snapshot(unit, revision),
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
}
