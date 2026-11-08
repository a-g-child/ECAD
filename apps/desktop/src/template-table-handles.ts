/** Screen-sized selection and table controls rendered above the retained paper scene. */
import type { TemplateTable, TemplateCell } from './types';
const ns = 'http://www.w3.org/2000/svg';
function node(tag: string, attrs: Record<string, string | number>) {
  const n = document.createElementNS(ns, tag);
  for (const [k, v] of Object.entries(attrs)) n.setAttribute(k, String(v));
  return n;
}
export function tableHandles(
  svg: SVGSVGElement,
  table: TemplateTable,
  cells: TemplateCell[],
  invalid: boolean,
  active: boolean,
) {
  const group = node('g', { class: `table-handles${invalid ? ' invalid' : ''}` });
  const px = 1 / Math.max(0.000001, Math.abs(svg.getScreenCTM()?.a ?? 1));
  const [tx, ty, w, h] = table.bounds.map((v) => Number(v) / 1000);
  for (const c of cells) {
    const [x, y, cw, ch] = c.bounds.map((v) => Number(v) / 1000);
    group.append(
      node('rect', {
        x: tx + x,
        y: ty + y,
        width: cw,
        height: ch,
        fill: '#2383bf',
        opacity: 0.16,
        'pointer-events': 'none',
        'data-selected-cell': c.id,
      }),
    );
  }
  const edge = (
    axis: string,
    from: number,
    x1: number,
    y1: number,
    x2: number,
    y2: number,
    outer?: string,
  ) => {
    group.append(
      node('line', {
        x1,
        y1,
        x2,
        y2,
        class: 'table-edge',
        'data-table': table.id,
        'data-axis': axis,
        'data-from': Math.round(from * 1000),
        'data-outer': outer ?? '',
        stroke: 'transparent',
        'stroke-width': 8 * px,
        'pointer-events': active ? 'none' : 'stroke',
        cursor: axis === 'x' ? 'col-resize' : 'row-resize',
      }),
    );
  };
  // Interior hit segments follow actual cell sides, including T junctions; spanning cells have no fake edge.
  const seen = new Set<string>();
  for (const c of table.cells) {
    const [x, y, cw, ch] = c.bounds.map((v) => Number(v) / 1000);
    for (const [axis, from, a, b] of [
      ['x', x, y, y + ch],
      ['x', x + cw, y, y + ch],
      ['y', y, x, x + cw],
      ['y', y + ch, x, x + cw],
    ] as const) {
      const extent = axis === 'x' ? w : h;
      if (from === 0 || from === extent) continue;
      const key = `${axis}:${from}:${a}:${b}`;
      if (seen.has(key)) continue;
      seen.add(key);
      if (axis === 'x') edge(axis, from, tx + from, ty + a, tx + from, ty + b);
      else edge(axis, from, tx + a, ty + from, tx + b, ty + from);
    }
  }
  edge('x', 0, tx, ty, tx, ty + h, 'start');
  edge('x', w, tx + w, ty, tx + w, ty + h, 'end');
  edge('y', 0, tx, ty, tx + w, ty, 'start');
  edge('y', h, tx, ty + h, tx + w, ty + h, 'end');
  group.append(
    node('rect', {
      x: tx,
      y: ty,
      width: w,
      height: h,
      fill: 'none',
      stroke: invalid ? '#bc3434' : '#2383bf',
      'stroke-width': px,
      'pointer-events': 'none',
    }),
  );
  const button = (label: string, text: string, kind: string, x: number, y: number) => {
    const control = node('g', {
      class: 'table-control',
      'data-table': table.id,
      'data-control': kind,
      role: 'button',
      'aria-label': label,
      tabindex: active ? -1 : 0,
      transform: `translate(${x} ${y})`,
    });
    const title = node('title', {});
    title.textContent = label;
    control.append(
      title,
      node('rect', {
        x: -8 * px,
        y: -8 * px,
        width: 16 * px,
        height: 16 * px,
        rx: 2 * px,
        fill: '#fff',
        stroke: '#2383bf',
        'stroke-width': px,
      }),
    );
    const glyph = node('text', {
      'text-anchor': 'middle',
      y: 4 * px,
      'font-size': 13 * px,
      fill: '#1e678f',
      'pointer-events': 'none',
    });
    glyph.textContent = text;
    control.append(glyph);
    group.append(control);
  };
  button('Move title-block table', '✥', 'move', tx - 12 * px, ty - 12 * px);
  if (cells.length) {
    const right = Math.max(...cells.map((c) => Number(c.bounds[0]) + Number(c.bounds[2]))) / 1000;
    const bottom = Math.max(...cells.map((c) => Number(c.bounds[1]) + Number(c.bounds[3]))) / 1000;
    button(
      'Insert row below selection; keep bottom edge',
      '+',
      'row',
      tx + right / 2,
      ty + bottom + 12 * px,
    );
    button(
      'Insert column after selection; keep right edge',
      '+',
      'column',
      tx + right + 12 * px,
      ty + bottom / 2,
    );
  }
  svg.append(group);
}

/** Highlight only borders actually chosen by the validated Rust pointer preview. */
export function marginSnapGuides(svg: SVGSVGElement, paper: string[], snapped: string[]) {
  if (!snapped.length) return;
  const [w, h, l, r, t, b] = paper.map((v) => Number(v) / 1000);
  const group = node('g', { class: 'margin-snap-guides', 'pointer-events': 'none' });
  for (const edge of snapped) {
    const vertical = edge === 'left' || edge === 'right';
    const at = edge === 'left' ? l : edge === 'right' ? w - r : edge === 'top' ? t : h - b;
    group.append(
      node('line', {
        x1: vertical ? at : l,
        y1: vertical ? t : at,
        x2: vertical ? at : w - r,
        y2: vertical ? h - b : at,
        stroke: '#14875a',
        'stroke-width': 2,
        'stroke-dasharray': '5 3',
        'vector-effect': 'non-scaling-stroke',
        'data-snap-edge': edge,
      }),
    );
  }
  svg.append(group);
}
