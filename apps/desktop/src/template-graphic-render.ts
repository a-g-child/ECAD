/** Shared artwork rendering for template placeholders and resolved project sheets. */
import type { TemplateGraphic } from './types';
import { renderText } from './text';
const ns = 'http://www.w3.org/2000/svg';
let serial = 0;
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
      const id = `template-artwork-${++serial}`;
      const clip = node('clipPath', { id });
      clip.append(node('rect', { x, y, width, height }));
      const text = node('text', {
        fill: g.colour,
        'font-family': g.font,
        'font-size': size,
        'clip-path': `url(#${id})`,
      });
      renderText(text, value, x, y + size, size);
      const title = node('title', {});
      title.textContent = value;
      group.append(clip, text, title);
      // Transparent bounds support selecting blank or clipped text without changing its appearance.
      group.append(node('rect', { x, y, width, height, fill: 'transparent' }));
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
      const text = group.querySelector('text')!;
      const b = text.getBBox();
      if (b.x < x || b.y < y || b.x + b.width > x + width || b.y + b.height > y + height) {
        group.dataset.diagnostic = 'text-overflow';
        group.querySelector('title')!.textContent +=
          '\nText exceeds bounds; full content is preserved';
        group.append(
          node('path', {
            d: `M ${x + width - 2} ${y} l 2 0 l 0 2 Z`,
            fill: '#b96b00',
            class: 'overflow-marker',
          }),
        );
      }
    }
  }
}
