# Exact WGSL frontend profile

Status: **Accepted R0 frontend-profile contract**.

This profile specializes the [semantic model](semantic-model.md) for the first
concrete RunenShader source-to-artifact path. It selects exact WGSL as the first
profile because doing so exercises RunenShader identity, validation, artifact,
provenance, diagnostic, and reproducibility semantics without introducing module
composition or source rewriting before those semantics are implemented.

## 1. Profile identity and external language baseline

**WGSL-PROFILE-001 — Accepted.** The first frontend profile is identified
semantically as `wgsl-exact-2026-08-17`.

**WGSL-PROFILE-002 — Accepted.** `wgsl-exact-2026-08-17` incorporates the WGSL
language semantics of the W3C WebGPU Shading Language Candidate Recommendation
Draft dated 17 August 2026.

**WGSL-PROFILE-003 — Accepted.** The profile MUST NOT float with a `latest`,
editor's-draft, compiler-default, browser, or `wgpu` interpretation of WGSL. A
later external WGSL revision changes RunenShader support only through accepted
profile-evolution work.

**WGSL-PROFILE-004 — Accepted.** RunenShader does not redefine WGSL syntax,
typing, execution semantics, or standard language-extension meaning. This file
owns only the RunenShader profile selection, restrictions, artifact behavior,
and realization/conformance boundary.

## 2. Input specialization

The generic source/package/module model remains authoritative. This profile
specializes it rather than introducing WGSL-specific parallel identities.

**WGSL-IN-001 — Accepted.** One compilation input for this profile MUST contain
exactly one package identity, exactly one root module identity, and exactly one
participating source snapshot.

**WGSL-IN-002 — Accepted.** The single source snapshot MUST be the complete source
of the root WGSL module. The profile admits no dependency-resolution edges,
imports, implicit includes, generated companion sources, or multi-unit module
composition.

**WGSL-IN-003 — Accepted.** The exact source snapshot bytes defined by
`RS-SRC-001` are the WGSL text presented to the profile realization. No Unicode,
line-ending, byte-order-mark, whitespace, comment, spelling, or formatting
normalization occurs before profile validation.

**WGSL-IN-004 — Accepted.** A valid WGSL module does not need to declare a shader
entry point. Entry-point availability and downstream pipeline usability are not
requirements for source-profile acceptance.

**WGSL-IN-005 — Accepted.** Filesystem path, filename extension, working
directory, source origin, and display name remain non-semantic provenance. The
profile MUST NOT infer its root module or source identity from them.

## 3. Profile language surface

**WGSL-LANG-001 — Accepted.** Source MUST conform to the external WGSL baseline
selected by `WGSL-PROFILE-002` and to the additional RunenShader restrictions in
this profile.

**WGSL-LANG-002 — Accepted.** A WGSL extension, directive, feature name, syntax
form, or semantic construct is inside this profile only when it is defined by the
selected external baseline. Recognition by the selected compiler realization is
not sufficient.

**WGSL-LANG-003 — Accepted.** Naga-specific, `wgpu`-specific, experimental, or
other implementation extensions absent from the selected external baseline MUST
be rejected as outside this profile even when the realization can parse or
validate them.

**WGSL-LANG-004 — Accepted.** The implementation MUST maintain an explicit
profile gate for external-baseline extension/directive admission wherever the
selected realization recognizes a wider surface. Compiler defaults MUST NOT
silently broaden this profile.

**WGSL-LANG-005 — Accepted.** Device-specific availability is not a source
profile restriction. A construct valid in the selected WGSL baseline MUST NOT be
rejected merely because a particular future GPU device lacks the capability to
execute it. Device and pipeline admission remain downstream authority.

## 4. Canonical artifact formation

**WGSL-ART-001 — Accepted.** If the invocation is accepted, the canonical WGSL
artifact byte sequence MUST be byte-for-byte identical to the admitted source
snapshot byte sequence.

**WGSL-ART-002 — Accepted.** The realization MUST NOT use a WGSL backend,
pretty-printer, formatter, normalizer, minifier, optimizer, IR round trip, or
other source re-emission step to form the canonical artifact.

**WGSL-ART-003 — Accepted.** Parse or validation IR MAY exist transiently as
private realization state. It MUST be discarded or retained only as derived
non-authoritative data and MUST NOT replace the exact source snapshot as artifact
authority.

**WGSL-ART-004 — Accepted.** Equal output bytes do not merge distinct source,
compilation-input, invocation, provenance, or artifact identities established by
the generic semantic model.

## 5. Source mapping

Because artifact bytes are unchanged, this profile has an identity source map.

**WGSL-MAP-001 — Accepted.** Artifact and source offsets use zero-based UTF-8 byte
offsets into the exact byte sequence. A range is half-open: `[start, end)`.

**WGSL-MAP-002 — Accepted.** For a source snapshot of byte length `N`, the
canonical mapping is total over `[0, N)`: every artifact byte offset `x` maps to
byte offset `x` in the same logical source-unit identity and source revision.

**WGSL-MAP-003 — Accepted.** The identity relation MAY be represented compactly;
an implementation need not materialize one mapping record per byte or range.
Representation does not change the mapping semantics.

**WGSL-MAP-004 — Accepted.** Line and column positions are derived diagnostic
views of the authoritative byte range. They MUST NOT replace byte offsets in the
portable mapping contract.

**WGSL-MAP-005 — Accepted.** Accepted artifacts for this profile have no generated
or unattributable WGSL ranges.

## 6. Initial validation realization

The profile and realization identities remain distinct as required by the generic
semantic model.

**WGSL-REAL-001 — Accepted.** The initial realization target is the Rust `naga`
crate version `30.0.1`.

**WGSL-REAL-002 — Accepted.** The production dependency surface for this
realization MUST disable default features and enable only `wgsl-in` unless later
accepted implementation evidence proves another Naga feature is required by this
profile.

**WGSL-REAL-003 — Accepted.** Naga's WGSL backend (`wgsl-out`) MUST NOT be enabled
or used to form this profile's canonical artifact.

**WGSL-REAL-004 — Accepted.** Naga parsing is realized through its WGSL frontend.
Semantic IR validation MUST use all Naga validation checks and a capability
configuration broad enough not to impose a target-device policy. The initial
configuration target is `ValidationFlags::all()` and `Capabilities::all()`, with
all shader stages and subgroup operation sets admitted for validator purposes.

**WGSL-REAL-005 — Accepted.** The broad Naga validation capability configuration
is not a claim that any GPU or RunenGPU context supports those capabilities. It
exists only to prevent the shader-source validator from becoming device
admission authority.

**WGSL-REAL-006 — Accepted.** The realization MUST apply `WGSL-LANG-002` through
`WGSL-LANG-004` independently of Naga accepting a wider syntax or extension
surface.

**WGSL-REAL-007 — Accepted.** Realization identity evidence MUST distinguish at
least the exact Naga version, enabled Cargo features, validation flags,
capability mask, subgroup-stage configuration, subgroup-operation configuration,
and any RunenShader profile-gate revision that can affect acceptance or
diagnostics.

**WGSL-REAL-008 — Accepted.** Updating Naga or any output-affecting realization
configuration requires issue-owned conformance evidence. It does not by itself
change `wgsl-exact-2026-08-17`; a profile identity changes only when accepted
frontend semantics change.

## 7. Outcome specialization

The ordinary outcome classes remain those in the generic semantic model.

**WGSL-OUT-001 — Accepted.** A source that violates the selected WGSL baseline or
an explicit RunenShader profile restriction is `Rejected`.

**WGSL-OUT-002 — Accepted.** Naga parse and semantic-validation diagnostics are
realization evidence used to implement `Rejected`. They do not become the
portable RunenShader error taxonomy.

**WGSL-OUT-003 — Accepted.** If a source is valid under the selected RunenShader
profile but the selected Naga realization cannot represent or validate that
construct, the invocation is `Unsupported`; the profile MUST NOT be silently
narrowed to match the realization.

**WGSL-OUT-004 — Accepted.** Operational inability to execute the selected
realization, resource exhaustion outside shader validity, or an isolated
realization failure is `Failed` and publishes no artifact.

**WGSL-OUT-005 — Accepted.** A RunenShader internal invariant defect retains the
special failure treatment in `RS-OUT-002`; it is not relabeled as a WGSL source
rejection or Naga failure.

## 8. Diagnostics and provenance

**WGSL-DIAG-001 — Accepted.** User-facing diagnostics MUST identify the logical
source-unit identity and source revision and SHOULD carry the most precise
half-open UTF-8 byte range available from the realization.

**WGSL-DIAG-002 — Accepted.** Naga error types, handles, IR node identities, and
formatted diagnostic strings are private realization details. Public contracts
MUST translate them into RunenShader-owned outcome and diagnostic evidence.

**WGSL-DIAG-003 — Accepted.** When the realization supplies only a less precise
location, RunenShader MUST represent that lower precision truthfully rather than
invent an exact range.

**WGSL-PROV-001 — Accepted.** Artifact provenance for this profile MUST identify
the exact source-unit/revision snapshot, compilation-input identity, frontend
profile identity, and realization/configuration identity used by the invocation.

## 9. Reproducibility and conformance

**WGSL-CONF-001 — Accepted.** Conformance MUST prove that repeated successful
invocations over the same closed input and realization produce byte-identical
canonical WGSL and semantically equivalent normative evidence.

**WGSL-CONF-002 — Accepted.** Conformance MUST prove byte preservation across
semantically irrelevant presentation differences, including comments,
whitespace, and line-ending choices that are themselves valid source bytes. The
artifact preserves those differences rather than normalizing them.

**WGSL-CONF-003 — Accepted.** Positive fixtures MUST include valid modules with and
without entry points and representative baseline language features used by the
support claim.

**WGSL-CONF-004 — Accepted.** Negative fixtures MUST independently cover lexical
or parse invalidity, type/semantic invalidity, and constructs or extensions that
the realization recognizes but this pinned profile does not admit.

**WGSL-CONF-005 — Accepted.** Every baseline extension admitted by the
implementation's profile gate MUST have conformance evidence for its accepted
form; every wider realization-only extension relevant to the gate MUST have
rejection evidence.

**WGSL-CONF-006 — Accepted.** Diagnostic fixtures MUST prove logical source
identity/revision attribution and byte-range mapping without exposing Naga IDs as
portable identity.

**WGSL-CONF-007 — Accepted.** Tests MUST prove that cache presence, invocation
ordering, unrelated host state, working directory, filesystem contents, and
environment variables cannot alter the profile result.

**WGSL-CONF-008 — Accepted.** The implementation MUST contain no production path
through Naga WGSL output generation for this profile.

## 10. Rust toolchain consequence

**WGSL-RUST-001 — Accepted.** This profile decision does not establish a
RunenShader repository MSRV.

**WGSL-RUST-002 — Accepted.** The implementation slice MUST add the exact selected
Naga dependency, resolve its complete dependency graph, inspect the selected
package metadata, and prove any proposed repository MSRV in CI before adding a
`rust-version` claim.

**WGSL-RUST-003 — Accepted.** Upstream README badges, workspace MSRV statements,
or metadata from a neighboring Naga release are evidence only and MUST NOT become
RunenShader MSRV authority without the resolved implementation dependency.

## 11. Disposition of other frontend candidates

**WGSL-NEXT-001 — Accepted.** WESL is the preferred next composition candidate,
not an R0 supported frontend. Future WESL work must pin an exact WESL edition and
compiler realization, compile only from RunenShader's closed logical inputs, and
use an in-memory/custom resolver rather than ambient filesystem, registry, or
network discovery unless those adapters are separately accepted.

**WGSL-NEXT-002 — Accepted.** A future WESL profile must explicitly select its
stable/experimental feature surface, source-mapping contract, conditional inputs,
module/package correspondence, mangling behavior, and output reproducibility.
Compiler defaults or a floating WESL edition are not acceptable profile identity.

**WGSL-NEXT-003 — Accepted.** Slang frontend support is **Deferred** while its
WebGPU target remains work in progress or until a concrete RunenShader consumer
need justifies a fresh investigation. Current deferral does not imply rejection
of Slang as a future tool.

**WGSL-NEXT-004 — Accepted.** A generic frontend plugin ABI remains **Deferred**.
The exact-WGSL profile MUST be implemented directly against the accepted semantic
contracts rather than through a speculative universal frontend abstraction.
