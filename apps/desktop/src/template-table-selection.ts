/** Paper/local micrometre hit tests and Excel-style rectangular closure around merged cells. */
import type { TemplateTable, TemplateCell } from './types';
export type TableSelection = { table: TemplateTable; cells: TemplateCell[] };
export function cellAt(table: TemplateTable, x: number, y: number) {
  const [tx, ty] = table.bounds.map(Number);
  return table.cells.find((c) => {
    const [cx, cy, w, h] = c.bounds.map(Number);
    return x >= tx + cx && x < tx + cx + w && y >= ty + cy && y < ty + cy + h;
  });
}
export function cellRange(table: TemplateTable, first: TemplateCell, last: TemplateCell) {
  const a = first.bounds.map(Number),
    b = last.bounds.map(Number);
  let x = Math.min(a[0], b[0]),
    y = Math.min(a[1], b[1]),
    r = Math.max(a[0] + a[2], b[0] + b[2]),
    d = Math.max(a[1] + a[3], b[1] + b[3]);
  let expanded = true;
  while (expanded) {
    expanded = false;
    for (const c of table.cells) {
      const [cx, cy, w, h] = c.bounds.map(Number);
      if (cx < r && cx + w > x && cy < d && cy + h > y) {
        const next = [Math.min(x, cx), Math.min(y, cy), Math.max(r, cx + w), Math.max(d, cy + h)];
        if (next[0] !== x || next[1] !== y || next[2] !== r || next[3] !== d) {
          [x, y, r, d] = next;
          expanded = true;
        }
      }
    }
  }
  return table.cells.filter((c) => {
    const [cx, cy, w, h] = c.bounds.map(Number);
    return cx >= x && cy >= y && cx + w <= r && cy + h <= d;
  });
}
export function paperPoint(svg: SVGSVGElement, e: { clientX: number; clientY: number }) {
  const matrix = svg.getScreenCTM();
  if (!matrix) return null;
  const p = svg.createSVGPoint();
  p.x = e.clientX;
  p.y = e.clientY;
  const at = p.matrixTransform(matrix.inverse());
  return { x: Math.round(at.x * 1000), y: Math.round(at.y * 1000) };
}
