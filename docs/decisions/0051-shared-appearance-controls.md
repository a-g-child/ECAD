# 0051 — Shared appearance controls

Status: implemented for existing font-family and colour fields.

## Decision

Use one frontend appearance control in template artwork properties, schematic drawing properties and drawing defaults. Provide generic sans-serif/serif/monospace choices and a small named RGB palette alongside the exact editable value. Custom font requests and #rrggbb values remain supported; choosing Custom does not clear or reset a value. Opening or rendering a control never writes an override. Mixed schematic values remain visibly mixed until a deliberate choice.

The control owns presentation and input intent only. Template fields use captured contextual validation/history, schematic fields use existing typed property commands and applicability rules, and defaults remain staged until Apply. Selecting a schematic preset is an explicit property edit, including when its value matches an inherited value. Undo restores the original ownership. Read-only properties do not get selectors. Moving focus between a template's exact field and preset selector does not complete the field prematurely; leaving the composite field or making a choice follows normal completion.

Font requests stay multiline, including template artwork. Presets do not enumerate installed fonts or guarantee identical font metrics across platforms. Colour presets are opaque RGB values, not electrical conductor colour names. Shared layout rules bound controls inside their existing panes.

## Boundaries and revisit conditions

No persistence, bridge or domain style fields change. Font weight, italic, text orientation, advanced colour selection and table-cell typography remain future work. Add new style capabilities through domain validation, explicit persistence evolution, projections and rendering together. Expand this module when another editor needs existing appearance fields; do not flatten inherited schematic styles into template-style explicit values.

## Verification

Real bridge/browser workflows cover inherited schematic font editing and undo, mixed drawing colour edits, default cancellation, exact custom multiline font requests, invalid template field repair, one-step template history and fixed canvas/pane bounds. Existing default validation/persistence and template save/reopen tests remain part of the full browser suite. Native font/output parity and Windows/Linux runtime checks remain separate work.
