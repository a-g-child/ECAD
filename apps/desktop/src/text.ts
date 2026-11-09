/** One text value supports any line count; line layout never changes the document object type. */
import { element } from './ui';
export function textEditor(className: string, rows = 3) {
  const editor = element('textarea', className);
  editor.rows = rows;
  editor.wrap = 'off';
  editor.spellcheck = false;
  editor.title = 'Enter inserts a line break; Ctrl/Cmd+Enter applies; Escape cancels';
  return editor;
}
export function applyTextKey(event: KeyboardEvent) {
  return !event.isComposing && event.key === 'Enter' && (event.ctrlKey || event.metaKey);
}
/** Explicit baselines preserve blank lines, CRLF/CR imports, indentation and literal markup. */
export function renderText(text: SVGElement, value: string, x: number, y: number, height: number) {
  const lines = value.split(/\r\n?|\n/);
  text.setAttribute('aria-label', value);
  for (const [index, line] of lines.entries()) {
    const span = document.createElementNS('http://www.w3.org/2000/svg', 'tspan');
    span.setAttribute('x', String(x));
    span.setAttribute('y', String(y + index * height * 1.3));
    span.style.whiteSpace = 'pre';
    span.textContent = line;
    text.append(span);
  }
  // Semantic line boxes also include empty leading/trailing lines. Real glyph bounds
  // are unioned after attachment so unusual fonts cannot be clipped by these boxes.
  text.dataset.lines = String(lines.length);
}
export function reserveTextBounds(
  group: SVGElement,
  text: SVGGraphicsElement,
  x: number,
  y: number,
  height: number,
  className = 'text-bounds',
  includeSingleLine = false,
) {
  const count = Number(text.dataset.lines);
  const glyphs = text.getBBox();
  // Empty strings have an SVG box at (0, 0), not at the text anchor.
  const b = glyphs.width === 0 && glyphs.height === 0 ? { x, y, width: 0, height: 0 } : glyphs;
  if (count <= 1 && !includeSingleLine) return;
  const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
  const top = Math.min(b.y, y - height),
    bottom = Math.max(b.y + b.height, y + (count - 1) * height * 1.3 + height * 0.3);
  for (const [name, value] of Object.entries({
    x: Math.min(x, b.x),
    y: top,
    width: Math.max(
      includeSingleLine ? height * 0.3 : 0,
      Math.max(x, b.x + b.width) - Math.min(x, b.x),
    ),
    height: bottom - top,
  }))
    rect.setAttribute(name, String(value));
  rect.setAttribute('class', className);
  rect.setAttribute('fill', 'transparent');
  group.prepend(rect);
}
