/** Display-only inverse placement. Commands and numeric fields retain exact source micrometres. */
import type { SheetRow } from './types';
export function sheetTransform(sheet: SheetRow) {
  const mm = (v: string) => Number(v) / 1000;
  const placed = !!sheet.view;
  const sx = placed ? mm(sheet.source_x) : 0,
    sy = placed ? mm(sheet.source_y) : 0;
  const px = placed ? mm(sheet.paper_x) : 0,
    py = placed ? mm(sheet.paper_y) : 0;
  const numerator = placed ? Number(sheet.numerator) : 1;
  const denominator = placed ? Number(sheet.denominator) : 1;
  const width = mm(sheet.width),
    height = mm(sheet.height);
  if (
    ![sx, sy, px, py, numerator, denominator, width, height].every(Number.isFinite) ||
    numerator <= 0 ||
    denominator <= 0 ||
    width <= 0 ||
    height <= 0
  )
    throw new Error('Invalid sheet display transform');
  const scale = denominator / numerator;
  return {
    inverse: `translate(${sx} ${sy}) scale(${scale}) translate(${-px} ${-py})`,
    box: ([x, y, w, h]: number[]) => [
      sx + (x - px) * scale,
      sy + (y - py) * scale,
      w * scale,
      h * scale,
    ],
    bounds: {
      x: sx - px * scale,
      y: sy - py * scale,
      width: width * scale,
      height: height * scale,
    },
  };
}
