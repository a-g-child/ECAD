/** Optional appearance choices preserve exact custom values; owners retain validation and history. */
import { element } from './ui';
export type AppearanceKind = 'font' | 'colour';
const fonts = [
  ['sans-serif', 'Sans serif'],
  ['serif', 'Serif'],
  ['monospace', 'Monospace'],
] as const;
const colours = [
  ['#000000', 'Black'],
  ['#ffffff', 'White'],
  ['#808080', 'Grey'],
  ['#ff0000', 'Red'],
  ['#008000', 'Green'],
  ['#0000ff', 'Blue'],
  ['#ffff00', 'Yellow'],
] as const;
/** Choices emit normal input/change intent, never mutate documents or infer style ownership. */
export function appearanceControl(
  input: HTMLInputElement | HTMLTextAreaElement,
  kind: AppearanceKind,
  chosen?: () => void,
) {
  const root = element('div', 'appearance-control');
  const select = element('select', 'property-input');
  select.setAttribute('aria-label', `${input.getAttribute('aria-label')} preset`);
  const custom = element('option', '', input.placeholder === 'Mixed' ? 'Mixed / custom' : 'Custom');
  custom.value = '';
  select.append(custom);
  for (const [value, name] of kind === 'font' ? fonts : colours) {
    const option = element('option', '', name);
    option.value = value;
    select.append(option);
  }
  const sync = () => {
    const value = kind === 'colour' ? input.value.toLowerCase() : input.value;
    select.value = Array.from(select.options).some((o) => o.value === value) ? value : '';
  };
  input.addEventListener('input', sync);
  select.addEventListener('change', () => {
    if (!select.value) return; // Custom retains the exact field; it never clears an override.
    input.value = select.value;
    input.dispatchEvent(new Event('input', { bubbles: true }));
    chosen?.();
  });
  sync();
  root.append(select, input);
  return root;
}
