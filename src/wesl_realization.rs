use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use wesl::error::{Diagnostic as WeslDiagnostic, ImportError, ResolveError};
use wesl::resolver::{Constants, VirtualResolver};
use wesl::syntax::{
    Attribute, GlobalDirective, ImportStatement, ModulePath, PathOrigin, TranslationUnit,
};
use wesl::{CompileOptions, Compiler, Feature, Features, ManglerKind};

use crate::artifact::{
    ShaderArtifact, ShaderArtifactIdentity, ShaderArtifactProvenance, ShaderArtifactSourceMap,
    ShaderByteRange,
};
use crate::identity::{ShaderModuleIdentity, ShaderSourceRevision, ShaderSourceUnitIdentity};
use crate::input::{ShaderCompilationInvocation, ShaderWeslFeatureValue, ShaderWeslModuleBinding};
use crate::outcome::{
    ShaderCompilationOutcome, ShaderCompilationResult, ShaderDiagnostic, ShaderInvariantError,
    ShaderSourceSubject,
};
use crate::wgsl_gate::{
    ExactWgslDirectiveKind, ExactWgslGateDecision, WgslExtensionDisposition,
    classify_wgsl_extension_name, gate_wgsl_profile_text,
};
use crate::wgsl_validation::{parse_wgsl, validate_wgsl};

struct PreparedModule<'a> {
    path: ModulePath,
    binding: &'a ShaderWeslModuleBinding,
    syntax: TranslationUnit,
}

type SourceBindingKey = (ShaderSourceUnitIdentity, ShaderSourceRevision);

pub(crate) fn compile_wesl(invocation: &ShaderCompilationInvocation) -> ShaderCompilationResult {
    let input = invocation.input();
    let Some(root) = input.wesl_root() else {
        return Err(invariant(
            "WESL realization received input without a WESL root binding",
        ));
    };
    let Some(modules) = input.wesl_modules() else {
        return Err(invariant(
            "WESL realization received input without WESL module evidence",
        ));
    };
    let Some(feature_values) = input.wesl_features() else {
        return Err(invariant(
            "WESL realization received input without WESL feature evidence",
        ));
    };

    let prepared = match prepare_modules(modules) {
        Ok(prepared) => prepared,
        Err(outcome) => return Ok(outcome),
    };
    let root_path = match parse_admitted_module_path(root) {
        Ok(path) => path,
        Err(diagnostic) => return Ok(ShaderCompilationOutcome::Rejected(vec![diagnostic])),
    };

    let Some(prepared_root) = prepared.iter().find(|module| {
        module.path == root_path
            && module.binding.module() == root.module()
            && module.binding.source().source_unit() == root.source().source_unit()
            && module.binding.source().revision() == root.source().revision()
            && module.binding.source().text() == root.source().text()
    }) else {
        return Ok(ShaderCompilationOutcome::Rejected(vec![
            ShaderDiagnostic::new(
                "the WESL root binding is not uniquely represented by the admitted resolution table",
            ),
        ]));
    };

    if let Some(outcome) = preflight_profile_surface(&prepared) {
        return Ok(outcome);
    }

    let features = match prepare_features(feature_values) {
        Ok(features) => features,
        Err(diagnostic) => return Ok(ShaderCompilationOutcome::Rejected(vec![diagnostic])),
    };

    let mut resolver = VirtualResolver::new();
    for module in &prepared {
        resolver.add_module(
            module.path.clone(),
            Cow::Borrowed(module.binding.source().text()),
        );
    }

    let options = CompileOptions {
        imports: true,
        condcomp: true,
        generics: false,
        strip: false,
        lower: false,
        validate: true,
        sourcemap: true,
        mangler: ManglerKind::Escape,
        mangle_main: false,
        keep: None,
        keep_main: false,
        features,
        constants: Constants::default(),
        dependencies: Vec::new(),
    };
    let compiler = Compiler::new_with_resolver(options, resolver);
    let compiled = match compiler.compile_module(&prepared_root.path) {
        Ok(compiled) => compiled,
        Err(error) => return classify_compiler_error(error, &prepared),
    };
    let canonical_wgsl = compiled.to_string();

    if let Some(diagnostic) = generated_wgsl_coverage_failure(&canonical_wgsl) {
        return Ok(ShaderCompilationOutcome::Unsupported(vec![diagnostic]));
    }

    let canonical_wgsl: Arc<str> = Arc::from(canonical_wgsl);
    let byte_len = canonical_wgsl.len();
    Ok(ShaderCompilationOutcome::Accepted(ShaderArtifact {
        identity: ShaderArtifactIdentity::for_invocation(invocation),
        canonical_wgsl,
        provenance: ShaderArtifactProvenance::for_invocation(invocation),
        source_map: ShaderArtifactSourceMap::unattributable(byte_len),
    }))
}

fn prepare_modules<'a>(
    modules: &'a [ShaderWeslModuleBinding],
) -> Result<Vec<PreparedModule<'a>>, ShaderCompilationOutcome> {
    let mut prepared = Vec::<PreparedModule<'a>>::with_capacity(modules.len());
    let mut paths = HashMap::<ModulePath, usize>::new();
    let mut module_bindings =
        HashMap::<ShaderModuleIdentity, (ModulePath, SourceBindingKey)>::new();
    let mut source_modules = HashMap::<SourceBindingKey, ShaderModuleIdentity>::new();

    for binding in modules {
        let path = parse_admitted_module_path(binding)
            .map_err(|diagnostic| ShaderCompilationOutcome::Rejected(vec![diagnostic]))?;

        if let Some(&existing_index) = paths.get(&path) {
            let existing = prepared[existing_index].binding;
            if !same_binding(existing, binding) {
                return Err(ShaderCompilationOutcome::Rejected(vec![
                    ShaderDiagnostic::new(
                        "the WESL resolution table contains contradictory bindings for one canonical module path",
                    ),
                ]));
            }
            continue;
        }

        let source_key = (binding.source().source_unit(), binding.source().revision());
        if let Some((existing_path, existing_source)) = module_bindings.get(&binding.module()) {
            if existing_path != &path || *existing_source != source_key {
                return Err(ShaderCompilationOutcome::Rejected(vec![
                    ShaderDiagnostic::new(
                        "one logical WESL module identity has contradictory resolution or source bindings",
                    ),
                ]));
            }
        } else {
            module_bindings.insert(binding.module(), (path.clone(), source_key));
        }

        if let Some(existing_module) = source_modules.get(&source_key) {
            if *existing_module != binding.module() {
                return Err(ShaderCompilationOutcome::Rejected(vec![
                    ShaderDiagnostic::new(
                        "one logical source snapshot is bound to multiple logical WESL modules",
                    ),
                ]));
            }
        } else {
            source_modules.insert(source_key, binding.module());
        }

        let syntax = match binding.source().text().parse::<TranslationUnit>() {
            Ok(syntax) => syntax,
            Err(error) => {
                let range = checked_source_range(
                    binding,
                    error.span.start,
                    error.span.end,
                    binding.source().text(),
                );
                return Err(ShaderCompilationOutcome::Rejected(vec![source_diagnostic(
                    binding,
                    "the WESL source could not be parsed",
                    range,
                    error.to_string(),
                )]));
            }
        };

        if let Some(outcome) = preflight_module_syntax(binding, &syntax) {
            return Err(outcome);
        }

        let index = prepared.len();
        paths.insert(path.clone(), index);
        prepared.push(PreparedModule {
            path,
            binding,
            syntax,
        });
    }

    Ok(prepared)
}

fn parse_admitted_module_path(
    binding: &ShaderWeslModuleBinding,
) -> Result<ModulePath, ShaderDiagnostic> {
    let raw = binding.resolution_path().as_str();
    let path = raw.parse::<ModulePath>().map_err(|error| {
        source_diagnostic(
            binding,
            "the WESL module-resolution key is malformed",
            None,
            error.to_string(),
        )
    })?;
    if !matches!(&path.origin, PathOrigin::Absolute) {
        return Err(source_diagnostic(
            binding,
            "the WESL module-resolution key is not anchored at the admitted package root",
            None,
            format!("non-canonical WESL module key `{raw}`"),
        ));
    }
    if path.to_string() != raw {
        return Err(source_diagnostic(
            binding,
            "the WESL module-resolution key is not in canonical package form",
            None,
            format!("canonical form is `{path}`"),
        ));
    }

    let probe = format!("import {raw}::__runen_shader_path_probe;");
    if probe.parse::<ImportStatement>().is_err() {
        return Err(source_diagnostic(
            binding,
            "the WESL module-resolution key is not a canonical WESL module path",
            None,
            format!("invalid WESL module key `{raw}`"),
        ));
    }
    Ok(path)
}

fn same_binding(left: &ShaderWeslModuleBinding, right: &ShaderWeslModuleBinding) -> bool {
    left.module() == right.module()
        && left.source().source_unit() == right.source().source_unit()
        && left.source().revision() == right.source().revision()
        && left.source().text() == right.source().text()
}

fn preflight_module_syntax(
    binding: &ShaderWeslModuleBinding,
    syntax: &TranslationUnit,
) -> Option<ShaderCompilationOutcome> {
    for import in &syntax.imports {
        if import.attributes.iter().any(is_conditional_attribute) {
            return Some(ShaderCompilationOutcome::Rejected(vec![source_diagnostic(
                binding,
                "conditional attributes on WESL import statements are outside the accepted profile",
                None,
                "wesl-composition-2026-08-22 excludes conditional import attributes".to_string(),
            )]));
        }
        if import
            .path
            .as_ref()
            .is_some_and(|path| matches!(&path.origin, PathOrigin::Package(_)))
        {
            return Some(ShaderCompilationOutcome::Rejected(vec![source_diagnostic(
                binding,
                "external WESL package references are outside the accepted single-package profile",
                None,
                "external package import rejected before resolver dispatch".to_string(),
            )]));
        }
    }

    for directive in &syntax.global_directives {
        let invalid_extension = match directive {
            GlobalDirective::Enable(directive) => directive.extensions.iter().find(|name| {
                classify_wgsl_extension_name(ExactWgslDirectiveKind::Enable, name)
                    == WgslExtensionDisposition::Rejected
            }),
            GlobalDirective::Requires(directive) => directive.extensions.iter().find(|name| {
                classify_wgsl_extension_name(ExactWgslDirectiveKind::Requires, name)
                    == WgslExtensionDisposition::Rejected
            }),
            GlobalDirective::Diagnostic(_) => None,
        };
        if let Some(name) = invalid_extension {
            return Some(ShaderCompilationOutcome::Rejected(vec![source_diagnostic(
                binding,
                "the WESL source requests a WGSL extension outside the accepted WGSL language envelope",
                None,
                format!("out-of-profile WGSL extension `{name}`"),
            )]));
        }
    }

    None
}

fn preflight_profile_surface(prepared: &[PreparedModule<'_>]) -> Option<ShaderCompilationOutcome> {
    for module in prepared {
        if !module.syntax.global_directives.is_empty() {
            return Some(ShaderCompilationOutcome::Unsupported(vec![
                source_diagnostic(
                    module.binding,
                    "WESL global-directive composition is outside wesl-rs 0.5.0 V1 coverage",
                    None,
                    "upstream wesl-rs directive composition remains incomplete (issue #85)"
                        .to_string(),
                ),
            ]));
        }
    }
    None
}

fn prepare_features(
    feature_values: &[ShaderWeslFeatureValue],
) -> Result<Features, ShaderDiagnostic> {
    let mut flags = HashMap::<String, Feature>::new();
    for feature in feature_values {
        let value = if feature.enabled() {
            Feature::Enable
        } else {
            Feature::Disable
        };
        match flags.insert(feature.name().to_string(), value) {
            Some(previous) if previous != value => {
                return Err(ShaderDiagnostic::new(
                    "the WESL conditional-feature table contains contradictory values for one feature name",
                ));
            }
            _ => {}
        }
    }
    Ok(Features {
        default: Feature::Error,
        flags,
    })
}

fn is_conditional_attribute(attribute: &Attribute) -> bool {
    matches!(
        attribute,
        Attribute::If(_) | Attribute::Elif(_) | Attribute::Else
    )
}

fn generated_wgsl_coverage_failure(canonical_wgsl: &str) -> Option<ShaderDiagnostic> {
    match gate_wgsl_profile_text(canonical_wgsl) {
        ExactWgslGateDecision::Continue => {}
        ExactWgslGateDecision::Rejected(_) => {
            return Some(ShaderDiagnostic::new(
                "the WESL realization generated WGSL outside the accepted WGSL language envelope",
            ));
        }
        ExactWgslGateDecision::Unsupported(_) | ExactWgslGateDecision::DeferToParser => {
            return Some(ShaderDiagnostic::new(
                "generated WGSL is outside this WESL realization's proven WGSL-gate coverage",
            ));
        }
    }

    let module = match parse_wgsl(canonical_wgsl) {
        Ok(module) => module,
        Err(error) => {
            return Some(
                ShaderDiagnostic::new(
                    "generated WGSL could not be parsed by the independent pinned WGSL admission gate",
                )
                .with_realization_detail(format!("Naga WGSL parse detail: {error}")),
            );
        }
    };
    if let Err(error) = validate_wgsl(&module) {
        return Some(
            ShaderDiagnostic::new(
                "generated WGSL could not be validated by the independent pinned WGSL admission gate",
            )
            .with_realization_detail(format!("Naga semantic validation detail: {error}")),
        );
    }
    None
}

fn classify_compiler_error(
    error: wesl::Error,
    prepared: &[PreparedModule<'_>],
) -> ShaderCompilationResult {
    match compiler_error_class(&error) {
        CompilerErrorClass::Rejected => {
            let diagnostic: WeslDiagnostic = error.into();
            Ok(ShaderCompilationOutcome::Rejected(vec![
                translate_diagnostic(
                    diagnostic,
                    prepared,
                    "the WESL source could not be translated under the accepted profile",
                ),
            ]))
        }
        CompilerErrorClass::Failed => {
            let diagnostic: WeslDiagnostic = error.into();
            Ok(ShaderCompilationOutcome::Failed(vec![
                translate_diagnostic(
                    diagnostic,
                    prepared,
                    "the WESL compiler could not execute the selected realization",
                ),
            ]))
        }
        CompilerErrorClass::Invariant => Err(invariant(
            "closed WESL realization attempted forbidden ambient resolution or configuration",
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompilerErrorClass {
    Rejected,
    Failed,
    Invariant,
}

fn compiler_error_class(error: &wesl::Error) -> CompilerErrorClass {
    match error {
        wesl::Error::ParseError(_)
        | wesl::Error::ValidateError(_)
        | wesl::Error::UsageError(_)
        | wesl::Error::CondCompError(_) => CompilerErrorClass::Rejected,
        wesl::Error::ResolveError(error) => resolve_error_class(error),
        wesl::Error::ImportError(error) => match error {
            ImportError::ResolveError(error) => resolve_error_class(error),
            ImportError::DuplicateSymbol(_) => CompilerErrorClass::Rejected,
        },
        wesl::Error::TomlError(_) => CompilerErrorClass::Invariant,
        wesl::Error::Error(diagnostic) => compiler_error_class(&diagnostic.error),
        wesl::Error::Custom(_) => CompilerErrorClass::Failed,
    }
}

fn resolve_error_class(error: &ResolveError) -> CompilerErrorClass {
    match error {
        ResolveError::ModuleNotFound(_, _) => CompilerErrorClass::Rejected,
        ResolveError::Error(diagnostic) => compiler_error_class(&diagnostic.error),
        ResolveError::Io(_)
        | ResolveError::FileNotFound(_, _)
        | ResolveError::FilesystemNotSupported
        | ResolveError::FileEscapesRoot(_, _) => CompilerErrorClass::Invariant,
    }
}

fn translate_diagnostic(
    diagnostic: WeslDiagnostic,
    prepared: &[PreparedModule<'_>],
    summary: &'static str,
) -> ShaderDiagnostic {
    let realization_detail = diagnostic.to_string();
    let binding = diagnostic
        .detail
        .module_path
        .as_ref()
        .and_then(|path| prepared.iter().find(|module| &module.path == path))
        .map(|module| module.binding);

    let mut translated = ShaderDiagnostic::new(summary).with_realization_detail(realization_detail);
    if let Some(binding) = binding {
        let range = diagnostic.detail.span.and_then(|span| {
            diagnostic
                .detail
                .source
                .as_deref()
                .and_then(|source| checked_source_range(binding, span.start, span.end, source))
        });
        translated = translated.with_source(source_subject(binding), range);
    }
    translated
}

fn source_diagnostic(
    binding: &ShaderWeslModuleBinding,
    summary: &'static str,
    range: Option<ShaderByteRange>,
    realization_detail: String,
) -> ShaderDiagnostic {
    ShaderDiagnostic::new(summary)
        .with_source(source_subject(binding), range)
        .with_realization_detail(realization_detail)
}

fn source_subject(binding: &ShaderWeslModuleBinding) -> ShaderSourceSubject {
    ShaderSourceSubject::new(binding.source().source_unit(), binding.source().revision())
}

fn checked_source_range(
    binding: &ShaderWeslModuleBinding,
    start: usize,
    end: usize,
    diagnostic_source: &str,
) -> Option<ShaderByteRange> {
    let source = binding.source().text();
    if source != diagnostic_source
        || start > end
        || end > source.len()
        || !source.is_char_boundary(start)
        || !source.is_char_boundary(end)
    {
        return None;
    }
    ShaderByteRange::new(start, end)
}

const fn invariant(summary: &'static str) -> ShaderInvariantError {
    ShaderInvariantError { summary }
}

#[cfg(test)]
mod tests {
    use crate::identity::{
        ShaderModuleIdentity, ShaderPackageIdentity, ShaderSourceRevision, ShaderSourceUnitIdentity,
    };
    use crate::input::{
        ShaderCompilationInput, ShaderWeslFeatureValue, ShaderWeslModuleBinding,
        ShaderWeslModulePath,
    };
    use crate::profile::ShaderCompilerRealization;
    use crate::source::ShaderSourceSnapshot;

    use super::*;

    fn binding(path: &str, module: u64, source_unit: u64, source: &str) -> ShaderWeslModuleBinding {
        ShaderWeslModuleBinding::new(
            ShaderWeslModulePath::new(path),
            ShaderModuleIdentity::try_from_raw(module).unwrap(),
            ShaderSourceSnapshot::new(
                ShaderSourceUnitIdentity::try_from_raw(source_unit).unwrap(),
                ShaderSourceRevision::try_from_raw(1).unwrap(),
                source,
            ),
        )
    }

    fn invocation(
        root: ShaderWeslModuleBinding,
        modules: Vec<ShaderWeslModuleBinding>,
        features: Vec<ShaderWeslFeatureValue>,
    ) -> ShaderCompilationInvocation {
        ShaderCompilationInvocation::new(
            ShaderCompilationInput::wesl_composition(
                ShaderPackageIdentity::try_from_raw(1).unwrap(),
                root,
                modules,
                features,
            ),
            ShaderCompilerRealization::Wesl050Composition20260822V1,
        )
    }

    #[test]
    fn duplicate_resolution_keys_with_conflicting_targets_are_rejected() {
        let root = binding("package::main", 1, 10, "fn main() {}\n");
        let result = compile_wesl(&invocation(
            root,
            vec![
                binding("package::dup", 2, 11, "fn a() {}\n"),
                binding("package::dup", 3, 12, "fn b() {}\n"),
            ],
            vec![],
        ))
        .unwrap();
        assert!(matches!(result, ShaderCompilationOutcome::Rejected(_)));
    }

    #[test]
    fn contradictory_logical_bindings_are_rejected() {
        let root = binding("package", 1, 10, "fn main() {}\n");
        let same_module_different_path = compile_wesl(&invocation(
            root.clone(),
            vec![binding("package::other", 1, 10, "fn main() {}\n")],
            vec![],
        ))
        .unwrap();
        assert!(matches!(
            same_module_different_path,
            ShaderCompilationOutcome::Rejected(_)
        ));

        let same_source_different_module = compile_wesl(&invocation(
            root,
            vec![binding("package::other", 2, 10, "fn main() {}\n")],
            vec![],
        ))
        .unwrap();
        assert!(matches!(
            same_source_different_module,
            ShaderCompilationOutcome::Rejected(_)
        ));
    }

    #[test]
    fn contradictory_feature_values_are_rejected() {
        let root = binding("package::main", 1, 10, "fn main() {}\n");
        let result = compile_wesl(&invocation(
            root,
            vec![],
            vec![
                ShaderWeslFeatureValue::new("FLAG", true),
                ShaderWeslFeatureValue::new("FLAG", false),
            ],
        ))
        .unwrap();
        assert!(matches!(result, ShaderCompilationOutcome::Rejected(_)));
    }

    #[test]
    fn profile_valid_global_directives_fail_closed_as_unsupported() {
        let root = binding("package::main", 1, 10, "enable f16;\nfn main() {}\n");
        let result = compile_wesl(&invocation(root, vec![], vec![])).unwrap();
        assert!(matches!(result, ShaderCompilationOutcome::Unsupported(_)));
    }

    #[test]
    fn profile_invalid_global_directives_are_rejected_before_upstream_coverage_gate() {
        let root = binding(
            "package::main",
            1,
            10,
            "enable wgpu_mesh_shader;\nfn main() {}\n",
        );
        let result = compile_wesl(&invocation(root, vec![], vec![])).unwrap();
        assert!(matches!(result, ShaderCompilationOutcome::Rejected(_)));
    }

    #[test]
    fn generated_wgsl_outside_the_pinned_envelope_is_not_accepted() {
        let diagnostic = generated_wgsl_coverage_failure(
            "enable wgpu_mesh_shader;\n@compute @workgroup_size(1) fn main() {}\n",
        );
        assert!(diagnostic.is_some());
    }
}
