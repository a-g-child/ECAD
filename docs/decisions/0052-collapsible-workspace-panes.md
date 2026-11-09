# 0052 — Collapsible workspace panes

Status: implemented for the project and template desktop views.

## Decision

The shell owns independent explorer/properties visibility. Header buttons collapse each pane into a 28 px labelled rail; the rail reopens it with ordinary button keyboard semantics. The center track receives the freed space. Both editors use the same session state and shared controls. Existing pane contents stay mounted, retaining scroll, selection and exact uncommitted field values. Visibility and camera layout do not enter document history or persistence.

Register `workspace.panes.explorer` and `workspace.panes.properties` as unbound, nonrepeating View actions. Header buttons, rails, View menu and command palette use the existing dispatcher and monotonic input stream. Template dispatch retains its independent editor revision checks. Host replies with unchanged project/editor revisions update input synchronization and shell visibility without rebuilding inspectors. Changed revisions use normal resynchronization.

Explicit Properties requests reveal the inspector. Explorer mode/library focus requests reveal the explorer. Pending template fields and active tools retain existing guards; invalid fields cannot be hidden by collapsing their pane. The buttons expose expanded state and control identity, and keyboard focus transfers to the visible counterpart after dispatch completes.

## Boundaries

Visibility lasts for the current app session and is shared between project/template views. Restart persistence, user-adjustable widths, docking, floating panes and independent per-editor layouts remain future work. Revisit the session preference model when durable workspace preferences are implemented; keep those preferences outside project and template files.

## Verification

Bridge regressions check advertised action availability, transient requests, replay rejection and unchanged document/template state. Browser workflows cover both panes, menu/palette equivalence, keyboard reopening, retained draft text, Properties auto-reveal, shared template visibility, pending-field guards, unchanged paper view coordinates and small-window operation. Existing editing/viewport regressions remain required.
