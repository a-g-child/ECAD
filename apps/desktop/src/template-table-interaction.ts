/** Pointer selection/manipulation over immutable table snapshots; previews are validated in Rust. */
import type { Context, Reply, TemplateVariant, TemplateTable, TemplateCell } from './types';
import { TemplatePreview } from './template-preview';
import { TemplateGesture } from './template-gesture';
import { cellAt, cellRange, paperPoint, type TableSelection } from './template-table-selection';
import { marginSnapGuides, tableHandles } from './template-table-handles';
type Point = { x: number; y: number };
type Drag =
  | { kind: 'select'; table: TemplateTable; first: TemplateCell; restore: () => void }
  | {
      kind: 'move' | 'edge';
      table: TemplateTable;
      start: Point;
      axis: string;
      from: number;
      outer: string;
      tolerance: number;
      moved: boolean;
    };
export class TemplateTableInteraction {
  private model: TemplateVariant | null = null;
  private selected: TableSelection | null = null;
  private shown: TemplateTable[] | null = null;
  private invalid = false;
  private snapped: string[] = [];
  private drag: Drag | null = null;
  private gesture: TemplateGesture;
  get active() {
    return this.drag !== null || this.gesture.active;
  }
  constructor(
    private preview: TemplatePreview,
    context: () => Context,
    apply: (r: Reply) => Promise<void>,
    private select: (table: TemplateTable, cells: TemplateCell[]) => boolean,
    private canStart: () => boolean,
    private changed: () => void,
    private status: (s: string, error?: boolean) => void,
    private captureSelection: () => () => void,
  ) {
    this.gesture = new TemplateGesture(
      context,
      (tables, invalid, snapped) => {
        this.snapped = snapped;
        this.shown = tables;
        this.invalid = invalid;
        if (this.model)
          this.preview.render({ ...this.model, tables: tables ?? this.model.tables }, false);
      },
      apply,
      () => {
        this.preview.locked = this.active;
        this.changed();
      },
      status,
    );
    preview.onRender = () => this.paint();
    const svg = preview.svg;
    svg.addEventListener('pointerdown', (e) => this.down(e));
    svg.addEventListener('pointermove', (e) => this.move(e));
    svg.addEventListener('pointerup', (e) => void this.up(e));
    svg.addEventListener('pointercancel', () => void this.cancel());
    svg.addEventListener('lostpointercapture', () => {
      if (this.drag) void this.cancel();
    });
    svg.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      if (this.active || !this.canStart()) return;
      const hit = this.hit(e);
      if (hit && !this.selected?.cells.some((c) => c.id === hit.cell.id))
        this.choose(hit.table, [hit.cell]);
      if (hit) this.preview.properties();
    });
    svg.addEventListener('keydown', (e) => {
      const control = (e.target as Element).closest<SVGGElement>('[data-control]');
      if (control && (e.key === 'Enter' || e.key === ' ')) {
        e.preventDefault();
        e.stopPropagation();
        void this.plus(control.dataset.control!, control.dataset.table!);
      }
    });
  }
  update(model: TemplateVariant, selection: TableSelection | null) {
    this.model = model;
    this.selected = selection;
    if (!this.gesture.active) {
      this.shown = null;
      this.invalid = false;
      this.snapped = [];
    }
    this.paint();
  }
  selection(table: TemplateTable, cells: TemplateCell[]) {
    this.selected = { table, cells };
    this.paint();
  }
  paint() {
    this.preview.svg.querySelector('.table-handles')?.remove();
    this.preview.svg.querySelector('.margin-snap-guides')?.remove();
    if (this.model && this.gesture.active)
      marginSnapGuides(this.preview.svg, this.model.paper, this.snapped);
    if (!this.selected) return;
    const table = (this.shown ?? this.model?.tables)?.find((t) => t.id === this.selected?.table.id);
    if (!table) return;
    const ids = new Set(this.selected.cells.map((c) => c.id));
    tableHandles(
      this.preview.svg,
      table,
      table.cells.filter((c) => ids.has(c.id)),
      this.invalid,
      this.active,
    );
  }
  private choose(table: TemplateTable, cells: TemplateCell[]) {
    if (!this.select(table, cells)) return false;
    this.selected = { table, cells };
    this.paint();
    return true;
  }
  private hit(e: { clientX: number; clientY: number }) {
    const p = paperPoint(this.preview.svg, e);
    if (!p) return null;
    for (const table of this.model?.tables ?? []) {
      const cell = cellAt(table, p.x, p.y);
      if (cell) return { table, cell };
    }
    return null;
  }
  private down(e: PointerEvent) {
    if (e.button !== 0 || this.active || !this.canStart()) return;
    const svg = this.preview.svg;
    const p = paperPoint(svg, e);
    if (!p) return;
    const control = (e.target as Element).closest<SVGGElement>('[data-control]');
    if (control && control.dataset.control !== 'move') {
      e.preventDefault();
      svg.focus();
      void this.plus(control.dataset.control!, control.dataset.table!);
      return;
    }
    const edge = (e.target as Element).closest<SVGLineElement>('.table-edge');
    const table = this.model?.tables.find(
      (t) => t.id === (control?.dataset.table ?? edge?.dataset.table),
    );
    if (table && (control || edge)) {
      const kind = control ? 'move' : 'edge';
      const axis = edge?.dataset.axis ?? '';
      const outer = edge?.dataset.outer ?? '';
      const action = kind === 'move' || outer ? 'template.table.resize' : 'template.table.boundary';
      if (!this.gesture.begin(this.model!.id, table.id, action)) return;
      const matrix = svg.getScreenCTM();
      const scale = Math.hypot(matrix?.a ?? 1, matrix?.b ?? 0);
      const tolerance = Math.min(4294967295, Math.ceil(8000 / Math.max(scale, Number.EPSILON)));
      this.drag = {
        kind,
        table,
        start: p,
        axis,
        from: Number(edge?.dataset.from ?? 0),
        outer,
        tolerance,
        moved: false,
      };
      this.propose(p, e.altKey);
    } else {
      const hit = this.hit(e);
      if (!hit) return;
      const restore = this.captureSelection();
      if (!this.choose(hit.table, [hit.cell])) return;
      this.drag = { kind: 'select', table: hit.table, first: hit.cell, restore };
    }
    e.preventDefault();
    svg.focus();
    svg.setPointerCapture(e.pointerId);
    this.preview.locked = true;
    this.changed();
  }
  private move(e: PointerEvent) {
    if (!this.drag) return;
    if (this.drag.kind === 'select') {
      const p = paperPoint(this.preview.svg, e);
      if (!p) return;
      // Clamping outside a table lets the user sweep a complete row/column without losing the range.
      const [x, y, w, h] = this.drag.table.bounds.map(Number);
      const last = cellAt(
        this.drag.table,
        Math.max(x, Math.min(x + w - 1, p.x)),
        Math.max(y, Math.min(y + h - 1, p.y)),
      );
      if (last) this.choose(this.drag.table, cellRange(this.drag.table, this.drag.first, last));
    } else {
      const p = paperPoint(this.preview.svg, e);
      if (p) this.propose(p, e.altKey);
    }
  }
  private propose(p: Point, bypass: boolean) {
    const drag = this.drag;
    if (!drag || drag.kind === 'select') return;
    const b = drag.table.bounds.map(Number),
      dx = p.x - drag.start.x,
      dy = p.y - drag.start.y;
    drag.moved ||= dx !== 0 || dy !== 0;
    const snapTolerance = drag.moved && !bypass ? String(drag.tolerance) : null;
    if (drag.kind === 'move') {
      this.gesture.propose({
        operation: 'pointer',
        drag: { kind: 'move', x: String(b[0] + dx), y: String(b[1] + dy) },
        snap_tolerance: snapTolerance,
      });
      return;
    }
    const delta = drag.axis === 'x' ? dx : dy;
    if (!drag.outer) {
      this.gesture.propose({
        operation: 'boundary',
        axis: drag.axis,
        from: String(drag.from),
        to: String(drag.from + delta),
      });
      return;
    }
    const origin = drag.axis === 'x' ? 0 : 1;
    const edge =
      drag.axis === 'x'
        ? drag.outer === 'start'
          ? 'left'
          : 'right'
        : drag.outer === 'start'
          ? 'top'
          : 'bottom';
    const coordinate = b[origin] + (drag.outer === 'start' ? 0 : b[origin + 2]) + delta;
    this.gesture.propose({
      operation: 'pointer',
      drag: { kind: 'resize', edge, coordinate: String(coordinate) },
      snap_tolerance: snapTolerance,
    });
  }

  private async up(e: PointerEvent) {
    if (e.button !== 0 || !this.drag) return;
    if (this.drag.kind !== 'select') {
      const p = paperPoint(this.preview.svg, e);
      if (p) this.propose(p, e.altKey);
    }
    this.drag = null;
    if (this.preview.svg.hasPointerCapture(e.pointerId))
      this.preview.svg.releasePointerCapture(e.pointerId);
    if (this.gesture.active) await this.gesture.finish(true);
    this.preview.locked = false;
    this.paint();
    this.changed();
  }
  async cancel() {
    if (!this.active) return false;
    const drag = this.drag;
    this.drag = null;
    if (drag?.kind === 'select') drag.restore();
    await this.gesture.finish(false);
    this.preview.locked = false;
    this.paint();
    this.changed();
    return true;
  }
  private async plus(kind: string, tableId: string) {
    if (this.active || !this.canStart() || !this.model || !['row', 'column'].includes(kind)) return;
    const table = this.model.tables.find((t) => t.id === tableId);
    if (!table) return;
    const selected = this.selected?.table.id === tableId ? this.selected.cells : table.cells;
    const axis = kind === 'row' ? 'y' : 'x',
      index = axis === 'x' ? 0 : 1;
    const at = Math.max(
      ...selected.map((c) => Number(c.bounds[index]) + Number(c.bounds[index + 2])),
    );
    const size = Math.min(
      ...selected
        .filter((c) => Number(c.bounds[index]) + Number(c.bounds[index + 2]) === at)
        .map((c) => Number(c.bounds[index + 2])),
    );
    if (this.gesture.begin(this.model.id, table.id, `template.table.insert_${kind}`)) {
      this.gesture.propose({
        operation: 'insert',
        axis,
        at: String(at),
        size: String(size),
        anchor: 'end',
        fit: kind === 'column',
      });
      await this.gesture.finish(true);
      this.paint();
      this.preview.svg.querySelector<SVGGElement>(`[data-control="${kind}"]`)?.focus();
    }
  }
}
