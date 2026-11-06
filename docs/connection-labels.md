# Pinouts and connection labels

Status: proposed requirements based on inspection of the existing Pinout Generator. No integration or code migration has been performed.

## Existing work reviewed

Source project: `/Users/andrew/dev/python/Pinout Generator`.

The supplied `__pycache__` path is a generated Python cache; the editable implementation is in the parent project. Inspection covered the README, document serialization/validation, connector layout model, connection-label test source, an example JSON document, and its rendered PNG. Existing tests were read, not executed, and the application was not launched.

Relevant existing capabilities include:

- A Tauri desktop editor with an SVG drawing canvas and Python-backed rendering/export.
- Built-in connector arrangements, including circular and rectangular families, plus editable custom profiles.
- Contact identifiers, pin/socket graphics, shell outlines, keys and orientation marks.
- Reference viewing direction, contact gender, and rotation metadata.
- Pin assignment descriptions and colours, with an editable table and inspector.
- Versioned JSON documents preserving geometry, identifiers, label options, and editing counters.
- Described pin tables, blank SWE connection-label templates, and symbol/custom export layouts.
- Connector name, path, description, and per-contact assignment fields.
- SVG/PNG exports and metadata identifying reserved label-field positions.
- Validation and test coverage in source for identifiers, label settings, orientation, document round trips, and export behavior.

The README explicitly describes SWE exports as static artwork/templates, without native SWE electrical bindings. Native CAD integration should use the source model and definitions where practical rather than relying on generated image pixels as editable electrical data.

These observations establish useful prior work, not verified compatibility with the future CAD document model or confirmation of manufacturer accuracy for every bundled profile.

## Proposed model

Separate four responsibilities:

| Responsibility | Contents |
| --- | --- |
| Connector profile | Contact arrangement and identifiers, outlines/keying, reference orientation, source metadata, and schematic/physical-scale designation. |
| Device terminal mapping | Explicit association of profile contacts with the actual device's terminal identities. |
| Project assignments | Device designation, signal descriptions, wire/conductor assignments, and other engineering values owned by the electrical model. |
| Label template and placement | Diagram/table composition, field bindings, columns, styles, orientation, scale, position, and local presentation overrides. |

The same connector profile can serve different devices and different signal assignments. Selecting a geometry must not infer a protocol or electrical pin assignment. A connection label is an additional representation of an existing device, so placing another label does not add another component to the bill of materials.

Generic devices can use profiles before any manufacturer part is assigned. Assigning a manufacturer part later requires mapping/compatibility checks and must not silently replace a custom arrangement or reassign its contacts.

## Library and custom workflows

1. Select a device or connector and choose **Place connection label**.
2. Select an available profile or create a project-local profile from blank geometry or a copied definition.
3. Map contacts to device terminals and resolve missing or duplicate mappings.
4. Select a template: face diagram, terminal table, or a combined presentation.
5. Choose viewing direction and orientation, preview, and place it on a sheet.
6. Adapt presentation through the standard property inspector and drawing tools.
7. Optionally save the custom profile/template to a reusable library with its source and version metadata.

Create project-local definitions without forcing global library publication. Adapting a shared definition creates a deliberate local revision or copy; it does not mutate unrelated projects. Projects retain the versions they use and adopt library updates explicitly.

Profiles and templates are reusable independently. For example, one company-standard table layout can display many connector families, and one connector arrangement can appear in several documentation layouts.

## Live values and editing

Bind displayed values to stable model identities, not formatted text matches. Device renames, conductor reassignment, and hierarchy changes update every affected label.

Candidate table fields include contact identifier, terminal designation, function/description, connected wire identifier and colour, cable/core identifier, and destination. Select the initial fields during requirements work. Multiple wires or destinations at a terminal need an explicit representation rather than an arbitrary single value.

Keep all contacts available, including unassigned contacts, with an explicit filtering option. Distinguish an unassigned contact from a broken terminal mapping or an unresolved binding. Sort numeric and lettered identifiers predictably without changing their identities.

Editing a bound assignment through a label, if supported, is an electrical-model command with normal validation and undo. Presentation text overrides must be distinguishable from authoritative electrical values. Derived fields should explain their source and remain read-only unless a meaningful underlying edit is available.

Blank templates or unpopulated exports may be useful, but hiding values for output must not clear assignments in the model. Static SVG/PNG export is a snapshot and should be distinguishable from a live placed label.

## Orientation and geometry

Record the reference face explicitly, such as mating face or wiring side, with contact type and orientation/key references. Derive supported alternate views through a defined profile transform. Do not assume that every connector family or gender change follows a universal mirroring rule.

Rotating or mirroring a presentation preserves contact-to-terminal bindings. Keep text readable according to a defined annotation policy and always make the viewed face clear. Do not allow a decorative flip to masquerade as a verified opposite-face pinout.

Distinguish schematic geometry from dimensionally accurate geometry. A connector diagram is not automatically a manufacturing drawing or mounting footprint. Manufacturer-specific arrangements and numbering require exact profile/source identification, not just matching the contact count.

## Interaction with the wider application

- Use the common right-click Properties entry, full inspector, and bulk editing for compatible presentation properties.
- Inherit fonts, text sizes, line widths, and table conventions from project styles, with explicit overrides and reset behavior.
- Preserve the label as an editable whole while allowing deliberate access to layout elements and contact mappings.
- Define sizing, wrapping, pagination, or overflow behavior for long values and large pin tables; do not silently obscure or truncate essential engineering information in output.
- Include label definitions, mappings, placements, and template versions in undoable document edits and deterministic project files.
- Keep generated rendering artifacts rebuildable where possible, so routine label refreshes do not create unrelated Git changes.
- Diagnose missing contacts, incompatible mappings, deleted devices, and stale references visibly. Deleting a label leaves the device intact; deleting its referenced device needs an explicit affected-label policy.

## Reuse and import assessment

The existing JSON formats, connector profiles, example documents, orientation logic, and rendering tests are candidates for reuse. Evaluate a one-way import adapter before deciding whether to retain the Python renderer, port selected functionality, or use a shared future rendering pipeline.

An importer must distinguish profile geometry from example project assignments. It must explicitly map imported contacts into the CAD terminal model, preserve source metadata, and report unsupported fields. Initial format candidates include versioned editor documents and older custom-profile formats handled by the existing project; supported versions remain to be specified.

Copying or linking SVG/PNG artwork remains useful for reference material, but does not reconstruct a native connector profile or create electrical bindings automatically.

## First acceptance scenario

1. Create a generic seven-terminal connector device and select a suitable library profile.
2. Place a combined face diagram and pin table, then map its contacts to the device terminals.
3. Assign two conductors and leave other terminals unassigned; verify every contact remains represented.
4. Change an assignment from the wiring view and verify the label updates without manual regeneration.
5. Change the label's viewing direction and rotation; verify mappings and assignments remain unchanged.
6. Copy a profile into a custom project-local definition, adjust its geometry, and use it without publishing a library entry.
7. Change a shared text style and confirm inherited label text updates while explicit overrides remain.
8. Save, reopen, and inspect the Git diff; verify geometry, mappings, and live bindings are preserved.
9. Remove a mapped terminal and verify a diagnostic appears rather than silently remapping its contact.

## Decisions to resolve

- Whether connection labels also cover terminal strips and non-connector devices in the first version.
- Initial profile families, import versions, and template authoring tools.
- Which bound fields can be edited through a label and how multiple connections per terminal appear.
- Supported orientation transforms and required profile verification/source metadata.
- Large-table layout rules, blank-template outputs, and the policy for deleted or unmapped objects.
- How much existing implementation to reuse after confirming the CAD architecture.
