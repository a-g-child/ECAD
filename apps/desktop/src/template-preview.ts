/** Paper preview owns only transient camera/selection; committed geometry comes from Rust. */
import type { TemplateVariant, TemplateCell, TemplateTable } from './types';
import { renderFrame } from './frame';
const ns = 'http://www.w3.org/2000/svg';
export class TemplatePreview {
  readonly svg = document.createElementNS(ns, 'svg');
  private model: TemplateVariant | null = null;
  locked = false;
  onRender: () => void = () => {};
  private box = [-10, -10, 317, 230];
  constructor(readonly properties: () => void) {
    this.svg.classList.add('template-preview');
    this.svg.setAttribute('role', 'img');
    this.svg.setAttribute('aria-label', 'Template paper preview');
    this.svg.tabIndex = 0;
    this.svg.dataset.focus = 'canvas';
    this.svg.addEventListener(
      'wheel',
      (e) => {
        e.preventDefault();
        if (this.locked) return;
        const scale = e.deltaMode === 1 ? 16 : e.deltaMode === 2 ? this.svg.clientHeight : 1;
        if (e.shiftKey || e.ctrlKey) {
          const amount = (e.deltaY * scale * this.box[2]) / Math.max(1, this.svg.clientWidth);
          this.box[e.shiftKey ? 0 : 1] += amount;
        } else {
          const matrix = this.svg.getScreenCTM();
          if (!matrix) return;
          const p = this.svg.createSVGPoint();
          p.x = e.clientX;
          p.y = e.clientY;
          const at = p.matrixTransform(matrix.inverse());
          const factor = Math.exp(Math.max(-1, Math.min(1, e.deltaY * scale * 0.0015)));
          this.box = [
            at.x + (this.box[0] - at.x) * factor,
            at.y + (this.box[1] - at.y) * factor,
            this.box[2] * factor,
            this.box[3] * factor,
          ];
        }
        this.camera();
      },
      { passive: false },
    );
    let pan: { x: number; y: number; box: number[] } | null = null;
    this.svg.addEventListener('pointerdown', (e) => {
      if (e.button === 1 && !this.locked) {
        e.preventDefault();
        pan = { x: e.clientX, y: e.clientY, box: [...this.box] };
        this.svg.setPointerCapture(e.pointerId);
      }
    });
    this.svg.addEventListener('pointermove', (e) => {
      if (pan) {
        const ratio = pan.box[2] / Math.max(1, this.svg.clientWidth);
        this.box = [
          pan.box[0] - (e.clientX - pan.x) * ratio,
          pan.box[1] - (e.clientY - pan.y) * ratio,
          pan.box[2],
          pan.box[3],
        ];
        this.camera();
      }
    });
    this.svg.addEventListener('pointerup', () => {
      pan = null;
    });
    this.svg.addEventListener('pointercancel', () => {
      pan = null;
    });
  }
  fit() {
    if (this.model) {
      const [w, h] = this.model.paper.map((s) => Number(s) / 1000);
      this.box = [-10, -10, w + 20, h + 20];
      this.camera();
    }
  }
  private camera() {
    this.svg.setAttribute('viewBox', this.box.join(' '));
    this.onRender();
  }
  render(model: TemplateVariant, reset: boolean) {
    this.model = model;
    this.svg.replaceChildren();
    const [w, h, l, r, t, b] = model.paper.map((v) => Number(v) / 1000);
    const rect = document.createElementNS(ns, 'rect');
    for (const [k, v] of Object.entries({
      x: 0,
      y: 0,
      width: w,
      height: h,
      fill: '#fff',
      stroke: '#b0b8c0',
      'stroke-width': 0.3,
    }))
      rect.setAttribute(k, String(v));
    this.svg.append(rect);
    renderFrame(this.svg, {
      template: model.id,
      graphics: model.graphics,
      border: { x: l, y: t, width: w - l - r, height: h - t - b },
      cells: model.tables.flatMap((table) =>
        table.cells.map((cell) => ({
          key: cell.id,
          picture: cell.picture,
          caption: cell.caption,
          text_styles: {
            caption: Object.fromEntries(
              Object.entries(model.text_defaults.caption).map(([k, v]) => [
                k,
                cell.text_styles.caption[k as keyof typeof cell.text_styles.caption] ?? v,
              ]),
            ),
            content: Object.fromEntries(
              Object.entries(model.text_defaults.content).map(([k, v]) => [
                k,
                cell.text_styles.content[k as keyof typeof cell.text_styles.content] ?? v,
              ]),
            ),
          } as import('./types').TemplateTextStyles,
          value: cell.contents
            .map((r) => (r.kind === 'literal' ? r.value : `{${r.value}}`))
            .join(''),
          bounds: {
            x: (Number(table.bounds[0]) + Number(cell.bounds[0])) / 1000,
            y: (Number(table.bounds[1]) + Number(cell.bounds[1])) / 1000,
            width: Number(cell.bounds[2]) / 1000,
            height: Number(cell.bounds[3]) / 1000,
          },
        })),
      ),
      title_block_too_small: false,
      font: 'sans-serif',
      text_height: 2.5,
      stroke_width: 0.25,
      colour: '#283c46',
    });
    if (reset) this.fit();
    else this.camera();
  }
}
