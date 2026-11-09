/** One-shot pointer artwork tools; authoritative previews and closed commits stay in Rust. */
import type { Context, Reply, TemplateGraphic, TemplateVariant } from './types';
import { TemplatePreview } from './template-preview';
import { TemplateGesture } from './template-gesture';
import { paperPoint } from './template-table-selection';
type Point = { x: number; y: number };
type Kind = 'text' | 'line' | 'rectangle' | 'ellipse' | 'move';
export class TemplateArtworkInteraction {
  private model: TemplateVariant | null = null;
  private selected: string | null = null;
  private tool: { kind: Kind; id: string; original: TemplateGraphic | null } | null = null;
  private drag: { start: Point; pointer: number } | null = null;
  private ending = false;
  private gesture: TemplateGesture;
  get active() {
    return this.tool !== null || this.gesture.active;
  }
  constructor(
    private preview: TemplatePreview,
    context: () => Context,
    apply: (r: Reply) => Promise<void>,
    private canStart: () => boolean,
    private changed: () => void,
    private status: (s: string, error?: boolean) => void,
    private select: (id: string) => void,
  ) {
    this.gesture = new TemplateGesture(
      context,
      (_tables, invalid, _snapped, graphics) => {
        if (this.model)
          this.preview.render({ ...this.model, graphics: graphics ?? this.model.graphics }, false);
        this.preview.svg.classList.toggle('artwork-invalid', invalid);
      },
      apply,
      () => {
        this.preview.locked = this.active;
        this.changed();
      },
      status,
      true,
    );
    const svg = preview.svg;
    // Capture precedes table hit-testing: visible artwork receives whole-object dragging.
    svg.addEventListener(
      'contextmenu',
      (e) => {
        const id = (e.target as Element).closest<SVGElement>('[data-template-graphic]')?.dataset
          .templateGraphic;
        if (id && !this.active && this.canStart()) {
          e.preventDefault();
          e.stopImmediatePropagation();
          this.selected = id;
          this.select(id);
        }
      },
      true,
    );
    svg.addEventListener('pointerdown', (e) => this.down(e), true);
    svg.addEventListener(
      'pointermove',
      (e) => {
        if (this.drag && !this.ending) {
          const p = paperPoint(svg, e);
          if (p) this.propose(p);
        }
      },
      true,
    );
    svg.addEventListener('pointerup', (e) => void this.up(e), true);
    svg.addEventListener('pointercancel', () => void this.cancel());
    svg.addEventListener('lostpointercapture', () => {
      if (this.drag && !this.ending) void this.cancel();
    });
  }
  update(model: TemplateVariant) {
    this.model = model;
    if (!model.graphics.some((g) => g.id === this.selected)) this.selected = null;
  }
  start(kind: Kind) {
    if (!this.model || this.active) return;
    const original =
      kind === 'move' ? (this.model.graphics.find((g) => g.id === this.selected) ?? null) : null;
    if (kind === 'move' && !original) {
      this.status('Select artwork in the preview before choosing Move.');
      return;
    }
    const id = original?.id ?? crypto.randomUUID().replaceAll('-', '');
    this.arm(kind, id, original);
    this.gesture.propose({ start: null, end: null });
    this.status(
      kind === 'move'
        ? 'Drag the selected artwork · Escape cancels'
        : `Drag to place ${kind} · Escape cancels`,
    );
    this.preview.svg.focus();
  }
  private arm(kind: Kind, id: string, original: TemplateGraphic | null) {
    this.tool = { kind, id, original };
    this.gesture.begin(
      this.model!.id,
      id,
      kind === 'move' ? 'template.artwork.move' : `template.draw.${kind}`,
    );
    this.preview.svg.dataset.artworkTool = kind;
    this.preview.locked = true;
    this.changed();
  }
  private down(e: PointerEvent) {
    if (e.button !== 0 || this.drag || this.ending || !this.model) return;
    const p = paperPoint(this.preview.svg, e);
    if (!p) return;
    const hit = (e.target as Element).closest<SVGElement>('[data-template-graphic]')?.dataset
      .templateGraphic;
    if (!this.tool) {
      if (!hit || !this.canStart()) return;
      const g = this.model.graphics.find((g) => g.id === hit);
      if (!g) return;
      this.selected = g.id;
      this.arm('move', g.id, g);
    } else if (this.tool.kind === 'move' && hit !== this.tool.id) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    this.drag = { start: p, pointer: e.pointerId };
    this.preview.svg.focus();
    this.preview.svg.setPointerCapture(e.pointerId);
    if (this.tool?.kind === 'move') this.propose(p);
  }
  private propose(p: Point) {
    if (!this.drag || !this.tool) return;
    const a = this.drag.start;
    const end = this.tool.original
      ? {
          x: Number(this.tool.original.bounds[0]) + p.x - a.x,
          y: Number(this.tool.original.bounds[1]) + p.y - a.y,
        }
      : p;
    this.gesture.propose({
      start: [String(a.x), String(a.y)],
      end: [String(end.x), String(end.y)],
    });
  }
  private async up(e: PointerEvent) {
    if (!this.drag || this.drag.pointer !== e.pointerId || this.ending) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    const p = paperPoint(this.preview.svg, e);
    if (p) this.propose(p);
    await this.finish(true);
  }
  async cancel() {
    if (this.active && !this.ending) await this.finish(false);
  }
  private async finish(commit: boolean) {
    this.ending = true;
    const id = this.tool?.id;
    try {
      await this.gesture.finish(commit);
    } finally {
      const pointer = this.drag?.pointer;
      this.drag = null;
      this.tool = null;
      if (pointer !== undefined && this.preview.svg.hasPointerCapture(pointer))
        this.preview.svg.releasePointerCapture(pointer);
      delete this.preview.svg.dataset.artworkTool;
      this.preview.svg.classList.remove('artwork-invalid');
      this.preview.locked = false;
      this.ending = false;
      this.changed();
      if (commit && id && this.model?.graphics.some((g) => g.id === id)) {
        this.selected = id;
        this.select(id);
      }
    }
  }
}
