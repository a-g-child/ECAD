/** Bridge-v1 read DTOs. Coordinates in the scene are approximate display millimetres only. */
export interface Target {
  kind: string;
  id: string;
}
export interface Revision {
  epoch: string;
  sequence: string;
}
export interface Context {
  editor?: Revision | null;
  revision: Revision;
  selection: Target[];
  view: string | null;
  sheet?: string | null;
  focus: string;
}
export interface TreeRow {
  target: Target;
  label: string;
  parent: Target | null;
  view: string | null;
}
export interface SceneRow {
  target: Target;
  view: string;
  label: string;
  shape: string;
  points: [number, number][];
  colour: string;
  width: number;
  text_height: number;
  font: string;
}
export interface ActionRow {
  id: string;
  label: string;
  enabled: boolean;
  reason: string | null;
  shortcuts: string[];
}
export interface DrawingDocumentRow {
  id: string;
  is_default: boolean;
  number: string;
  title: string;
  revision: string;
  default_template: string;
  sheets: string[];
}
export interface Snapshot {
  template_editor: TemplateView | null;
  template_choices: SavedTemplateRow[];
  drawing_documents: DrawingDocumentRow[];
  sheets: SheetRow[];
  drawing_lists: DrawingListRow[];
  sheet_frames: FrameRow[];
  bridge_version: 1;
  next_input: string;
  revision: Revision;
  name: string;
  path: string | null;
  dirty: boolean;
  views: { id: string; name: string }[];
  objects: TreeRow[];
  scene: SceneRow[];
  actions: ActionRow[];
}
export interface PropertyRow {
  key: string;
  label: string;
  kind: string;
  display: string;
  value: string | null;
  mixed: boolean;
  editable: boolean;
  applicable: number;
  unsupported: number;
  restricted: number;
  origin: string;
  reason: string;
}
export interface MoveRow {
  session: string;
  token: string;
  x: string;
  y: string;
  scene: SceneRow[];
  candidates: number;
  chosen: number | null;
  snap: string | null;
}
export interface TransformRow {
  session: string;
  token: string;
  x: string;
  y: string;
  operation: string;
  scene: SceneRow[];
  valid: boolean;
  changed: boolean;
}
export interface AnnotationRow extends MoveRow {
  text: string;
  valid: boolean;
}
export interface PathRow extends MoveRow {
  vertices: [string, string][];
  rubber: SceneRow | null;
  can_add: boolean;
  can_finish: boolean;
}
export interface DrawingDefaults {
  font: string;
  text_height: string;
  stroke_width: string;
  colour: string;
}
export interface MetadataRow {
  key: string;
  label: string;
  value: string;
}
export interface ProfileRow {
  id: string;
  name: string;
  values: Record<string, string>;
}
export interface ProfileCatalog {
  token: string;
  entries: ProfileRow[];
  fields: MetadataRow[];
}
export interface ProfileImport {
  token: string;
  changes: { key: string; label: string; before: string | null; after: string | null }[];
}
export interface SavedTemplateRow {
  new_assignments: boolean;
  key: string;
  label: string;
  width: string;
  height: string;
  left: string;
  right: string;
  top: string;
  bottom: string;
}
export interface SheetRow {
  inherited_variant: string | null;
  id: string;
  kind: string;
  template: string | null;
  document: string | null;
  number: string;
  title: string;
  width: string;
  height: string;
  left: string;
  right: string;
  top: string;
  bottom: string;
  view: string | null;
  paper_x: string;
  paper_y: string;
  source_x: string;
  source_y: string;
  numerator: string;
  denominator: string;
}
export interface FrameRow {
  sheet: string;
  template: string;
  border: { x: number; y: number; width: number; height: number };
  cells: { key: string; caption: string; value: string | null; bounds: FrameRow['border'] }[];
  title_block_too_small: boolean;
  font: string;
  text_height: number;
  stroke_width: number;
  colour: string;
}
export interface TemplateDefaults {
  schematic: string;
  cover: string;
  drawing_list: string;
}
export interface TemplateCatalog {
  defaults: TemplateDefaults;
  choices: [string, string][];
}
export interface DrawingListRow {
  sheet: string;
  x: number;
  y: number;
  width: number;
  height: number;
  row_height: number;
  capacity: string;
  ordinal: string;
  page_count: string;
  total_entries: string;
  remaining: string;
  rows: { sheet: string; document: string; number: string; title: string }[];
}
export interface SheetPreview {
  token: string;
  sheet: SheetRow;
  frame: FrameRow;
  list: DrawingListRow | null;
  scene: SceneRow[];
}
export interface Reply {
  table_preview?: { token: string; tables: TemplateTable[] };
  project_templates: TemplateCatalog | null;
  activate_sheet: string | null;
  sheet_draft: SheetRow | null;
  sheet_preview: SheetPreview | null;
  profiles: ProfileCatalog | null;
  profile_import: ProfileImport | null;
  metadata: MetadataRow[] | null;
  defaults: DrawingDefaults | null;
  path: PathRow | null;
  annotation: AnnotationRow | null;
  transform: TransformRow | null;
  movement: MoveRow | null;
  snapshot: Snapshot | null;
  properties: PropertyRow[] | null;
  report: string[][] | null;
  request: string | null;
  message: string | null;
}
export const keyOf = (target: Target) => `${target.kind}:${target.id}`;
/** Fail closed on incompatible bridge versions rather than rendering an obsolete document. */
export function validateReply(value: unknown): Reply {
  if (!value || typeof value !== 'object') throw new Error('Invalid bridge response');
  const reply = value as Reply;
  if (reply.snapshot) {
    const s = reply.snapshot;
    if (
      s.bridge_version !== 1 ||
      typeof s.revision?.epoch !== 'string' ||
      typeof s.revision?.sequence !== 'string' ||
      !Array.isArray(s.drawing_documents) ||
      !Array.isArray(s.sheets) ||
      !Array.isArray(s.sheet_frames) ||
      !Array.isArray(s.drawing_lists) ||
      !Array.isArray(s.objects) ||
      !Array.isArray(s.scene) ||
      !Array.isArray(s.actions)
    )
      throw new Error('Incompatible document projection');
    for (const object of [...s.objects, ...s.scene])
      if (!/^[0-9a-f]{32}$/.test(object.target.id)) throw new Error('Invalid object identity');
    for (const row of s.scene)
      if (row.points.some((p) => p.length !== 2 || !p.every(Number.isFinite)))
        throw new Error('Invalid scene coordinate');
  }
  return reply;
}

export interface TemplateRun {
  kind: 'literal' | 'field';
  value: string;
}
export interface TemplateCell {
  id: string;
  caption: string;
  bounds: string[];
  contents: TemplateRun[];
}
export interface TemplateTable {
  id: string;
  bounds: string[];
  cells: TemplateCell[];
}
export interface TemplateVariant {
  id: string;
  name: string;
  paper: string[];
  content: string[];
  tables: TemplateTable[];
}
export interface TemplateView {
  revision: Revision;
  definition: string;
  published: string | null;
  selected: string;
  dirty: boolean;
  pending: boolean;
  variants: TemplateVariant[];
  catalog: [string, string][];
  catalog_error: string | null;
}
