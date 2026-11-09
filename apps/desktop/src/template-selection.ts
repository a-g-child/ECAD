/** One workspace selection; controllers retain only gesture-local state or rendering projections. */
import type { TemplateVariant } from './types';
export type TemplateSelection =
  | { kind: 'paper' }
  | { kind: 'cells'; table: string; cells: string[] }
  | { kind: 'artwork'; id: string }
  | { kind: 'artwork-catalog' }; // Transitional numeric insertion surface, not an object selection.
export function selectedTable(model: TemplateVariant | null, selected: TemplateSelection) {
  if (selected.kind !== 'cells') return null;
  const table = model?.tables.find((t) => t.id === selected.table);
  const cells = table?.cells.filter((c) => selected.cells.includes(c.id)) ?? [];
  return table && cells.length ? { table, cells } : null;
}
export function reconcileSelection(
  model: TemplateVariant,
  selected: TemplateSelection,
): TemplateSelection {
  if (selected.kind === 'cells') {
    const resolved = selectedTable(model, selected);
    return resolved
      ? { kind: 'cells', table: resolved.table.id, cells: resolved.cells.map((c) => c.id) }
      : { kind: 'paper' };
  }
  if (selected.kind === 'artwork' && !model.graphics.some((g) => g.id === selected.id))
    return { kind: 'paper' };
  return selected;
}
