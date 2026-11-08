# 0045 — Retained template artwork and variable text

Templates need free text and coherent vector artwork alongside structured tables. These objects belong to a saved template variant, not to a schematic view or an electrical occurrence.

## Decision

Add stable typed artwork identities and exact integer paper-space geometry to `TemplateLayout`. The initial shapes are multiline text, rectangles, ellipses and ordered polylines (including two-point lines). Text uses the same literal/typed-field runs and resolver as title-block cells. Project sheets resolve bindings from their own portable metadata; library bodies retain bindings and never capture example project values.

Each object owns explicit font, positive text height/line width and RGB colour. Rectangles/ellipses are outlines. Polylines keep all vertices together and allow horizontal/vertical zero-height/width bounds. Bounds and vertices must fit the paper, including its margin area; artwork is not silently translated by the table margin-fit operation. Stroke centre-lines define geometric extents. Text is clipped to its explicit box with an overflow marker and full-content tooltip. Paint order follows retained artwork order, behind title-block tables.

The editor's Artwork action shares the registry, dispatcher and input stream. Its property controls stage additions, edits and removals; Apply submits one complete revision-bound replacement through the existing independent template document history. Revert discards fields. Table edits and legacy clients omitting artwork retain existing objects. UI selection and open property sections remain transient. New shape controls are property-form subcontrols, like adding a text run; they do not mutate the document directly.

Standalone variant schema v3 adds strict artwork records. Artwork-free variants continue encoding as v1/v2 so existing immutable library bodies remain canonical. Old readers reject a v3 body instead of silently discarding artwork. Project v13 and library v1 already contain individually versioned variant bodies and need no outer format change. Persistence, bridge DTOs and domain records stay separate.

## Bounds and revisit conditions

This first authoring slice uses exact property coordinates and a local-point editor, with canvas/explorer access to object properties. Pointer placement/dragging, artwork snapping, transforms, z-order commands, style inheritance, arcs, filled shapes and image/logo assets remain future steps. Image import needs a portable asset contract, not machine-local paths or arbitrary SVG markup. The per-layout 4096-object and per-polyline 4096-vertex limits are defensive bounds, not measured capacity claims. Revisit them with artwork scalability fixtures.

Verify real project sheet resolution, independent history and failure atomicity, table-edit preservation, strict versioning, local publication/reload and shared browser rendering. Runtime platform validation remains separate from host build/lint checks.
