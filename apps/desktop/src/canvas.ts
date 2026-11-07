/** SVG view state is transient; scene rendering never writes domain coordinates. */
import type { SceneRow, Target } from './types';
import { keyOf } from './types';
import type { Shell } from './shell';
import { renderText, reserveTextBounds } from './text';
import { CanvasInput, type DragPlacement, type PointerPosition } from './canvas-input';
export class DrawingCanvas {
  private box = [-15, -45, 145, 90];
  private input: CanvasInput;
  startMove: (target: Target, point: PointerPosition) => Promise<DragPlacement | null> = async () =>
    null;
  cancelGesture() {
    return this.input.cancel();
  }
  private selected = new Set<string>();
  private rows: SceneRow[] = [];
  movement: {
    point: (x: string, y: string, tolerance: number) => void;
    commit: () => void;
  } | null = null;
  preview(rows: SceneRow[], snap?: [number, number]) {
    const base = this.rows;
    const patches = new Map(rows.map((r) => [keyOf(r.target), r]));
    const existing = new Set(base.map((r) => keyOf(r.target)));
    this.render([
      ...base.map((r) => patches.get(keyOf(r.target)) ?? r),
      ...rows.filter((r) => !existing.has(keyOf(r.target))),
    ]);
    this.rows = base;
    for (const node of this.shell.svg.querySelectorAll<SVGElement>('[data-object]'))
      node.classList.toggle('move-preview', patches.has(node.dataset.object!));
    if (snap) {
      const marker = document.createElementNS(this.shell.svg.namespaceURI, 'circle');
      marker.setAttribute('cx', String(snap[0]));
      marker.setAttribute('cy', String(snap[1]));
      marker.setAttribute(
        'r',
        String((this.box[2] * 6) / Math.max(1, this.shell.svg.getBoundingClientRect().width)),
      );
      marker.setAttribute('class', 'snap-marker');
      this.shell.svg.append(marker);
    }
  }
  pathPreview(
    rows: SceneRow[],
    rubber: SceneRow | null,
    vertices: [string, string][],
    cursor: [number, number],
    snapped: boolean,
  ) {
    this.preview(rows, snapped ? cursor : undefined);
    const ns = 'http://www.w3.org/2000/svg';
    if (!snapped) {
      const marker = document.createElementNS(ns, 'circle');
      marker.setAttribute('cx', String(cursor[0]));
      marker.setAttribute('cy', String(cursor[1]));
      marker.setAttribute('r', '0.8');
      marker.setAttribute('class', 'path-cursor');
      this.shell.svg.append(marker);
    }
    if (rubber) {
      const line = document.createElementNS(ns, 'polyline');
      for (const [name, value] of Object.entries({
        points: rubber.points.map((p) => p.join(',')).join(' '),
        fill: 'none',
        stroke: rubber.colour,
        'stroke-width': rubber.width,
        class: 'path-rubber',
      }))
        line.setAttribute(name, String(value));
      this.shell.svg.append(line);
    }
    for (const [x, y] of vertices) {
      const dot = document.createElementNS(ns, 'circle');
      dot.setAttribute('cx', String(Number(x) / 1000));
      dot.setAttribute('cy', String(Number(y) / 1000));
      dot.setAttribute('r', '0.65');
      dot.setAttribute('class', 'path-vertex');
      this.shell.svg.append(dot);
    }
  }
  clearPreview() {
    this.render(this.rows);
  }
  constructor(
    private shell: Shell,
    private choose: (target: Target | null, add: boolean) => void,
    selectMany: (targets: Target[], add: boolean) => void,
  ) {
    this.input = new CanvasInput({
      svg: shell.svg,
      canvas: shell.canvas,
      rows: () => this.rows,
      selected: () => this.selected,
      box: () => this.box,
      setBox: (box) => {
        this.box = box;
        this.paintBox();
      },
      choose,
      selectMany,
      startMove: (target, point) => this.startMove(target, point),
      movement: () => this.movement,
    });
    shell.svg.addEventListener('contextmenu', (event) => {
      if (this.movement) return;
      const hit = (event.target as Element).closest<SVGElement>('[data-object]');
      const row = this.rows.find((r) => keyOf(r.target) === hit?.dataset.object);
      if (row && !this.selected.has(keyOf(row.target))) this.choose(row.target, false);
      shell.canvas.focus();
    });
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
    const make = <K extends keyof SVGElementTagNameMap>(
      tag: K,
      attrs: Record<string, string | number>,
    ) => {
      const n = document.createElementNS('http://www.w3.org/2000/svg', tag);
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
        renderText(text, row.label, x, y + 6, 2.3);
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
        renderText(text, row.label, x, y, row.text_height);
        group.append(text);
      }
      this.shell.svg.append(group);
      if (row.shape === 'text')
        reserveTextBounds(
          group,
          group.querySelector<SVGGraphicsElement>('text.ink')!,
          row.points[0][0],
          row.points[0][1],
          row.text_height,
        );
      if (row.shape === 'anchor')
        reserveTextBounds(
          group,
          group.querySelector<SVGGraphicsElement>('text.anchor-caption')!,
          row.points[0][0],
          row.points[0][1] + 6,
          2.3,
          'caption-bounds',
        );
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
    for (const node of this.shell.svg.querySelectorAll<SVGGraphicsElement>('[data-object]')) {
      const b = node.getBBox();
      minX = Math.min(minX, b.x);
      maxX = Math.max(maxX, b.x + b.width);
      minY = Math.min(minY, b.y);
      maxY = Math.max(maxY, b.y + b.height);
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
