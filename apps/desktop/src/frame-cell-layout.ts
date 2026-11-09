/** Caption/value block flow, measured in paper units and shared by preview and real sheets. */
import type { FrameRow } from './types';
import { frameTextStyle } from './frame-text';
import { renderText } from './text';
type Cell = FrameRow['cells'][number];
const ns = 'http://www.w3.org/2000/svg';
let serial = 0;
function node<K extends keyof SVGElementTagNameMap>(
  tag: K,
  attrs: Record<string, string | number> = {},
) {
  const n = document.createElementNS(ns, tag);
  for (const [key, value] of Object.entries(attrs)) n.setAttribute(key, String(value));
  return n;
}
/** Render a configured cell block. Returns whether exact content exceeds its inset, without shrinking. */
export function renderCellTextLayout(
  parent: SVGElement,
  cell: Cell,
  frame: Omit<FrameRow, 'sheet'>,
) {
  const layout = cell.text_styles?.layout ?? {};
  const cap = frameTextStyle(cell.text_styles?.caption, frame.font, 1.5, frame.colour);
  const val = frameTextStyle(
    cell.text_styles?.content,
    frame.font,
    frame.text_height,
    frame.colour,
  );
  const caption = cell.caption,
    value = cell.value ?? '';
  const inline = layout.caption_position === 'inline';
  const horizontal = layout.horizontal ?? cell.text_styles?.content.horizontal ?? 'left';
  const vertical = layout.vertical ?? cell.text_styles?.content.vertical ?? 'top';
  const clip = node('clipPath', { id: `cell-block-${++serial}` });
  const { x, y, width, height } = cell.bounds;
  const inner = {
    x: x + 1,
    y: y + 1,
    width: Math.max(0, width - 2),
    height: Math.max(0, height - 2),
  };
  clip.append(node('rect', inner));
  const clipped = node('g', { 'clip-path': `url(#${clip.id})` });
  const block = node('g', {
    class: 'cell-text-layout',
    'data-caption-position': inline ? 'inline' : 'above',
  });
  const capText = node('text', {
    ...cap.attrs,
    'font-size': cap.size,
    'text-anchor': 'start',
    class: 'frame-caption',
  });
  const valText = node('text', {
    ...val.attrs,
    'font-size': val.size,
    'text-anchor': 'start',
    class: 'frame-value',
  });
  block.append(capText, valText);
  clipped.append(block);
  parent.append(clip, clipped);
  const capLines = caption.split(/\r\n?|\n/).length;
  const capHeight = caption ? cap.size * (1 + (capLines - 1) * 1.3) : 0;
  let capY = cap.size,
    valueX = 0,
    valueY = val.size;
  if (inline && caption && value) capY = Math.max(cap.size, val.size);
  renderText(capText, caption, 0, capY, cap.size);
  let separator: SVGTextElement | null = null;
  if (caption && value) {
    if (inline) {
      const last = capText.querySelector<SVGTSpanElement>('tspan:last-child');
      valueY = capY + (capLines - 1) * cap.size * 1.3;
      valueX = last?.getComputedTextLength() ?? 0;
      separator = node('text', {
        ...cap.attrs,
        'font-size': cap.size,
        'text-anchor': 'start',
        class: 'frame-separator',
      });
      renderText(separator, layout.separator ?? ': ', valueX, valueY, cap.size);
      block.append(separator);
      valueX += separator.getComputedTextLength();
    } else valueY += capHeight + 0.75;
  }
  renderText(valText, value, valueX, valueY, val.size);
  // Paragraph alignment in above mode stays independent of whole-block justification.
  if (!inline) {
    const width = Math.max(capText.getBBox().width, valText.getBBox().width);
    for (const [text, content, style, size, baseline] of [
      [capText, caption, cell.text_styles?.caption, cap.size, capY],
      [valText, value, cell.text_styles?.content, val.size, valueY],
    ] as const) {
      const align = style?.horizontal ?? horizontal;
      const at = align === 'center' ? width / 2 : align === 'right' ? width : 0;
      text.setAttribute(
        'text-anchor',
        align === 'center' ? 'middle' : align === 'right' ? 'end' : 'start',
      );
      text.replaceChildren();
      renderText(text, content, at, baseline, size);
    }
  }
  const measured = block.getBBox();
  // Include blank lines and a nominal font-height box even where no glyphs are painted.
  const semanticBottom = Math.max(
    caption ? capY + (capLines - 1) * cap.size * 1.3 : 0,
    value ? valueY + (value.split(/\r\n?|\n/).length - 1) * val.size * 1.3 : 0,
  );
  const top = Math.min(0, measured.y),
    bottom = Math.max(semanticBottom, measured.y + measured.height);
  const extent = bottom - top;
  const dx =
    inner.x -
    measured.x +
    (horizontal === 'center'
      ? (inner.width - measured.width) / 2
      : horizontal === 'right'
        ? inner.width - measured.width
        : 0);
  const dy =
    inner.y -
    top +
    (vertical === 'middle'
      ? (inner.height - extent) / 2
      : vertical === 'bottom'
        ? inner.height - extent
        : 0);
  block.setAttribute('transform', `translate(${dx} ${dy})`);
  return (
    !!(caption || value) && (measured.width > inner.width + 1e-6 || extent > inner.height + 1e-6)
  );
}
