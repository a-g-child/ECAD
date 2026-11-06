# ECAD

An electrical CAD application in early development, targeting equipment systems around the scale of a car. The intended desktop platforms are Linux, Windows, and macOS.

The first working slice is a **headless Rust engine**, not a graphical CAD application. It can create devices and terminals, connect individual conductors, commit multi-object transactions, undo/redo complete operations, track saved checkpoints, and save/reopen deterministic versioned JSON. Session-specific revision tokens reject edits from old or unrelated sessions. The project scope includes hierarchy, libraries, styles, bulk properties, keyboard-first control, and integrated Git workflows; those are tracked separately from implemented features.

The document controller now supports guarded saves, save-as, external-change detection, and validated reloads. Persistent project IDs and OS-random UUID-v4 object generation are available without adding IO dependencies to the core.

## Start here

- [Implementation checklist](docs/implementation-checklist.md): completed work, next slices, and pending platform checks.
- [Product scope](docs/product-scope.md): intended product behavior and boundaries.
- [Architecture](docs/architecture.md) and [coding standards](docs/coding-standards.md).
- [Testing strategy](docs/testing-strategy.md): local suites and scalability experiments.

## Local setup

Install Rust 1.97 or newer with Cargo, rustfmt, and Clippy, plus Git for the local repository integration test. No Node, Tauri, Python, database, or GitHub account is needed for this headless slice. The current baseline was tested with Rust 1.97.0 on macOS/aarch64; Windows and Linux execution is pending. An exact rustup toolchain pin remains a checklist item; `rust-version` currently records the minimum version.

With dependencies available locally:

```sh
cargo xtask full
```

On a fresh machine, populate the dependency cache once first:

```sh
cargo fetch --locked
```

`Cargo.lock` is included and should be versioned with the project. The local check runner invokes Cargo with `--offline --locked`; subsequent checks do not need GitHub or network access. A fresh platform may need its own dependency cache and system tooling.

## Working commands

| Command | What it does |
| --- | --- |
| `cargo xtask quick` | Dependency-boundary checks, formatting, Clippy, unit/integration/regression tests |
| `cargo xtask full` | Quick checks plus example-target compilation, doctests, and documentation with warnings treated as errors |
| `cargo xtask scale` | Optimized headless probe at 100, 1,000, and 10,000 devices; writes `target/scale/latest.json` |
| `cargo run --offline --locked -p ecad-adapters --example headless` | Creates two devices and one conductor, exercises undo/redo, and saves/reopens in an automatically cleaned temporary directory |
| `cargo doc --offline --locked --workspace --no-deps --open` | Opens generated Rust API documentation |

The full suite currently contains 56 behavior tests and two doctests. Integration tests use isolated temporary files and repositories, including a real Git branch/commit/checkout workflow and safe reload after checkout. They do not alter this workspace's repository or contact a remote. This workspace's `origin` is configured as [a-g-child/ECAD](https://github.com/a-g-child/ECAD).

The scale probe varies unique device/terminal/conductor counts while editing the same local connection 1,000 times. It records build, validation, encoding/decoding, and local-edit latency and verifies round trips and undo correctness. It does not yet measure UI rendering, memory, hierarchy, disk IO, or release capacity. Results are measurements, not pass/fail timing guarantees.

## Source boundaries

```text
crates/ecad-domain/    Electrical objects, typed identities, validation
crates/ecad-app/       Commands, session revisions, undo/redo, storage contract
crates/ecad-adapters/  Explicit JSON schema, file IO, integration tests/examples
xtask/                Portable local checks and dependency-boundary enforcement
docs/                 Product plans, decisions, and implementation checklist
```

`ecad-domain` has no dependencies. `ecad-app` depends only on the domain. Adapters use serde/serde_json for explicit disk DTOs, tempfile for staged replacement, getrandom/uuid for fallible UUID-v4 generation, and SHA-256 for observed content tokens. Public APIs require rustdoc; unsafe code is forbidden throughout the workspace.

## Prototype boundaries

The experimental v2 JSON format contains a persistent project lineage ID, devices, terminals, and conductors. IDs have canonical lowercase 32-digit hexadecimal encoding. `RandomIds` generates UUID-v4 identities from OS entropy; explicit constructors remain useful for imported data and deterministic fixtures. Save-as/file copies and Git branches retain the lineage ID; creating a distinct design requires choosing a new lineage rather than treating the path as identity. A general design-fork operation is not implemented yet.

Opening v1 returns `MigrationRequired(1)` rather than inventing an ID. Explicitly convert with `migrate_v1(legacy_text, project_id)`, choosing and retaining the assigned project ID, then save the result to a new destination through `Document`. The migration preserves object IDs and assignments and never rewrites the source. Unknown fields and future versions are rejected.

Undo history is session-local and limited by transaction count, not bytes. Every session receives a process-local epoch; revision guards reject tokens from other sessions but are not persistent identifiers across application restarts. New documents start unsaved. `Document::open` starts clean, and `Document::save` moves the checkpoint only after success. Undo/redo tracks saved history positions; independently recreated equivalent contents are conservatively treated as dirty.

Normal editing should use `Document<JsonFile>`. It checks observed path/content versions before replacing a file, rejects unobserved existing destinations on first save/save-as, and requires an explicit policy before discarding local edits during reload. Invalid, missing, or different-lineage replacements leave the live model, history, and file checkpoint intact. Successful reload creates a fresh session epoch and clears history. Low-level `ProjectStore`/`Session` saves remain unconditional primitives for fixtures/imports and do not provide those external-change checks.

File saves stage and synchronize a replacement file. Change detection is optimistic: an uncooperative writer can still race between the last content check and rename. Full locking/concurrent-writer coordination, power-loss recovery, permission preservation, path-alias policy, and multi-file transactions remain pending. See the checklist for these next requirements.
