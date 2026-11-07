# ECAD

An electrical CAD application in early development, targeting equipment systems around the scale of a car. The intended desktop platforms are Linux, Windows, and macOS.

The working implementation is a **headless Rust engine**, not a graphical CAD application. It supports devices, terminals, conductors, project-local nested subsystems, explicit interfaces, and cable assemblies. Multi-object transactions, undo/redo, saved checkpoints, and deterministic JSON are implemented. Shared subsystem definitions/revisions, libraries, styles, bulk properties, keyboard-first control, and the desktop workspace remain planned.

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
| `cargo run --offline --locked -p ecad-adapters --example conveyor` | Builds a controller, nested conveyor/sensor assembly, interfaces, cable, and internal wiring as one transaction; verifies undo/redo and save/reopen |
| `cargo doc --offline --locked --workspace --no-deps --open` | Opens generated Rust API documentation |

The full suite currently contains 76 behavior tests and two doctests. Integration tests use isolated temporary files and repositories, including a real Git branch/commit/checkout workflow and safe reload after checkout. They do not alter this workspace's repository or contact a remote. This workspace's `origin` is configured as [a-g-child/ECAD](https://github.com/a-g-child/ECAD).

The scale probe varies unique device/terminal/conductor counts, unrelated subsystem counts, and hierarchy depth while editing a local connection 1,000 times per case. It records core timing and verifies round trips and undo correctness. Hierarchy cases cover 100/1,000/10,000 unrelated blocks and depth 16/128/256, with explicit terminal exposure and a real cable grouping. It does not measure UI rendering, memory, shared-definition expansion, disk IO, or release capacity. Results are measurements, not pass/fail timing guarantees.

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

The experimental v3 JSON format adds subsystems, device ownership, interface exposures, and cable membership to the v2 electrical model. IDs have canonical lowercase 32-digit hexadecimal encoding. `RandomIds` generates UUID-v4 identities from OS entropy; explicit constructors remain useful for imported data and deterministic fixtures. Save-as/file copies and Git branches retain project lineage. A general design-fork operation is not implemented yet.

V2 files load losslessly as root-level designs and save in v3 on the next explicit save. Opening v1 returns `MigrationRequired(1)` rather than inventing a project ID. Convert with `migrate_v1(legacy_text, project_id)`, retaining that ID, then save to a new destination through `Document`. Old fixtures are retained and object identities/assignments are preserved. Unknown fields and future versions are rejected.

Subsystems currently represent local occurrences, not shared templates. A port exposes an existing internal terminal; every boundary crossed by a completed conductor must expose its terminal. Cable ends reference devices/subsystems and validate their cores against those endpoints. Spare ends and empty cables remain valid drafts. Traversal is iterative with a provisional 256-level guard. Reparenting, device-scope changes, and interface removal currently validate the full model; ordinary termination edits follow only the affected ancestry and cable constraints.

Undo history is session-local and limited by transaction count, not bytes. Every session receives a process-local epoch; revision guards reject tokens from other sessions but are not persistent identifiers across application restarts. New documents start unsaved. `Document::open` starts clean, and `Document::save` moves the checkpoint only after success. Undo/redo tracks saved history positions; independently recreated equivalent contents are conservatively treated as dirty.

Normal editing should use `Document<JsonFile>`. It checks observed path/content versions before replacing a file, rejects unobserved existing destinations on first save/save-as, and requires an explicit policy before discarding local edits during reload. Invalid, missing, or different-lineage replacements leave the live model, history, and file checkpoint intact. Successful reload creates a fresh session epoch and clears history. Low-level `ProjectStore`/`Session` saves remain unconditional primitives for fixtures/imports and do not provide those external-change checks.

File saves stage and synchronize a replacement file. Change detection is optimistic: an uncooperative writer can still race between the last content check and rename. Full locking/concurrent-writer coordination, power-loss recovery, permission preservation, path-alias policy, and multi-file transactions remain pending. See the checklist for these next requirements.
