# Status

RunenShader is an active `rust-framework` repository in R0 semantic-kernel work.
This document records durable capability maturity only. GitHub owns live issue,
pull-request, workflow, and priority state.

## Current capability

| Capability | State |
| --- | --- |
| repository identity and licensing authority | established |
| normative source/toolchain semantic model | established under `spec/` |
| public semantic data kernel | implemented for identities, exact snapshots, closed exact-WGSL inputs, artifact evidence, source mapping, diagnostics, and outcome types |
| first frontend profile contract | exact WGSL profile selected and realized through the pinned Naga 30.0.1 path |
| repository-owned merge-readiness validation | established |
| public shader-compilation entry point | stateful `ShaderCompiler` implemented |
| concrete frontend implementation | Naga 30.0.1 WGSL parser/validator behind the private profile gate; tested coverage is limited to the demonstrated exact-WGSL surface |
| canonical WGSL artifact formation by a compiler | implemented with byte-preserving source artifacts and identity source mapping |
| WESL composition support | not accepted; next composition candidate |
| Slang support | deferred pending stronger WebGPU/WGSL maturity or demonstrated need |
| persistent artifact/cache format | none accepted |
| RunenGPU integration | not implemented; intentionally downstream-owned |
| renderer/application integration | not implemented; outside core authority |

## Maturity constraints

The semantic specification and exact WGSL profile define what the compiler path
must preserve. The current realization proves ordinary valid modules with and
without entry points, exact source-byte preservation, Naga parse and semantic
rejection, private gate reconciliation, source-revision binding, host-state
reproducibility, logical diagnostics, and representative constructs for every
Gate V1 `Continue` extension. It does not claim device or downstream pipeline
support merely because Naga source validation uses broad capabilities.

RunenShader does not support WESL, Slang, a generic frontend plugin mechanism,
filesystem discovery, package-registry resolution, source watching, hot reload,
or downstream GPU admission merely because those concerns have been evaluated.

RunenShader currently makes no repository MSRV claim. The Naga dependency is
pinned and Cargo-resolved, but no minimum Rust version is promoted from that
evidence.

## Acceptance model

A capability moves from absent to supported only when repository-local contracts,
implementation, conformance evidence, and exact-head validation agree. Derived
caches, compiler-private structures, paths, and downstream GPU identities do not
become RunenShader authority through use.
