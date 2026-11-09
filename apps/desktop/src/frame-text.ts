/** Paper text style resolution and alignment shared by template and sheet frame rendering. */
import type { TemplateTextStyle } from './types';
export function frameTextStyle(
  style: TemplateTextStyle | undefined,
  font: string,
  height: number,
  colour: string,
) {
  return {
    size: style?.height ? Number(style.height) / 1000 : height,
    attrs: {
      'font-family': style?.font ?? font,
      fill: style?.colour ?? colour,
      'font-weight': style?.bold ? 'bold' : 'normal',
      'font-style': style?.italic ? 'italic' : 'normal',
      'text-anchor':
        style?.horizontal === 'center' ? 'middle' : style?.horizontal === 'right' ? 'end' : 'start',
    },
  };
}
export function frameTextPosition(
  value: string,
  style: TemplateTextStyle | undefined,
  size: number,
  x: number,
  y: number,
  width: number,
  height: number,
) {
  const lines = value.split(/\r\n?|\n/).length;
  const extent = size + (lines - 1) * size * 1.3;
  return {
    x: x + (style?.horizontal === 'center' ? width / 2 : style?.horizontal === 'right' ? width : 0),
    y:
      y +
      size +
      (style?.vertical === 'middle'
        ? (height - extent) / 2
        : style?.vertical === 'bottom'
          ? height - extent
          : 0),
  };
}
