# Status

RunenShader is an active `rust-framework` repository in semantic-kernel bootstrap.
This document records durable capability maturity only. GitHub owns live issue,
pull-request, workflow, and priority state.

## Current capability

| Capability | State |
| --- | --- |
| repository identity and licensing authority | established by the bootstrap contract |
| normative source/toolchain semantic model | established under `spec/` |
| repository-owned merge-readiness validation | established |
| public shader-compilation Rust API | not implemented |
| concrete frontend support | none accepted |
| canonical WGSL generation implementation | not implemented |
| persistent artifact/cache format | none accepted |
| RunenGPU integration | not implemented; intentionally downstream-owned |
| renderer/application integration | not implemented; outside core authority |

## Maturity constraints

The semantic specification defines what future implementations must preserve; it
does not itself prove a compiler path. No source language, compiler library,
frontend plugin mechanism, reflection model, filesystem resolver, hot-reload
policy, or package registry is supported merely because it is discussed as a
possible future realization.

RunenShader currently makes no MSRV claim. Stable Rust is used for repository
validation until concrete product evidence justifies a minimum supported version.

## Acceptance model

A capability moves from absent to supported only when repository-local contracts,
implementation, conformance evidence, and exact-head validation agree. Derived
caches, compiler-private structures, paths, and downstream GPU identities do not
become RunenShader authority through use.
