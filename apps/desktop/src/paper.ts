/** Paper-space review projection; the editable canvas shares this background/frame renderer. */
import type { DrawingListRow, FrameRow, SceneRow, SheetRow } from './types';
import { renderText } from './text';
import { renderDrawingList } from './drawing-list';
import { renderFrame } from './frame';
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
  list: DrawingListRow | null = null,
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
  renderPaperBackground(svg, sheet, frame, list);
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

export function renderPaperBackground(
  svg: SVGElement,
  sheet: SheetRow,
  frame: FrameRow,
  list: DrawingListRow | null = null,
  navigate?: (id: string) => void,
) {
  const w = Number(sheet.width) / 1000,
    h = Number(sheet.height) / 1000;
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
  if (list) renderDrawingList(svg, list, frame, navigate);
}
