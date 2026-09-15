# WESL composition frontend profile

Status: **Accepted R1 frontend-profile contract**.

This profile specializes the [semantic model](semantic-model.md) for deterministic
multi-module WESL composition into canonical WGSL. It accepts WESL language
semantics only through an explicit pinned external revision and keeps WESL module
paths, compiler internals, and host storage outside RunenShader identity.

This profile does not select a concrete WESL compiler realization. A later
realization must prove its own supported coverage against this contract.

## 1. Profile identity and external language baselines

**WESL-PROFILE-001 — Accepted.** This frontend profile is identified semantically
as `wesl-composition-2026-08-22`.

**WESL-PROFILE-002 — Accepted.** `wesl-composition-2026-08-22` incorporates the
WESL import and conditional-translation semantics from the
`webgpu-tools/wesl-spec` repository at Git revision
`78d932876ae17e0a41e63fa17ea66804725625b0`, dated 22 August 2026, including
the subordinate visibility, module-path, and other definitions required to give
those two enhancements their meaning.

**WESL-PROFILE-003 — Accepted.** The WGSL language embedded in this WESL profile
uses the same W3C WGSL Candidate Recommendation Draft dated 17 August 2026 that
is selected by `WGSL-PROFILE-002`. This imports the pinned WGSL language
baseline, not the exact-WGSL profile's one-source, byte-identity artifact,
identity-map, or Naga-realization specializations.

**WESL-PROFILE-004 — Accepted.** The profile MUST NOT float with a WESL `latest`,
an unpinned WESL edition, compiler defaults, package-manager state, an
editor's-draft WGSL interpretation, or a concrete compiler release. Changing
either external language baseline requires accepted profile-evolution work.

**WESL-PROFILE-005 — Accepted.** RunenShader does not redefine WESL or WGSL
syntax, typing, visibility, import, conditional-translation, or execution
semantics. This file owns only the RunenShader profile selection, restrictions,
closed-input mapping, artifact boundary, and realization/conformance contract.

## 2. Accepted WESL language surface

**WESL-LANG-001 — Accepted.** The newly accepted WESL enhancement surface is
limited to imports and conditional translation from the pinned WESL revision.
WGSL syntax and constructs otherwise remain inside the pinned WGSL baseline
selected by `WESL-PROFILE-003`.

**WESL-LANG-002 — Accepted.** The import surface includes the pinned WESL
semantics required for module paths, item/module imports, aliases, collections,
wildcards, public imports and visibility, relative `super`/`package` addressing,
package-root addressing, cycles, and module attributes that are part of the
pinned import contract. RunenShader restrictions on what modules and packages
may actually resolve are defined separately in this profile.

**WESL-LANG-003 — Accepted.** Conditional translation includes the pinned
`@if`, `@elif`, and `@else` semantics and translate-time boolean feature
expressions. One ordinary RunenShader invocation performs complete translation;
incremental or partially translated WESL is not an accepted artifact.

**WESL-LANG-004 — Accepted.** Experimental WESL generics, execution/evaluation
extensions, experimental lowering or polyfill semantics, implementation-specific
Naga or WGPU extensions, and any WESL enhancement outside
`WESL-LANG-001` through `WESL-LANG-003` are outside this profile.

**WESL-LANG-005 — Accepted.** Stripping, dead-declaration elimination, lazy
module loading, and related reachability strategies are not additional authored
language features in this profile. A realization MAY use them only as
deterministic output-affecting implementation strategies fixed by realization
identity/configuration, and only when conformance proves that they preserve the
pinned WESL semantics, including the language's treatment of unused or
unresolved imports. Compiler defaults do not become profile authority.

**WESL-LANG-006 — Accepted.** Package publication, package-manager dependency
selection, registry resolution, and network acquisition are not accepted
language-input mechanisms for this profile. Their absence does not change the
pinned WESL syntax; it restricts which WESL package references can resolve in one
RunenShader invocation.

## 3. Closed package, module, and source input

The generic RunenShader package/module/source identities remain authoritative.
WESL module paths are resolution keys layered on those identities.

**WESL-IN-001 — Accepted.** One invocation contains exactly one
`ShaderPackageIdentity`, one explicit compilation root `ShaderModuleIdentity`, a
finite explicit set of participating logical modules, and exact immutable UTF-8
source-unit/revision snapshots for those modules.

**WESL-IN-002 — Accepted.** Each admitted WESL module path MUST resolve to
exactly one admitted logical module and exactly one complete source snapshot for
that module. One source snapshot MUST NOT ambiguously provide two logical WESL
modules within the same invocation.

**WESL-IN-003 — Accepted.** The compilation root and the WESL package-root module
are distinct concepts. The compilation root is always explicit RunenShader input.
The WESL `package` root module is present only when the admitted WESL
module-path mapping explicitly provides it; neither is inferred from a filename,
directory, or package manifest.

**WESL-IN-004 — Accepted.** The exact bytes of every participating
`ShaderSourceSnapshot` are presented to WESL translation without Unicode,
line-ending, byte-order-mark, whitespace, comment, spelling, or formatting
normalization by RunenShader.

**WESL-IN-005 — Accepted.** WESL module paths, import spellings, aliases,
package-relative paths, and display names are profile-specific resolution facts.
They MUST NOT become or determine `ShaderModuleIdentity`,
`ShaderSourceUnitIdentity`, `ShaderSourceRevision`, filesystem-path identity, or
compiler-private identity.

**WESL-IN-006 — Accepted.** This initial profile admits only references that
resolve within the single explicit RunenShader package. A path anchored at any
external WESL package name is invalid under this profile even if the pinned WESL
language or a future compiler can resolve it.

**WESL-IN-007 — Accepted.** The admitted WESL module-path mapping MUST be finite,
explicit, and unambiguous. Duplicate keys, contradictory module/source bindings,
or resolution to a source revision absent from the closed input are `Rejected`.

**WESL-IN-008 — Accepted.** Collection enumeration order for modules, source
snapshots, module-path bindings, and conditional features has no semantic meaning
unless the selected external WESL semantics explicitly assign meaning to that
order. Incidental container or hash iteration order MUST NOT affect the result.

## 4. Closed WESL resolution

**WESL-RES-001 — Accepted.** Import and declaration-path resolution follows the
pinned WESL semantics, but every module lookup MUST be satisfied only by the
explicit admitted WESL module-path mapping.

**WESL-RES-002 — Accepted.** After input admission, the realization MUST NOT
consult ambient filesystem contents, working directory, environment variables,
`wesl.toml`, mutable package-manager state, Cargo/npm registries, network state,
implicit latest versions, or compiler-global resolver/search-path state.

**WESL-RES-003 — Accepted.** Filesystem, editor, build-system, or package-manager
adapters MAY construct candidate closed inputs before admission. After admission,
the resulting explicit RunenShader identities, source snapshots, module-path
bindings, root, and profile configuration are authoritative for the invocation;
the adapter remains non-authoritative.

**WESL-RES-004 — Accepted.** Cycles, visibility, re-exports, wildcard behavior,
and declaration resolution are accepted or rejected according to the pinned WESL
language semantics. RunenShader MUST NOT impose an independent import-language
meaning merely to simplify a realization.

## 5. Conditional-translation input

**WESL-COND-001 — Accepted.** One invocation carries an explicit finite mapping
from every supplied translate-time feature name to one boolean value. The
complete mapping is semantic compilation input.

**WESL-COND-002 — Accepted.** Every translate-time feature referenced by source
that survives to the complete conditional-translation phase MUST have an
explicit admitted value. A referenced feature with no admitted value is
`Rejected`; RunenShader MUST NOT invent a default enabled or disabled value.

**WESL-COND-003 — Accepted.** Explicit values for unused feature names MAY be
present, matching the pinned WESL semantics. They remain part of the exact closed
input and therefore participate in compilation-input evidence even when they do
not change artifact bytes.

**WESL-COND-004 — Accepted.** A RunenShader invocation MUST complete conditional
translation before canonical artifact formation. An output that still contains
an unresolved translate-time WESL attribute is not canonical WGSL and MUST NOT be
`Accepted`.

## 6. Canonical WGSL artifact

**WESL-ART-001 — Accepted.** Successful translation produces canonical UTF-8
WGSL as a generated RunenShader artifact. Artifact bytes are not required to
equal any one participating source snapshot.

**WESL-ART-002 — Accepted.** Final generated WGSL MUST conform to the external
WGSL language baseline selected by `WESL-PROFILE-003` and MUST NOT contain a
WGSL extension or construct outside that pinned baseline merely because a WESL
compiler or downstream implementation accepts it.

**WESL-ART-003 — Accepted.** The one-source and byte-identity rules
`WGSL-IN-001` through `WGSL-IN-003`, `WGSL-ART-001` through `WGSL-ART-003`, and
`WGSL-MAP-001` through `WGSL-MAP-005` are exact-WGSL profile specializations and
do not apply to this transformed WESL profile.

**WESL-ART-004 — Accepted.** WESL translation necessarily performs the
name-resolution and rewriting required by the pinned language. Optimization,
minification, formatter-only rewriting, experimental lowering, or another
non-required transformation MUST NOT silently enter profile semantics. A
semantics-preserving reachability/stripping strategy permitted by
`WESL-LANG-005` remains realization configuration and must be deterministic and
conformance-proven.

**WESL-ART-005 — Accepted.** The generic canonical-artifact rule remains
authoritative: canonical WGSL need not be a universal normal form. Any
output-affecting realization choice not fixed by the external WESL semantics,
including a concrete name-mangling strategy or deterministic text-emission
strategy, MUST be fixed by realization identity/configuration and conformance.

**WESL-ART-006 — Accepted.** Equal generated WGSL bytes do not merge distinct
source, compilation-input, invocation, provenance, profile, realization, or
artifact identities established by the semantic model.

## 7. Profile versus compiler realization

**WESL-REAL-001 — Accepted.** No concrete WESL compiler realization is accepted
by this profile decision. A WESL compiler crate, executable, parser, linker, AST,
IR, resolver, source-map object, or mangler remains implementation evidence.

**WESL-REAL-002 — Accepted.** A future realization MUST identify every
output-affecting implementation choice, including at least compiler/tool version,
enabled dependency features, WESL feature toggles, import/conditional-translation
configuration, stripping/lazy/lowering settings, validation strategy, resolver
mode, name-mangling strategy, and deterministic output-emission configuration
where applicable.

**WESL-REAL-003 — Accepted.** A future realization MUST provide a strict
in-memory or otherwise closed resolver that can resolve only the module-path
bindings admitted by `WESL-IN-007`. A realization's filesystem or package
resolver MUST NOT be used as ambient fallback.

**WESL-REAL-004 — Accepted.** A future realization MUST independently prove that
its generated output satisfies `WESL-ART-002`; acceptance by the WESL compiler
alone is not proof that the output lies inside RunenShader's pinned WGSL language
envelope.

**WESL-REAL-005 — Accepted.** If this accepted WESL profile admits a valid
request that a selected realization cannot parse, translate, map, or validate
within its demonstrated coverage, the invocation is `Unsupported`. Realization
limitations MUST NOT silently narrow this profile.

**WESL-REAL-006 — Accepted.** Updating a compiler/tool version or another
output-affecting realization choice requires issue-owned conformance evidence. It
does not by itself change `wesl-composition-2026-08-22`.

## 8. Diagnostics, provenance, and source mapping

**WESL-DIAG-001 — Accepted.** User-facing diagnostics MUST identify the logical
source-unit identity and source revision whenever the failing WESL source
location is truthfully known and SHOULD carry the most precise truthful
zero-based half-open UTF-8 byte range available.

**WESL-DIAG-002 — Accepted.** WESL module paths, filesystem/display names,
mangled identifiers, compiler-native diagnostics, AST/IR identities, and
realization source-map objects are supplemental evidence only. Public contracts
MUST translate them into RunenShader-owned logical subjects and outcomes.

**WESL-DIAG-003 — Accepted.** When the realization proves only a module/source
subject or a less precise location, RunenShader MUST represent that lower
precision truthfully rather than invent an exact byte range.

**WESL-PROV-001 — Accepted.** Artifact provenance MUST identify the complete
closed compilation-input identity, frontend profile, realization/configuration,
root module, module/source relationships, and every participating logical
source-unit/revision snapshot.

**WESL-MAP-001 — Accepted.** Artifact mapping uses the generic semantic-model
contract: a canonical WGSL artifact range MAY map to a logical
source-unit/revision byte range when that relation is proven, or be classified
explicitly as generated or unattributable when it is not.

**WESL-MAP-002 — Accepted.** Mapping coverage MUST be explicit. A realization
MUST NOT present declaration-name/module-origin evidence as byte-precise artifact
mapping unless that byte relation is actually established.

**WESL-MAP-003 — Accepted.** A conforming first realization MAY conservatively
classify transformed output regions as generated or unattributable while still
providing complete participating-source provenance. More precise source mapping
may be added only when truthful evidence supports it.

## 9. Outcomes and failure boundary

The ordinary outcome classes remain those in the semantic model.

**WESL-OUT-001 — Accepted.** Invalid WESL syntax or semantics, a WGSL construct
outside the pinned underlying language baseline, an import invalid under the
pinned language, an external-package reference forbidden by this profile,
ambiguous/contradictory closed resolution, or a missing referenced conditional
feature is `Rejected`.

**WESL-OUT-002 — Accepted.** Generated WGSL outside the pinned WGSL language
envelope MUST NOT be published as `Accepted`. Its ordinary classification follows
the semantic cause: explicit invalid source/profile input is `Rejected`; a
profile-valid request that the selected realization cannot translate within its
proven coverage is `Unsupported`; a contradiction in an accepted realization
contract fails closed as an invariant defect.

**WESL-OUT-003 — Accepted.** A request valid under this profile but outside the
selected realization's proven feature, directive, translation, diagnostic, or
mapping coverage is `Unsupported`.

**WESL-OUT-004 — Accepted.** Operational inability to execute the selected
realization or an isolated tool failure that is not shader/profile invalidity is
`Failed` and publishes no artifact.

**WESL-OUT-005 — Accepted.** A RunenShader invariant defect retains the special
failure treatment in `RS-OUT-002`; it MUST NOT be relabeled as WESL rejection,
unsupported coverage, or ordinary compiler failure.

## 10. Reproducibility and conformance

**WESL-CONF-001 — Accepted.** Conformance for a future realization MUST prove
that repeated successful invocations over the same exact closed input and
realization produce byte-identical canonical WGSL and semantically equivalent
RunenShader evidence.

**WESL-CONF-002 — Accepted.** Positive conformance MUST cover at least:
single-module WESL, multi-module relative imports, package-root-relative imports,
aliases/collections, a cycle that is valid under pinned WESL semantics,
conditional imports or declarations where accepted by the pinned combination,
and explicit true/false feature selections.

**WESL-CONF-003 — Accepted.** Negative conformance MUST independently cover at
least unresolved imports, forbidden external-package references, ambiguous closed
resolution, missing referenced conditional features, invalid WESL syntax or
semantics, and generated WGSL outside the pinned WGSL language envelope.

**WESL-CONF-004 — Accepted.** Conformance MUST prove that module/source storage
enumeration order, resolver-map iteration order, cache state, invocation order,
working directory, unrelated filesystem contents, environment variables, and
network availability cannot change the semantic result.

**WESL-CONF-005 — Accepted.** Provenance fixtures MUST prove every participating
logical source revision remains represented even when generated artifact mapping
for some or all output regions is only generated/unattributable.

**WESL-CONF-006 — Accepted.** Diagnostic fixtures MUST prove logical
source-unit/revision attribution without promoting WESL paths, filenames, or
compiler IDs to portable identity.

**WESL-CONF-007 — Accepted.** The realization MUST prove its exact
stripping/reachability/lazy-resolution strategy and that it preserves the pinned
WESL semantics; it MUST also prove that experimental lowering, generic expansion
outside this profile, ambient package resolution, and unresolved incremental
conditional translation are absent from the accepted production path.

## 11. Deferred extensions

**WESL-NEXT-001 — Deferred.** External WESL package dependencies, package
publication, package manifests, registry/network acquisition, and package-manager
integration require separate accepted work.

**WESL-NEXT-002 — Deferred.** WESL generics, execution/evaluation extensions,
experimental lowering/polyfills, and other authored WESL enhancements outside
the selected pinned surface require separate profile-evolution work. A
semantics-preserving stripping/reachability strategy remains realization
configuration under `WESL-LANG-005`, not a newly accepted authored language
feature.

**WESL-NEXT-003 — Deferred.** A public generic frontend plugin/provider ABI is
not introduced by this profile.

**WESL-NEXT-004 — Deferred.** Node-graph/editor-native authored identities and a
custom RunenShader language are not introduced by this profile. Future authored
representations may feed explicit textual source snapshots through adapters until
concrete evidence proves a broader RunenShader semantic origin model is required.
