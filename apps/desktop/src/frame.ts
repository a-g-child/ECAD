/** Paper-unit frame rendering. Literal values stay intact; overflowing text is clipped and flagged. */
import type { FrameRow } from './types';
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
export function renderFrame(svg: SVGElement, frame: Omit<FrameRow, 'sheet'>) {
  const group = node('g', {
    class: 'sheet-frame',
    'data-template': frame.template,
    fill: frame.colour,
    'font-family': frame.font,
  });
  const border = node('rect', {
    ...frame.border,
    fill: 'none',
    stroke: frame.colour,
    'stroke-width': frame.stroke_width,
    class: 'frame-border',
  });
  group.append(border);
  svg.append(group);
  if (frame.title_block_too_small) {
    const title = node('title', {});
    title.textContent = 'Paper interior is too small for the standard 180 × 40 mm title block';
    border.append(title);
    group.dataset.diagnostic = 'title-block-too-small';
  }
  for (const cell of frame.cells) {
    const { x, y, width, height } = cell.bounds;
    const container = node('g', { class: 'frame-cell', 'data-field': cell.key });
    const title = node('title', {});
    title.textContent = `${cell.caption}: ${cell.value ?? '(not set)'}`;
    const box = node('rect', {
      x,
      y,
      width,
      height,
      fill: 'none',
      stroke: frame.colour,
      'stroke-width': frame.stroke_width,
    });
    const caption = node('text', { x: x + 1, y: y + 2, 'font-size': 1.5 });
    renderText(caption, cell.caption, x + 1, y + 2, 1.5);
    const id = `frame-value-${++serial}`;
    const clip = node('clipPath', { id });
    // Reserve the caption strip and a 1 mm inset. Text never silently shrinks.
    clip.append(node('rect', { x: x + 1, y: y + 3, width: width - 2, height: height - 4 }));
    const text = node('text', {
      x: x + 1,
      y: y + 3 + frame.text_height,
      'font-size': frame.text_height,
      'clip-path': `url(#${id})`,
      class: 'frame-value',
    });
    renderText(text, cell.value ?? '', x + 1, y + 3 + frame.text_height, frame.text_height);
    container.append(title, box, caption, clip, text);
    group.append(container);
    const b = text.getBBox();
    const lines = (cell.value ?? '').split(/\r\n?|\n/).length;
    const semanticBottom = y + 3 + frame.text_height + (lines - 1) * frame.text_height * 1.3;
    const overflow =
      cell.value &&
      (b.x < x + 1 ||
        b.x + b.width > x + width - 1 ||
        b.y < y + 3 ||
        Math.max(b.y + b.height, semanticBottom) > y + height - 1);
    if (overflow) {
      container.classList.add('frame-overflow');
      title.textContent += '\nText exceeds cell bounds; full value is preserved';
      container.append(
        node('path', {
          d: `M ${x + width - 2} ${y} l 2 0 l 0 2 Z`,
          fill: '#b96b00',
          class: 'overflow-marker',
        }),
      );
    }
  }
}
