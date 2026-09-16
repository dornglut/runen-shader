# Architecture

## System boundary

RunenShader is the standalone authority for reusable shader-source and
shader-compilation semantics.

Its canonical direction is:

```text
consumer / authoring host
    -> RunenShader semantic contracts
        -> selected frontend realization
            -> external compiler/tooling implementation
        -> canonical WGSL artifact + RunenShader evidence

consumer integration
    -> RunenGPU source admission / renderer / other downstream authority
```

The second path is composition, not a dependency inside the RunenShader semantic
core. RunenShader and RunenGPU are sibling frameworks.

## Semantic versus realization authority

The normative semantic model is under [spec/](spec/README.md). It owns logical
identity, immutable source revision meaning, explicit compilation input,
dependency resolution, frontend-profile boundaries, outcome classes, canonical
artifact formation, provenance, source mapping, and reproducibility.

Concrete compiler libraries, their private IR, reflection structures, filesystem
resolvers, caches, thread scheduling, and process state are implementation
realizations. They may implement accepted contracts but do not define them.

The public Rust crate realizes the shared semantic data boundary and two accepted
frontend paths through one stateful `ShaderCompiler`: exact WGSL through pinned
Naga 30.0.1 and closed WESL composition through pinned `wesl-rs` 0.5.0 V1.
The private RunenShader WGSL profile gate plus Naga parse/validation govern the
exact path and independently readmit WESL-generated WGSL before artifact
publication. WESL translation and resolution remain private realization work;
neither compiler becomes normative language or RunenShader identity authority.

## Current repository shape

The current source tree contains:

- a public semantic data kernel for explicit package/module/source identities,
  exact source snapshots, closed exact-WGSL and multi-source WESL inputs,
  deterministic artifact and provenance evidence, truthful source-mapping
  classifications, diagnostics, and outcome classes;
- one public stateful `ShaderCompiler` with closed dispatch to the Naga 30.0.1
  exact-WGSL and `wesl-rs` 0.5.0 WESL composition realizations;
- exact-WGSL source-byte artifact formation with total identity source mapping
  after successful private gate reconciliation, parsing, and validation;
- WESL composition of explicitly admitted single-package in-memory modules with
  explicit conditional-feature values into generated WGSL, independently checked
  by the RunenShader WGSL gate and pinned Naga validator before publication;
- complete participating-source provenance for WESL and a conservative total
  `Unattributable` transformed-artifact map rather than unproven byte attribution;
- the normative `spec/` authority;
- root architecture, roadmap, status, testing, licensing, and agent entrypoints;
- a repository-local `xtask` that owns merge-readiness validation;
- a thin immutable CI caller.

The WESL realization consults only the closed in-memory resolution table after
admission; filesystem contents, cwd, environment, `wesl.toml`, registries,
network lookup, and package dependencies are not compilation authority.
Conditional attributes on imports are outside the accepted initial profile.
Profile-valid WGSL global-directive composition affected by upstream
`wesl-rs` issue #85 remains `Unsupported`; the realization does not claim full
coverage of the accepted WESL profile.

Both compiler paths keep source-byte observations in the compiler instance,
translate upstream diagnostics into RunenShader-owned logical subjects where
truthfully possible, and never expose or store upstream compiler IR, reflection,
resolver handles, or source-map objects in accepted artifacts. The exact-WGSL
path uses no WGSL writer or re-emission.

## Implementation boundary

The root crate keeps implementation modules private and re-exports the accepted
public semantic types. Opaque identity representation, storage choices, and
module layout are implementation details unless `spec/` promotes a concrete
boundary.

Canonical WGSL artifact formation remains compiler-owned inside the crate:
ordinary callers can inspect `ShaderArtifact` but cannot construct
success-shaped artifact data directly. The exact-WGSL realization publishes
unchanged admitted source bytes and identity mapping; the WESL realization
publishes deterministic generated WGSL only after independent pinned-WGSL
validation, with full closed-input provenance and total unattributable mapping.
Neither artifact implies GPU program, binding, pipeline, device, or execution
admission: downstream consumers map the product into their own contracts.

## Documentation authority

| Concern | Canonical location |
| --- | --- |
| normative shader semantics | `spec/` |
| repository/system boundary | `ARCHITECTURE.md` |
| durable outcome sequence | `ROADMAP.md` |
| durable maturity/capability state | `STATUS.md` |
| merge-readiness and evidence | `TESTING.md` plus repository validator |
| executor rules | `AGENTS.md` |
| public landing and navigation | `README.md` |
| current licensing representation | `LICENSE` and `LICENSING.md` |
| implementation and tests | Rust source/tests when accepted |
| repository-local decision rationale | `docs/adr/` only when a real local ADR exists |
| dated durable verification reports | `docs/reports/` only when such evidence is accepted |
| live work state | GitHub issues, pull requests, and Projects |

Do not create empty documentation taxonomies for symmetry. Normative `spec/`
content must remain self-contained and may link only within `spec/`.

## Dependency rules

- The semantic core does not require RunenGPU, RunenRender, Runenwerk, or Runen
  language packages.
- A concrete frontend dependency is admitted only by accepted RunenShader work
  that proves its supported profile and conformance.
- Filesystem, registry, network, watcher, and hot-reload adapters are outside the
  semantic core unless later accepted work defines an explicit boundary.
- Downstream integrations map RunenShader products into downstream contracts;
  they do not share identity or mutate RunenShader authority.

## Failure boundary

Compilation failures are typed by the normative outcome model. Invalid input,
unsupported realization coverage, and operational tool failure remain distinct.
An internal RunenShader invariant violation is an implementation defect and must
fail closed rather than masquerade as an ordinary source error or publish an
artifact.
