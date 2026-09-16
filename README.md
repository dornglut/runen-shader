# RunenShader

RunenShader is Dornglut's standalone Rust framework for reusable shader-source and
shader-compilation semantics.

It defines the stable source-to-artifact boundary used by renderers, compute
consumers, and other tooling without making a renderer, GPU executor, filesystem,
or compiler implementation the owner of shader-source semantics. Its concrete
compiler paths are pinned Naga 30.0.1 exact WGSL and bounded `wesl-rs` 0.5.0
WESL composition.

## Maturity

RunenShader has completed its R0 semantic-kernel outcome. Repository and normative
specification authority are established, and the public crate exposes the semantic
data kernel plus a stateful compiler for the accepted exact-WGSL and closed WESL
composition profiles.

The exact-WGSL path preserves source bytes and identity source mapping. The WESL
path translates an explicit single-package, in-memory module set with explicit
conditional-feature values into generated canonical WGSL; its V1 artifact map is
total but `Unattributable`. The pinned WESL profile is broader than this first
realization: profile-valid global-directive composition affected by upstream
`wesl-rs` issue #85 remains `Unsupported`, not silently accepted.

Only the frontend surfaces demonstrated by their respective compiler conformance
fixtures are implemented. Neither path implies downstream GPU/device admission.
See [STATUS.md](STATUS.md) for durable maturity and [ROADMAP.md](ROADMAP.md) for
outcome sequencing.

## Boundary

RunenShader owns reusable contracts for:

- logical shader package, module, and source-unit identity;
- immutable source revisions and explicit source snapshots;
- closed compilation inputs and deterministic dependency resolution;
- frontend-profile and compiler-realization distinction;
- compilation outcomes and source-facing diagnostics;
- canonical WGSL artifact formation, exact or generated according to the profile;
- provenance, source mapping, and reproducibility evidence.

RunenShader does not own GPU resources, program admission, bindings, pipelines,
submission, renderer material or lighting semantics, application recovery,
filesystem watching, hot-reload activation, or package-registry/network-fetch
policy.

The normative semantic contract lives under [spec/](spec/README.md). RunenShader
and RunenGPU are sibling authorities: a consumer may explicitly submit the exact
WGSL from a RunenShader artifact into a separately owned RunenGPU source-admission
operation.

## Validation

```text
cargo validate
```

is the single repository-owned merge-readiness command. CI is a thin immutable
caller of that command. See [TESTING.md](TESTING.md).

No minimum supported Rust version is claimed yet. The repository validates on the
stable toolchain declared by [rust-toolchain.toml](rust-toolchain.toml); an MSRV
will be accepted only from concrete product evidence.

## Repository authority

- [Architecture](ARCHITECTURE.md)
- [Normative specification](spec/README.md)
- [Roadmap](ROADMAP.md)
- [Status](STATUS.md)
- [Testing and validation](TESTING.md)
- [Agent guide](AGENTS.md)
- [Organization contribution guidance](https://github.com/dornglut/.github/blob/main/CONTRIBUTING.md)
- [Organization security policy](https://github.com/dornglut/.github/blob/main/SECURITY.md)

Repository profile: `rust-framework`. Lifecycle: `active`. Contribution mode:
`owner-only`.

## License

The current public RunenShader representation is
[GPL-3.0-only](LICENSE). A separately negotiated commercial license may be
available as described in [LICENSING.md](LICENSING.md).
