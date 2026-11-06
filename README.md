# ECAD

An electrical CAD application in early development, targeting equipment systems around the scale of a car. The intended desktop platforms are Linux, Windows, and macOS.

The first working slice is a **headless Rust engine**, not a graphical CAD application. It can create devices and terminals, connect individual conductors, commit multi-object transactions, undo/redo complete operations, track saved checkpoints, and save/reopen deterministic versioned JSON. Session-specific revision tokens reject edits from old or unrelated sessions. The project scope includes hierarchy, libraries, styles, bulk properties, keyboard-first control, and integrated Git workflows; those are tracked separately from implemented features.

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

The full suite currently contains 36 behavior tests and two doctests. Integration tests use isolated temporary files and repositories, including a real Git branch/commit/checkout workflow. They do not alter this workspace's repository or contact a remote. This workspace's `origin` is configured as [a-g-child/ECAD](https://github.com/a-g-child/ECAD).

The scale probe varies unique device/terminal/conductor counts while editing the same local connection 1,000 times. It records build, validation, encoding/decoding, and local-edit latency and verifies round trips and undo correctness. It does not yet measure UI rendering, memory, hierarchy, disk IO, or release capacity. Results are measurements, not pass/fail timing guarantees.

## Source boundaries

```text
crates/ecad-domain/    Electrical objects, typed identities, validation
crates/ecad-app/       Commands, session revisions, undo/redo, storage contract
crates/ecad-adapters/  Explicit JSON schema, file IO, integration tests/examples
xtask/                Portable local checks and dependency-boundary enforcement
docs/                 Product plans, decisions, and implementation checklist
```

`ecad-domain` has no dependencies. `ecad-app` depends only on the domain. The adapter implements the application's storage contract using serde/serde_json for explicit disk DTOs and tempfile for staged file replacement. Public APIs require rustdoc; unsafe code is forbidden throughout the workspace.

## Prototype boundaries

The v1 JSON format is experimental and contains only devices, terminals, and conductors. Future schema changes must be deliberate; unknown versions and fields are rejected. IDs are caller-supplied, nonzero opaque 128-bit values with canonical hexadecimal encoding; a production ID-generation and project identity scheme is pending.

Undo history is session-local and limited by transaction count, not bytes. Every session receives a process-local epoch; revision guards reject tokens from other sessions but are not persistent identifiers across application restarts. New sessions start unsaved; `Session::open` starts clean, and `Session::save` moves the checkpoint only after success. Undo/redo tracks saved history positions; independently recreated equivalent contents are conservatively treated as dirty. External reload orchestration remains pending.

File saves stage and synchronize a replacement file but do not yet provide external-change protection, full power-loss recovery, permission preservation, or multi-file transactional saves. See the checklist for these next requirements.
