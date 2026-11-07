/** Canvas gestures own transient navigation/selection; all movement delegates to the move tool. */
import type { SceneRow, Target } from './types';
import { keyOf } from './types';
import { marqueeTargets, type Point } from './marquee';
export interface PointerPosition {
  x: string;
  y: string;
  tolerance: number;
}
export interface DragPlacement {
  point: (point: PointerPosition) => void;
  finish: (point: PointerPosition) => Promise<void>;
  cancel: () => void;
}
interface InputHost {
  svg: SVGSVGElement;
  canvas: HTMLElement;
  rows: () => SceneRow[];
  selected: () => Set<string>;
  box: () => number[];
  setBox: (box: number[]) => void;
  choose: (target: Target | null, add: boolean) => void;
  selectMany: (targets: Target[], add: boolean) => void;
  startMove: (target: Target, position: PointerPosition) => Promise<DragPlacement | null>;
  movement: () => {
    point: (x: string, y: string, tolerance: number) => void;
    commit: () => void;
  } | null;
}
interface Gesture {
  id: number;
  kind: 'pan' | 'marquee' | 'object';
  x: number;
  y: number;
  from: Point;
  box: number[];
  add: boolean;
  moved: boolean;
  target?: Target;
  placement?: Promise<DragPlacement | null>;
  cancelled: boolean;
}
export class CanvasInput {
  private gesture: Gesture | null = null;
  private overlay: SVGRectElement | null = null;
  private lastClient: Point | null = null;
  constructor(private host: InputHost) {
    host.svg.addEventListener('pointerdown', (e) => this.down(e));
    host.svg.addEventListener('pointermove', (e) => this.move(e));
    host.svg.addEventListener('pointerup', (e) => this.up(e));
    host.svg.addEventListener('pointercancel', () => this.cancel());
    host.svg.addEventListener('lostpointercapture', () => this.cancel());
    host.svg.addEventListener('wheel', (e) => this.wheel(e), { passive: false });
    window.addEventListener('blur', () => this.cancel());
  }
  private world(x: number, y: number): Point {
    const matrix = this.host.svg.getScreenCTM();
    const p = matrix ? new DOMPoint(x, y).matrixTransform(matrix.inverse()) : new DOMPoint();
    return [p.x, p.y];
  }
  private position(x: number, y: number): PointerPosition | null {
    const p = this.world(x, y).map((v) => Math.round(v * 1000));
    if (!p.every(Number.isSafeInteger)) return null;
    return {
      x: String(p[0]),
      y: String(p[1]),
      tolerance: Math.min(
        4294967295,
        Math.round(
          (this.host.box()[2] * 10000) / Math.max(1, this.host.svg.getBoundingClientRect().width),
        ),
      ),
    };
  }
  private down(e: PointerEvent) {
    if ((e.button !== 0 && e.button !== 1) || this.gesture) return;
    if (e.button === 0 && this.host.movement()) {
      e.preventDefault();
      this.host.movement()!.commit();
      this.host.canvas.focus();
      return;
    }
    const hit = (e.target as Element).closest<SVGElement>('[data-object]');
    const row = this.host.rows().find((r) => keyOf(r.target) === hit?.dataset.object);
    if (row && e.button === 0) {
      if (e.shiftKey || !this.host.selected().has(keyOf(row.target)))
        this.host.choose(row.target, e.shiftKey);
    }
    this.gesture = {
      id: e.pointerId,
      kind: e.button === 1 ? 'pan' : row ? 'object' : 'marquee',
      x: e.clientX,
      y: e.clientY,
      from: this.world(e.clientX, e.clientY),
      box: [...this.host.box()],
      add: e.shiftKey,
      moved: false,
      target: row?.target,
      cancelled: false,
    };
    this.host.svg.setPointerCapture(e.pointerId);
    e.preventDefault();
    this.host.canvas.focus();
  }
  private move(e: PointerEvent) {
    this.lastClient = [e.clientX, e.clientY];
    const g = this.gesture;
    const point = this.position(e.clientX, e.clientY);
    if (!g) {
      if (point) this.host.movement()?.point(point.x, point.y, point.tolerance);
      return;
    }
    if (e.pointerId !== g.id) return;
    if (!g.moved && Math.hypot(e.clientX - g.x, e.clientY - g.y) < 4) return;
    g.moved = true;
    if (g.kind === 'pan') {
      const b = this.host.svg.getBoundingClientRect();
      this.host.setBox([
        g.box[0] - ((e.clientX - g.x) * g.box[2]) / b.width,
        g.box[1] - ((e.clientY - g.y) * g.box[3]) / b.height,
        g.box[2],
        g.box[3],
      ]);
    } else if (g.kind === 'marquee') this.drawMarquee(g.from, this.world(e.clientX, e.clientY));
    else if (!g.add && g.target && point) {
      if (!g.placement) {
        const start = this.position(g.x, g.y);
        if (!start) return;
        g.placement = this.host.startMove(g.target, start);
      }
      void g.placement.then((handle) => {
        if (!g.cancelled) handle?.point(point);
      });
    }
  }
  private up(e: PointerEvent) {
    const g = this.gesture;
    if (!g || e.pointerId !== g.id) return;
    this.gesture = null;
    this.removeOverlay();
    if (this.host.svg.hasPointerCapture(g.id)) this.host.svg.releasePointerCapture(g.id);
    if (g.kind === 'marquee') {
      if (g.moved)
        this.host.selectMany(
          marqueeTargets(this.host.svg, this.host.rows(), g.from, this.world(e.clientX, e.clientY)),
          g.add,
        );
      else this.host.choose(null, g.add);
    }
    const point = this.position(e.clientX, e.clientY);
    if (g.placement)
      void g.placement.then((handle) => {
        if (g.cancelled || !point) handle?.cancel();
        else void handle?.finish(point);
      });
  }
  /** Escape, pointer loss and focus loss drop gestures without a partial edit. */
  cancel(): boolean {
    const g = this.gesture;
    if (!g) return false;
    g.cancelled = true;
    this.gesture = null;
    this.removeOverlay();
    if (this.host.svg.hasPointerCapture(g.id)) this.host.svg.releasePointerCapture(g.id);
    if (g.placement) void g.placement.then((handle) => handle?.cancel());
    return true;
  }
  private removeOverlay() {
    this.overlay?.remove();
    this.overlay = null;
  }
  private drawMarquee(a: Point, b: Point) {
    if (!this.overlay) {
      this.overlay = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
      this.host.svg.append(this.overlay);
    }
    for (const [name, value] of Object.entries({
      x: Math.min(a[0], b[0]),
      y: Math.min(a[1], b[1]),
      width: Math.abs(b[0] - a[0]),
      height: Math.abs(b[1] - a[1]),
    }))
      this.overlay.setAttribute(name, String(value));
    const window = b[0] >= a[0];
    this.overlay.setAttribute('class', `selection-marquee ${window ? 'window' : 'crossing'}`);
    this.overlay.setAttribute('aria-label', window ? 'Enclosure selection' : 'Crossing selection');
  }
  private wheel(e: WheelEvent) {
    e.preventDefault();
    // Avoid mixing navigation with an unresolved drag or a window in old coordinates.
    if (this.gesture) return;
    const bounds = this.host.svg.getBoundingClientRect();
    const box = this.host.box();
    const unit =
      e.deltaMode === WheelEvent.DOM_DELTA_LINE
        ? 16
        : e.deltaMode === WheelEvent.DOM_DELTA_PAGE
          ? bounds.height
          : 1;
    const dx = e.deltaX * unit,
      dy = e.deltaY * unit;
    if (e.shiftKey || e.ctrlKey) {
      const horizontal = e.shiftKey; // Shift wins if both modifiers are held.
      const delta = horizontal ? dx || dy : dy || dx;
      this.host.setBox([
        box[0] + (horizontal ? (delta * box[2]) / bounds.width : 0),
        box[1] + (!horizontal ? (delta * box[3]) / bounds.height : 0),
        box[2],
        box[3],
      ]);
    } else if (dy) {
      const factor = Math.exp(Math.max(-1, Math.min(1, dy * 0.0015)));
      const w = Math.min(100000, Math.max(2, box[2] * factor)),
        h = (box[3] * w) / box[2];
      const fx = (e.clientX - bounds.left) / bounds.width,
        fy = (e.clientY - bounds.top) / bounds.height;
      this.host.setBox([box[0] + (box[2] - w) * fx, box[1] + (box[3] - h) * fy, w, h]);
    }
    // The point under the cursor changes when panning/zooming during placement.
    if (this.lastClient) {
      const point = this.position(...this.lastClient);
      if (point) this.host.movement()?.point(point.x, point.y, point.tolerance);
    }
  }
}
