/** View-only whole-object selection geometry; never inferred electrical connectivity. */
import type { SceneRow, Target } from './types';
import { keyOf } from './types';
export type Point = [number, number];
export type Rect = [number, number, number, number];
const contains = (r: Rect, p: Point) =>
  p[0] >= r[0] && p[0] <= r[2] && p[1] >= r[1] && p[1] <= r[3];
const overlaps = (a: Rect, b: Rect) => a[0] <= b[2] && a[2] >= b[0] && a[1] <= b[3] && a[3] >= b[1];
function distanceSquared(p: Point, a: Point, b: Point) {
  const dx = b[0] - a[0],
    dy = b[1] - a[1];
  const t = Math.max(
    0,
    Math.min(1, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (dx * dx + dy * dy || 1)),
  );
  return (p[0] - a[0] - t * dx) ** 2 + (p[1] - a[1] - t * dy) ** 2;
}
function crosses(a: Point, b: Point, r: Rect) {
  let lo = 0,
    hi = 1;
  for (let axis = 0; axis < 2; axis++) {
    const delta = b[axis] - a[axis];
    if (!delta) {
      if (a[axis] < r[axis] || a[axis] > r[axis + 2]) return false;
    } else {
      const x = (r[axis] - a[axis]) / delta,
        y = (r[axis + 2] - a[axis]) / delta;
      lo = Math.max(lo, Math.min(x, y));
      hi = Math.min(hi, Math.max(x, y));
      if (lo > hi) return false;
    }
  }
  return true;
}
function strokeTouches(a: Point, b: Point, r: Rect, radius: number) {
  if (crosses(a, b, r)) return true;
  const pointRectDistance = (p: Point) =>
    Math.max(r[0] - p[0], 0, p[0] - r[2]) ** 2 + Math.max(r[1] - p[1], 0, p[1] - r[3]) ** 2;
  const corners: Point[] = [
    [r[0], r[1]],
    [r[2], r[1]],
    [r[2], r[3]],
    [r[0], r[3]],
  ];
  return (
    Math.min(
      pointRectDistance(a),
      pointRectDistance(b),
      ...corners.map((p) => distanceSquared(p, a, b)),
    ) <=
    radius * radius
  );
}
// Square endpoint caps extend beyond the round segment-distance test at their corners.
function cap(a: Point, b: Point, radius: number): Point[] {
  const length = Math.hypot(a[0] - b[0], a[1] - b[1]);
  if (!length) return [];
  const x = ((a[0] - b[0]) * radius) / length,
    y = ((a[1] - b[1]) * radius) / length;
  return [
    [a[0] - y, a[1] + x],
    [a[0] - y + x, a[1] + x + y],
    [a[0] + y + x, a[1] - x + y],
    [a[0] + y, a[1] - x],
  ];
}
function polygonTouches(points: Point[], rect: Rect) {
  if (!points.length) return false;
  if (points.some((p) => contains(rect, p))) return true;
  if (points.some((p, i) => crosses(p, points[(i + 1) % points.length], rect))) return true;
  // If no edge crosses, a rectangle can still lie entirely inside the cap.
  const p: Point = [rect[0], rect[1]];
  const signs = points.map((a, i) => {
    const b = points[(i + 1) % points.length];
    return (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
  });
  return signs.every((v) => v >= 0) || signs.every((v) => v <= 0);
}
/** Annotation bounds come from the actual rendered font; route tests use segments, not their bounding box. */
export function marqueeTargets(
  svg: SVGSVGElement,
  rows: SceneRow[],
  from: Point,
  to: Point,
): Target[] {
  const rect: Rect = [
    Math.min(from[0], to[0]),
    Math.min(from[1], to[1]),
    Math.max(from[0], to[0]),
    Math.max(from[1], to[1]),
  ];
  const enclosed = to[0] >= from[0];
  const ink = new Map(
    [...svg.querySelectorAll<SVGGraphicsElement>('[data-object] .ink')].map((node) => [
      node.parentElement!.getAttribute('data-object')!,
      node,
    ]),
  );
  return rows
    .filter((row) => {
      const node = ink.get(keyOf(row.target));
      if (!node) return false;
      const b = row.shape === 'text' ? (node.parentNode as SVGGElement).getBBox() : node.getBBox();
      const radius = row.shape === 'path' ? row.width / 2 : row.shape === 'anchor' ? 0.175 : 0;
      const bounds: Rect = [
        b.x - radius,
        b.y - radius,
        b.x + b.width + radius,
        b.y + b.height + radius,
      ];
      const caps =
        row.shape === 'path' && row.points.length > 1
          ? [
              cap(row.points[0], row.points[1], radius),
              cap(row.points.at(-1)!, row.points.at(-2)!, radius),
            ]
          : [];
      for (const p of caps.flat()) {
        bounds[0] = Math.min(bounds[0], p[0]);
        bounds[1] = Math.min(bounds[1], p[1]);
        bounds[2] = Math.max(bounds[2], p[0]);
        bounds[3] = Math.max(bounds[3], p[1]);
      }
      if (enclosed)
        return contains(rect, [bounds[0], bounds[1]]) && contains(rect, [bounds[2], bounds[3]]);
      if (!overlaps(rect, bounds)) return false;
      if (row.shape === 'path')
        return (
          row.points.slice(1).some((p, i) => strokeTouches(row.points[i], p, rect, radius)) ||
          caps.some((points) => polygonTouches(points, rect))
        );
      if (row.shape === 'anchor') {
        const [x, y] = row.points[0];
        return (
          Math.max(rect[0] - x, 0, x - rect[2]) ** 2 + Math.max(rect[1] - y, 0, y - rect[3]) ** 2 <=
          (1.25 + radius) ** 2
        );
      }
      return true;
    })
    .map((row) => row.target);
}
