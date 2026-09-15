# R0B frontend evaluation — 2026-09-15

Status: non-normative dated evidence.

This report records the external evidence used to choose the first RunenShader
frontend profile. Normative RunenShader semantics live under `spec/`; external
tool documentation and this report do not override that authority.

## Question

What is the cheapest first frontend path that proves the accepted RunenShader
identity, validation, artifact, provenance, diagnostic, and reproducibility model
without allowing one compiler implementation or an unstable composition language
to define the semantic kernel?

## WGSL baseline

The W3C WebGPU Shading Language page currently publishes a Candidate
Recommendation Draft dated 17 August 2026:

- https://www.w3.org/TR/WGSL/
- https://www.w3.org/TR/2026/CRD-WGSL-20260817/

WGSL is already the canonical source product expected at the RunenShader to
RunenGPU boundary. Starting with exact WGSL therefore needs no composition,
translation, import resolver, or source re-emission merely to prove the semantic
kernel.

Decision consequence: pin the first profile to the dated 17 August 2026 external
baseline. Do not use the floating `latest` or editor's draft as semantic identity.

## Naga as first validation realization

Current docs.rs material identifies `naga` 30.0.1 as the current crate release and
lists WGSL input as a primary, fully validated frontend behind `wgsl-in`:

- https://docs.rs/crate/naga/30.0.1
- https://docs.rs/crate/naga/30.0.1/features
- https://docs.rs/naga/30.0.1/naga/front/wgsl/
- https://docs.rs/naga/30.0.1/naga/valid/

Naga's own example parses WGSL and then validates the resulting module with all
validation flags and broad validation capabilities. That is suitable as a
language-validation realization when RunenShader separately gates the pinned
WGSL profile and does not treat Naga capabilities as GPU-device capabilities:

- https://docs.rs/naga/30.0.1/src/naga/lib.rs.html

The crate has no default features and exposes `wgsl-in` independently from
`wgsl-out`. RunenShader therefore does not need WGSL re-emission for the exact
profile.

Naga also recognizes implementation/language extensions that are not guaranteed
in every environment. This is evidence that the RunenShader profile must not
silently equal "whatever Naga accepts":

- https://docs.rs/naga/30.0.1/naga/front/wgsl/

Decision consequence: use Naga 30.0.1 with the minimal `wgsl-in` dependency
surface as the initial validation realization, while keeping its IR, validator
capabilities, handles, reflection, and error taxonomy private.

## Why output remains the exact input bytes

For the first profile, parsing and validation are required, but source
transformation is not. Naga provides a separate WGSL output backend, which would
introduce formatting/re-emission behavior that RunenShader does not need to prove
R0:

- https://docs.rs/naga/30.0.1/naga/back/wgsl/

Keeping output byte-identical to the admitted source has stronger properties for
this first slice:

- no formatter becomes artifact authority;
- comments and presentation remain exactly attributable;
- source mapping is a total identity relation;
- reproducibility does not depend on backend writer stability;
- later compiler upgrades can change validation without gratuitously changing
  artifact bytes.

Decision consequence: the exact-WGSL profile validates through Naga but never
forms its canonical artifact through Naga's WGSL writer.

## WESL evaluation

WESL remains the strongest next composition candidate. Its stated goal is to add
shared WGSL enhancements such as imports, conditional translation, and packages,
then translate them to vanilla WGSL:

- https://wesl-lang.dev/spec/README
- https://wesl-lang.dev/spec/Imports

The Rust implementation provides an in-memory `VirtualResolver` and a custom
`Resolver` interface, which are compatible with RunenShader's requirement that
compilation operate over explicit closed inputs rather than ambient discovery:

- https://docs.rs/wesl/latest/wesl/struct.VirtualResolver.html
- https://docs.rs/wesl/latest/wesl/trait.Resolver.html

However, the WESL configuration specification currently identifies `2026_pre` as
the current edition and says defaults may evolve when no `wesl.toml` is present:

- https://wesl-lang.dev/spec/WeslToml

The compiler API also exposes source mapping, feature toggles, custom resolvers,
mangling, imports, conditional translation, generics, and other choices that must
be normalized deliberately rather than inherited wholesale:

- https://docs.rs/wesl/latest/wesl/struct.Wesl.html

Decision consequence: WESL should follow the exact-WGSL kernel as a separately
accepted composition profile. A future profile must pin its edition and feature
surface, use closed in-memory/custom resolution, and define source-map and
mangling semantics explicitly.

## Slang evaluation

Slang can target WGSL, but its own current target documentation still labels
WebGPU support as work in progress:

- https://shader-slang.org/slang/user-guide/targets

Its WGSL target documentation also records target-specific unsupported system
values and other translation limitations:

- https://docs.shader-slang.org/en/stable/external/slang/docs/user-guide/a2-03-wgsl-target-specific.html

Decision consequence: defer Slang from R0. Re-evaluate when WebGPU/WGSL support
matures or a concrete consumer requires Slang's broader language/toolchain
capabilities.

## Rust-version evidence

The current Naga README advertises a Rust compiler baseline, while package
metadata observed across the 30.x line has changed between releases. That is not
sufficient evidence to set a RunenShader repository MSRV before the exact
implementation dependency is resolved:

- https://docs.rs/crate/naga/30.0.1/source/README.md
- https://docs.rs/crate/naga/30.0.0/source/Cargo.toml

Decision consequence: R0B makes no repository MSRV claim. The implementation
slice must add the exact dependency, inspect the resolved package graph, and prove
any proposed `rust-version` through repository validation.

## Conclusion

The evaluated order is:

```text
R0 kernel: exact WGSL profile + private Naga validation realization
    -> prove RunenShader semantic/public contract

next composition candidate: WESL
    -> imports/packages/conditional composition over closed inputs

later evidence-driven candidate: Slang
```

This minimizes semantic surface in R0 while preserving a direct path to the
composition capability that motivated RunenShader. It also avoids creating a
generic frontend abstraction before two concrete supported frontend profiles
have demonstrated a real common contract.
