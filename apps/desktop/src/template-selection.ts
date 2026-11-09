/** One workspace selection; controllers retain only gesture-local state or rendering projections. */
import type { TemplateVariant } from './types';
export type TemplateSelection =
  | { kind: 'paper' }
  | { kind: 'cells'; table: string; cells: string[]; whole?: boolean }
  | { kind: 'artwork'; id: string }
  | { kind: 'artwork-catalog' }; // Transitional numeric insertion surface, not an object selection.
export function selectedTable(model: TemplateVariant | null, selected: TemplateSelection) {
  if (selected.kind !== 'cells') return null;
  const table = model?.tables.find((t) => t.id === selected.table);
  const cells =
    (selected.whole ? table?.cells : table?.cells.filter((c) => selected.cells.includes(c.id))) ??
    [];
  return table && cells.length ? { table, cells } : null;
}
export function reconcileSelection(
  model: TemplateVariant,
  selected: TemplateSelection,
): TemplateSelection {
  if (selected.kind === 'cells') {
    const resolved = selectedTable(model, selected);
    return resolved
      ? {
          kind: 'cells',
          table: resolved.table.id,
          cells: resolved.cells.map((c) => c.id),
          whole: selected.whole,
        }
      : { kind: 'paper' };
  }
  if (selected.kind === 'artwork' && !model.graphics.some((g) => g.id === selected.id))
    return { kind: 'paper' };
  return selected;
}

/** Resolve an explicit pointer selection before asynchronous field completion replaces DOM nodes. */
export function selectionAt(model: TemplateVariant, target: Element): TemplateSelection | null {
  const tableId = target.closest<HTMLElement>('[data-template-table]')?.dataset.templateTable;
  const table = model.tables.find((t) => t.id === tableId);
  if (table)
    return { kind: 'cells', table: table.id, cells: table.cells.map((c) => c.id), whole: true };
  const cellNode = target.closest<HTMLElement>('[data-template-cell],.frame-cell');
  const cellId = cellNode?.dataset.templateCell ?? cellNode?.dataset.field;
  if (cellId) {
    const table = model.tables.find((t) => t.cells.some((c) => c.id === cellId));
    if (table) return { kind: 'cells', table: table.id, cells: [cellId] };
  }
  const artwork = target.closest<HTMLElement>('[data-template-graphic],[data-template-artwork]');
  const id = artwork?.dataset.templateGraphic ?? artwork?.dataset.templateArtwork;
  if (id && model.graphics.some((g) => g.id === id)) return { kind: 'artwork', id };
  if (
    target.closest('[data-template-paper]') ||
    (target.closest('.template-preview') && !target.closest('.table-handles'))
  )
    return { kind: 'paper' };
  return null;
}
