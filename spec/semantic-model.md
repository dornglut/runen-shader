# RunenShader semantic model

Status: **Accepted core semantic contract**.

This document normalizes the semantic source-to-artifact boundary. Concrete
frontend profiles and compiler realizations specialize this generic model through
separately accepted normative specifications under `spec/`.

## 1. Authority

**RS-AUTH-001 — Accepted.** RunenShader MUST be the sole authority for the
shader-source and shader-compilation semantics defined here.

**RS-AUTH-002 — Accepted.** Filesystem layout, compiler-private objects, compiler
IR, reflection models, cache entries, process-local handles, content digests, and
downstream GPU identities MUST NOT become semantic authority by coincidence.

**RS-AUTH-003 — Accepted.** External shader-language syntax and language semantics
remain owned by the corresponding external language/profile specification. A
RunenShader frontend profile states exactly which external semantics it accepts
and realizes; it MUST NOT silently redefine that language.

## 2. Logical identity

RunenShader distinguishes logical identity from physical representation.

**RS-ID-001 — Accepted.** A `ShaderPackageIdentity` identifies one logical shader
package namespace. Its identity MUST NOT be inferred from a directory path,
repository URL, package registry location, hash, or compiler object.

**RS-ID-002 — Accepted.** A `ShaderModuleIdentity` identifies one logical module
inside a package namespace. Module identity MUST be explicit and MUST NOT be
inferred from a source filename or compiler-private module handle.

**RS-ID-003 — Accepted.** A `ShaderSourceUnitIdentity` identifies one logical
source unit. Association between source units and modules MUST be explicit in the
compilation input; path containment is not an authority rule.

**RS-ID-004 — Accepted.** A `ShaderSourceRevision` is meaningful only together
with its source-unit identity. The same `(source unit, revision)` pair MUST always
denote the same exact source snapshot whenever it is reused.

**RS-ID-005 — Accepted.** Source revision identity is opaque semantic identity. It
MUST NOT inherently imply ordering, wall-clock time, freshness, filesystem
modification state, or content-hash identity.

**RS-ID-006 — Accepted.** A `ShaderCompilationInputIdentity` identifies one exact
closed semantic compilation input. Reusing that identity for a semantically
different input is invalid.

**RS-ID-007 — Accepted.** A `ShaderArtifactIdentity` identifies one RunenShader
artifact product. It is independent from downstream program-source identity and
MUST NOT be represented as a RunenGPU identity.

The concrete Rust representation, persistence encoding, and allocation strategy
for these identities are **Open**. Implementations MUST preserve the accepted
identity semantics regardless of representation.

## 3. Source snapshots

**RS-SRC-001 — Accepted.** A source snapshot contains a source-unit identity, a
source revision, and the exact UTF-8 source byte sequence associated with that
revision.

**RS-SRC-002 — Accepted.** RunenShader MUST NOT normalize Unicode, line endings,
byte-order marks, or other source bytes before establishing snapshot identity.
Any frontend-defined normalization or preprocessing occurs only as an explicit
compilation transform over an already identified source snapshot; it MUST NOT
change the exact bytes denoted by the source revision.

**RS-SRC-003 — Accepted.** Display names, diagnostic paths, repository locations,
and origin URLs MAY be retained as provenance, but they MUST NOT affect source
identity or resolution unless an accepted profile explicitly promotes a value to
semantic input.

**RS-SRC-004 — Accepted.** Two source snapshots with different source-unit or
revision identities are distinct semantic snapshots even when their source bytes
are equal.

## 4. Package, module, and source relationships

**RS-REL-001 — Accepted.** A compilation input MUST explicitly identify its root
module or equivalent profile-defined root selection.

**RS-REL-002 — Accepted.** The compilation input MUST explicitly enumerate the
finite module and source snapshots available to that compilation.

**RS-REL-003 — Accepted.** The relationship between a module and its participating
source snapshots MUST be represented by the input contract, not discovered by
ambient directory traversal.

**RS-REL-004 — Accepted.** A frontend profile MAY permit one source unit to
participate in more than one module or one module to be formed from multiple
source units, but such composition MUST be explicit and deterministic.

**RS-REL-005 — Accepted.** Package/module/source identity correspondence MUST be
validated before compilation. Ambiguous or contradictory relationships are
invalid input rather than implementation-defined behavior.

## 5. Frontend profile and compiler realization

RunenShader separates semantic frontend support from concrete compiler
realization.

**RS-FRONT-001 — Accepted.** A `ShaderFrontendProfile` identifies the exact
language/profile semantics RunenShader claims to accept, including any
RunenShader-owned restrictions required for deterministic realization.

**RS-FRONT-002 — Accepted.** A `ShaderCompilerRealization` identifies one concrete
compiler implementation/build and the realization contract used to implement a
frontend profile.

**RS-FRONT-003 — Accepted.** Frontend-profile identity and compiler-realization
identity MUST remain distinct. Replacing a compiler realization does not by
itself create a new language profile, and supporting the same external language
name does not prove profile equivalence.

**RS-FRONT-004 — Accepted.** Every output-affecting compiler option, extension,
feature toggle, target choice, optimization choice, compatibility mode, or
realization configuration that can change the formed artifact MUST be explicit
in the compilation invocation evidence.

**RS-FRONT-005 — Accepted.** Compiler defaults MAY be used only when the selected
realization contract fixes those defaults sufficiently that the same identified
realization/configuration has one reproducible meaning. Environment-dependent or
host-dependent defaults MUST NOT silently affect artifact semantics.

The first concrete frontend profile and initial compiler-realization contract are
accepted by [the exact WGSL frontend profile](frontend-wgsl.md). That accepted
selection specializes this model; it does not by itself establish shipped
implementation or conformance support.

## 6. Closed compilation input

A compilation input is semantic data; a compilation invocation combines that
input with an explicit realization.

**RS-IN-001 — Accepted.** `ShaderCompilationInput` MUST be closed: it contains all
semantic source, dependency, frontend-profile, root-selection, and
output-contract choices required to determine the requested shader product.

**RS-IN-002 — Accepted.** Compilation semantics MUST NOT depend on ambient working
directory, unlisted filesystem contents, process environment, registry state,
network state, process-global mutable configuration, cache contents, wall-clock
time, or implicit "latest" dependency selection.

**RS-IN-003 — Accepted.** `ShaderCompilationInvocation` MUST combine one exact
compilation input with one identified compiler realization and all
output-affecting realization configuration.

**RS-IN-004 — Accepted.** If an implementation cannot make a compiler dependency
closed because the compiler performs uncontrolled ambient discovery, that
realization is non-conforming for the affected profile until the discovery is
intercepted, disabled, or otherwise made explicit.

**RS-IN-005 — Accepted.** Input construction MAY be assisted by filesystem,
registry, editor, build-system, or network adapters, but those adapters produce a
candidate closed input. Their mutable external state is not consulted after the
input is admitted for compilation.

## 7. Dependency resolution

**RS-RES-001 — Accepted.** Every compilation-relevant dependency request MUST
resolve through an explicit finite relation contained in or referenced by the
closed input.

**RS-RES-002 — Accepted.** A resolved dependency target MUST identify the exact
module and exact source revisions that participate in the target for that
compilation.

**RS-RES-003 — Accepted.** Resolution MUST NOT silently fall back to the host
filesystem, package registry, network, current process directory, environment
variables, or compiler search paths.

**RS-RES-004 — Accepted.** Missing, ambiguous, contradictory, or profile-invalid
resolution is a rejected semantic input. A compiler realization lacking coverage
for an otherwise valid accepted profile is unsupported realization coverage.

**RS-RES-005 — Accepted.** Dependency cycles, visibility, import/export meaning,
and other language-specific dependency rules are interpreted according to the
selected frontend profile. Private resolver structures do not become portable
RunenShader semantics.

A package-registry protocol and network acquisition contract are **Deferred**.
They may later produce closed inputs but are not part of compilation authority.

## 8. Canonical artifact

The initial canonical RunenShader program product is exact WGSL plus
RunenShader-owned evidence.

**RS-ART-001 — Accepted.** An accepted artifact MUST contain the exact UTF-8 WGSL
byte sequence selected as the canonical program product for that invocation.

**RS-ART-002 — Accepted.** `canonical` means the exact chosen RunenShader product,
not a claim that all semantically equivalent WGSL programs have one universal
normal form.

**RS-ART-003 — Accepted.** An accepted artifact MUST carry enough normative
evidence to identify its artifact identity, compilation-input identity, frontend
profile, compiler realization/configuration, and source provenance.

**RS-ART-004 — Accepted.** Artifact publication is atomic. A failed, rejected,
unsupported, cancelled, or internally defective compilation MUST NOT publish a
partially accepted canonical artifact.

**RS-ART-005 — Accepted.** Compiler reflection, intermediate representation, or
backend objects MAY be retained as derived implementation data, but they are not
part of the canonical artifact unless later accepted specification work promotes
a concrete field.

**RS-ART-006 — Accepted.** A content digest MAY accelerate comparison, caching, or
diagnostics, but full contract equality remains authoritative unless a future
accepted digest contract explicitly states otherwise.

A durable serialized artifact format and stable digest algorithm are **Deferred**.
No persistence compatibility is implied by the bootstrap model.

## 9. Provenance and source mapping

**RS-PROV-001 — Accepted.** RunenShader owns provenance that explains which exact
source snapshots, frontend profile, realization, and configuration formed an
artifact.

**RS-PROV-002 — Accepted.** Provenance references logical source identities and
revisions. Human-readable paths or labels MAY accompany them only as diagnostic
metadata.

**RS-MAP-001 — Accepted.** Any source mapping published by RunenShader MUST map
canonical artifact regions to logical source-unit/revision regions or explicitly
classify regions as generated/unattributable.

**RS-MAP-002 — Accepted.** Mapping coverage MUST be explicit. Partial or
unavailable mapping MUST NOT be represented as complete mapping.

**RS-MAP-003 — Accepted.** The concrete source-map encoding is **Open**. A
frontend cannot claim mapping support until its accepted conformance evidence
proves the published coverage semantics.

## 10. Diagnostics

**RS-DIAG-001 — Accepted.** Source-facing diagnostics MUST use RunenShader logical
subjects where the semantic contract knows them. Compiler-native paths, handles,
or opaque IDs MAY appear only as supplemental realization evidence.

**RS-DIAG-002 — Accepted.** Diagnostic ordering MUST be deterministic whenever it
is observable as part of a reproducible semantic outcome.

**RS-DIAG-003 — Accepted.** Diagnostics SHOULD include a stable subject, outcome
class, useful context, and actionable correction when the boundary has enough
information to provide one.

The concrete public diagnostic code vocabulary is **Open** and requires public
API work before stabilization.

## 11. Reproducibility

**RS-REP-001 — Accepted.** For the same exact compilation input, frontend profile,
compiler realization/build, and output-affecting realization configuration, a
successful conforming invocation MUST produce byte-identical canonical WGSL and
semantically equivalent normative artifact evidence independent of scheduling,
cache state, storage enumeration order, or unrelated host state.

**RS-REP-002 — Accepted.** Rejected and unsupported semantic outcomes SHOULD be
deterministic for the same closed invocation. Observable diagnostic ordering
MUST follow the deterministic diagnostic rule.

**RS-REP-003 — Accepted.** Operational failure such as process launch failure,
resource exhaustion, interrupted execution, or compiler crash does not become a
deterministic shader semantic result. Such failure publishes no artifact and
remains distinguishable from rejection or unsupported coverage.

**RS-REP-004 — Accepted.** A realization whose output depends on unrecorded host,
environment, filesystem, network, random, time, or process state is
non-conforming for reproducible artifact formation.

Reproducibility across different compiler realizations is **Open** and MUST NOT
be assumed merely because both realizations implement the same frontend profile.

## 12. Outcome model

A compilation invocation has exactly one ordinary public outcome class:

- `Accepted` — one complete canonical artifact was formed;
- `Rejected` — the explicit input is invalid under accepted RunenShader/profile
  semantics;
- `Unsupported` — the semantic request is valid but the selected realization
  does not provide accepted coverage for it;
- `Failed` — tool/execution failure prevented a semantic result.

**RS-OUT-001 — Accepted.** These classes MUST remain distinguishable at the public
boundary. A caller must not receive success-shaped data for rejected,
unsupported, or failed work.

**RS-OUT-002 — Accepted.** Internal RunenShader invariant violation is an
implementation defect, not an ordinary source-compilation outcome. It MUST fail
closed, publish no artifact, and MUST NOT be relabeled as user rejection,
unsupported coverage, or ordinary tool failure.

**RS-OUT-003 — Accepted.** The exact Rust mechanism used to expose an internal
defect is **Open** until the public API contract is accepted.

## 13. Cache and persisted derivatives

**RS-CACHE-001 — Accepted.** Caches, indexes, compiler IR snapshots, reflection
records, generated dependency indexes, and persisted derivative products are
non-authoritative unless future accepted work explicitly promotes a durable
format.

**RS-CACHE-002 — Accepted.** A cache hit MAY avoid recomputation only when it is
proven equivalent to the complete invocation required by the accepted contract.
A stale, partial, or ambiguous cache entry MUST fail closed to recomputation or
rejection rather than publish mismatched authority.

**RS-CACHE-003 — Accepted.** Cache absence, eviction, insertion order, or storage
location MUST NOT change semantic compilation results.

## 14. Downstream boundary

**RS-DOWN-001 — Accepted.** RunenShader canonical WGSL is a product that a
consumer may submit to another authority. RunenShader MUST NOT own RunenGPU
adapter/device/resource/work/submission semantics or GPU program-interface,
binding, layout, pipeline, or execution admission.

**RS-DOWN-002 — Accepted.** A consumer integrating RunenShader with RunenGPU MUST
establish a separately owned RunenGPU source identity/admission operation. A
RunenShader artifact identity or source revision MUST NOT be reused as if it were
a RunenGPU identity.

**RS-DOWN-003 — Accepted.** RunenShader reflection or frontend metadata MUST NOT be
presented as a second authority for downstream GPU interface facts. A downstream
authority may independently derive or validate facts from the exact canonical
WGSL according to its own contract.

**RS-DOWN-004 — Accepted.** Renderer materials, lighting, visibility, frame
composition, application recovery, last-known-good behavior, filesystem
watching, and hot-reload activation remain outside RunenShader semantic
authority.

## 15. Product decision state

The first concrete frontend profile and initial compiler-realization contract are
**Accepted** by [the exact WGSL frontend profile](frontend-wgsl.md). Their
acceptance selects semantics and realization requirements; implementation and
conformance remain separately required before support is claimed.

The following are **Open** unless marked otherwise. Their unresolved state does
not authorize arbitrary incompatible implementation:

- public Rust API/type representation of the accepted semantic concepts;
- concrete source-map representation and stable diagnostic-code vocabulary;
- artifact identity encoding and any digest algorithm;
- cross-realization equivalence claims.

The following are **Deferred**:

- durable serialized artifact/cache compatibility;
- package-registry protocol and network dependency acquisition;
- filesystem watching and hot-reload activation;
- last-known-good/application recovery policy;
- generic frontend-plugin ABI.

Any of these becomes supported only through later accepted specification,
implementation, and conformance work.
