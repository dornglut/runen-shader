# Status

RunenShader is an active `rust-framework` repository in R0 semantic-kernel work.
This document records durable capability maturity only. GitHub owns live issue,
pull-request, workflow, and priority state.

## Current capability

| Capability | State |
| --- | --- |
| repository identity and licensing authority | established |
| normative source/toolchain semantic model | established under `spec/` |
| dependency-free public semantic data kernel | implemented for identities, exact snapshots, closed exact-WGSL inputs, artifact evidence, source mapping, diagnostics, and outcome types |
| first frontend profile contract | exact WGSL profile selected; compiler implementation and conformance not yet accepted |
| repository-owned merge-readiness validation | established |
| public shader-compilation entry point | not implemented |
| concrete frontend implementation | none accepted |
| canonical WGSL artifact formation by a compiler | not implemented |
| WESL composition support | not accepted; next composition candidate |
| Slang support | deferred pending stronger WebGPU/WGSL maturity or demonstrated need |
| persistent artifact/cache format | none accepted |
| RunenGPU integration | not implemented; intentionally downstream-owned |
| renderer/application integration | not implemented; outside core authority |

## Maturity constraints

The semantic specification and exact WGSL profile define what compiler
implementations must preserve. The public semantic data kernel realizes those
non-compiler contracts, but selecting a profile/realization and exposing its data
model is not the same as shipping frontend support.

RunenShader does not support WESL, Slang, a generic frontend plugin mechanism,
filesystem discovery, package-registry resolution, source watching, hot reload,
or downstream GPU admission merely because those concerns have been evaluated.

RunenShader currently makes no repository MSRV claim. The first dependency-bearing
compiler slice must resolve the selected dependency set and prove any claimed
minimum Rust version before that claim becomes repository authority.

## Acceptance model

A capability moves from absent to supported only when repository-local contracts,
implementation, conformance evidence, and exact-head validation agree. Derived
caches, compiler-private structures, paths, and downstream GPU identities do not
become RunenShader authority through use.
