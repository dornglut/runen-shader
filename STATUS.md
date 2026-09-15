# Status

RunenShader is an active `rust-framework` repository. The R0 semantic-kernel exit
properties are achieved through the accepted exact-WGSL production path and its
repository-owned conformance evidence. R1 has accepted its first additional
frontend contract for closed WESL composition, but no WESL compiler realization
or public multi-source Rust input is implemented yet. This document records
durable capability maturity only. GitHub owns live issue, pull-request,
workflow, and priority state.

## Current capability

| Capability | State |
| --- | --- |
| repository identity and licensing authority | established |
| normative source/toolchain semantic model | established under `spec/` |
| public semantic data kernel | implemented for identities, exact snapshots, closed exact-WGSL inputs, artifact evidence, source mapping, diagnostics, and outcome types |
| exact WGSL frontend profile | selected and realized through the pinned Naga 30.0.1 path |
| WESL composition frontend profile | accepted for the pinned 22 Aug 2026 import/visibility + conditional-translation surface; no compiler realization yet |
| repository-owned merge-readiness validation | established |
| public shader-compilation entry point | stateful `ShaderCompiler` implemented for the exact-WGSL realization |
| concrete frontend implementation | Naga 30.0.1 WGSL parser/validator behind the private profile gate; tested coverage is limited to the demonstrated exact-WGSL surface |
| canonical WGSL artifact formation by a compiler | implemented for exact WGSL with byte-preserving source artifacts and identity source mapping; generated WESL artifact formation is not implemented |
| Slang support | architecture-targeted but production-deferred while WGSL/WebGPU realization evidence remains immature |
| generic frontend plugin/provider mechanism | deferred; accepted profiles remain repository-owned |
| persistent artifact/cache format | none accepted |
| RunenGPU integration | not implemented; intentionally downstream-owned |
| renderer/application integration | not implemented; outside core authority |

## Maturity constraints

The semantic specification, exact-WGSL profile, and WESL composition profile
define what their respective compiler paths must preserve. The current production
realization proves ordinary valid WGSL modules with and without entry points,
exact source-byte preservation, Naga parse and semantic rejection, private gate
reconciliation, source-revision binding, host-state reproducibility, logical
diagnostics, and representative constructs for every Gate V1 `Continue`
extension. It does not claim device or downstream pipeline support merely
because Naga source validation uses broad capabilities.

The accepted WESL profile pins a closed single-package multi-module contract for
import/visibility composition and complete conditional translation. It does not yet provide a Rust
multi-source input representation, a WESL compiler dependency, generated-WGSL
artifact production, or WESL conformance evidence. Accepting the profile is not
a claim that WESL source can already be compiled by the public crate.

RunenShader does not support Slang, a generic frontend plugin mechanism,
filesystem discovery, package-registry resolution, source watching, hot reload,
or downstream GPU admission merely because those concerns have been evaluated.

RunenShader currently makes no repository MSRV claim. The Naga dependency is
pinned and Cargo-resolved, but no minimum Rust version is promoted from that
evidence. Researching a future WESL compiler dependency does not establish an
MSRV either.

## Acceptance model

A capability moves from absent to implemented only when repository-local
contracts, implementation, conformance evidence, and exact-head validation agree.
A profile contract may be accepted before its realization; status must keep those
states distinct. Derived caches, compiler-private structures, paths, and
downstream GPU identities do not become RunenShader authority through use.
