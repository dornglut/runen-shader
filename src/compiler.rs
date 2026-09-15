use std::collections::HashMap;
use std::sync::Arc;

use crate::artifact::{
    ExactWgslSourceMap, ShaderArtifact, ShaderArtifactIdentity, ShaderArtifactProvenance,
    ShaderByteRange,
};
use crate::identity::{ShaderSourceRevision, ShaderSourceUnitIdentity};
use crate::input::ShaderCompilationInvocation;
use crate::outcome::{
    ShaderCompilationOutcome, ShaderCompilationResult, ShaderDiagnostic, ShaderInvariantError,
    ShaderSourceSubject,
};
use crate::profile::{ShaderCompilerRealization, ShaderFrontendProfile};
use crate::wgsl_gate::{ExactWgslGateDecision, ExactWgslGateFinding, gate_exact_wgsl_profile};

/// Stateful authority for the pinned exact-WGSL compiler realization.
///
/// The compiler records the exact bytes first observed for each logical
/// `(source unit, revision)` pair. This is instance state rather than semantic
/// identity or a process-global registry.
#[derive(Debug, Default)]
pub struct ShaderCompiler {
    source_bindings: HashMap<(ShaderSourceUnitIdentity, ShaderSourceRevision), Arc<str>>,
}

impl ShaderCompiler {
    /// Creates an empty compiler authority with no observed source bindings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Compiles one explicit invocation using the pinned Naga realization.
    pub fn compile(&mut self, invocation: &ShaderCompilationInvocation) -> ShaderCompilationResult {
        let source = invocation.input().source();
        let subject = ShaderSourceSubject::new(source.source_unit(), source.revision());
        let binding = (source.source_unit(), source.revision());

        if let Some(previous) = self.source_bindings.get(&binding) {
            if previous.as_ref() != source.text() {
                return Ok(ShaderCompilationOutcome::Rejected(vec![
                    ShaderDiagnostic::new(
                        "the source revision is already bound to different exact source bytes",
                    )
                    .with_source(subject, None),
                ]));
            }
        } else {
            self.source_bindings
                .insert(binding, Arc::clone(&source.source));
        }

        if invocation.input().profile() != ShaderFrontendProfile::WgslExact20260817
            || invocation.realization() != ShaderCompilerRealization::Naga3001ExactWgslGateV1
        {
            return Err(ShaderInvariantError {
                summary: "the invocation selects an unsupported RunenShader realization",
            });
        }

        let gate = gate_exact_wgsl_profile(source);
        if let ExactWgslGateDecision::Rejected(finding) = gate {
            return Ok(ShaderCompilationOutcome::Rejected(vec![gate_diagnostic(
                subject, finding, false,
            )]));
        }

        let mut frontend =
            naga::front::wgsl::Frontend::new_with_options(naga::front::wgsl::Options {
                parse_doc_comments: false,
                capabilities: naga::valid::Capabilities::all(),
            });
        let parsed = frontend.parse(source.text());

        let module = match parsed {
            Ok(module) => module,
            Err(error) => {
                let diagnostics = parse_diagnostics(subject, &error, source.text());
                if let ExactWgslGateDecision::Unsupported(finding) = gate
                    && error_overlaps_finding(&error, source.text(), finding)
                {
                    return Ok(ShaderCompilationOutcome::Unsupported(vec![
                        gate_diagnostic(subject, finding, true),
                    ]));
                }
                return Ok(ShaderCompilationOutcome::Rejected(diagnostics));
            }
        };

        if let ExactWgslGateDecision::DeferToParser = gate {
            return Ok(ShaderCompilationOutcome::Unsupported(vec![
                ShaderDiagnostic::new(
                    "the source parsed, but the profile gate could not establish pinned-profile admission",
                )
                .with_source(subject, None),
            ]));
        }

        if matches!(gate, ExactWgslGateDecision::Unsupported(_)) {
            return Err(ShaderInvariantError {
                summary: "the profile gate reported unsupported coverage for source accepted by Naga",
            });
        }

        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator.subgroup_stages(naga::valid::ShaderStages::all());
        validator.subgroup_operations(naga::valid::SubgroupOperationSet::all());

        if let Err(error) = validator.validate(&module) {
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
            source_map: ExactWgslSourceMap::for_source(source),
        }))
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

fn error_overlaps_finding(
    error: &naga::front::wgsl::ParseError,
    source: &str,
    finding: ExactWgslGateFinding,
) -> bool {
    error.labels().any(|(span, _)| {
        checked_range(span.to_range(), source)
            .map(|range| ranges_overlap(range, finding.name_range()))
            .unwrap_or(false)
    })
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
    use crate::input::ShaderCompilationInput;
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
            .unwrap();
        assert_eq!(mapped.range(), full_range);
        assert_eq!(mapped.source_unit(), artifact.provenance().source_unit());
        assert_eq!(mapped.revision(), artifact.provenance().source_revision());

        let mut compiler = ShaderCompiler::new();
        let accepted = compiler
            .compile(&invocation(
                5,
                6,
                "enable f16;\nfn half() -> f16 { return f16(1.0); }",
            ))
            .unwrap();
        assert!(matches!(accepted, ShaderCompilationOutcome::Accepted(_)));
    }

    #[test]
    fn rejects_parse_and_semantic_errors_with_logical_ranges() {
        let mut compiler = ShaderCompiler::new();
        let parse = outcome(&mut compiler, "fn broken( {");
        let ShaderCompilationOutcome::Rejected(parse_diagnostics) = parse else {
            panic!("expected parse rejection");
        };
        assert_eq!(
            parse_diagnostics[0]
                .subject()
                .unwrap()
                .source_unit()
                .diagnostic_raw(),
            3
        );
        assert!(
            parse_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.range().is_some())
        );

        let semantic = compiler
            .compile(&invocation(8, 9, "fn bad() { break; }"))
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
    fn unsupported_gate_requires_correlated_parser_evidence() {
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
        let names = [
            "unrestricted_pointer_parameters",
            "uniform_buffer_standard_layout",
            "subgroup_id",
            "subgroup_uniformity",
            "texture_and_sampler_let",
            "texture_formats_tier1",
            "linear_indexing",
            "immediate_address_space",
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
    fn identical_repeated_source_and_distinct_units_are_allowed() {
        let mut compiler = ShaderCompiler::new();
        let first = outcome(&mut compiler, "fn helper() {}\n");
        let second = outcome(&mut compiler, "fn helper() {}\n");
        assert!(matches!(&first, ShaderCompilationOutcome::Accepted(_)));
        assert_eq!(first, second);

        let distinct_unit = compiler
            .compile(&invocation(7, 4, "fn helper() {}\n"))
            .unwrap();
        assert!(matches!(
            distinct_unit,
            ShaderCompilationOutcome::Accepted(_)
        ));
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
}
