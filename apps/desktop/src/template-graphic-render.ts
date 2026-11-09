/** Shared artwork rendering for template placeholders and resolved project sheets. */
import type { TemplateGraphic } from './types';
import { renderText, reserveTextBounds } from './text';
const ns = 'http://www.w3.org/2000/svg';
function node<K extends keyof SVGElementTagNameMap>(
  tag: K,
  attrs: Record<string, string | number>,
) {
  const n = document.createElementNS(ns, tag);
  for (const [key, value] of Object.entries(attrs)) n.setAttribute(key, String(value));
  return n;
}
export function renderTemplateGraphics(parent: SVGElement, graphics: TemplateGraphic[]) {
  for (const g of graphics) {
    const [x, y, width, height] = g.bounds.map((v) => Number(v) / 1000);
    const group = node('g', { 'data-template-graphic': g.id, 'pointer-events': 'none' });
    const style = { fill: 'none', stroke: g.colour, 'stroke-width': Number(g.stroke_width) / 1000 };
    if (g.kind === 'text') {
      const size = Number(g.text_height) / 1000;
      const value = g.contents
        .map((r) => (r.kind === 'literal' ? r.value : `{${r.value}}`))
        .join('');
      const text = node('text', {
        fill: g.colour,
        'font-family': g.font,
        'font-size': size,
      });
      renderText(text, value, x, y + size, size);
      const title = node('title', {});
      title.textContent = value;
      group.append(text, title);
    } else if (g.kind === 'rectangle')
      group.append(node('rect', { x, y, width, height, ...style }));
    else if (g.kind === 'ellipse')
      group.append(
        node('ellipse', {
          cx: x + width / 2,
          cy: y + height / 2,
          rx: width / 2,
          ry: height / 2,
          ...style,
        }),
      );
    else
      group.append(
        node('polyline', {
          points: g.points
            .map((p) => `${x + Number(p[0]) / 1000},${y + Number(p[1]) / 1000}`)
            .join(' '),
          ...style,
        }),
      );
    parent.append(group);
    if (g.kind === 'text') {
      reserveTextBounds(
        group,
        group.querySelector('text')!,
        x,
        y + Number(g.text_height) / 1000,
        Number(g.text_height) / 1000,
        'text-bounds',
        true,
      );
    }
  }
}
