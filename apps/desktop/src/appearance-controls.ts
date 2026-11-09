/** Shared non-text colour control; all font editing belongs to FontPalette. */
import { element } from './ui';
import { colourPicker } from './colour-picker';
export type AppearanceKind = 'colour';
export function appearanceControl(
  input: HTMLInputElement | HTMLTextAreaElement,
  _kind: AppearanceKind,
  chosen?: () => void,
) {
  const root = element('div', 'appearance-control');
  root.append(colourPicker(input, chosen), input);
  return root;
}
