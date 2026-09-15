use runen_shader::{
    ShaderArtifactSourceMap, ShaderByteRange, ShaderCompilationInput, ShaderCompilationInvocation,
    ShaderCompilationOutcome, ShaderCompiler, ShaderCompilerRealization, ShaderModuleIdentity,
    ShaderPackageIdentity, ShaderSourceRevision, ShaderSourceSnapshot, ShaderSourceUnitIdentity,
    ShaderWeslFeatureValue, ShaderWeslModuleBinding, ShaderWeslModulePath,
};

fn module(
    path: &str,
    module: u64,
    source_unit: u64,
    revision: u64,
    source: &str,
) -> ShaderWeslModuleBinding {
    ShaderWeslModuleBinding::new(
        ShaderWeslModulePath::new(path),
        ShaderModuleIdentity::try_from_raw(module).unwrap(),
        ShaderSourceSnapshot::new(
            ShaderSourceUnitIdentity::try_from_raw(source_unit).unwrap(),
            ShaderSourceRevision::try_from_raw(revision).unwrap(),
            source,
        ),
    )
}

fn invocation(
    root: ShaderWeslModuleBinding,
    additional: Vec<ShaderWeslModuleBinding>,
    features: Vec<ShaderWeslFeatureValue>,
) -> ShaderCompilationInvocation {
    ShaderCompilationInvocation::new(
        ShaderCompilationInput::wesl_composition(
            ShaderPackageIdentity::try_from_raw(100).unwrap(),
            root,
            additional,
            features,
        ),
        ShaderCompilerRealization::Wesl050Composition20260822V1,
    )
}

fn accepted(
    root: ShaderWeslModuleBinding,
    additional: Vec<ShaderWeslModuleBinding>,
    features: Vec<ShaderWeslFeatureValue>,
) -> runen_shader::ShaderArtifact {
    let mut compiler = ShaderCompiler::new();
    let result = compiler
        .compile(&invocation(root, additional, features))
        .unwrap();
    let ShaderCompilationOutcome::Accepted(artifact) = result else {
        panic!("expected WESL acceptance, got {result:?}");
    };
    artifact
}

fn compile(
    root: ShaderWeslModuleBinding,
    additional: Vec<ShaderWeslModuleBinding>,
    features: Vec<ShaderWeslFeatureValue>,
) -> ShaderCompilationOutcome {
    ShaderCompiler::new()
        .compile(&invocation(root, additional, features))
        .unwrap()
}

#[test]
fn wesl_only_visibility_syntax_executes_through_the_wesl_realization() {
    let artifact = accepted(
        module(
            "package",
            1,
            11,
            1,
            "public fn helper() -> u32 { return 7u; }\n@compute @workgroup_size(1) fn main() { let value = helper(); }\n",
        ),
        vec![],
        vec![],
    );

    assert!(!artifact.canonical_wgsl().contains("public fn"));
    assert!(artifact.canonical_wgsl().contains("fn helper"));
    assert!(artifact.canonical_wgsl().contains("@compute"));
    assert!(artifact.canonical_wgsl().contains("fn main"));
    assert!(matches!(
        artifact.source_map(),
        ShaderArtifactSourceMap::Unattributable { .. }
    ));
    let full = ShaderByteRange::new(0, artifact.canonical_wgsl().len()).unwrap();
    assert!(
        artifact
            .source_map()
            .map_artifact_range(full)
            .unwrap()
            .exact_source()
            .is_none()
    );
}

#[test]
fn multi_module_package_root_and_relative_imports_compile_from_closed_memory() {
    let root = module(
        "package::main",
        1,
        11,
        1,
        "import package::math::add;\nimport super::util::bias;\n@compute @workgroup_size(1) fn main() { let value = add(1u, bias()); }\n",
    );
    let artifact = accepted(
        root,
        vec![
            module(
                "package::math",
                2,
                12,
                1,
                "public fn add(a: u32, b: u32) -> u32 { return a + b; }\n",
            ),
            module(
                "package::util",
                3,
                13,
                1,
                "public fn bias() -> u32 { return 2u; }\n",
            ),
        ],
        vec![],
    );

    assert!(!artifact.canonical_wgsl().contains("import "));
    assert!(artifact.canonical_wgsl().contains("fn main"));
}

#[test]
fn collections_aliases_visibility_and_public_reexports_are_realized() {
    let artifact = accepted(
        module(
            "package",
            1,
            11,
            1,
            "import package::prelude::forwarded as forwarded_alias;\nimport package::util::{VALUE, helper};\n@compute @workgroup_size(1) fn main() { let value = forwarded_alias() + helper() + VALUE; }\n",
        ),
        vec![
            module(
                "package::util",
                2,
                12,
                1,
                "public const VALUE: u32 = 3u;\npublic fn helper() -> u32 { return VALUE; }\n",
            ),
            module(
                "package::prelude",
                3,
                13,
                1,
                "public import super::util::helper as forwarded;\n",
            ),
        ],
        vec![],
    );

    assert!(!artifact.canonical_wgsl().contains("import "));
    assert!(!artifact.canonical_wgsl().contains("public "));
}

#[test]
fn package_visibility_is_accepted_and_private_cross_module_access_is_rejected() {
    let package_visible = accepted(
        module(
            "package",
            1,
            11,
            1,
            "import package::util::helper;\n@compute @workgroup_size(1) fn main() { helper(); }\n",
        ),
        vec![module("package::util", 2, 12, 1, "fn helper() {}\n")],
        vec![],
    );
    assert!(package_visible.canonical_wgsl().contains("fn main"));

    let private = compile(
        module(
            "package",
            1,
            21,
            1,
            "import package::util::helper;\n@compute @workgroup_size(1) fn main() { helper(); }\n",
        ),
        vec![module(
            "package::util",
            2,
            22,
            1,
            "private fn helper() {}\n",
        )],
        vec![],
    );
    assert!(matches!(private, ShaderCompilationOutcome::Rejected(_)));
}

#[test]
fn a_profile_valid_module_import_cycle_is_accepted() {
    let artifact = accepted(
        module(
            "package",
            1,
            11,
            1,
            "import package::cycle_a::ping;\n@compute @workgroup_size(1) fn main() { let value = ping(); }\n",
        ),
        vec![
            module(
                "package::cycle_a",
                2,
                12,
                1,
                "import package::cycle_b::VALUE;\npublic const HELPER: u32 = 1u;\npublic fn ping() -> u32 { return VALUE; }\n",
            ),
            module(
                "package::cycle_b",
                3,
                13,
                1,
                "import package::cycle_a::HELPER;\npublic const VALUE: u32 = HELPER;\n",
            ),
        ],
        vec![],
    );

    assert!(artifact.canonical_wgsl().contains("fn main"));
}

#[test]
fn conditional_translation_true_and_false_are_explicit_and_deterministic() {
    let source = "@if(feature_x)\nconst VALUE: u32 = 1u;\n@else\nconst VALUE: u32 = 2u;\n@compute @workgroup_size(1) fn main() { let value = VALUE; }\n";
    let root = || module("package", 1, 11, 1, source);

    let true_artifact = accepted(
        root(),
        vec![],
        vec![ShaderWeslFeatureValue::new("feature_x", true)],
    );
    let false_artifact = accepted(
        root(),
        vec![],
        vec![ShaderWeslFeatureValue::new("feature_x", false)],
    );

    assert_ne!(
        true_artifact.canonical_wgsl().as_bytes(),
        false_artifact.canonical_wgsl().as_bytes()
    );
    assert!(!true_artifact.canonical_wgsl().contains("@if"));
    assert!(!false_artifact.canonical_wgsl().contains("@if"));
}

#[test]
fn unused_explicit_features_affect_input_identity_but_not_output_bytes() {
    let source = "@compute @workgroup_size(1) fn main() {}\n";
    let without = accepted(module("package", 1, 11, 1, source), vec![], vec![]);
    let with_unused = accepted(
        module("package", 1, 11, 1, source),
        vec![],
        vec![ShaderWeslFeatureValue::new("unused_feature", true)],
    );

    assert_eq!(
        without.canonical_wgsl().as_bytes(),
        with_unused.canonical_wgsl().as_bytes()
    );
    assert_ne!(without.identity(), with_unused.identity());
}

#[test]
fn repeated_output_and_collection_order_are_deterministic() {
    let root = module(
        "package",
        1,
        11,
        1,
        "import package::math::add;\n@if(feature_x) const VALUE: u32 = 1u;\n@else const VALUE: u32 = 2u;\n@compute @workgroup_size(1) fn main() { let value = add(VALUE); }\n",
    );
    let math = module(
        "package::math",
        2,
        12,
        1,
        "public fn add(value: u32) -> u32 { return value + 1u; }\n",
    );
    let unused = module(
        "package::unused",
        3,
        13,
        1,
        "public fn unused() -> u32 { return 9u; }\n",
    );
    let first = invocation(
        root.clone(),
        vec![math.clone(), unused.clone()],
        vec![
            ShaderWeslFeatureValue::new("feature_x", true),
            ShaderWeslFeatureValue::new("unused_flag", false),
        ],
    );
    let second = invocation(
        root,
        vec![unused, math],
        vec![
            ShaderWeslFeatureValue::new("unused_flag", false),
            ShaderWeslFeatureValue::new("feature_x", true),
        ],
    );
    let mut compiler = ShaderCompiler::new();
    let first_result = compiler.compile(&first).unwrap();
    let repeat_result = compiler.compile(&first).unwrap();
    let reordered_result = compiler.compile(&second).unwrap();

    assert_eq!(first_result, repeat_result);
    assert_eq!(first_result, reordered_result);
}

#[test]
fn complete_provenance_retains_exact_closed_input_evidence_including_unused_sources() {
    let artifact = accepted(
        module("package", 1, 11, 3, "fn helper() {}\n"),
        vec![module(
            "package::unused",
            2,
            12,
            7,
            "public fn unused() {}\n",
        )],
        vec![ShaderWeslFeatureValue::new("unused_feature", false)],
    );

    let provenance = artifact.provenance();
    let modules = provenance.wesl_modules().unwrap();
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].resolution_path().as_str(), "package");
    assert_eq!(modules[0].source_unit().diagnostic_raw(), 11);
    assert_eq!(modules[0].source_revision().diagnostic_raw(), 3);
    assert_eq!(modules[1].resolution_path().as_str(), "package::unused");
    assert_eq!(modules[1].source_unit().diagnostic_raw(), 12);
    assert_eq!(modules[1].source_revision().diagnostic_raw(), 7);
    assert_eq!(provenance.wesl_features().unwrap().len(), 1);
    assert_eq!(
        provenance.wesl_root_resolution_path().unwrap().as_str(),
        "package"
    );
}

#[test]
fn unused_unresolved_import_is_not_forced_by_strip_false() {
    let artifact = accepted(
        module(
            "package",
            1,
            11,
            1,
            "import package::missing::unused;\n@compute @workgroup_size(1) fn main() {}\n",
        ),
        vec![],
        vec![],
    );
    assert!(artifact.canonical_wgsl().contains("fn main"));
}

#[test]
fn semantically_required_unresolved_import_is_rejected() {
    let result = compile(
        module(
            "package",
            1,
            11,
            1,
            "import package::missing::required;\n@compute @workgroup_size(1) fn main() { required(); }\n",
        ),
        vec![],
        vec![],
    );
    assert!(matches!(result, ShaderCompilationOutcome::Rejected(_)));
}

#[test]
fn missing_and_contradictory_conditional_features_are_rejected() {
    let missing = compile(
        module(
            "package",
            1,
            11,
            1,
            "@if(required_feature)\nfn helper() {}\n",
        ),
        vec![],
        vec![],
    );
    assert!(matches!(missing, ShaderCompilationOutcome::Rejected(_)));

    let contradictory = compile(
        module("package", 1, 21, 1, "fn helper() {}\n"),
        vec![],
        vec![
            ShaderWeslFeatureValue::new("feature_x", false),
            ShaderWeslFeatureValue::new("feature_x", true),
        ],
    );
    assert!(matches!(
        contradictory,
        ShaderCompilationOutcome::Rejected(_)
    ));
}

#[test]
fn malformed_noncanonical_and_external_resolution_table_keys_are_rejected() {
    for (offset, path) in ["self::main", "external::main", "package::"]
        .into_iter()
        .enumerate()
    {
        let result = compile(
            module(path, 1, 30 + offset as u64, 1, "fn helper() {}\n"),
            vec![],
            vec![],
        );
        assert!(
            matches!(result, ShaderCompilationOutcome::Rejected(_)),
            "expected resolution-key rejection for {path:?}, got {result:?}"
        );
    }
}

#[test]
fn contradictory_module_and_source_bindings_are_rejected() {
    let same_module_two_paths = compile(
        module("package", 1, 41, 1, "fn helper() {}\n"),
        vec![module("package::alias", 1, 41, 1, "fn helper() {}\n")],
        vec![],
    );
    assert!(matches!(
        same_module_two_paths,
        ShaderCompilationOutcome::Rejected(_)
    ));

    let same_source_two_modules = compile(
        module("package", 1, 51, 1, "fn helper() {}\n"),
        vec![module("package::other", 2, 51, 1, "fn helper() {}\n")],
        vec![],
    );
    assert!(matches!(
        same_source_two_modules,
        ShaderCompilationOutcome::Rejected(_)
    ));
}

#[test]
fn invalid_syntax_and_external_package_references_are_rejected() {
    let syntax = compile(
        module("package", 1, 11, 1, "fn broken( {\n"),
        vec![],
        vec![],
    );
    assert!(matches!(syntax, ShaderCompilationOutcome::Rejected(_)));

    let external = compile(
        module(
            "package",
            1,
            21,
            1,
            "import external_pkg::math::helper;\nfn main() { helper(); }\n",
        ),
        vec![],
        vec![],
    );
    assert!(matches!(external, ShaderCompilationOutcome::Rejected(_)));
}

#[test]
fn conditional_imports_are_rejected() {
    let conditional_import = compile(
        module(
            "package",
            1,
            11,
            1,
            "@if(feature_x)\nimport package::math::helper;\nfn main() {}\n",
        ),
        vec![module("package::math", 2, 12, 1, "public fn helper() {}\n")],
        vec![ShaderWeslFeatureValue::new("feature_x", true)],
    );
    assert!(matches!(
        conditional_import,
        ShaderCompilationOutcome::Rejected(_)
    ));
}

#[test]
fn profile_valid_global_directive_surface_is_unsupported() {
    let fixtures = [
        "enable f16;\nfn main() {}\n",
        "requires pointer_composite_access;\nfn main() {}\n",
        "diagnostic(off, derivative_uniformity);\nfn main() {}\n",
    ];
    for (offset, source) in fixtures.into_iter().enumerate() {
        let result = compile(
            module("package", 1, 60 + offset as u64, 1, source),
            vec![],
            vec![],
        );
        assert!(
            matches!(result, ShaderCompilationOutcome::Unsupported(_)),
            "expected directive surface to be unsupported, got {result:?}"
        );
    }

    let conditional_directive = compile(
        module(
            "package",
            1,
            70,
            1,
            "@if(feature_x) diagnostic(off, derivative_uniformity);\nfn main() {}\n",
        ),
        vec![],
        vec![ShaderWeslFeatureValue::new("feature_x", true)],
    );
    assert!(matches!(
        conditional_directive,
        ShaderCompilationOutcome::Unsupported(_)
    ));
}

#[test]
fn forbidden_authored_wgsl_extension_is_rejected_before_directive_coverage() {
    let result = compile(
        module(
            "package",
            1,
            11,
            1,
            "enable wgpu_mesh_shader;\nfn main() {}\n",
        ),
        vec![],
        vec![],
    );
    assert!(matches!(result, ShaderCompilationOutcome::Rejected(_)));
}

#[test]
fn syntax_diagnostics_map_back_to_the_exact_admitted_logical_source() {
    let result = compile(
        module("package", 1, 11, 1, "fn main() {}\n"),
        vec![module("package::broken", 2, 12, 7, "fn broken( {\n")],
        vec![],
    );
    let ShaderCompilationOutcome::Rejected(diagnostics) = result else {
        panic!("expected rejection, got {result:?}");
    };
    assert_eq!(
        diagnostics[0]
            .subject()
            .unwrap()
            .source_unit()
            .diagnostic_raw(),
        12
    );
    assert_eq!(
        diagnostics[0]
            .subject()
            .unwrap()
            .revision()
            .diagnostic_raw(),
        7
    );
    assert!(diagnostics[0].range().is_some());
}

#[test]
fn exact_wgsl_path_remains_byte_preserving_after_mapping_generalization() {
    let source = "// exact\r\n@compute @workgroup_size(1) fn main() {}\n";
    let input = ShaderCompilationInput::exact_wgsl(
        ShaderPackageIdentity::try_from_raw(200).unwrap(),
        ShaderModuleIdentity::try_from_raw(201).unwrap(),
        ShaderSourceSnapshot::new(
            ShaderSourceUnitIdentity::try_from_raw(202).unwrap(),
            ShaderSourceRevision::try_from_raw(3).unwrap(),
            source,
        ),
    );
    let invocation =
        ShaderCompilationInvocation::new(input, ShaderCompilerRealization::Naga3001ExactWgslGateV1);
    let result = ShaderCompiler::new().compile(&invocation).unwrap();
    let ShaderCompilationOutcome::Accepted(artifact) = result else {
        panic!("expected exact-WGSL acceptance, got {result:?}");
    };
    assert_eq!(artifact.canonical_wgsl().as_bytes(), source.as_bytes());
    assert!(matches!(
        artifact.source_map(),
        ShaderArtifactSourceMap::ExactIdentity(_)
    ));
    let full = ShaderByteRange::new(0, source.len()).unwrap();
    let mapped = artifact
        .source_map()
        .map_artifact_range(full)
        .unwrap()
        .exact_source()
        .unwrap();
    assert_eq!(mapped.source_unit().diagnostic_raw(), 202);
    assert_eq!(mapped.revision().diagnostic_raw(), 3);
    assert_eq!(mapped.range(), full);
}

#[test]
fn wesl_results_do_not_depend_on_isolated_cwd_environment_or_filesystem_state() {
    let expected = accepted(
        module(
            "package",
            1,
            11,
            1,
            "import package::math::helper;\n@compute @workgroup_size(1) fn main() { helper(); }\n",
        ),
        vec![module("package::math", 2, 12, 1, "public fn helper() {}\n")],
        vec![],
    );
    let expected_marker = format!(
        "RUNEN_SHADER_WESL_HOST_STATE_RESULT={}",
        normative_representation(&expected)
    );

    let base = std::env::temp_dir().join(format!(
        "runen-shader-wesl-host-state-{}",
        std::process::id()
    ));
    let with_noise = base.join("with-noise");
    let without_noise = base.join("without-noise");
    std::fs::create_dir_all(&with_noise).unwrap();
    std::fs::create_dir_all(&without_noise).unwrap();
    std::fs::write(with_noise.join("wesl.toml"), b"[package]\nname='ambient'\n").unwrap();
    std::fs::write(with_noise.join("math.wesl"), b"this must never be read").unwrap();

    let executable = std::env::current_exe().unwrap();
    let run_child = |directory: &std::path::Path, marker: &str| {
        std::process::Command::new(&executable)
            .arg("--exact")
            .arg("wesl_host_state_child_probe")
            .arg("--nocapture")
            .current_dir(directory)
            .env("RUNEN_SHADER_WESL_HOST_STATE_PROBE", "1")
            .env("RUNEN_SHADER_UNRELATED_STATE", marker)
            .output()
            .unwrap()
    };

    let first = run_child(&with_noise, "with-noise");
    let second = run_child(&without_noise, "without-noise");
    assert!(first.status.success(), "first child failed: {first:?}");
    assert!(second.status.success(), "second child failed: {second:?}");
    let marker = |output: &std::process::Output| {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find(|line| line.starts_with("RUNEN_SHADER_WESL_HOST_STATE_RESULT="))
            .map(str::to_owned)
    };
    assert_eq!(marker(&first), Some(expected_marker.clone()));
    assert_eq!(marker(&second), Some(expected_marker));
    std::fs::remove_dir_all(base).unwrap();
}

#[test]
fn wesl_host_state_child_probe() {
    if std::env::var_os("RUNEN_SHADER_WESL_HOST_STATE_PROBE").is_none() {
        return;
    }
    let artifact = accepted(
        module(
            "package",
            1,
            11,
            1,
            "import package::math::helper;\n@compute @workgroup_size(1) fn main() { helper(); }\n",
        ),
        vec![module("package::math", 2, 12, 1, "public fn helper() {}\n")],
        vec![],
    );
    println!(
        "RUNEN_SHADER_WESL_HOST_STATE_RESULT={}",
        normative_representation(&artifact)
    );
}

fn normative_representation(artifact: &runen_shader::ShaderArtifact) -> String {
    format!(
        "bytes={:?};identity={:?};provenance={:?};map={:?}",
        artifact.canonical_wgsl().as_bytes(),
        artifact.identity(),
        artifact.provenance(),
        artifact.source_map(),
    )
}
