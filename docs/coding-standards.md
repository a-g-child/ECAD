# Coding standards

Status: initial repository conventions, applied to the working headless Rust workspace. Formatting, Clippy, public rustdoc, doctests, and core dependency checks run through `cargo xtask full`. Desktop/frontend tooling remains open.

## Readable structure

Organize files around one cohesive responsibility. Prefer domain names such as `terminal_mapping`, `route_edit`, and `style_resolution` over broad `utils`, `helpers`, or `manager` modules. Avoid splitting code solely to meet a line count or adding abstractions that only rename an expression.

Use these review thresholds for authored source, including frontend components:

| Unit | Guidance |
| --- | --- |
| Source file | Aim for roughly 150–350 lines where natural; smaller files are welcome. |
| File over 500 lines | Review responsibilities and extract cohesive units when useful. |
| File over 800 lines | Split, or record a concrete reason to retain it and a follow-up review trigger. |
| Function | Prefer one understandable operation; roughly 20–60 lines is a useful working range, not a minimum. |
| Function over 100 lines | Review control flow, responsibilities, and opportunities for meaningful extraction. |

These are maintainability guidelines, not reasons to compress formatting or remove documentation. Generated code, fixtures, and declarative tables are exceptions; keep them separate from ordinary behavior. Test code should not be moved arbitrarily just to hide an oversized implementation file. Routine exceptions need a rationale, not a user approval step.

Keep orchestration separate from calculations and IO. Name functions by the operation they perform. Prefer explicit arguments and return types over hidden global state, flag-heavy functions, and long positional parameter lists. Use parameter structs where inputs form a meaningful concept.

## Rust APIs and visibility

- Start private. Use `pub(crate)` for internal collaboration and `pub` for intentional cross-crate APIs.
- Expose behavior and read access rather than unrestricted mutable fields that can violate invariants.
- Use typed IDs, enums, and unit-aware values to express domain distinctions.
- Use traits for real IO boundaries or genuine alternate implementations; do not create a trait for every type or function.
- Prefer concrete types and straightforward control flow until additional generic machinery has a clear benefit.
- Return typed errors for recoverable failures. Preserve context at boundaries and avoid collapsing validation into a generic message.
- Do not use `unwrap`, `expect`, or panics for expected user, file, repository, or IPC errors. Tests and documented internal invariants may justify them.
- Avoid unchecked casts, unexplained tolerances, hidden rounding, and floating-point equality assumptions in geometry code.
- Avoid `unsafe` by default. Any necessary block needs a documented safety argument and a narrowly contained implementation.

## Documentation that helps readers

Document every intentional public Rust API, including types, fields, variants, traits, and functions. Internal cross-module interfaces and non-obvious private functions also need explanations of their purpose and invariants.

Use `//!` module/crate documentation to explain responsibility, key types, dependency boundaries, and an example workflow. Use `///` item documentation to explain observable behavior rather than repeat the signature.

For functions, include what matters to a caller:

- Purpose and meaning of the result.
- Units, coordinate frame, ordering, and preconditions where relevant.
- Side effects, identity preservation, and whether failure leaves state unchanged.
- `# Errors`, `# Panics`, and `# Safety` sections where applicable.
- A short runnable example for important workflows, or a link to a shared example covering related functions.

For example, a route-edit function should explain whether endpoints remain attached, which bends can move, and what happens when the requested edit conflicts with a pinned point. "Updates the route" is insufficient.

Keep examples small and test them as doctests where practical. Use `no_run` when an example should compile but needs an external runtime or resource. Do not mark broken examples `ignore` simply to make checks pass. Explain algorithms and tradeoffs in comments where they are not evident from the code; do not narrate each line.

Follow the [rustdoc guidance](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html) and [Rust API documentation guidelines](https://rust-lang.github.io/api-guidelines/documentation.html). These support the documentation conventions; the file-size thresholds above are project choices.

## Frontend conventions

Prefer TypeScript for application and bridge code, subject to the frontend decision. Keep shared controls separate from CAD features and tool interaction logic separate from rendering components. Do not put the whole inspector, tool controller, and bridge client into one component.

Use shared UI tokens and controls for dimensions and behavior. Avoid per-feature sizing rules that bypass the shell's layout contracts. Frontend code must not write project files or bypass core commands to mutate committed electrical state.

Register user-facing operations through the shared action model. Do not add feature-specific global shortcut listeners or separate implementations for toolbar, menu, and keyboard invocation. Use effective keymap data for shortcut labels, and preserve typing, focus, and input-composition behavior. See [Keyboard and action model](keyboard-actions.md).

Keep transient selection, hover, tool previews, and workspace preferences separate from saved document data. Use typed bridge contracts and preserve revision information. UI labels and diagnostics should describe the user's design and action, without exposing internal module structure unnecessarily.

## Dependencies and changes

Before adding a dependency, identify the capability needed, its owner, and the boundary at which it is used. Consider maintenance, licensing, platform support, and replacement cost at a depth appropriate to its role. Keep heavy rendering/CAD/IO dependencies out of the domain crate.

When changing a public contract, update its callers, documentation, and relevant tests together. File-schema changes need explicit compatibility/migration handling. Make broad architectural changes reviewable through a short decision record.

## Implementation checks

Unit, regression, and integration tests are required parts of the implementation strategy. See [Testing and platform strategy](testing-strategy.md) for locally runnable profiles, test isolation, native desktop checks, and Linux/Windows/macOS coverage. GitHub backup does not replace local verification.

Use `cargo xtask full` for the portable offline/locked check workflow. Its underlying Rust checks are equivalent to:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --doc
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

Enable `#![deny(missing_docs)]` in reusable public Rust libraries; use module-level explanations and review for internal APIs that the lint does not cover. Keep unsafe code linted according to each crate's needs. Add frontend formatting, linting, type checks, and behavior checks once its tooling is selected. Lock the Rust toolchain once chosen so these checks are reproducible.

The environment assignment shown for rustdoc uses POSIX shell syntax. The implemented portable runner sets that environment programmatically. Separate host-independent core checks from desktop/system prerequisites when necessary; cross-compiling alone does not count as testing the other operating systems. Current execution has been verified on macOS/aarch64; Windows/Linux runs remain pending.

Use tests for meaningful invariants and failure cases: bulk-edit atomicity, undo/redo, endpoint preservation, serialization stability, and property applicability. Formatting, static checks, and documentation review cover routine structural edits; do not add tests that merely reproduce trivial implementation details.
