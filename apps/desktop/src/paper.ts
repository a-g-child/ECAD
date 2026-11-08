/** Read-only paper-space projection. Source scene coordinates and document records remain intact. */
import type { FrameRow, SceneRow, SheetRow, Snapshot } from './types';
import type { Shell } from './shell';
import { element } from './ui';
import { renderText } from './text';
import { renderFrame } from './frame';
import { CanvasInput } from './canvas-input';
const ns = 'http://www.w3.org/2000/svg';
function node(tag: string, attrs: Record<string, string | number>) {
  const n = document.createElementNS(ns, tag);
  for (const [key, value] of Object.entries(attrs)) n.setAttribute(key, String(value));
  return n;
}
let serial = 0;
export function renderPaper(
  svg: SVGSVGElement,
  sheet: SheetRow,
  scene: SceneRow[],
  frame: FrameRow,
) {
  svg.replaceChildren();
  const mm = (value: string) => Number(value) / 1000,
    w = mm(sheet.width),
    h = mm(sheet.height);
  const clip = `paper-clip-${++serial}`;
  const defs = node('defs', {}),
    path = node('clipPath', { id: clip });
  path.append(node('rect', { width: w, height: h }));
  defs.append(path);
  svg.append(defs);
  svg.append(
    node('rect', {
      width: w,
      height: h,
      fill: '#fff',
      stroke: '#9aa5ad',
      'stroke-width': 0.3,
      class: 'paper-boundary',
    }),
  );
  renderFrame(svg, frame);
  if (!sheet.view) return;
  const clipped = node('g', { 'clip-path': `url(#${clip})` });
  const placed = node('g', {
    transform: `translate(${mm(sheet.paper_x)} ${mm(sheet.paper_y)}) scale(${Number(sheet.numerator) / Number(sheet.denominator)}) translate(${-mm(sheet.source_x)} ${-mm(sheet.source_y)})`,
    class: 'sheet-placement',
  });
  for (const row of scene.filter((r) => r.view === sheet.view)) {
    const group = node('g', {});
    const [x, y] = row.points[0];
    if (row.shape === 'path')
      group.append(
        node('polyline', {
          points: row.points.map((p) => p.join(',')).join(' '),
          fill: 'none',
          stroke: row.colour,
          'stroke-width': row.width,
          'stroke-linejoin': 'round',
          'stroke-linecap': 'square',
        }),
      );
    else if (row.shape === 'anchor') {
      group.append(
        node('circle', {
          cx: x,
          cy: y,
          r: 1.25,
          fill: '#fff',
          stroke: row.colour,
          'stroke-width': 0.35,
        }),
      );
      const text = node('text', {
        x,
        y: y + 6,
        'text-anchor': 'middle',
        'font-size': 2.3,
        fill: '#46525e',
      });
      renderText(text, row.label, x, y + 6, 2.3);
      group.append(text);
    } else {
      const text = node('text', {
        x,
        y,
        fill: row.colour,
        'font-size': row.text_height,
        'font-family': row.font,
      });
      renderText(text, row.label, x, y, row.text_height);
      group.append(text);
    }
    placed.append(group);
  }
  clipped.append(placed);
  svg.append(clipped);
}
export function paperBox(sheet: SheetRow, aspect: number) {
  const w = Number(sheet.width) / 1000,
    h = Number(sheet.height) / 1000;
  const width = Math.max(w + 20, (h + 20) * aspect),
    height = width / aspect;
  return [(w - width) / 2, (h - height) / 2, width, height];
}
export class PaperSurface {
  private surface = element('div', 'sheet-surface');
  private svg = document.createElementNS(ns, 'svg');
  private badge = element('div', 'paper-badge');
  private tip = element(
    'div',
    'paper-tip',
    'Paper preview · F fits sheet · Return to Model view to edit',
  );
  private snapshot: Snapshot | null = null;
  private box = [0, 0, 317, 230];
  private current = '';
  private showing = false;
  get active() {
    return !!this.current;
  }
  get id() {
    return this.current;
  }
  constructor(
    private shell: Shell,
    private changed: () => void,
  ) {
    this.surface.hidden = true;
    this.tip.hidden = true;
    shell.placement.append(this.tip);
    this.svg.setAttribute('aria-label', 'Paper sheet preview');
    this.svg.setAttribute('role', 'img');
    this.surface.append(this.svg, this.badge);
    shell.canvas.append(this.surface);
    // Reuse CAD navigation only; paper preview never selects or moves model objects.
    this.svg.addEventListener(
      'pointerdown',
      (e) => {
        if (e.button === 0) {
          e.preventDefault();
          e.stopImmediatePropagation();
        }
      },
      true,
    );
    new CanvasInput({
      svg: this.svg,
      canvas: shell.canvas,
      rows: () => [],
      selected: () => new Set(),
      box: () => this.box,
      setBox: (box) => {
        this.box = box;
        this.paint();
      },
      choose: () => {},
      selectMany: () => {},
      startMove: async () => null,
      movement: () => null,
    });
    shell.sheets.addEventListener('change', () => this.show(shell.sheets.value));
    new ResizeObserver(() => {
      if (this.active) this.paint();
    }).observe(shell.canvas);
  }
  update(snapshot: Snapshot, replaced: boolean) {
    this.snapshot = snapshot;
    const previous = this.current;
    if (replaced || !snapshot.sheets.some((s) => s.id === this.current)) this.current = '';
    this.shell.sheets.replaceChildren();
    for (const s of [{ id: '', number: '', title: 'Model view' }, ...snapshot.sheets]) {
      const option = element('option', '', s.id ? `${s.number} · ${s.title}` : s.title);
      option.value = s.id;
      this.shell.sheets.append(option);
    }
    this.shell.sheets.value = this.current;
    this.render(previous !== this.current);
  }
  show(id: string) {
    const change = this.current !== id;
    this.current = id;
    this.shell.sheets.value = id;
    this.render(change);
    this.changed();
  }
  private render(fit: boolean) {
    const sheet = this.snapshot?.sheets.find((s) => s.id === this.current);
    const previouslyShowing = this.showing;
    this.showing = !!sheet;
    this.surface.hidden = !sheet;
    this.tip.hidden = !sheet;
    this.shell.placement.classList.toggle('paper-mode', !!sheet);
    if (sheet || previouslyShowing) {
      this.shell.views.disabled = !!sheet;
      this.shell.inspector.inert = !!sheet;
    }
    if (!sheet) return;
    const frame = this.snapshot!.sheet_frames.find((f) => f.sheet === sheet.id)!;
    renderPaper(this.svg, sheet, this.snapshot!.scene, frame);
    this.badge.textContent = `${sheet.number} · ${sheet.title} · Paper preview · edit source in Model view`;
    if (frame.title_block_too_small) this.badge.textContent += ' · Title block cannot fit';
    this.badge.title = this.badge.textContent;
    if (fit) this.fit();
    else this.paint();
  }
  fit() {
    const sheet = this.snapshot?.sheets.find((s) => s.id === this.current);
    if (!sheet) return;
    this.box = paperBox(sheet, this.aspect());
    this.paint();
  }
  private aspect() {
    return Math.max(1, this.shell.canvas.clientWidth) / Math.max(1, this.shell.canvas.clientHeight);
  }
  private paint() {
    this.box[3] = this.box[2] / this.aspect();
    this.svg.setAttribute('viewBox', this.box.join(' '));
    if (this.active) this.shell.zoom.textContent = 'Paper';
  }
}
