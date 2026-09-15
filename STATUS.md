# Status

RunenShader is an active `rust-framework` repository in R0 semantic-kernel work.
This document records durable capability maturity only. GitHub owns live issue,
pull-request, workflow, and priority state.

## Current capability

| Capability | State |
| --- | --- |
| repository identity and licensing authority | established |
| normative source/toolchain semantic model | established under `spec/` |
| first frontend profile contract | exact WGSL profile selected; implementation and conformance not yet accepted |
| repository-owned merge-readiness validation | established |
| public shader-compilation Rust API | not implemented |
| concrete frontend implementation | none accepted |
| canonical WGSL artifact implementation | not implemented |
| WESL composition support | not accepted; next composition candidate |
| Slang support | deferred pending stronger WebGPU/WGSL maturity or demonstrated need |
| persistent artifact/cache format | none accepted |
| RunenGPU integration | not implemented; intentionally downstream-owned |
| renderer/application integration | not implemented; outside core authority |

## Maturity constraints

The semantic specification and exact WGSL profile define what future
implementations must preserve; they do not themselves prove a compiler path.
Selecting a profile or realization contract is not the same as shipping frontend
support.

RunenShader does not support WESL, Slang, a generic frontend plugin mechanism,
filesystem discovery, package-registry resolution, source watching, hot reload,
or downstream GPU admission merely because those concerns have been evaluated.

RunenShader currently makes no repository MSRV claim. The first implementation
slice must resolve the selected dependency set and prove any claimed minimum Rust
version before that claim becomes repository authority.

## Acceptance model

A capability moves from absent to supported only when repository-local contracts,
implementation, conformance evidence, and exact-head validation agree. Derived
caches, compiler-private structures, paths, and downstream GPU identities do not
become RunenShader authority through use.
