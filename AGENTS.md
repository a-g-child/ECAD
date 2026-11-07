# Repository working conventions

This repository contains planning documents and an initial headless Rust workspace. Read the [implementation checklist](docs/implementation-checklist.md) to distinguish working behavior from future scope. There is no desktop UI yet.

Read the relevant planning documents before implementation:

- [Product scope](docs/product-scope.md)
- [Architecture direction](docs/architecture.md)
- [Coding standards](docs/coding-standards.md)
- [Testing and platform strategy](docs/testing-strategy.md)
- [Implementation checklist](docs/implementation-checklist.md)
- [Drawing workflows](docs/drawing-workflows.md) for editor work
- [Connection labels](docs/connection-labels.md) for pinout/library work
- [Keyboard and action model](docs/keyboard-actions.md) for user actions and input handling
- [Hierarchy, scale, and shared revisions](docs/hierarchy-and-revisions.md) for subsystem reuse, identity, and scale work

## Architecture

- Preserve a headless Rust domain/application core independent of Tauri, widgets, concrete storage, Git libraries, and rendering backends.
- Keep committed document edits behind application commands with validation and coherent undo/redo.
- Keep internal model, persistence schema, and bridge DTOs distinct.
- Keep UI projections, caches, previews, and workspace preferences separate from the authoritative document.
- Use the application `Document` controller and versioned store contract for normal save/reload behavior. Low-level `ProjectStore` saves are unconditional. Preserve session/history/checkpoints on lifecycle failures, and do not overstate optimistic conflict detection as a filesystem lock.
- Respect stable identities, explicit units, structured routes, property ownership, and style inheritance.
- Distinguish reusable definitions, immutable revisions, and occurrences. Publish shared changes explicitly and keep consuming projects pinned until they adopt an update. Known-use discovery is advisory and may be incomplete.
- New electrical mutators must respect indexed pinned-instance ownership. Make-local retains historical ancestry; forgetting it is explicit and undoable. Preserve exact instance state through transaction rollback and undo.
- Keep shared-library publication outside project undo. Library writes preserve published bodies, validate recommendations, and coordinate cooperating writers; never infer adoption from a recommendation change or automatically steal retained locks.
- Known-use discovery is read-only and advisory. Preserve labelled last-known references on failed scans, expose registration coverage/freshness, and keep copied file locations distinct from project lineage. Never interpret empty results as global absence.
- Electrical properties map typed identities to real fields. Skip only inapplicable pairs in mixed edits; read-only/invalid changes fail coherently. Keep graphical style/default fields on drawing representations and preserve constructor/topology/pin validation.
- Creation presets copy appearance intent only into new drawing objects. Resolve each property through object, view, named style, then project; preserve equal-valued explicit overrides and restore PropertyState during undo. Reusable electrical bodies reject drawing-bearing content until template drawing instantiation exists.
- Route geometry never rewires conductors. Keep view-local anchors tied to real terminals and reject electrical changes that invalidate their routes. Preserve derived route indexes and exact affected-layout undo; reverse movement is not exact undo. Keep route and graphical-polyline creation presets distinct.
- Action surfaces share registry availability and one dispatcher. Preserve captured targets/revisions, protect text/IME/control input, flag native repeats, and share monotonic input IDs across delivery paths. Never install a partially compiled keymap or silently drop unknown overrides.
- Start with modules and crates needed by working features; avoid speculative plugin frameworks and empty scaffolding.

## Maintainability

- Use cohesive, plainly named modules and explanatory APIs. Avoid catch-all utility modules.
- Aim for manageable source files, commonly 150–350 lines. Review files above 500 lines and functions above 100 lines. Above 800 lines, split the file or document a concrete exception and review trigger. Generated data and fixtures are exceptions.
- Do not sacrifice readability, documentation, or cohesion to satisfy line counts.
- Document public Rust APIs with rustdoc, module responsibilities with `//!`, and meaningful errors, invariants, units, and side effects. Keep important examples executable where practical.
- Prefer private or `pub(crate)` APIs, typed errors/IDs/units, and explicit data flow.
- Use shared UI controls and layout tokens; do not introduce feature-specific layout behavior that shifts the workspace unexpectedly.
- Register user operations centrally for menus, toolbars, keyboard, and command-palette access. Avoid scattered global shortcut listeners, protect text editing/composition, and keep action dispatch separate from committed document commands.

## Verification and decisions

- Run `cargo xtask full` for the current local formatting, Clippy, unit/integration/regression, doctest, rustdoc, and core dependency checks. `cargo xtask scale` runs the separate bounded release probe. Dependencies must be cached; the runner uses offline/locked Cargo operations.
- Add relevant checks with implementation and keep the implementation checklist current.
- Keep unit, regression, and integration tests runnable locally without GitHub or remote services. Run the appropriate local checks before treating implementation as ready; report unavailable checks explicitly.
- Substantive bug fixes normally include a regression reproducer. Use temporary local repositories for Git integration tests.
- Target Linux, Windows, and macOS. Use portable tooling and distinguish host testing from cross-compilation and actual native validation on other platforms.
- Use reproducible local scalability workloads to investigate engine limits. Record environments and seeds, check correctness under load, and support performance changes with measurements. Keep expensive sweeps separate from routine checks.
- Verify meaningful behavior and failure cases appropriate to the change. Do not add tests for documentation-only edits.
- Record consequential architecture choices and revisit conditions in short decision records.
- Routine refactoring and justified guideline exceptions do not require user approval. The user's current instructions take precedence.
