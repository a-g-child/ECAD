/** Generated paper-space navigation. Cell text stays literal; link targets are stable sheet IDs. */
import type { DrawingListRow, FrameRow } from './types';
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
export function renderDrawingList(
  parent: SVGElement,
  list: DrawingListRow,
  frame: FrameRow,
  navigate?: (id: string) => void,
) {
  const group = node('g', { class: 'drawing-list', fill: frame.colour, 'font-family': frame.font });
  parent.append(group);
  const warning =
    list.remaining !== '0' ? ` · ${list.remaining} drawings need more list pages` : '';
  const heading = node('text', {
    x: list.x,
    y: list.y + 2.5,
    'font-size': 2.5,
    class: 'drawing-list-heading',
  });
  renderText(
    heading,
    `Drawing list · ${list.ordinal} / ${list.page_count}${warning}`,
    list.x,
    list.y + 2.5,
    2.5,
  );
  group.append(heading);
  if (list.capacity === '0') {
    const note = node('text', { x: list.x, y: list.y + 6, 'font-size': 2.5 });
    renderText(
      note,
      'No room for table rows. Increase paper size or choose another template.',
      list.x,
      list.y + 6,
      2.5,
    );
    group.append(note);
    return;
  }
  const column = Math.min(40, list.width / 3);
  for (const [caption, x] of [
    ['Drawing number', list.x],
    ['Title', list.x + column],
  ] as [string, number][]) {
    const text = node('text', { x: x + 1, y: list.y + 7, 'font-size': 1.8 });
    renderText(text, caption, x + 1, list.y + 7, 1.8);
    group.append(text);
  }
  for (const [index, row] of list.rows.entries()) {
    const y = list.y + (index + 1) * list.row_height;
    const link = navigate
      ? node('a', {
          href: `#sheet-${row.sheet}`,
          tabindex: 0,
          'aria-label': `Open ${row.number} · ${row.title}`,
        })
      : node('g', {});
    link.dataset.sheetLink = row.sheet;
    if (navigate) {
      link.addEventListener('pointerdown', (e) => e.stopPropagation());
      link.addEventListener('click', (e) => {
        e.preventDefault();
        e.stopPropagation();
        navigate(row.sheet);
      });
      link.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
          e.preventDefault();
          e.stopPropagation();
          navigate(row.sheet);
        }
      });
    }
    const title = node('title', {});
    title.textContent = `${row.number}\n${row.title}`;
    link.append(title);
    group.append(link);
    for (const [text, x, width] of [
      [row.number, list.x, column],
      [row.title, list.x + column, list.width - column],
    ] as [string, number, number][]) {
      const id = `list-cell-${++serial}`;
      const clip = node('clipPath', { id });
      clip.append(
        node('rect', { x: x + 1, y: y + 0.5, width: width - 2, height: list.row_height - 1 }),
      );
      const label = node('text', {
        x: x + 1,
        y: y + frame.text_height + 1,
        'font-size': frame.text_height,
        'clip-path': `url(#${id})`,
      });
      renderText(label, text, x + 1, y + frame.text_height + 1, frame.text_height);
      link.append(
        node('rect', {
          x,
          y,
          width,
          height: list.row_height,
          fill: 'transparent',
          stroke: frame.colour,
          'stroke-width': frame.stroke_width,
        }),
        clip,
        label,
      );
      const b = label.getBBox();
      if (b.x + b.width > x + width - 1 || b.y + b.height > y + list.row_height - 0.5) {
        const mark = node('path', {
          d: `M ${x + width - 2} ${y} h 2 v 2 Z`,
          fill: '#b96b00',
          class: 'overflow-marker',
        });
        link.append(mark);
      }
    }
  }
}
