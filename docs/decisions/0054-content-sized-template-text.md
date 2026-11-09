# 0054 — Content-sized template text

Free template text should grow with its contents rather than require a manually sized clipping box. This supersedes the free-text clipping and box-placement portions of decisions 0045 and 0049; structured table cells retain their explicit geometry and overflow policy.

The browser derives bounds from actual glyph metrics plus semantic line boxes, including empty leading/trailing lines and an empty-text hit area. The shared renderer serves library placeholders and resolved sheet values. Bounds are transient; fonts and project variables can change them without modifying a reusable template or publishing a revision. Text has no automatic wrapping or clipping; paper/export boundaries remain a separate concern.

Keep the existing persisted bounds fields unchanged to avoid rewriting immutable revisions. For text only, width/height are positive legacy hints, not clipping extents or translation limits. Validation checks that the anchor is inside paper; vector geometry retains full paper-bound validation. New pointer text uses a 1 µm extent hint and the initial pointer anchor. Contextual properties expose position, contents and appearance, not width/height. Existing contents, identities, library bodies and exact undo remain intact.

Older application versions still interpret these extents as clipping boxes and can reject moved legacy text whose hint crosses paper. Backward rendering compatibility is not promised; no schema/body rewrite is performed. Revisit the stored representation when introducing portable font metrics, text orientation or print/export layout. Browser measurements do not establish font parity between operating systems. Runtime font loading would require explicit bounds invalidation if downloadable fonts are introduced.

Validation covers glyph/blank-line bounds, font-height changes, unclipped text, empty-text selection, anchor-only translation, and the existing artwork history/save/reopen browser workflow.
