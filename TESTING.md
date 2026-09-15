# Testing and validation

## Canonical command

```text
cargo validate
```

This command is implemented by the repository-local `xtask` and is the single
merge-readiness baseline for RunenShader.

## Baseline checks

Validation fails closed when:

- required RunenShader authority files are missing;
- removed template authority such as `BOOTSTRAP.md` reappears;
- active repository surfaces retain template identity or Apache product-license
  claims;
- Cargo metadata and the lockfile disagree;
- Markdown links are broken or normative `spec/` content escapes its authority
  boundary;
- repository text required by the validator is not valid UTF-8 or lacks a final
  newline;
- Rust formatting is not clean;
- workspace tests fail;
- Clippy emits warnings;
- rustdoc emits warnings;
- Git whitespace checks fail;
- validation changes repository state.

The repository is intentionally small during bootstrap, but the semantic and
repository-authority guards are real acceptance checks rather than placeholders.

## Focused validation

Focused commands may be used while developing a bounded change, for example:

```text
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo doc --workspace --no-deps --locked
```

They do not replace `cargo validate`.

## CI and exact-head evidence

`.github/workflows/validation.yml` is deliberately thin. It pins Dornglut's
shared Rust validation workflow to an immutable commit and invokes the
repository-owned command with read-only repository permission.

Pull-request acceptance requires successful independent validation of the exact
reviewed feature-head SHA. If the feature head moves, earlier validation and
review evidence is stale and must not be reused.

When a suitable local Rust executor is unavailable, hosted exact-head CI is the
first executable compile/test gate; local/model inspection must be reported only
as preparation, not as executable validation.

## Exact-WGSL compiler conformance

The focused exact-WGSL compiler tests exercise the public stateful
`ShaderCompiler` with valid modules with and without entry points, exact
source-byte preservation, parse and semantic rejection, Naga-only extension
rejection, pinned-but-unrealized extension outcomes, malformed-directive
deferral, source-revision rebinding, repeated compilation, distinct logical
identities, logical diagnostic subjects and truthful byte ranges, and
representative actual constructs for every Gate V1 `Continue` extension. The
tests also cover every unsupported `enable` and `requires` name recognized by
the private R0D2 gate, including `fragment_depth`.

Naga is inspected only through its public WGSL parse labels and validation spans;
its module, IR, reflection, and WGSL writer are not part of the RunenShader
artifact boundary. Primary parse-label ordering is used for gate reconciliation
while all public labels remain available as translated diagnostics. A
child-process test isolates cwd and environment mutation while varying unrelated
files and cache-like state, and compares the same semantic result across those
host states. The production dependency enables `wgsl-in` only.

## WESL composition conformance

`tests/wesl_compiler_conformance.rs` exercises the public compiler path for the
pinned `wesl-rs` 0.5.0 realization. Positive coverage includes WESL-only
visibility syntax, package-root and relative multi-module imports,
aliases/collections/public re-exports, package/private visibility, a valid module
import cycle, main-module pipeline exposure, explicit true/false conditional
translation, unused explicit features, repeated-output and collection-order
determinism, and complete closed-input provenance.

Negative and boundary coverage includes malformed/non-canonical/external table
keys, contradictory path/module/source/feature bindings, syntax diagnostics for
all admitted sources, missing required features, external package references,
conditional imports, required unresolved imports, and the bounded global
directive surface. An unused unresolved import is also exercised explicitly to
prove the selected `strip = false` configuration does not turn static WESL module
loading into eager resolution.

Every accepted WESL result is independently checked by the production path
against the RunenShader WGSL language gate and pinned Naga validator before
artifact publication. The V1 transformed artifact map is total but
`Unattributable`; tests separately preserve exact-WGSL identity mapping and the
public model's exact/generated/unattributable classifications. A child-process
probe varies cwd, environment, `wesl.toml`, and shader-like filesystem noise and
compares canonical bytes plus RunenShader identity/provenance/mapping evidence.
