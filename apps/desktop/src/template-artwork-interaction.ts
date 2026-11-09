/** One-shot pointer artwork tools; authoritative previews and closed commits stay in Rust. */
import type { Context, Reply, TemplateGraphic, TemplateVariant } from './types';
import { TemplatePreview } from './template-preview';
import { TemplatePathPlacement } from './template-path-placement';
import { TemplateTextPlacement } from './template-text-placement';
import { TemplateGesture } from './template-gesture';
import { paperPoint } from './template-table-selection';
type Point = { x: number; y: number };
type Kind = 'text' | 'polyline' | 'line' | 'rectangle' | 'ellipse' | 'move';
export class TemplateArtworkInteraction {
  private model: TemplateVariant | null = null;

  private tool: { kind: Kind; id: string; original: TemplateGraphic | null } | null = null;
  private drag: { start: Point; pointer: number | null; screen: Point; second: boolean } | null =
    null;
  private ending = false;
  private path: TemplatePathPlacement | null = null;
  private textPlacement: TemplateTextPlacement | null = null;
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
    private select: (id: string, focus?: boolean) => void,
    private selection: () => string | null,
  ) {
    this.gesture = new TemplateGesture(
      context,
      (_tables, invalid, _snapped, graphics) => {
        if (this.model)
          this.preview.render({ ...this.model, graphics: graphics ?? this.model.graphics }, false);
        this.preview.svg.classList.toggle('artwork-invalid', invalid);
        this.paintAnchor();
        this.path?.paint();
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
          this.select(id, true);
        }
      },
      true,
    );
    svg.addEventListener('pointerdown', (e) => this.down(e), true);
    svg.addEventListener(
      'pointermove',
      (e) => {
        if (this.path && !this.ending) {
          const p = paperPoint(svg, e);
          if (p) this.path.hover(p);
          return;
        }
        if (
          this.drag &&
          !this.ending &&
          !this.textPlacement &&
          (this.drag.pointer === null || this.drag.pointer === e.pointerId)
        ) {
          const p = paperPoint(svg, e);
          if (p) this.propose(p);
        }
      },
      true,
    );
    svg.addEventListener('pointerup', (e) => void this.up(e), true);
    svg.addEventListener('pointercancel', () => void this.cancel());
    svg.addEventListener('lostpointercapture', (e) => {
      if (this.drag?.pointer === e.pointerId && !this.ending) void this.cancel();
    });
  }
  update(model: TemplateVariant) {
    this.model = model;
  }
  start(kind: Kind) {
    if (!this.model || this.active) return;
    const original =
      kind === 'move' ? (this.model.graphics.find((g) => g.id === this.selection()) ?? null) : null;
    if (kind === 'move' && !original) {
      this.status('Select artwork in the preview before choosing Move.');
      return;
    }
    const id = original?.id ?? crypto.randomUUID().replaceAll('-', '');
    this.arm(kind, id, original);
    this.gesture.propose({ start: null, end: null });
    if (kind === 'polyline') {
      this.path = new TemplatePathPlacement(
        this.preview.svg,
        (edit) => this.gesture.propose(edit),
        async () => {
          if (!(await this.gesture.ready())) return false;
          await this.finish(true);
          return true;
        },
        () => this.cancel(),
        this.status,
      );
      this.preview.svg.focus();
      return;
    }
    this.status(
      kind === 'move'
        ? 'Drag the selected artwork · Escape cancels'
        : kind === 'text'
          ? 'Click a text anchor or drag a text box · Escape cancels'
          : `Click start, then finish ${kind} · Drag also works · Escape cancels`,
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
    if (
      e.button !== 0 ||
      this.drag?.pointer != null ||
      this.ending ||
      this.textPlacement ||
      !this.model
    )
      return;
    const p = paperPoint(this.preview.svg, e);
    if (!p) return;
    if (this.path) {
      e.preventDefault();
      e.stopImmediatePropagation();
      this.preview.svg.focus();
      this.path.add(p);
      return;
    }
    const hit = (e.target as Element).closest<SVGElement>('[data-template-graphic]')?.dataset
      .templateGraphic;
    if (!this.tool) {
      if (!hit || !this.canStart()) return;
      const g = this.model.graphics.find((g) => g.id === hit);
      if (!g) return;
      this.select(g.id);
      this.arm('move', g.id, g);
    } else if (this.tool.kind === 'move' && hit !== this.tool.id) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    if (this.drag) {
      this.drag.pointer = e.pointerId;
      this.drag.second = true;
    } else {
      this.drag = {
        start: p,
        pointer: e.pointerId,
        screen: { x: e.clientX, y: e.clientY },
        second: false,
      };
    }
    this.paintAnchor();
    this.preview.svg.focus();
    this.preview.svg.setPointerCapture(e.pointerId);
    if (this.tool?.kind === 'move' || this.drag.second) this.propose(p);
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
    const distance = Math.hypot(e.clientX - this.drag.screen.x, e.clientY - this.drag.screen.y);
    if (this.tool?.kind === 'text') {
      const a = this.drag.start;
      const p = paperPoint(this.preview.svg, e);
      if (
        !p ||
        !this.model ||
        a.x < 0 ||
        a.y < 0 ||
        a.x >= Number(this.model.paper[0]) ||
        a.y >= Number(this.model.paper[1])
      ) {
        await this.finish(false);
        return;
      }
      const b =
        distance < 4
          ? {
              x: Math.min(a.x + 60000, Number(this.model.paper[0])),
              y: Math.min(a.y + 20000, Number(this.model.paper[1])),
            }
          : p;
      if (
        b.x < 0 ||
        b.y < 0 ||
        b.x > Number(this.model.paper[0]) ||
        b.y > Number(this.model.paper[1]) ||
        b.x === a.x ||
        b.y === a.y
      ) {
        await this.finish(false);
        this.status('Place a positive text box inside the paper.', true);
        return;
      }
      this.drag.pointer = null;
      if (this.preview.svg.hasPointerCapture(e.pointerId))
        this.preview.svg.releasePointerCapture(e.pointerId);
      this.preview.svg.dataset.artworkPhase = 'text';
      this.textPlacement = new TemplateTextPlacement(
        this.preview.svg.parentElement!,
        this.preview.svg,
        [Math.min(a.x, b.x), Math.min(a.y, b.y)],
        (contents) =>
          this.gesture.propose({
            start: [String(a.x), String(a.y)],
            end: [String(b.x), String(b.y)],
            contents,
          }),
        async () => {
          if (!(await this.gesture.ready())) return false;
          await this.finish(true);
          return true;
        },
        () => this.cancel(),
      );
      return;
    }

    if (!this.drag.second && this.tool?.kind !== 'move' && distance < 4) {
      // A click establishes the anchor. Clear capture ownership before releasing it.
      this.drag.pointer = null;
      if (this.preview.svg.hasPointerCapture(e.pointerId))
        this.preview.svg.releasePointerCapture(e.pointerId);
      this.preview.svg.dataset.artworkPhase = 'finish';
      this.status('Click the finish point · Escape cancels');
      return;
    }
    const p = paperPoint(this.preview.svg, e);
    if (p) this.propose(p);
    await this.finish(p !== null);
  }
  private paintAnchor() {
    const svg = this.preview.svg;
    svg.querySelector('[data-artwork-anchor]')?.remove();
    if (!this.drag || this.tool?.kind === 'move' || this.tool?.kind === 'text') return;
    const anchor = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
    anchor.setAttribute('data-artwork-anchor', '');
    anchor.setAttribute('cx', String(this.drag.start.x / 1000));
    anchor.setAttribute('cy', String(this.drag.start.y / 1000));
    anchor.setAttribute('r', String(4 / Math.max(0.000001, Math.abs(svg.getScreenCTM()?.a ?? 1))));
    anchor.setAttribute('fill', 'none');
    anchor.setAttribute('stroke', '#2383bf');
    anchor.setAttribute('stroke-width', '1');
    anchor.setAttribute('vector-effect', 'non-scaling-stroke');
    anchor.setAttribute('pointer-events', 'none');
    svg.append(anchor);
  }
  key(event: KeyboardEvent) {
    if (!this.path) return false;
    this.path.key(event);
    return true;
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
      this.path?.close();
      this.path = null;
      this.textPlacement?.close();
      this.textPlacement = null;
      this.drag = null;
      this.tool = null;
      if (pointer != null && this.preview.svg.hasPointerCapture(pointer))
        this.preview.svg.releasePointerCapture(pointer);
      delete this.preview.svg.dataset.artworkTool;
      delete this.preview.svg.dataset.artworkPhase;
      this.preview.svg.querySelector('[data-artwork-anchor]')?.remove();
      this.preview.svg.classList.remove('artwork-invalid');
      this.preview.locked = false;
      this.ending = false;
      this.changed();
      if (commit && id && this.model?.graphics.some((g) => g.id === id)) {
        this.select(id, true);
      }
    }
  }
}
