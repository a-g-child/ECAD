# Electrical CAD — product scope

Current editor integration direction: [contextual editor roadmap](contextual-editor-integration.md). It supersedes earlier form-heavy template UI proposals: one selection owner, automatic contextual properties, simplified Insert/Draw tools, shared field/tool modules and explicit persistence evolution. Historical implementation sections remain records of their respective slices.

Status: initial scope for discussion, not a final requirements specification or architecture.

Implementation has begun with a small headless Rust slice. See the [implementation checklist](implementation-checklist.md) for working behavior, verification, and remaining scope.

## Product intent

Build an electrical CAD application for designing connected electrical systems at several levels of detail. Its central strength is a coherent hierarchy: users can understand a complete system through cable-level overviews, then drill into subsystems to design individual conductors and terminal connections.

Support generic electrical design before manufacturer selection. Treat document editing, undo/redo, and Git version control as foundational workflows.

The initial target is a complete equipment system at roughly the scale of a car, covering interconnected components, cables, and their conductors. Building services are outside the initial target. Keep the model extensible to larger systems without claiming those workflows are already supported.

## Agreed direction

- Maintain one underlying electrical model with multiple coordinated views.
- Support nested functional subsystems with explicit external interfaces.
- Show cables as individual connections in overview views and expose their contents in detailed views.
- Keep functional hierarchy, physical location, and drawing organization distinct.
- Allow generic devices and symbols to be placed before manufacturer parts are assigned.
- Provide reusable libraries for devices, symbols, wires, cables, and manufacturer parts.
- Support reusable connector pinouts and connection-label templates, with custom project definitions and live bindings to electrical data.
- Use a document-based editing model with undo and redo.
- Build defaults, shared drawing styles, and explicit overrides into the document model from the beginning.
- Provide consistent, complete property access and bulk editing across both similar and mixed selections.
- Use a stable desktop workspace with shared UI controls and explicit layout contracts for panes, toolbars, inspectors, and notifications. Organise tools into File, Drawing and Library ribbons, preserving the canvas when switching.
- Treat routes, connected drawing paths, groups, and image attachments as coherent editable objects, with deterministic placement and explicit connectivity behavior.
- Make version control an important product capability, with Git accessible through the application interface.
- Prefer a deterministic, text-based authoritative project format suitable for Git review and merging.
- Establish explicit module ownership, dependency boundaries, documented interfaces, and manageable source files before implementation grows.
- Build meaningful unit, regression, and integration tests into development, with locally runnable verification independent of GitHub.
- Target Linux, Windows, and macOS as desktop platforms from the beginning.
- Design keyboard-first control around a complete registry of user actions, with direct shortcuts for frequent operations and extensible, contextual remapping.

Tauri with a Rust core is a preferred candidate, subject to interaction and rendering prototypes. SQLite is a candidate for indexing and caching; its role is not settled. Neither choice is a finalized architecture.

## Drawing sheets and reusable layouts

The application workspace can hold several open projects. The electrical/component explorer is scoped to the selected project; each project keeps independent topology, history, save state and Git/report context.

The left Project explorer should primarily show pages: cover, generated drawing list, overview schematics and detailed wire/terminal/connector sheets, with document groups where useful. Open sheets in selectable tabs/panes in the main workspace. Keep electrical structure available separately. Every page adopts its CAD template; hierarchical sheet blocks and off-sheet connectors navigate and represent shared electrical interfaces without duplicating the underlying model.

Organize actual drawings as Project → Documents → Sheets. Cable overviews, hierarchical diagrams and detailed wire/connector schematics are edited directly on template-backed CAD sheets with borders and drawing information. The separate Model view/read-only paper workflow in the v8 prototype is superseded. Symbol/component/footprint and template-definition editors are different workspace kinds with their own appropriate coordinate surfaces. Logical drawing documents and sheet organization remain independent of electrical subsystem hierarchy and output drawing sets.

Include structured sheet layouts and title-block tables with row/cell split/merge, multiline entries, bulk formatting, scoped variables and explicit local overrides. Personal/company profiles supply copied project values; sheet-local and derived values remain distinct. Plan standard, draft, cover and generated contents layouts, pinned template revisions, paper-space sizing, portable logos and reproducible issued output. See [sheets and templates](sheets-and-templates.md) for the feature contract and delivery checklist.

## Core concepts

| Concept | Meaning |
| --- | --- |
| Project | A portable design containing the electrical model, views, settings, and required library definitions or pinned references. |
| Functional subsystem | A nested block representing a function, with an explicit external interface and internal implementation. |
| Subsystem definition and revision | A reusable subsystem design with a stable definition identity and immutable revisions, instantiated within one or more projects. |
| Physical location | A cabinet, mounting area, installation zone, or other place occupied by equipment. |
| Device | An instance of a generic electrical component with identifiable terminals and electrical roles. |
| Terminal | An identifiable electrical connection point on a device or interface. |
| Connection | A modeled electrical relationship between connection points; its precise relationship to wires and nets remains to be specified. |
| Wire or conductor | An individual conducting element with properties and terminations. A conductor may belong to a cable. |
| Cable | An identifiable assembly of conductors and applicable shields, pairs, and other structure. |
| Symbol | A graphical representation of a device, interface, or part of a device. |
| Drawing document | A logical project drawing package with its own metadata, ordered sheets and default sheet template. |
| Sheet | An editable template-backed CAD page with paper, drawing information and schematic representations. |
| Drawing view | An internal representation/coordinate context; project schematics are presented and edited on their sheets. |
| Definition editor | A dedicated symbol, component, footprint or template workspace, separate from project drawing sheets. |
| Manufacturer part | An orderable item with technical properties and mappings to compatible generic devices and representations. |
| Library definition | A reusable, versioned definition from which project instances are created. |
| Drawing style | A named, reusable set of presentation properties that objects can inherit, with explicit overrides. |
| Connector profile | A reusable contact arrangement, identifiers, orientation metadata, and graphical geometry that can be mapped to device terminals. |
| Connection label | A placed pinout diagram and/or terminal table bound to a device or connector, using a reusable presentation template. |

Drawing geometry must not implicitly determine electrical identity. A device may have several representations while remaining one device. Hierarchical interfaces must resolve to real internal connections without creating duplicate devices or cables.

## Functional scope

### Hierarchy and navigation

Users can create nested subsystems, define their interfaces, and navigate between an overview and internal detail. Renaming, moving, or reorganizing objects preserves their identities and valid references.

A subsystem may span multiple sheets and physical locations. A sheet does not own the electrical objects it displays. Cable continuity can extend through hierarchy boundaries.

The finest electrical element in the working model is an individual conductor; an ordinary component/device is the highest non-hierarchical equipment object. Terminals and contacts remain required connection anchors. Cables group conductors, while nested subsystems organize components and subordinate subsystems.

Allow deeply nested hierarchy without assigning a small fixed product limit now. Reject containment and definition-dependency cycles. A depth cap does not itself create deterministic behavior; define traversal order and explicit limits for depth, expanded instance count, and other resources, with clear diagnostics rather than silent truncation.

Reusable subsystem definitions may be instantiated within and across projects. Offer a deliberate choice between a project-local revision and publishing a new shared revision/default. Existing projects retain pinned revisions until an explicit update is accepted. Known-use checks help show impact, but must not claim to discover every offline or unknown project. See [Hierarchy, scale, and shared revisions](hierarchy-and-revisions.md).

The first implementation slice may begin with ordinary nested instances, but its identity model must distinguish definitions, revisions, and occurrences from the start.

### Overview and detailed wiring

Overview views display functional blocks or devices connected by cables. Detailed views expose individual conductors, shields, terminals, and connector pins. Both views operate on the same underlying objects.

Cable properties include its type, identifier, contents, and terminations. Individual conductors can be assigned to connection points. The application can identify incomplete assignments and inconsistent connections.

In the initial scope, routing means electrical connectivity and drawn paths. Physical installation routes, bend radii, and calculated cable lengths require separate requirements.

### Drawing, placement, and attachment workflows

The editor must retain the structure and intent of drawn objects after placement. A multi-segment wire route or graphical polyline remains an editable whole, while allowing deliberate segment and vertex editing. Drawing primitives, graphical groups, device instances, and electrical connections have distinct identities and behaviors.

Provide predictable snapping, numeric placement, alignment, distribution, repeat placement, and editable arrays. Selection must make whole-object versus sub-element editing explicit. Moving connected equipment should support preserving its connections; detaching or reconnecting must be a distinct, visible action. Local route adjustments must preserve unaffected geometry and electrical identity.

Crossings, junctions, and wire hops are separate concepts. A hop is a presentation convention for a non-connected crossing, generated from the actual route geometry and project style. Repositioning crossing wires must not silently connect them.

Images are first-class document attachments with editable transforms, cropping, anchoring, stacking order, locking, and defined print/export behavior. Projects should carry the image assets needed for portability, with external references an explicit alternative rather than an accidental dependency.

See [Drawing and editing workflows](drawing-workflows.md) for the detailed draft, including selection, constraints, connected moves, arrays, image handling, and acceptance scenarios. A full general-purpose geometric constraint solver and automatic global routing are not prerequisites for the initial editor.

### Generic devices and libraries

Users can place generic devices, define or select suitable symbols, and work with terminal counts and terminal functions before selecting a manufacturer part.

Wire and cable definitions store reusable attributes such as cross-section, colour, conductor count, pairing, shielding, and ratings where applicable. The requirements phase will distinguish mandatory properties from optional metadata.

Project instances retain enough definition data and version information to reopen reliably without depending on a changing external library. Library updates must not silently alter existing designs.

### Connector pinouts and connection labels

Provide a general library of connector profiles and connection-label templates, together with the ability to create or adapt custom definitions within a project. A one-off definition should not require publishing a global library item; users can promote it into a reusable library later.

Connection labels can combine connector face diagrams, contact identifiers, assignment tables, device designations, hierarchy/location paths, descriptions, and applicable wire or cable information. Their displayed electrical values come from the project model. They are additional views of existing devices and terminals, not duplicate components or manually maintained copies of their wiring.

Separate contact geometry, physical terminal mapping, project-specific signal assignments, and label presentation. Record the source orientation, contact type, and keying explicitly. Changing viewing direction must preserve contact identities and assignments rather than renumbering by screen position.

Custom creation should use the same drawing, properties, styles, and undo/redo conventions as the rest of the editor. Preserve project-owned versions of definitions and explicit library updates. Missing mappings and incompatible connector profiles need visible diagnostics.

The existing Python/Tauri Pinout Generator is a reference implementation and potential source of profiles, examples, import formats, and rendering logic. It has not been integrated or selected as a runtime dependency. See [Pinouts and connection labels](connection-labels.md) for the inspected capabilities and proposed workflow.

### Manufacturer parts and representations

Users can assign manufacturer parts to compatible generic devices and explicitly map terminals. Compatibility includes electrical roles and relevant ratings, rather than terminal count alone.

The product direction includes links between an electrical device, schematic symbols, 2D physical representations, 3D assets, and an actual orderable part. Interactive 2D panel layout and 3D assembly editing are separate capabilities and are not assumed to be first-version requirements.

### Future 2D CAD layout and mechanical handoff

These are long-term capabilities to preserve in the architecture, not additions to the initial delivery milestone.

**2D layout using genuine CAD geometry:** Support dimensionally accurate equipment and panel layouts using formats such as DXF. Users should eventually be able to import manufacturer drawings as reusable component representations, place them at true scale, and use ordinary 2D CAD operations such as geometry editing, snapping, layers, dimensions, blocks, trimming, and offsets. DXF export is also an intended exchange capability; supported entities, versions, and fidelity need explicit definition.

Keep physical footprints distinct from schematic symbols and illustrative pinout diagrams. A footprint can share a device identity and manufacturer part while having its own units, origin, mounting reference, geometry, and placement. Preserve imported CAD as structured geometry where supported, rather than reducing it to a raster image. Referenced blocks should remain reusable objects where practical; unsupported entities and conversions must be reported rather than silently discarded. Full DWG compatibility is not implied by DXF support.

**Outward mechanical compatibility:** Provide a neutral handoff suitable for downstream mechanical CAD, including SOLIDWORKS. A proposed minimum package contains STEP model assets, a versioned machine-readable manifest, and an electrical connection table. An assembled STEP export can be an additional capability when physical placement data and suitable geometry support are available.

The manifest should associate stable device/instance identifiers and designations with part identifiers, relative model asset paths, assembly membership, units, coordinate conventions, and any known placement transforms. Repeated instances can reference one shared model asset. Record model-to-device offsets and mounting planes explicitly. A 2D placement alone does not establish a complete 3D placement; missing depth, orientation, models, and transforms must remain explicit rather than being guessed.

The connection table should refer to the same instance and terminal identities, and include source/destination terminals, wire or cable/core identifiers, and applicable conductor attributes. If terminal connection points exist in a 3D model, record their model-local positions and orientation separately from schematic pin locations. Branches, shields, and multiple terminations require a defined schema rather than an assumed single source/destination row for every case.

The STEP geometry and connection data serve different roles. A neutral package does not by itself create native SOLIDWORKS Electrical objects or routing features. A future importer, macro, or adapter could consume the manifest and table; that integration requires separate implementation and validation. Initial compatibility means a documented, usable handoff, not a promised bidirectional synchronization workflow.

Exports should identify the source project revision and report missing assets, incomplete placements, unsupported geometry, and unmapped terminals. Keep required assets portable and include a schema version so future consumers can interpret the package reliably. Asset revisions and hashes can support repeatable exports; deterministic text manifests should avoid unrelated Git churn.

SOLIDWORKS documentation confirms [DXF/DWG import into drawings or sketches](https://help.solidworks.com/2020/English/SolidWorks/slddxf/t_importingdxfdwg.htm) and [STEP import](https://help.solidworks.com/2024/english/SolidWorks/sldworks/t_reading_step_iges_acis_sw.htm). These support the neutral-format direction; the proposed manifest and electrical table remain our own exchange contract, to be validated against a selected downstream workflow and software version.

### Defaults, styles, and overrides

Defaults are a foundational capability that must be designed alongside the document model, symbol system, rendering, and property editor. Users should be able to establish consistent drawing conventions without formatting every object individually.

Cover fonts, text sizes, wire and cable stroke widths, symbol line widths, label placement, and other drawing properties. Keep visual stroke width distinct from physical conductor cross-section and other electrical properties. Creation defaults for electrical attributes may also be useful, but their application must not silently change existing engineering selections.

Distinguish two behaviors explicitly:

- **Creation defaults:** Initial values used when creating objects; changing these defaults does not retroactively alter existing objects.
- **Inherited styles:** Shared presentation values that remain linked to existing objects; changing a style updates the objects that inherit it, while preserving explicit overrides.

The proposed scope is application/user presets for starting projects, portable project defaults and named styles, optional view-level overrides, and explicit object-level overrides. Define precise precedence during requirements work. Personal settings must not silently alter the appearance of an existing shared project. Functional hierarchy should not automatically introduce another style inheritance hierarchy.

Inheritance operates per property: overriding a label's text size should not break its inherited font. The property editor must show the effective value and where it comes from, distinguish inherited values from explicit overrides, and offer a reset to inheritance. An explicit value equal to the inherited value still has different intent and must remain distinguishable.

Library symbols should participate in project styles through defined graphical roles, such as device labels, terminal labels, and symbol outlines. Define how library-specified values interact with those roles, including any geometry or presentation constraints that must be preserved.

Project styles and defaults are saved with the project, reviewed through Git, and editable through undo/redo. Style references should allow a shared style change to produce a focused document diff without rewriting every dependent object. Imported objects need explicit handling for missing or conflicting styles.

Rendering, printing, and export must resolve the same document styles. Font availability and substitution must have a defined portability policy, with missing fonts made visible to users. Theme and selection colours remain separate from drawing properties intended for output.

### Property inspection and bulk editing

Every user-facing document object, including components, wires, cables, symbols, subsystems, and pages/views, exposes Properties through its right-click menu. Provide the same capability through the main property inspector and a keyboard action. A single-object inspection exposes its full applicable property set, organized and searchable, rather than a reduced subset that requires unrelated dialogs to access other properties.

Include electrical attributes, identification, library and manufacturer assignments, presentation, and supported custom properties where applicable. Distinguish editable values from derived or read-only values and explain restrictions. Where a symbol represents a device, make the ownership and scope of properties clear: changing the device's designation and changing this symbol's text size affect different things.

Multi-selection editing supports both common properties and the union of properties available across the selection. A property must remain editable when only some selected objects support it. Editing that property changes the applicable objects and leaves unsupported objects untouched, without requiring users to rebuild the selection.

For example, a selection containing labels and wires can expose text font and size for the labels and stroke width for the wires. Users can change all of these while retaining the selection. An additional object without a font property does not prevent bulk font editing. Compatible properties can also span object types, such as a shared drawing colour where its meaning is consistent.

Show applicability for each property, such as "Font size: 8 of 12 objects", and distinguish uniform values, mixed values, inherited values, and explicit overrides. A mixed value is not a blank value. Objects with the same effective value may still have different inheritance states. Make any restriction to a subgroup or property category visible, and allow users to inspect which objects will be affected.

Combine properties by compatible meaning, type, and units, not merely by matching display names. For example, graphical wire width and physical conductor cross-section must remain separate. Unsupported properties, temporarily unavailable properties, and read-only properties are different states; do not silently treat validation failures as unsupported objects.

Changing an inherited presentation property through the inspector creates explicit overrides on the applicable selection unless the user explicitly chooses to edit a shared style. Bulk reset to inheritance is also available. Editing a shared style must make its wider impact clear because it may affect unselected objects.

Each bulk edit is one coherent undoable operation. Validate the proposed values against the applicable objects before applying the edit; report restrictions and avoid silently leaving a partially changed selection after a failure. The interface should support successive edits to different subsets within one retained selection. Whether several pending edits can be applied as one compound operation remains a requirements decision.

Define property metadata as a foundational contract shared by inspection, editing, validation, and persistence. It must describe stable property identity, meaning, type, units, applicability, editability, constraints, ownership, and inheritance behavior. The exact implementation remains an architecture decision.

### Desktop workspace and UI consistency

The interface should behave like a focused desktop engineering tool: a central drawing workspace surrounded by predictable menu, toolbar, dock/pane, and status regions. Content changes must not incidentally resize the surrounding workspace. Deliberate window resizing, splitter movement, pane visibility changes, and accessibility settings may change its geometry.

The application shell owns pane geometry and overflow rules. Individual tools supply content within those boundaries. Define initial, minimum, and maximum pane sizes; explicit resize behavior; internal scrolling; and stable scrollbar space where supported. Long identifiers, large numeric values, loading states, and validation messages must not force panes wider or move the drawing canvas. Full values remain accessible through editing or inspection when display text is truncated.

Build shared UI primitives for pane headers, toolbars, property rows, typed editors, tree views, tables, context menus, dialogs, and status reporting. Centralize spacing, typography, icon sizes, row heights, focus indicators, and interaction states. Features compose these controls rather than inventing their own sizing and interaction conventions. Application UI tokens remain separate from document drawing styles.

Property inspectors use consistent label/value columns and defined handling for mixed values, units, inheritance controls, validation, and long content. Ordinary content updates should preserve active edits, keyboard focus, and scroll position. Menus, shortcuts, toolbar actions, and context menus should share action definitions so availability and behavior remain consistent.

Use a reserved status region for routine feedback and an accessible notification area or overlay for transient messages. Notifications and background progress must not insert new layout rows that displace the canvas. Detailed errors can use a dedicated problems pane or an explicitly opened dialog. Avoid covering the current editing target with transient feedback.

Workspace layout preferences, pane visibility, and splitter positions are user/workspace state, separate from the shared electrical document. Provide a reset-layout action. Full docking, floating windows, and saved layout presets remain scope decisions; a stable split-pane workspace can establish the initial behavior.

Support keyboard navigation, visible focus, text scaling, and high-DPI displays. Stable geometry does not mean clipping essential content or disabling accessibility scaling. Define a minimum supported window size and intentional behavior when available space is constrained.

Validate the shell early with long names and values, empty and large selections, mixed properties, missing data, validation errors, progress updates, notifications, and different display scales. Check geometry stability and interaction continuity, rather than relying only on static screenshots. Confirm behavior on each supported operating system and webview before finalizing the UI stack.

### Keyboard control and action mapping

Make frequent CAD operations directly accessible through mnemonic single keys in the drawing context, with `W` for wire as an initial candidate. Define all eligible user-facing operations as named, discoverable, bindable actions from the beginning. This covers tools and operations, not a separate action for every property value or internal function.

Use one action registry for menus, toolbars, context menus, keyboard bindings, and a searchable command palette. Give each action a stable identity, applicability/enabled rules, required context, and binding policy. Keyboard invocation must use the same behavior and validation as other entry points.

Support modifier combinations, multi-key sequences, contextual scopes, user remapping, conflict diagnostics, and import/export/reset of keymaps. Preserve essential editing and accessibility conventions. Drawing shortcuts must not fire while typing into property fields or during text composition. Define explicit behavior for selection versus hovered targets, active tools, dialogs, cancellation, and key repeat.

New actions must not require adding bespoke keyboard listeners or expanding a hard-coded table with a fixed action limit. Every eligible action must be assignable even if its default access is through search or a menu rather than a dedicated shortcut. Keymaps are user/workspace preferences, not project drawing settings. See [Keyboard and action model](keyboard-actions.md) for the detailed draft and proposed verification cases.

### Document editing and persistence

Users can create, open, save, and reopen complete projects. User-visible editing actions support coherent undo and redo, including actions affecting several objects or views.

An unchanged document saves without producing spurious Git differences. Objects have stable identifiers, serialization is deterministic, and document schema versions are explicit. Electrical data and drawing placement should be independently reviewable where practical.

Project safety includes atomic or recoverable saves, detection of external changes, and a defined recovery strategy. Undo-history persistence across restarts remains an open decision.

### Git workflow

Use Git as the version-control system and expose its operations through the application. Maintain interoperability with ordinary Git tools.

The initial integrated workflow should support repository initialization or opening, repository status, change review, explicit commits, history, and local branch creation and switching. Saving and committing are separate operations.

The application must handle unsaved documents and externally changed files when repository operations affect the working tree. It must detect unresolved conflicts and avoid loading or saving a conflicted project as though it were valid.

Text diffs can provide the initial review experience. Graphical and electrical-aware diffs, guided merge resolution, and remote authentication/synchronization are later candidates. A successful text merge still requires model validation.

### Validation and outputs

Start with checks for broken references, invalid terminal mappings, unresolved hierarchy interfaces, and incomplete or inconsistent conductor assignments. Requirements must distinguish intentional incompleteness from errors.

The intended product should provide useful engineering outputs such as cable schedules, wire lists, terminal connection lists, bills of materials, and printable drawings. The first deliverable and its export format must be selected during requirements work.

### Testing and platform support

Linux, Windows, and macOS are intended desktop targets. Select minimum OS versions, Linux distributions/display environments, CPU architectures, and package formats during implementation planning. Supporting a platform requires native execution and packaging checks, not only successful cross-compilation.

Maintain locally runnable unit, regression, and integration suites. Core and adapter tests should not require a GitHub account, remote repository, or network service. Git integration tests use temporary local repositories. GitHub is an intended source backup/version-control destination; hosted checks may supplement local verification later but must not be its only execution path.

Test document invariants, commands and undo/redo, property/style behavior, deterministic serialization, recovery, library mappings, and electrical connectivity. Preserve representative project fixtures and add regression cases for substantive defects. Include native desktop workflow and layout checks as the editor develops.

Add locally reproducible scalability experiments using representative car-scale designs, controlled synthetic growth, and bounded stress/soak workloads. Measure usability as well as throughput: interaction latency, frame times, memory, undo growth, loading, saving, and complete validation. Use measured growth and bottlenecks to refine the engine and set supported limits; verify electrical correctness throughout. See [scalability experiments](testing-strategy.md#scalability-experiments).

See [Testing and platform strategy](testing-strategy.md) for suite boundaries, local check profiles, and cross-platform verification requirements. Local core checks and a bounded headless scale probe are implemented; native desktop testing, Linux/Windows execution, and CI remain pending.

## Initial boundaries

Prioritize a reliable electrical model, hierarchical editing, coordinated views, foundational defaults and styles, complete property inspection and bulk editing, and a usable local Git workflow.

Defer the following unless requirements establish that they are essential:

- Full 3D mechanical assembly and cable routing.
- Advanced 2D cabinet layout and manufacturing preparation.
- Real-time multi-user editing and cloud collaboration infrastructure.
- Hosted manufacturer catalogues, procurement integrations, and automatic part matching.
- Automatic wire routing, automatic cable sizing, and engineering calculations.
- Broad import/export compatibility with established proprietary CAD formats.
- Comprehensive standards-specific drawing and compliance rule sets.

The future 2D CAD layout and neutral mechanical handoff described above remain deferred. Preserve stable identities, representation boundaries, explicit units and coordinate frames, and extensible asset references now; choose CAD kernels and exchange libraries when those workflows become active requirements.

## First end-to-end milestone

Design a small control system containing a controller and a nested conveyor subsystem with a sensor and a junction terminal device.

1. Create a project and place generic devices with explicit terminals.
2. Define the conveyor's external interface and internal connections.
3. Draw an overview cable from the controller to the conveyor subsystem.
4. Open a detailed view, select a cable definition, and assign its conductors to terminals.
5. Verify that overview and detail agree after changes from either view.
6. Assign at least one manufacturer part using an explicit terminal mapping.
7. Undo and redo a meaningful connectivity change across the affected views.
8. Save and reopen the project with identities and connections preserved.
9. Review and commit the project through the integrated Git interface.
10. Change a conductor assignment, inspect its diff, and save again without unrelated changes appearing.
11. Create a local branch, make and commit a design change, then switch back and verify the earlier model is restored.
12. Produce a basic cable or connection list from the model.
13. Change a project text style and wire stroke width, verify inherited objects update while explicit overrides remain, reset an override to inheritance, and undo/redo the changes. Verify that saving and reopening preserves the result and that the style change produces a focused Git diff.
14. Inspect the full properties of a device, symbol, wire, and page through their context menus. Select labels, wires, and an object without text properties together; change font size for the labels and stroke width for the wires while retaining the selection. Verify applicability counts, mixed-value display, preservation of unsupported objects, validation of invalid values, and undo/redo of each bulk edit.

Success means the model stays coherent through editing, persistence, and Git operations. A large catalogue or polished 3D presentation is not required to demonstrate this milestone.

## Requirements to resolve next

1. **Primary users and jobs:** For the car-scale equipment target, what is a representative combination of components, cables, connector assemblies, and reusable subsystems? What build documentation is required? Building services are outside the initial target.
2. **Electrical semantics:** How should nets, wire segments, cable conductors, shields, splices, terminal bridges, connectors, and hierarchical interfaces relate?
3. **Hierarchy behavior:** Which interfaces can a subsystem expose? How should repeated assemblies, device identifiers, and cross-references behave?
4. **Editing experience:** Which placement, wiring, snapping, selection, annotation, and keyboard workflows are essential? How are objects shown in multiple views?
5. **Libraries and parts:** Which attributes are required? How are symbols authored, parts mapped, library updates applied, and missing definitions handled?
6. **Document and Git behavior:** What are the serialization boundaries, undo units, save/recovery rules, branch-switch behavior, and initial conflict-handling expectations?
7. **Outputs and conventions:** Which reports, drawing conventions, identifier schemes, units, and export formats are needed first?
8. **Operating constraints:** With Linux, Windows, and macOS as targets, which OS versions, Linux distributions/display environments, CPU architectures, and package formats are supported? What project sizes, offline workflows, and interaction performance targets must be covered?
9. **Defaults and styles:** Which properties use creation defaults versus live inheritance? What is the precedence between project styles, library values, view overrides, and object overrides? How should presets, imports, missing fonts, and bulk resets behave?
10. **Properties and selection:** How are complete property sets grouped and searched? How are compatible properties combined across object types, applicability and restrictions shown, and subgroup edits targeted? Should edits apply immediately or support a compound Apply operation? How do custom properties participate?
11. **Desktop workspace:** Which panes and docking capabilities are essential initially? What are the minimum window size, density and scaling expectations, shared control conventions, notification behavior, and measurable layout-stability criteria?
12. **Drawing workflows:** What are the selection levels, snap priorities, route adjustment rules, array behaviors, image formats, and initial geometric constraints? Which operations preserve connectivity, and how are intentional connection changes previewed?
13. **Pinouts and connection labels:** Which connector profiles and label layouts are needed first? How are contacts mapped to terminals, viewing directions defined, wiring assignments displayed, and existing generator documents imported? Which bound fields can be edited through the label?
14. **Future CAD exchange:** Which DXF entities and layout operations matter first? Should the first mechanical handoff provide individual STEP assets, an assembled STEP, or both? What placement, terminal-point, and connection data must a downstream SOLIDWORKS workflow consume, and will it need a custom adapter?
15. **Keyboard controls:** What are the initial mnemonic bindings, active-tool and focus contexts, target-selection rules, and platform conventions? Which actions need default direct shortcuts versus key sequences or palette access?
16. **Hierarchy and shared revisions:** Which edits are instance parameters versus structural project-local revisions? Where are shared definitions stored, how is known usage indexed, and how are publication, default selection, and per-project updates reviewed? What measured depth and expanded-instance limits are appropriate?

## Architecture work after requirements

See [Architecture direction](architecture.md) for the proposed module boundaries and contracts, and [Coding standards](coding-standards.md) for documentation, source-size guidance, and verification conventions. These establish maintainability constraints now while leaving detailed architecture decisions open.

Use representative workflows and sample projects to specify the domain model and invariants first, including defaults, style references, explicit overrides, the property metadata and bulk-editing contract, and structured drawing objects. Then define the command/undo model, persistence boundaries, library resolution, Git lifecycle, rendering approach, and frontend/core interface.

Prototype the highest-risk assumptions: navigating and editing a large drawing smoothly; preserving connectivity across hierarchy and views; and producing stable, understandable project diffs. Use those results to confirm Tauri, the rendering technology, and any SQLite role before committing to the full implementation.
