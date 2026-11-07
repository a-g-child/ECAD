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
  revision: Revision;
  selection: Target[];
  view: string | null;
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
export interface Snapshot {
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
export interface Reply {
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
