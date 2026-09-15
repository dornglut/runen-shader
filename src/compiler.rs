use std::collections::HashMap;
use std::sync::Arc;

use crate::artifact::{
    ShaderArtifact, ShaderArtifactIdentity, ShaderArtifactProvenance, ShaderArtifactSourceMap,
    ShaderByteRange,
};
use crate::identity::{ShaderSourceRevision, ShaderSourceUnitIdentity};
use crate::input::ShaderCompilationInvocation;
use crate::outcome::{
    ShaderCompilationOutcome, ShaderCompilationResult, ShaderDiagnostic, ShaderInvariantError,
    ShaderSourceSubject,
};
use crate::profile::{ShaderCompilerRealization, ShaderFrontendProfile};
use crate::wesl_realization::compile_wesl;
use crate::wgsl_gate::{ExactWgslGateDecision, ExactWgslGateFinding, gate_exact_wgsl_profile};
use crate::wgsl_validation::{parse_wgsl, validate_wgsl};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GateParserDecision {
    ContinueToValidation,
    Rejected,
    Unsupported,
    Invariant,
}

/// Stateful authority for RunenShader compiler realization dispatch.
///
/// The compiler records the exact bytes first observed for every logical
/// `(source unit, revision)` pair in one admitted closed input before realization dispatch. This is
/// instance state rather than semantic identity or a process-global registry. Realization dispatch
/// is explicit: exact WGSL uses the pinned Naga gate, while WESL composition uses the pinned closed
/// wesl-rs realization.
#[derive(Debug, Default)]
pub struct ShaderCompiler {
    source_bindings: HashMap<(ShaderSourceUnitIdentity, ShaderSourceRevision), Arc<str>>,
}

impl ShaderCompiler {
    /// Creates an empty compiler authority with no observed source bindings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Compiles one explicit invocation using an accepted implemented realization.
    pub fn compile(&mut self, invocation: &ShaderCompilationInvocation) -> ShaderCompilationResult {
        if let Some(rejected) = self.bind_invocation_sources(invocation) {
            return Ok(rejected);
        }

        match (invocation.input().profile(), invocation.realization()) {
            (
                ShaderFrontendProfile::WgslExact20260817,
                ShaderCompilerRealization::Naga3001ExactWgslGateV1,
            ) => self.compile_exact_wgsl(invocation),
            (
                ShaderFrontendProfile::WeslComposition20260822,
                ShaderCompilerRealization::Wesl050Composition20260822V1,
            ) => compile_wesl(invocation),
            _ => {
                let source = invocation.input().source();
                let subject = ShaderSourceSubject::new(source.source_unit(), source.revision());
                Ok(ShaderCompilationOutcome::Unsupported(vec![
                    ShaderDiagnostic::new(
                        "the selected compiler realization does not provide accepted coverage for the requested frontend profile",
                    )
                    .with_source(subject, None),
                ]))
            }
        }
    }

    fn compile_exact_wgsl(
        &self,
        invocation: &ShaderCompilationInvocation,
    ) -> ShaderCompilationResult {
        let source = invocation.input().source();
        let subject = ShaderSourceSubject::new(source.source_unit(), source.revision());
        let gate = gate_exact_wgsl_profile(source);
        if let ExactWgslGateDecision::Rejected(finding) = gate {
            return Ok(ShaderCompilationOutcome::Rejected(vec![gate_diagnostic(
                subject, finding, false,
            )]));
        }

        let parsed = parse_wgsl(source.text());
        let module = match parsed {
            Ok(module) => match reconcile_gate_and_parse(gate, true, None) {
                GateParserDecision::ContinueToValidation => module,
                GateParserDecision::Unsupported => {
                    return Ok(ShaderCompilationOutcome::Unsupported(vec![
                        ShaderDiagnostic::new(
                            "the source parsed, but the profile gate could not establish pinned-profile admission",
                        )
                        .with_source(subject, None),
                    ]));
                }
                GateParserDecision::Invariant => {
                    return Err(ShaderInvariantError {
                        summary: "the profile gate reported unsupported coverage for source accepted by Naga",
                    });
                }
                GateParserDecision::Rejected => {
                    return Err(ShaderInvariantError {
                        summary: "gate/parser reconciliation rejected a successfully parsed source",
                    });
                }
            },
            Err(error) => {
                let diagnostics = parse_diagnostics(subject, &error, source.text());
                let primary_range = primary_error_range(&error, source.text());
                match reconcile_gate_and_parse(gate, false, primary_range) {
                    GateParserDecision::Unsupported => {
                        let ExactWgslGateDecision::Unsupported(finding) = gate else {
                            unreachable!("unsupported parser decision requires an unsupported gate")
                        };
                        return Ok(ShaderCompilationOutcome::Unsupported(vec![
                            gate_diagnostic(subject, finding, true),
                        ]));
                    }
                    GateParserDecision::Rejected => {
                        return Ok(ShaderCompilationOutcome::Rejected(diagnostics));
                    }
                    GateParserDecision::ContinueToValidation | GateParserDecision::Invariant => {
                        return Err(ShaderInvariantError {
                            summary: "gate/parser reconciliation returned an invalid parse-failure decision",
                        });
                    }
                }
            }
        };

        if let Err(error) = validate_wgsl(&module) {
            return Ok(ShaderCompilationOutcome::Rejected(validation_diagnostics(
                subject,
                source.text(),
                &error,
            )));
        }

        Ok(ShaderCompilationOutcome::Accepted(ShaderArtifact {
            identity: ShaderArtifactIdentity::for_invocation(invocation),
            canonical_wgsl: Arc::clone(&source.source),
            provenance: ShaderArtifactProvenance::for_invocation(invocation),
            source_map: ShaderArtifactSourceMap::exact(source),
        }))
    }

    fn bind_invocation_sources(
        &mut self,
        invocation: &ShaderCompilationInvocation,
    ) -> Option<ShaderCompilationOutcome> {
        let input = invocation.input();
        let sources = if let Some(modules) = input.wesl_modules() {
            modules
                .iter()
                .map(|module| module.source())
                .collect::<Vec<_>>()
        } else {
            vec![input.source()]
        };

        let mut pending =
            HashMap::<(ShaderSourceUnitIdentity, ShaderSourceRevision), Arc<str>>::new();

        for source in sources {
            let subject = ShaderSourceSubject::new(source.source_unit(), source.revision());
            let binding = (source.source_unit(), source.revision());
            let previous = self
                .source_bindings
                .get(&binding)
                .or_else(|| pending.get(&binding));

            if let Some(previous) = previous {
                if previous.as_ref() != source.text() {
                    return Some(ShaderCompilationOutcome::Rejected(vec![
                        ShaderDiagnostic::new(
                            "the source revision is already bound to different exact source bytes",
                        )
                        .with_source(subject, None),
                    ]));
                }
            } else {
                pending.insert(binding, Arc::clone(&source.source));
            }
        }

        self.source_bindings.extend(pending);
        None
    }
}

fn gate_diagnostic(
    subject: ShaderSourceSubject,
    finding: ExactWgslGateFinding,
    unsupported: bool,
) -> ShaderDiagnostic {
    let summary = if unsupported {
        "the source uses a pinned WGSL extension outside this realization's demonstrated coverage"
    } else {
        "the source uses an extension outside the selected exact-WGSL profile"
    };
    ShaderDiagnostic::new(summary).with_source(subject, Some(finding.name_range()))
}

fn parse_diagnostics(
    subject: ShaderSourceSubject,
    error: &naga::front::wgsl::ParseError,
    source: &str,
) -> Vec<ShaderDiagnostic> {
    let detail = format!("Naga WGSL parse detail: {}", error);
    let diagnostics = error
        .labels()
        .filter_map(|(span, _)| checked_range(span.to_range(), source))
        .map(|range| {
            ShaderDiagnostic::new("the WGSL source could not be parsed")
                .with_source(subject, Some(range))
                .with_realization_detail(detail.clone())
        })
        .collect::<Vec<_>>();

    if diagnostics.is_empty() {
        vec![
            ShaderDiagnostic::new("the WGSL source could not be parsed")
                .with_source(subject, None)
                .with_realization_detail(detail),
        ]
    } else {
        diagnostics
    }
}

fn validation_diagnostics(
    subject: ShaderSourceSubject,
    source: &str,
    error: &naga::WithSpan<naga::valid::ValidationError>,
) -> Vec<ShaderDiagnostic> {
    let detail = format!("Naga semantic validation detail: {}", error);
    let diagnostics = error
        .spans()
        .filter_map(|(span, _)| checked_range(span.to_range(), source))
        .map(|range| {
            ShaderDiagnostic::new("the WGSL source failed semantic validation")
                .with_source(subject, Some(range))
                .with_realization_detail(detail.clone())
        })
        .collect::<Vec<_>>();

    if diagnostics.is_empty() {
        vec![
            ShaderDiagnostic::new("the WGSL source failed semantic validation")
                .with_source(subject, None)
                .with_realization_detail(detail),
        ]
    } else {
        diagnostics
    }
}

fn reconcile_gate_and_parse(
    gate: ExactWgslGateDecision,
    parse_success: bool,
    primary_parse_range: Option<ShaderByteRange>,
) -> GateParserDecision {
    match (gate, parse_success) {
        (ExactWgslGateDecision::Continue, true) => GateParserDecision::ContinueToValidation,
        (ExactWgslGateDecision::Continue, false)
        | (ExactWgslGateDecision::DeferToParser, false)
        | (ExactWgslGateDecision::Rejected(_), false) => GateParserDecision::Rejected,
        (ExactWgslGateDecision::DeferToParser, true) => GateParserDecision::Unsupported,
        (ExactWgslGateDecision::Unsupported(finding), false) => {
            if primary_parse_range
                .map(|range| ranges_overlap(range, finding.name_range()))
                .unwrap_or(false)
            {
                GateParserDecision::Unsupported
            } else {
                GateParserDecision::Rejected
            }
        }
        (ExactWgslGateDecision::Unsupported(_), true) => GateParserDecision::Invariant,
        (ExactWgslGateDecision::Rejected(_), true) => GateParserDecision::Invariant,
    }
}

fn primary_error_range(
    error: &naga::front::wgsl::ParseError,
    source: &str,
) -> Option<ShaderByteRange> {
    // Naga documents labels().next() as the primary parse span; complementary labels are
    // translated for diagnostics but must not change gate/outcome classification.
    error
        .labels()
        .next()
        .and_then(|(span, _)| checked_range(span.to_range(), source))
}

fn ranges_overlap(left: ShaderByteRange, right: ShaderByteRange) -> bool {
    left.start() < right.end() && right.start() < left.end()
}

fn checked_range(range: Option<std::ops::Range<usize>>, source: &str) -> Option<ShaderByteRange> {
    let range = range?;
    (range.end <= source.len() && source.get(range.clone()).is_some())
        .then(|| ShaderByteRange::new(range.start, range.end))
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{
        ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision, ShaderSourceUnitIdentity,
    };
    use crate::input::{ShaderCompilationInput, ShaderWeslModuleBinding, ShaderWeslModulePath};
    use crate::source::ShaderSourceSnapshot;

    fn invocation(source_unit: u64, revision: u64, source: &str) -> ShaderCompilationInvocation {
        ShaderCompilationInvocation::new(
            ShaderCompilationInput::exact_wgsl(
                ShaderPackageIdentity::try_from_raw(1).unwrap(),
                ShaderModuleIdentity::try_from_raw(2).unwrap(),
                ShaderSourceSnapshot::new(
                    ShaderSourceUnitIdentity::try_from_raw(source_unit).unwrap(),
                    ShaderSourceRevision::try_from_raw(revision).unwrap(),
                    source,
                ),
            ),
            ShaderCompilerRealization::Naga3001ExactWgslGateV1,
        )
    }

    fn wesl_module(
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

    fn wesl_invocation(
        root: ShaderWeslModuleBinding,
        additional: Vec<ShaderWeslModuleBinding>,
    ) -> ShaderCompilationInvocation {
        ShaderCompilationInvocation::new(
            ShaderCompilationInput::wesl_composition(
                ShaderPackageIdentity::try_from_raw(700).unwrap(),
                root,
                additional,
                vec![],
            ),
            ShaderCompilerRealization::Naga3001ExactWgslGateV1,
        )
    }

    fn outcome(compiler: &mut ShaderCompiler, source: &str) -> ShaderCompilationOutcome {
        compiler.compile(&invocation(3, 4, source)).unwrap()
    }

    #[test]
    fn accepts_valid_modules_with_and_without_entry_points_and_preserves_bytes() {
        let mut compiler = ShaderCompiler::new();
        let source = "// exact\r\nfn helper() { }\n";
        let accepted = outcome(&mut compiler, source);
        let ShaderCompilationOutcome::Accepted(artifact) = accepted else {
            panic!("expected helper-only module to be accepted");
        };
        assert_eq!(artifact.canonical_wgsl().as_bytes(), source.as_bytes());
        let full_range = ShaderByteRange::new(0, source.len()).unwrap();
        let mapped = artifact
            .source_map()
            .map_artifact_range(full_range)
            .unwrap()
            .exact_source()
            .unwrap();
        assert_eq!(mapped.range(), full_range);
        assert_eq!(mapped.source_unit(), artifact.provenance().source_unit());
        assert_eq!(mapped.revision(), artifact.provenance().source_revision());

        let mut compiler = ShaderCompiler::new();
        let accepted = compiler
            .compile(&invocation(
                5,
                6,
                "@compute @workgroup_size(1)\nfn main() {}",
            ))
            .unwrap();
        assert!(matches!(accepted, ShaderCompilationOutcome::Accepted(_)));
    }

    #[test]
    fn accepts_every_gate_continue_extension_with_representative_constructs() {
        let fixtures = [
            ("f16", "enable f16;\nfn half() -> f16 { return f16(1.0); }"),
            (
                "clip_distances",
                "enable clip_distances;\nstruct Output { @builtin(position) position: vec4f, @builtin(clip_distances) clips: array<f32, 1> }\n@vertex fn main() -> Output { var output: Output; output.position = vec4f(0.0); output.clips = array<f32, 1>(0.0); return output; }",
            ),
            (
                "dual_source_blending",
                "enable dual_source_blending;\nstruct Output { @location(0) @blend_src(0) color0: vec4f, @location(0) @blend_src(1) color1: vec4f }\n@fragment fn main() -> Output { return Output(vec4f(1.0), vec4f(0.0)); }",
            ),
            (
                "primitive_index",
                "enable primitive_index;\n@fragment fn main(@builtin(primitive_index) index: u32) -> @location(0) vec4f { return vec4f(f32(index)); }",
            ),
            (
                "readonly_and_readwrite_storage_textures",
                "requires readonly_and_readwrite_storage_textures;\n@group(0) @binding(0) var image: texture_storage_2d<r32float, read>;\n@compute @workgroup_size(1) fn main() { let value = textureLoad(image, vec2u(0)); }",
            ),
            (
                "packed_4x8_integer_dot_product",
                "requires packed_4x8_integer_dot_product;\nfn packed() -> u32 { return dot4U8Packed(0u, 0u); }",
            ),
            (
                "pointer_composite_access",
                "requires pointer_composite_access;\nfn first() -> i32 { var values: array<i32, 2>; let pointer = &values; return (*pointer)[0]; }",
            ),
        ];

        for (index, (name, source)) in fixtures.into_iter().enumerate() {
            let mut compiler = ShaderCompiler::new();
            let result = compiler
                .compile(&invocation(30 + index as u64, 1, source))
                .unwrap();
            assert!(
                matches!(result, ShaderCompilationOutcome::Accepted(_)),
                "expected {name} fixture to be accepted, got {result:?}"
            );
        }
    }

    #[test]
    fn rejects_parse_and_semantic_errors_with_logical_ranges() {
        let mut compiler = ShaderCompiler::new();
        let parse = outcome(&mut compiler, "fn broken( {");
        let ShaderCompilationOutcome::Rejected(parse_diagnostics) = parse else {
            panic!("expected parse rejection");
        };
        let parse_subject = ShaderSourceSubject::new(
            ShaderSourceUnitIdentity::try_from_raw(3).unwrap(),
            ShaderSourceRevision::try_from_raw(4).unwrap(),
        );
        let expected_parse_range = ShaderByteRange::new(11, 12).unwrap();
        assert!(
            parse_diagnostics
                .iter()
                .all(|diagnostic| diagnostic.subject() == Some(parse_subject))
        );
        assert!(
            parse_diagnostics
                .iter()
                .all(|diagnostic| diagnostic.range() == Some(expected_parse_range))
        );

        let semantic_source = "fn bad() { break; }";
        let semantic = compiler
            .compile(&invocation(8, 9, semantic_source))
            .unwrap();
        let ShaderCompilationOutcome::Rejected(semantic_diagnostics) = semantic else {
            panic!("expected semantic rejection");
        };
        assert!(
            semantic_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.summary().contains("semantic validation")),
            "unexpected diagnostics: {semantic_diagnostics:?}"
        );
        let semantic_subject = ShaderSourceSubject::new(
            ShaderSourceUnitIdentity::try_from_raw(8).unwrap(),
            ShaderSourceRevision::try_from_raw(9).unwrap(),
        );
        assert!(
            semantic_diagnostics
                .iter()
                .all(|diagnostic| diagnostic.subject() == Some(semantic_subject))
        );
        assert!(semantic_diagnostics.iter().all(|diagnostic| {
            let Some(range) = diagnostic.range() else {
                return false;
            };
            range.end() <= semantic_source.len()
                && semantic_source.get(range.start()..range.end()).is_some()
                && range.start() <= semantic_source.find("break").unwrap()
                && range.end() >= semantic_source.find("break").unwrap() + "break".len()
        }));
    }

    #[test]
    fn gate_rejects_naga_only_extensions_before_parser_acceptance() {
        let mut compiler = ShaderCompiler::new();
        let result = outcome(&mut compiler, "enable wgpu_mesh_shader;\nfn helper() {}\n");
        let ShaderCompilationOutcome::Rejected(diagnostics) = result else {
            panic!("expected profile rejection");
        };
        assert_eq!(
            diagnostics[0].range().unwrap().byte_len(),
            "wgpu_mesh_shader".len()
        );
    }

    #[test]
    fn unsupported_gate_requires_correlated_primary_parser_evidence() {
        let mut compiler = ShaderCompiler::new();
        let result = outcome(&mut compiler, "enable subgroups;\n");
        assert!(matches!(result, ShaderCompilationOutcome::Unsupported(_)));

        let result = compiler
            .compile(&invocation(7, 4, "fn broken(\nenable subgroups;\n"))
            .unwrap();
        assert!(
            matches!(result, ShaderCompilationOutcome::Rejected(_)),
            "unexpected outcome: {result:?}"
        );
    }

    #[test]
    fn every_gate_unsupported_requires_name_has_realization_evidence() {
        for (offset, name) in ["subgroups", "subgroup_size_control"]
            .into_iter()
            .enumerate()
        {
            let mut compiler = ShaderCompiler::new();
            let result = compiler
                .compile(&invocation(
                    100 + offset as u64,
                    1,
                    &format!("enable {name};\n"),
                ))
                .unwrap();
            assert!(
                matches!(result, ShaderCompilationOutcome::Unsupported(_)),
                "expected unsupported result for {name}, got {result:?}"
            );
        }

        let names = [
            "unrestricted_pointer_parameters",
            "uniform_buffer_standard_layout",
            "subgroup_id",
            "subgroup_uniformity",
            "texture_and_sampler_let",
            "texture_formats_tier1",
            "linear_indexing",
            "immediate_address_space",
            "fragment_depth",
            "buffer_view",
        ];

        for (offset, name) in names.into_iter().enumerate() {
            let mut compiler = ShaderCompiler::new();
            let result = compiler
                .compile(&invocation(
                    100 + offset as u64,
                    1,
                    &format!("requires {name};\n"),
                ))
                .unwrap();
            assert!(
                matches!(result, ShaderCompilationOutcome::Unsupported(_)),
                "expected unsupported result for {name}, got {result:?}"
            );
        }
    }

    #[test]
    fn defer_to_parser_parse_failure_is_rejected() {
        let mut compiler = ShaderCompiler::new();
        let result = outcome(&mut compiler, "enable f16 ?;");
        assert!(matches!(result, ShaderCompilationOutcome::Rejected(_)));
    }

    #[test]
    fn defer_to_parser_parse_success_fails_closed_as_unsupported() {
        assert_eq!(
            reconcile_gate_and_parse(ExactWgslGateDecision::DeferToParser, true, None),
            GateParserDecision::Unsupported
        );
    }

    #[test]
    fn unsupported_parse_failure_uses_only_the_primary_label() {
        let source = ShaderSourceSnapshot::new(
            ShaderSourceUnitIdentity::try_from_raw(500).unwrap(),
            ShaderSourceRevision::try_from_raw(1).unwrap(),
            "enable subgroups;\n",
        );
        let ExactWgslGateDecision::Unsupported(finding) = gate_exact_wgsl_profile(&source) else {
            panic!("expected unsupported gate finding");
        };
        let gate = ExactWgslGateDecision::Unsupported(finding);

        assert_eq!(
            reconcile_gate_and_parse(gate, false, ShaderByteRange::new(0, 5)),
            GateParserDecision::Rejected
        );
        assert_eq!(
            reconcile_gate_and_parse(gate, false, ShaderByteRange::new(12, 16)),
            GateParserDecision::Unsupported
        );
    }

    #[test]
    fn source_revision_binding_records_rejected_first_observation() {
        let mut compiler = ShaderCompiler::new();
        assert!(matches!(
            outcome(&mut compiler, "fn broken( {"),
            ShaderCompilationOutcome::Rejected(_)
        ));
        let rebound = compiler
            .compile(&invocation(3, 4, "fn helper() {}"))
            .unwrap();
        assert!(matches!(rebound, ShaderCompilationOutcome::Rejected(_)));
    }

    #[test]
    fn source_revision_binding_records_unsupported_first_observation() {
        let mut compiler = ShaderCompiler::new();
        assert!(matches!(
            outcome(&mut compiler, "enable subgroups;\n"),
            ShaderCompilationOutcome::Unsupported(_)
        ));
        let rebound = compiler
            .compile(&invocation(3, 4, "fn helper() {}"))
            .unwrap();
        assert!(matches!(rebound, ShaderCompilationOutcome::Rejected(_)));
    }

    #[test]
    fn multi_source_binding_is_atomic_when_a_later_wesl_snapshot_conflicts() {
        let mut compiler = ShaderCompiler::new();
        let seeded = compiler
            .compile(&invocation(810, 1, "fn seeded() {}"))
            .unwrap();
        assert!(matches!(seeded, ShaderCompilationOutcome::Accepted(_)));

        let wesl = wesl_invocation(
            wesl_module("package::main", 811, 811, 1, "fn root() {}"),
            vec![wesl_module(
                "package::z_conflict",
                812,
                810,
                1,
                "fn conflicting() {}",
            )],
        );
        let result = compiler.compile(&wesl).unwrap();
        assert!(matches!(result, ShaderCompilationOutcome::Rejected(_)));

        // The unseen root from the rejected multi-source input must not have been partially bound.
        let root_reuse = compiler
            .compile(&invocation(811, 1, "fn different_root() {}"))
            .unwrap();
        assert!(matches!(root_reuse, ShaderCompilationOutcome::Accepted(_)));
    }

    #[test]
    fn all_wesl_sources_bind_before_profile_realization_mismatch_is_unsupported() {
        let mut compiler = ShaderCompiler::new();
        let wesl = wesl_invocation(
            wesl_module("package::main", 820, 820, 1, "fn root() {}"),
            vec![wesl_module("package::math", 821, 821, 1, "fn helper() {}")],
        );

        let result = compiler.compile(&wesl).unwrap();
        let ShaderCompilationOutcome::Unsupported(diagnostics) = result else {
            panic!("expected WESL request with the exact-WGSL realization to be unsupported");
        };
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].subject(),
            Some(ShaderSourceSubject::new(
                ShaderSourceUnitIdentity::try_from_raw(820).unwrap(),
                ShaderSourceRevision::try_from_raw(1).unwrap(),
            ))
        );

        let root_rebound = compiler
            .compile(&invocation(820, 1, "fn changed_root() {}"))
            .unwrap();
        assert!(matches!(
            root_rebound,
            ShaderCompilationOutcome::Rejected(_)
        ));
        let dependency_rebound = compiler
            .compile(&invocation(821, 1, "fn changed_helper() {}"))
            .unwrap();
        assert!(matches!(
            dependency_rebound,
            ShaderCompilationOutcome::Rejected(_)
        ));
    }

    #[test]
    fn exact_wgsl_with_the_wesl_realization_is_ordinary_unsupported_coverage() {
        let mut compiler = ShaderCompiler::new();
        let exact = ShaderCompilationInvocation::new(
            ShaderCompilationInput::exact_wgsl(
                ShaderPackageIdentity::try_from_raw(1).unwrap(),
                ShaderModuleIdentity::try_from_raw(2).unwrap(),
                ShaderSourceSnapshot::new(
                    ShaderSourceUnitIdentity::try_from_raw(900).unwrap(),
                    ShaderSourceRevision::try_from_raw(1).unwrap(),
                    "fn helper() {}",
                ),
            ),
            ShaderCompilerRealization::Wesl050Composition20260822V1,
        );
        let result = compiler.compile(&exact).unwrap();
        assert!(matches!(result, ShaderCompilationOutcome::Unsupported(_)));
    }

    #[test]
    fn identical_repeated_source_and_distinct_units_are_allowed() {
        let mut compiler = ShaderCompiler::new();
        let first = outcome(&mut compiler, "fn helper() {}\n");
        let second = outcome(&mut compiler, "fn helper() {}\n");
        let ShaderCompilationOutcome::Accepted(first_artifact) = first else {
            panic!("expected first identical source to be accepted");
        };
        let ShaderCompilationOutcome::Accepted(second_artifact) = second else {
            panic!("expected repeated identical source to be accepted");
        };
        assert_eq!(
            first_artifact.canonical_wgsl().as_bytes(),
            second_artifact.canonical_wgsl().as_bytes()
        );
        assert_eq!(first_artifact, second_artifact);

        let distinct_unit = compiler
            .compile(&invocation(7, 4, "fn helper() {}\n"))
            .unwrap();
        let ShaderCompilationOutcome::Accepted(distinct_artifact) = distinct_unit else {
            panic!("expected distinct logical source unit to be accepted");
        };
        assert_eq!(
            first_artifact.canonical_wgsl().as_bytes(),
            distinct_artifact.canonical_wgsl().as_bytes()
        );
        assert_ne!(first_artifact.identity(), distinct_artifact.identity());
        assert_ne!(
            first_artifact.provenance().source_unit(),
            distinct_artifact.provenance().source_unit()
        );
        let full_range = ShaderByteRange::new(0, first_artifact.source_map().byte_len()).unwrap();
        let first_mapped = first_artifact
            .source_map()
            .map_artifact_range(full_range)
            .unwrap()
            .exact_source()
            .unwrap();
        let distinct_mapped = distinct_artifact
            .source_map()
            .map_artifact_range(full_range)
            .unwrap()
            .exact_source()
            .unwrap();
        assert_ne!(first_mapped.source_unit(), distinct_mapped.source_unit());
    }

    #[test]
    fn compilation_results_do_not_depend_on_invocation_order() {
        let first_source = "fn first() {}\n";
        let second_source = "fn second() {}\n";

        let mut left = ShaderCompiler::new();
        let left_first = left.compile(&invocation(20, 1, first_source)).unwrap();
        let left_second = left.compile(&invocation(21, 1, second_source)).unwrap();

        let mut right = ShaderCompiler::new();
        let right_second = right.compile(&invocation(21, 1, second_source)).unwrap();
        let right_first = right.compile(&invocation(20, 1, first_source)).unwrap();

        assert_eq!(left_first, right_first);
        assert_eq!(left_second, right_second);
    }

    #[test]
    fn compilation_results_do_not_depend_on_isolated_host_state() {
        let expected = outcome(&mut ShaderCompiler::new(), "fn helper() {}\n");
        let ShaderCompilationOutcome::Accepted(artifact) = expected else {
            panic!("expected accepted host-state probe result")
        };
        let expected_marker = format!(
            "RUNEN_SHADER_HOST_STATE_RESULT={}",
            normative_test_representation(&artifact)
        );

        let base =
            std::env::temp_dir().join(format!("runen-shader-host-state-{}", std::process::id()));
        std::fs::create_dir(&base).expect("create isolated host-state directory");
        let with_cache = base.join("with-cache");
        let without_cache = base.join("without-cache");
        std::fs::create_dir(&with_cache).expect("create cache-present directory");
        std::fs::create_dir(&without_cache).expect("create cache-absent directory");
        std::fs::write(with_cache.join("unrelated.data"), b"unrelated")
            .expect("write unrelated filesystem contents");
        std::fs::write(with_cache.join("shader-cache-like"), b"derived")
            .expect("write cache-like contents");

        let executable = std::env::current_exe().expect("locate test executable");
        let run_child = |directory: &std::path::Path, marker: &str, cache: &str| {
            std::process::Command::new(&executable)
                .arg("--exact")
                .arg("compiler::tests::host_state_child_probe")
                .arg("--nocapture")
                .current_dir(directory)
                .env("RUNEN_SHADER_HOST_STATE_PROBE", "1")
                .env("RUNEN_SHADER_UNRELATED_STATE", marker)
                .env("RUNEN_SHADER_CACHE_LIKE", cache)
                .output()
                .expect("run isolated host-state child")
        };

        let first = run_child(&with_cache, "with-unrelated-files", "present");
        let second = run_child(&without_cache, "without-unrelated-files", "absent");

        assert!(first.status.success(), "first child failed: {first:?}");
        assert!(second.status.success(), "second child failed: {second:?}");
        let marker = |output: &std::process::Output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .find(|line| line.starts_with("RUNEN_SHADER_HOST_STATE_RESULT="))
                .map(str::to_owned)
        };
        assert_eq!(marker(&first), Some(expected_marker.clone()));
        assert_eq!(marker(&second), Some(expected_marker));
        std::fs::remove_dir_all(base).expect("remove isolated host-state directory");
    }

    #[test]
    fn host_state_child_probe() {
        if std::env::var_os("RUNEN_SHADER_HOST_STATE_PROBE").is_none() {
            return;
        }

        let result = outcome(&mut ShaderCompiler::new(), "fn helper() {}\n");
        let ShaderCompilationOutcome::Accepted(artifact) = result else {
            panic!("expected accepted host-state probe result")
        };
        println!(
            "RUNEN_SHADER_HOST_STATE_RESULT={}",
            normative_test_representation(&artifact)
        );
    }

    fn normative_test_representation(artifact: &ShaderArtifact) -> String {
        let input = artifact.identity().compilation_input();
        let provenance = artifact.provenance();
        let full_range = ShaderByteRange::new(0, artifact.source_map().byte_len()).unwrap();
        let mapped = artifact
            .source_map()
            .map_artifact_range(full_range)
            .unwrap()
            .exact_source()
            .unwrap();

        format!(
            "outcome=accepted;canonical_bytes={:?};artifact_identity.input.package={};artifact_identity.input.root_module={};artifact_identity.input.source_unit={};artifact_identity.input.revision={};artifact_identity.input.profile={};artifact_identity.realization={};provenance.package={};provenance.root_module={};provenance.source_unit={};provenance.revision={};provenance.profile={};provenance.realization={};source_map.byte_len={};source_map.source_unit={};source_map.revision={};source_map.range=[{}, {})",
            artifact.canonical_wgsl().as_bytes(),
            input.package().diagnostic_raw(),
            input.root_module().diagnostic_raw(),
            input.source_unit().diagnostic_raw(),
            input.source_revision().diagnostic_raw(),
            input.profile().semantic_name(),
            artifact.identity().realization().diagnostic_label(),
            provenance.package().diagnostic_raw(),
            provenance.root_module().diagnostic_raw(),
            provenance.source_unit().diagnostic_raw(),
            provenance.source_revision().diagnostic_raw(),
            provenance.profile().semantic_name(),
            provenance.realization().diagnostic_label(),
            artifact.source_map().byte_len(),
            mapped.source_unit().diagnostic_raw(),
            mapped.revision().diagnostic_raw(),
            mapped.range().start(),
            mapped.range().end(),
        )
    }
}
