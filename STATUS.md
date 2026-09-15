# Status

RunenShader is an active `rust-framework` repository. The R0 semantic-kernel exit
properties are achieved through the accepted exact-WGSL production path and its
repository-owned conformance evidence. R1 has accepted and realized its first
additional frontend contract for closed WESL composition through a pinned
`wesl-rs` 0.5.0 realization. This document records durable capability maturity
only. GitHub owns live issue, pull-request, workflow, and priority state.

## Current capability

| Capability | State |
| --- | --- |
| repository identity and licensing authority | established |
| normative source/toolchain semantic model | established under `spec/` |
| public semantic data kernel | implemented for identities, exact snapshots, closed exact-WGSL inputs, closed WESL multi-source input identity/evidence, artifact provenance, generic artifact mapping classifications, diagnostics, and outcome types |
| exact WGSL frontend profile | selected and realized through the pinned Naga 30.0.1 path |
| WESL composition frontend profile | accepted for the pinned 22 Aug 2026 import/visibility + conditional-translation surface and realized by `wesl-rs` 0.5.0 V1 over closed in-memory inputs |
| repository-owned merge-readiness validation | established |
| public shader-compilation entry point | stateful `ShaderCompiler` implemented for exact WGSL and the accepted WESL composition realization |
| concrete frontend implementation | Naga 30.0.1 exact-WGSL parser/validator plus `wesl-rs` 0.5.0 closed composition; each remains behind RunenShader-owned profile and outcome translation |
| canonical WGSL artifact formation by a compiler | implemented for exact WGSL with byte-preserving identity mapping and for WESL with generated canonical WGSL plus conservative total unattributable mapping |
| Slang support | architecture-targeted but production-deferred while WGSL/WebGPU realization evidence remains immature |
| generic frontend plugin/provider mechanism | deferred; accepted profiles remain repository-owned |
| persistent artifact/cache format | none accepted |
| RunenGPU integration | not implemented; intentionally downstream-owned |
| renderer/application integration | not implemented; outside core authority |

## Maturity constraints

The semantic specification, exact-WGSL profile, and WESL composition profile
define what their respective compiler paths must preserve. The exact-WGSL
realization proves ordinary valid WGSL modules with and without entry points,
exact source-byte preservation, Naga parse and semantic rejection, private gate
reconciliation, source-revision binding, host-state reproducibility, logical
diagnostics, and representative constructs for every Gate V1 `Continue`
extension. It does not claim device or downstream pipeline support merely
because Naga source validation uses broad capabilities.

The WESL realization is pinned to `wesl-rs` 0.5.0 with optional crate features
disabled and a fixed `CompileOptions` contract. It resolves only the explicit
closed single-package in-memory module table, completes admitted conditional
translation from explicit feature values, and independently readmits generated
WGSL through the existing RunenShader WGSL gate plus Naga 30.0.1 validation.
Ambient filesystem contents, the process working directory, environment state,
`wesl.toml`, registries, network lookup, and package dependencies are not
realization authority. Every admitted source remains in artifact provenance even
when WESL static usage analysis does not load that module.

The first WESL realization is deliberately bounded. Conditional attributes on
imports are rejected. Profile-valid global WGSL directive composition is
`Unsupported` while upstream `wesl-rs` directive behavior lacks the evidence
required by the accepted contract; profile-invalid directives remain rejected.
Transformed WESL artifact bytes are conservatively classified as
`Unattributable`; the public mapping model can also represent exact-source and
generated ranges, but the V1 realization does not claim byte-precise transformed
source mapping without proof.

RunenShader does not support Slang, a generic frontend plugin mechanism,
filesystem discovery, package-registry resolution, source watching, hot reload,
or downstream GPU admission merely because those concerns have been evaluated.

RunenShader currently makes no repository MSRV claim. The concrete dependencies
are pinned and Cargo-resolved, but their individual `rust-version` declarations
do not by themselves establish a repository MSRV policy.

## Acceptance model

A capability moves from absent to implemented only when repository-local
contracts, implementation, conformance evidence, and exact-head validation agree.
A profile contract or semantic representation may be accepted before its compiler
realization; status must keep those states distinct. Derived caches,
compiler-private structures, paths, and downstream GPU identities do not become
RunenShader authority through use.
