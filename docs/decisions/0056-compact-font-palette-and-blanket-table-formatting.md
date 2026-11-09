# 0056 — Compact font palette and blanket table formatting

Replace stacked formatting rows with a shared compact palette: editable font-family suggestions, regular/bold/italic/bold-italic style, physical text height in the current units, bold/italic toggles, native colour picker, alignment and a per-property/all-formatting reset menu. Only supported properties appear; underline, strikeout, scripts, font enumeration and point typography remain outside this slice. Font suggestions are portable names, not a claim that the fonts are installed.

One cell retains independent caption/content palettes. A range or explicit whole-table selection exposes one palette over both channels in every selected cell. Mixed ownership/values remain mixed; changing one field changes only that field. Choosing Regular/Bold/Italic/Bold italic intentionally sets both weight and slant in one edit. Reset returns only the selected properties to inheritance. No text content editor is shown for blanket selection, preventing an ambiguous first-cell-only edit. Template defaults retain independent caption/content palettes.

The explorer provides an explicit whole-table selection entry, also usable by right-click. It uses the existing selection/Properties pipeline and pending-field guards. Whole-table intent is transient UI state; bridge action context contains only the resolved existing cell IDs. That intent resolves against current table membership after structural edits and undo. Even a one-cell table has a blanket palette when explicitly selected as a table.

No domain, storage or history contract changes. The existing v4 cell-style proposal commits complete selections atomically, and existing validation, stale-target protection, inheritance and exact undo remain authoritative. The shared palette receives style references and owner callbacks, keeping it independent of template command dispatch.

Verification covers mixed ranges, both-channel size changes, whole-table font/style edits, preservation of content/unrelated properties, reset, undo/redo and saved revision reopening. Native font chooser/colour-picker behavior still requires platform-specific checks.
