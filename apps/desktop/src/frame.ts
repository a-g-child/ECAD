import { renderPicture } from './picture-render';
/** Paper-unit frame rendering. Literal values stay intact; overflowing text is clipped and flagged. */
import type { FrameRow } from './types';
import { renderText } from './text';
import { frameTextStyle, frameTextPosition } from './frame-text';
import { renderTemplateGraphics } from './template-graphic-render';
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
  renderTemplateGraphics(group, frame.graphics);
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
    const captionStyle = frameTextStyle(cell.text_styles?.caption, frame.font, 1.5, frame.colour);
    const valueStyle = frameTextStyle(
      cell.text_styles?.content,
      frame.font,
      frame.text_height,
      frame.colour,
    );
    const strip = Object.values(cell.text_styles?.caption ?? {}).some((v) => v != null)
      ? Math.max(
          3,
          captionStyle.size * (1 + (cell.caption.split(/\r\n?|\n/).length - 1) * 1.3) + 1.5,
        )
      : 3;
    const captionAt = frameTextPosition(
      cell.caption,
      cell.text_styles?.caption,
      captionStyle.size,
      x + 1,
      y + 0.5,
      width - 2,
      strip - 1.5,
    );
    const caption = node('text', { ...captionStyle.attrs, 'font-size': captionStyle.size });
    renderText(caption, cell.caption, captionAt.x, captionAt.y, captionStyle.size);
    const valueAt = frameTextPosition(
      cell.value ?? '',
      cell.text_styles?.content,
      valueStyle.size,
      x + 1,
      y + strip,
      width - 2,
      height - strip - 1,
    );
    const id = `frame-value-${++serial}`;
    const clip = node('clipPath', { id });
    // Reserve the caption strip and a 1 mm inset. Text never silently shrinks.
    clip.append(
      node('rect', {
        x: x + 1,
        y: y + strip,
        width: Math.max(0, width - 2),
        height: Math.max(0, height - strip - 1),
      }),
    );
    const text = node('text', {
      ...valueStyle.attrs,
      'font-size': valueStyle.size,
      'clip-path': `url(#${id})`,
      class: 'frame-value',
    });
    renderText(text, cell.value ?? '', valueAt.x, valueAt.y, valueStyle.size);
    if (cell.picture) renderPicture(container, cell.picture, { x, y, width, height });
    container.append(title, box, caption, clip, text);
    group.append(container);
    const b = text.getBBox();
    const lines = (cell.value ?? '').split(/\r\n?|\n/).length;
    const semanticBottom = valueAt.y + (lines - 1) * valueStyle.size * 1.3;
    const overflow =
      cell.value &&
      (b.x < x + 1 ||
        b.x + b.width > x + width - 1 ||
        b.y < y + strip ||
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
