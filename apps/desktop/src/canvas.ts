/** SVG view state is transient; scene rendering never writes domain coordinates. */
import type { SceneRow, Target } from './types';
import { keyOf } from './types';
import type { Shell } from './shell';
export class DrawingCanvas {
  private box = [-15, -45, 145, 90];
  private drag: { x: number; y: number; box: number[]; moved: boolean } | null = null;
  private selected = new Set<string>();
  private rows: SceneRow[] = [];
  constructor(
    private shell: Shell,
    private choose: (target: Target | null, add: boolean) => void,
  ) {
    shell.svg.addEventListener('pointerdown', (event) => {
      if (event.button !== 0 && event.button !== 1) return;
      const hit = (event.target as Element).closest<SVGElement>('[data-object]');
      if (hit && event.button === 0) {
        const row = this.rows.find((r) => keyOf(r.target) === hit.dataset.object);
        if (row) this.choose(row.target, event.shiftKey);
        shell.canvas.focus();
        return;
      }
      this.drag = { x: event.clientX, y: event.clientY, box: [...this.box], moved: false };
      shell.svg.setPointerCapture(event.pointerId);
      event.preventDefault();
      shell.canvas.focus();
    });
    shell.svg.addEventListener('contextmenu', (event) => {
      const hit = (event.target as Element).closest<SVGElement>('[data-object]');
      const row = this.rows.find((r) => keyOf(r.target) === hit?.dataset.object);
      if (row && !this.selected.has(keyOf(row.target))) this.choose(row.target, false);
      shell.canvas.focus();
    });
    shell.svg.addEventListener('pointermove', (event) => {
      if (!this.drag) return;
      const bounds = shell.svg.getBoundingClientRect();
      const dx = ((event.clientX - this.drag.x) * this.box[2]) / bounds.width;
      const dy = ((event.clientY - this.drag.y) * this.box[3]) / bounds.height;
      if (Math.abs(event.clientX - this.drag.x) + Math.abs(event.clientY - this.drag.y) > 3)
        this.drag.moved = true;
      this.box = [this.drag.box[0] - dx, this.drag.box[1] - dy, this.box[2], this.box[3]];
      this.paintBox();
    });
    shell.svg.addEventListener('pointerup', (event) => {
      if (this.drag && !this.drag.moved) this.choose(null, event.shiftKey);
      this.drag = null;
    });
    shell.svg.addEventListener('pointercancel', () => (this.drag = null));
    shell.svg.addEventListener(
      'wheel',
      (event) => {
        event.preventDefault();
        const factor = event.deltaY < 0 ? 0.85 : 1.18;
        const bounds = shell.svg.getBoundingClientRect();
        const fx = (event.clientX - bounds.left) / bounds.width,
          fy = (event.clientY - bounds.top) / bounds.height;
        const w = Math.min(100000, Math.max(2, this.box[2] * factor)),
          h = (this.box[3] * w) / this.box[2];
        this.box = [
          this.box[0] + (this.box[2] - w) * fx,
          this.box[1] + (this.box[3] - h) * fy,
          w,
          h,
        ];
        this.paintBox();
      },
      { passive: false },
    );
    new ResizeObserver(() => this.paintBox()).observe(shell.canvas);
  }
  setSelection(targets: Target[]) {
    this.selected = new Set(targets.map(keyOf));
    for (const node of this.shell.svg.querySelectorAll<SVGElement>('[data-object]'))
      node.classList.toggle('selected', this.selected.has(node.dataset.object!));
  }
  render(rows: SceneRow[], fit = false) {
    this.rows = rows;
    this.shell.svg.replaceChildren();
    const ns = this.shell.svg.namespaceURI;
    const make = (tag: string, attrs: Record<string, string | number>) => {
      const n = document.createElementNS(ns, tag);
      for (const [key, value] of Object.entries(attrs)) n.setAttribute(key, String(value));
      return n;
    };
    const defs = make('defs', {});
    const pattern = make('pattern', {
      id: 'grid',
      width: 5,
      height: 5,
      patternUnits: 'userSpaceOnUse',
    });
    pattern.append(make('circle', { cx: 0, cy: 0, r: 0.1, fill: '#dce1e4' }));
    defs.append(pattern);
    this.shell.svg.append(defs);
    this.shell.svg.append(
      make('rect', {
        x: -1000000,
        y: -1000000,
        width: 2000000,
        height: 2000000,
        fill: 'url(#grid)',
      }),
    );
    for (const row of rows) {
      const group = make('g', { 'data-object': keyOf(row.target), tabindex: -1 });
      const title = make('title', {});
      title.textContent = row.label;
      group.append(title);
      if (row.shape === 'path') {
        const points = row.points.map((p) => p.join(',')).join(' ');
        group.append(
          make('polyline', {
            points,
            fill: 'none',
            stroke: row.colour,
            'stroke-width': row.width,
            'stroke-linejoin': 'round',
            'stroke-linecap': 'square',
            class: 'ink',
          }),
        );
        group.append(
          make('polyline', {
            points,
            fill: 'none',
            stroke: 'transparent',
            'stroke-width': 12,
            'vector-effect': 'non-scaling-stroke',
            class: 'hit-path',
          }),
        );
      } else if (row.shape === 'anchor') {
        const [x, y] = row.points[0];
        group.append(
          make('circle', {
            cx: x,
            cy: y,
            r: 1.25,
            fill: '#fff',
            stroke: row.colour,
            'stroke-width': 0.35,
            class: 'ink',
          }),
        );
        const text = make('text', {
          x,
          y: y + 6,
          'text-anchor': 'middle',
          'font-size': 2.3,
          fill: '#46525e',
          class: 'anchor-caption',
        });
        text.textContent = row.label;
        group.append(text);
      } else {
        const [x, y] = row.points[0];
        const text = make('text', {
          x,
          y,
          fill: row.colour,
          'font-size': row.text_height,
          'font-family': row.font,
          class: 'ink',
        });
        text.textContent = row.label;
        group.append(text);
      }
      this.shell.svg.append(group);
    }
    this.setSelection(
      [...this.selected].flatMap((key) =>
        rows.filter((r) => keyOf(r.target) === key).map((r) => r.target),
      ),
    );
    if (fit) this.fit();
    else this.paintBox();
  }
  fit() {
    if (!this.rows.length) return;
    let minX = Infinity,
      maxX = -Infinity,
      minY = Infinity,
      maxY = -Infinity;
    for (const row of this.rows)
      for (const [x, y] of row.points) {
        minX = Math.min(minX, x);
        maxX = Math.max(maxX, x);
        minY = Math.min(minY, y);
        maxY = Math.max(maxY, y);
      }
    let w = Math.max(30, maxX - minX + 30),
      h = Math.max(30, maxY - minY + 35);
    const bounds = this.shell.svg.getBoundingClientRect();
    const aspect = Math.max(0.1, bounds.width / Math.max(1, bounds.height));
    if (w / h < aspect) w = h * aspect;
    else h = w / aspect;
    this.box = [(minX + maxX - w) / 2, (minY + maxY - h) / 2, w, h];
    this.paintBox();
  }
  private paintBox() {
    const bounds = this.shell.svg.getBoundingClientRect();
    if (bounds.width > 0 && bounds.height > 0)
      this.box[3] = (this.box[2] * bounds.height) / bounds.width;
    this.shell.svg.setAttribute('viewBox', this.box.join(' '));
    this.shell.zoom.textContent = `${Math.round((140 / this.box[2]) * 100)}%`;
  }
}
