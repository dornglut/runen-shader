use crate::artifact::ShaderByteRange;
use crate::source::ShaderSourceSnapshot;

/// Internal profile-gate result for the pinned exact-WGSL profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExactWgslGateDecision {
    /// Every observed relevant directive is in-profile and inside Gate V1 coverage.
    Continue,
    /// A complete observed extension name is outside the pinned WGSL profile.
    Rejected(ExactWgslGateFinding),
    /// A complete observed extension name is in-profile but outside Gate V1 coverage.
    Unsupported(ExactWgslGateFinding),
}

/// Relevant directive family for one profile-gate finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExactWgslDirectiveKind {
    Enable,
    Requires,
}

/// Source evidence for one profile-gate classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExactWgslGateFinding {
    directive: ExactWgslDirectiveKind,
    name_range: ShaderByteRange,
}

impl ExactWgslGateFinding {
    pub(crate) const fn directive(self) -> ExactWgslDirectiveKind {
        self.directive
    }

    pub(crate) const fn name_range(self) -> ShaderByteRange {
        self.name_range
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NameDisposition {
    Continue,
    Rejected,
    Unsupported,
}

/// Applies the RunenShader-owned directive-extension gate for `wgsl-exact-2026-08-17`.
///
/// This is deliberately not a WGSL parser. It recognizes only the initial global-directive
/// prefix needed to prevent the selected compiler realization from defining profile membership.
/// Any uncertain or malformed directive shape is left to the later WGSL parser.
pub(crate) fn gate_exact_wgsl_profile(source: &ShaderSourceSnapshot) -> ExactWgslGateDecision {
    let text = source.text();
    let mut cursor = 0;
    let mut first_rejected = None;
    let mut first_unsupported = None;

    loop {
        cursor = match skip_trivia(text, cursor) {
            Some(next) => next,
            None => return finish(first_rejected, first_unsupported),
        };

        if cursor == text.len() {
            return finish(first_rejected, first_unsupported);
        }

        let (keyword_end, keyword) = match scan_ascii_identifier(text, cursor) {
            Some(token) => token,
            None => return finish(first_rejected, first_unsupported),
        };

        match keyword {
            "enable" => {
                let Some((next, findings)) =
                    scan_extension_directive(text, keyword_end, ExactWgslDirectiveKind::Enable)
                else {
                    return finish(first_rejected, first_unsupported);
                };
                record_findings(
                    findings,
                    &mut first_rejected,
                    &mut first_unsupported,
                );
                cursor = next;
            }
            "requires" => {
                let Some((next, findings)) =
                    scan_extension_directive(text, keyword_end, ExactWgslDirectiveKind::Requires)
                else {
                    return finish(first_rejected, first_unsupported);
                };
                record_findings(
                    findings,
                    &mut first_rejected,
                    &mut first_unsupported,
                );
                cursor = next;
            }
            "diagnostic" => {
                let Some(next) = scan_diagnostic_directive(text, keyword_end) else {
                    return finish(first_rejected, first_unsupported);
                };
                cursor = next;
            }
            _ => return finish(first_rejected, first_unsupported),
        }
    }
}

fn finish(
    first_rejected: Option<ExactWgslGateFinding>,
    first_unsupported: Option<ExactWgslGateFinding>,
) -> ExactWgslGateDecision {
    if let Some(finding) = first_rejected {
        ExactWgslGateDecision::Rejected(finding)
    } else if let Some(finding) = first_unsupported {
        ExactWgslGateDecision::Unsupported(finding)
    } else {
        ExactWgslGateDecision::Continue
    }
}

fn record_findings(
    findings: Vec<(NameDisposition, ExactWgslGateFinding)>,
    first_rejected: &mut Option<ExactWgslGateFinding>,
    first_unsupported: &mut Option<ExactWgslGateFinding>,
) {
    for (disposition, finding) in findings {
        match disposition {
            NameDisposition::Continue => {}
            NameDisposition::Rejected if first_rejected.is_none() => {
                *first_rejected = Some(finding);
            }
            NameDisposition::Unsupported if first_unsupported.is_none() => {
                *first_unsupported = Some(finding);
            }
            NameDisposition::Rejected | NameDisposition::Unsupported => {}
        }
    }
}

fn scan_extension_directive(
    source: &str,
    after_keyword: usize,
    directive: ExactWgslDirectiveKind,
) -> Option<(usize, Vec<(NameDisposition, ExactWgslGateFinding)>)> {
    let mut cursor = skip_trivia(source, after_keyword)?;
    let mut findings = Vec::new();

    loop {
        let name_start = cursor;
        let (name_end, name) = scan_ascii_identifier(source, cursor)?;
        let range = ShaderByteRange::new(name_start, name_end)?;
        let disposition = classify_extension_name(directive, name);
        findings.push((
            disposition,
            ExactWgslGateFinding {
                directive,
                name_range: range,
            },
        ));
        cursor = skip_trivia(source, name_end)?;

        match source.as_bytes().get(cursor).copied() {
            Some(b';') => return Some((cursor + 1, findings)),
            Some(b',') => {
                cursor = skip_trivia(source, cursor + 1)?;
                if source.as_bytes().get(cursor) == Some(&b';') {
                    return Some((cursor + 1, findings));
                }
            }
            _ => return None,
        }
    }
}

fn scan_diagnostic_directive(source: &str, after_keyword: usize) -> Option<usize> {
    let mut cursor = skip_trivia(source, after_keyword)?;
    cursor = consume_ascii(source, cursor, b'(')?;
    cursor = skip_trivia(source, cursor)?;

    let (severity_end, _) = scan_ascii_identifier(source, cursor)?;
    cursor = skip_trivia(source, severity_end)?;
    cursor = consume_ascii(source, cursor, b',')?;
    cursor = skip_trivia(source, cursor)?;

    let (rule_end, _) = scan_ascii_identifier(source, cursor)?;
    cursor = skip_trivia(source, rule_end)?;
    if source.as_bytes().get(cursor) == Some(&b'.') {
        cursor = skip_trivia(source, cursor + 1)?;
        let (member_end, _) = scan_ascii_identifier(source, cursor)?;
        cursor = skip_trivia(source, member_end)?;
    }

    if source.as_bytes().get(cursor) == Some(&b',') {
        cursor = skip_trivia(source, cursor + 1)?;
    }

    cursor = consume_ascii(source, cursor, b')')?;
    cursor = skip_trivia(source, cursor)?;
    consume_ascii(source, cursor, b';')
}

fn classify_extension_name(
    directive: ExactWgslDirectiveKind,
    name: &str,
) -> NameDisposition {
    match directive {
        ExactWgslDirectiveKind::Enable => match name {
            "f16" | "clip_distances" | "dual_source_blending" | "primitive_index" => {
                NameDisposition::Continue
            }
            "subgroups" | "subgroup_size_control" => NameDisposition::Unsupported,
            _ => NameDisposition::Rejected,
        },
        ExactWgslDirectiveKind::Requires => match name {
            "readonly_and_readwrite_storage_textures"
            | "packed_4x8_integer_dot_product"
            | "pointer_composite_access" => NameDisposition::Continue,
            "unrestricted_pointer_parameters"
            | "uniform_buffer_standard_layout"
            | "subgroup_id"
            | "subgroup_uniformity"
            | "texture_and_sampler_let"
            | "texture_formats_tier1"
            | "linear_indexing"
            | "immediate_address_space"
            | "buffer_view" => NameDisposition::Unsupported,
            _ => NameDisposition::Rejected,
        },
    }
}

fn skip_trivia(source: &str, mut cursor: usize) -> Option<usize> {
    loop {
        if cursor >= source.len() {
            return Some(source.len());
        }

        let bytes = source.as_bytes();
        if bytes.get(cursor..cursor + 2) == Some(b"//") {
            cursor = skip_line_comment(source, cursor + 2)?;
            continue;
        }
        if bytes.get(cursor..cursor + 2) == Some(b"/*") {
            cursor = skip_block_comment(source, cursor)?;
            continue;
        }

        let ch = source.get(cursor..)?.chars().next()?;
        if is_pattern_whitespace(ch) {
            cursor += ch.len_utf8();
            continue;
        }

        return Some(cursor);
    }
}

fn skip_line_comment(source: &str, mut cursor: usize) -> Option<usize> {
    while cursor < source.len() {
        let ch = source.get(cursor..)?.chars().next()?;
        if is_line_break(ch) {
            return Some(cursor);
        }
        cursor += ch.len_utf8();
    }
    Some(source.len())
}

fn skip_block_comment(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut cursor = start + 2;
    let mut depth = 1usize;

    while cursor < bytes.len() {
        if bytes.get(cursor..cursor + 2) == Some(b"/*") {
            depth = depth.checked_add(1)?;
            cursor += 2;
        } else if bytes.get(cursor..cursor + 2) == Some(b"*/") {
            depth -= 1;
            cursor += 2;
            if depth == 0 {
                return Some(cursor);
            }
        } else {
            cursor += 1;
        }
    }

    None
}

fn scan_ascii_identifier(source: &str, start: usize) -> Option<(usize, &str)> {
    let bytes = source.as_bytes();
    let first = *bytes.get(start)?;
    if !is_ascii_identifier_start(first) {
        return None;
    }

    let mut end = start + 1;
    while let Some(&byte) = bytes.get(end) {
        if is_ascii_identifier_continue(byte) {
            end += 1;
        } else {
            break;
        }
    }

    if first == b'_' && end == start + 1 {
        return None;
    }

    // A non-ASCII code point immediately following the ASCII prefix may be an XID continuation.
    // Without a Unicode XID table, fail open rather than splitting one WGSL identifier incorrectly.
    if bytes.get(end).is_some_and(|byte| !byte.is_ascii()) {
        return None;
    }

    Some((end, source.get(start..end)?))
}

const fn is_ascii_identifier_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

const fn is_ascii_identifier_continue(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}

fn consume_ascii(source: &str, cursor: usize, expected: u8) -> Option<usize> {
    (source.as_bytes().get(cursor) == Some(&expected)).then_some(cursor + 1)
}

const fn is_pattern_whitespace(ch: char) -> bool {
    matches!(
        ch,
        '\u{0009}'
            | '\u{000a}'
            | '\u{000b}'
            | '\u{000c}'
            | '\u{000d}'
            | '\u{0020}'
            | '\u{0085}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{2028}'
            | '\u{2029}'
    )
}

const fn is_line_break(ch: char) -> bool {
    matches!(
        ch,
        '\u{000a}'
            | '\u{000b}'
            | '\u{000c}'
            | '\u{000d}'
            | '\u{0085}'
            | '\u{2028}'
            | '\u{2029}'
    )
}

#[cfg(test)]
mod tests {
    use crate::identity::{ShaderSourceRevision, ShaderSourceUnitIdentity};

    use super::*;

    fn snapshot(source: &str) -> ShaderSourceSnapshot {
        ShaderSourceSnapshot::new(
            ShaderSourceUnitIdentity::try_from_raw(1).unwrap(),
            ShaderSourceRevision::try_from_raw(2).unwrap(),
            source,
        )
    }

    fn decision(source: &str) -> ExactWgslGateDecision {
        gate_exact_wgsl_profile(&snapshot(source))
    }

    fn assert_unsupported(source: &str, expected_name: &str) {
        let ExactWgslGateDecision::Unsupported(finding) = decision(source) else {
            panic!("expected unsupported gate decision for {source:?}");
        };
        let range = finding.name_range();
        assert_eq!(&source[range.start()..range.end()], expected_name);
    }

    fn assert_rejected(source: &str, expected_name: &str) {
        let ExactWgslGateDecision::Rejected(finding) = decision(source) else {
            panic!("expected rejected gate decision for {source:?}");
        };
        let range = finding.name_range();
        assert_eq!(&source[range.start()..range.end()], expected_name);
    }

    #[test]
    fn baseline_enable_names_match_gate_v1_coverage() {
        for name in [
            "f16",
            "clip_distances",
            "dual_source_blending",
            "primitive_index",
        ] {
            assert_eq!(
                decision(&format!("enable {name};")),
                ExactWgslGateDecision::Continue
            );
        }

        for name in ["subgroups", "subgroup_size_control"] {
            assert_unsupported(&format!("enable {name};"), name);
        }
    }

    #[test]
    fn baseline_language_extension_names_match_gate_v1_coverage() {
        for name in [
            "readonly_and_readwrite_storage_textures",
            "packed_4x8_integer_dot_product",
            "pointer_composite_access",
        ] {
            assert_eq!(
                decision(&format!("requires {name};")),
                ExactWgslGateDecision::Continue
            );
        }

        for name in [
            "unrestricted_pointer_parameters",
            "uniform_buffer_standard_layout",
            "subgroup_id",
            "subgroup_uniformity",
            "texture_and_sampler_let",
            "texture_formats_tier1",
            "linear_indexing",
            "immediate_address_space",
            "buffer_view",
        ] {
            assert_unsupported(&format!("requires {name};"), name);
        }
    }

    #[test]
    fn realization_only_and_unknown_names_are_rejected() {
        for name in [
            "wgpu_mesh_shader",
            "wgpu_ray_query",
            "wgpu_cooperative_matrix",
            "wgpu_binding_array",
            "wgpu_int16",
            "draw_index",
            "future_extension",
        ] {
            assert_rejected(&format!("enable {name};"), name);
        }
        assert_rejected("requires future_extension;", "future_extension");
    }

    #[test]
    fn rejection_precedes_unsupported_across_complete_directives() {
        let source = "enable subgroups;\nrequires future_extension;";
        let ExactWgslGateDecision::Rejected(finding) = decision(source) else {
            panic!("expected out-of-profile rejection to take precedence");
        };
        assert_eq!(finding.directive(), ExactWgslDirectiveKind::Requires);
        let range = finding.name_range();
        assert_eq!(&source[range.start()..range.end()], "future_extension");
    }

    #[test]
    fn comma_lists_trailing_commas_and_multiple_directives_are_supported() {
        let source = concat!(
            "enable f16, primitive_index,;\n",
            "requires packed_4x8_integer_dot_product, pointer_composite_access;\n",
            "fn helper() {}\n"
        );
        assert_eq!(decision(source), ExactWgslGateDecision::Continue);

        assert_unsupported("enable f16, subgroups,;", "subgroups");
    }

    #[test]
    fn diagnostic_directives_do_not_hide_later_profile_directives() {
        let source = concat!(
            "diagnostic(off, derivative_uniformity);\n",
            "enable f16;\n",
            "diagnostic(warning, chromium.unreachable_code);\n",
            "requires pointer_composite_access;\n",
            "fn helper() {}\n"
        );
        assert_eq!(decision(source), ExactWgslGateDecision::Continue);
    }

    #[test]
    fn comments_and_nested_block_comments_cannot_forge_directives() {
        let source = concat!(
            "/* enable wgpu_mesh_shader; /* requires future_extension; */ */\n",
            "// enable wgpu_int16;\n",
            "enable f16;\n",
            "fn helper() {} // requires future_extension;\n"
        );
        assert_eq!(decision(source), ExactWgslGateDecision::Continue);
    }

    #[test]
    fn scanning_stops_at_first_non_directive_global_token() {
        let source = concat!(
            "fn enable() { let requires = 1; }\n",
            "enable wgpu_mesh_shader;\n"
        );
        assert_eq!(decision(source), ExactWgslGateDecision::Continue);
    }

    #[test]
    fn finding_ranges_are_exact_utf8_byte_offsets() {
        let source = "\u{200e}/* é */ enable wgpu_mesh_shader;";
        let ExactWgslGateDecision::Rejected(finding) = decision(source) else {
            panic!("expected rejection");
        };
        let expected_start = source.find("wgpu_mesh_shader").unwrap();
        assert_eq!(finding.name_range().start(), expected_start);
        assert_eq!(
            finding.name_range().end(),
            expected_start + "wgpu_mesh_shader".len()
        );
    }

    #[test]
    fn malformed_or_incomplete_directives_fail_open_to_later_parser() {
        for source in [
            "enable wgpu_mesh_shader",
            "enable wgpu_mesh_shader ?;",
            "requires unrestricted_pointer_parameters",
            "diagnostic(off, derivative_uniformity",
            "enable ;",
        ] {
            assert_eq!(decision(source), ExactWgslGateDecision::Continue);
        }
    }

    #[test]
    fn possible_unicode_identifier_continuation_fails_open() {
        let source = "enable\u{0301} wgpu_mesh_shader;";
        assert_eq!(decision(source), ExactWgslGateDecision::Continue);
    }

    #[test]
    fn repeated_scans_are_deterministic() {
        let source = "enable subgroups;\nfn helper() {}";
        assert_eq!(decision(source), decision(source));
    }
}
