/** Exact unit conversion at the UI boundary. Stored/editor bridge values remain integer µm. */
export type LengthUnit = 'mm' | 'in' | 'um';
const factors = { mm: 1000n, in: 25400n, um: 1n };
const labels = { mm: 'mm', in: 'in', um: 'µm' };
let unit: LengthUnit = 'mm';
try {
  const saved = localStorage.getItem('ecad.length-unit');
  if (saved === 'mm' || saved === 'in' || saved === 'um') unit = saved;
} catch {
  /* A restricted host still has session preferences. */
}
export const lengthUnit = () => unit;
export const unitLabel = () => labels[unit];
export function parseLength(value: string, selected: LengthUnit = unit): string {
  const match = /^\s*([+-]?)(\d+(?:\.\d*)?|\.\d+)\s*(mm|in|um|µm|\")?\s*$/i.exec(value);
  if (!match) throw new Error('Enter a length in mm or inches, optionally with a unit suffix.');
  const suffix = match[3]?.toLowerCase();
  const chosen = suffix === '"' ? 'in' : suffix === 'µm' ? 'um' : (suffix ?? selected);
  const [whole, fraction = ''] = match[2].split('.');
  const denominator = 10n ** BigInt(fraction.length);
  const numerator = BigInt((whole || '0') + fraction) * factors[chosen as LengthUnit];
  if (numerator % denominator !== 0n)
    throw new Error('This length is finer than the supported 0.001 mm precision.');
  return (((match[1] === '-' ? -1n : 1n) * numerator) / denominator).toString();
}
export function formatLength(value: string, selected: LengthUnit = unit): string {
  if (!/^[+-]?\d+$/.test(value)) return value;
  const n = BigInt(value),
    sign = n < 0n ? '-' : '',
    absolute = n < 0n ? -n : n;
  const factor = factors[selected];
  // Display inches to six decimals; untouched controls retain the exact original integer.
  const scaled = (absolute * 1000000n + factor / 2n) / factor;
  const fraction = (scaled % 1000000n).toString().padStart(6, '0').replace(/0+$/, '');
  return `${sign}${scaled / 1000000n}${fraction ? '.' + fraction : ''}`;
}
type Field = HTMLInputElement | HTMLTextAreaElement;
type State = {
  original: string;
  shown: string;
  unit: LengthUnit;
  label?: HTMLElement;
  name?: string;
  points?: boolean;
  validTitle?: string;
};
const states = new WeakMap<Field, State>();
const fields = new Set<WeakRef<Field>>();
export function dimension(input: Field, name?: string, label?: HTMLElement) {
  const existing = states.get(input);
  if (existing) {
    existing.name = name ?? existing.name;
    existing.label = label ?? existing.label;
    if (existing.name)
      input.setAttribute('aria-label', existing.name.replace('(µm', `(${unitLabel()}`));
    return input;
  }
  states.set(input, { original: input.value, shown: input.value, unit, name, label });
  fields.add(new WeakRef(input));
  input.inputMode = 'decimal';
  input.dataset.dimension = '';
  writeDimension(input, input.value);
  input.addEventListener('input', () => {
    try {
      const state = states.get(input)!;
      if (state.points) parsePoints(input.value, state.unit);
      else parseLength(input.value, state.unit);
      if (state.validTitle !== undefined) input.title = state.validTitle;
      input.removeAttribute('aria-invalid');
    } catch (error) {
      const state = states.get(input)!;
      if (!input.hasAttribute('aria-invalid')) state.validTitle = input.title;
      input.setAttribute('aria-invalid', 'true');
      input.title = (error as Error).message;
    }
  });
  return input;
}
export function writeDimension(input: Field, value: string) {
  const state = states.get(input);
  if (!state) {
    input.value = value;
    return;
  }
  state.original = value;
  state.unit = unit;
  state.shown = state.points
    ? value
        .split(/\r\n?|\n/)
        .map((line) =>
          line
            .split(',')
            .map((v) => formatLength(v.trim()))
            .join(','),
        )
        .join('\n')
    : formatLength(value);
  input.value = state.shown;
  input.removeAttribute('aria-invalid');
  if (state.name) {
    const text = state.name.replace('(µm', `(${unitLabel()}`);
    input.setAttribute('aria-label', text);
    if (state.label) state.label.textContent = text;
  }
}
export function readDimension(input: Field): string {
  const state = states.get(input);
  if (!state || input.value === '') return input.value;
  if (input.value === state.shown) return state.original;
  try {
    return state.points
      ? parsePoints(input.value, state.unit)
      : parseLength(input.value, state.unit);
  } catch {
    return input.value + ' [invalid length]';
  } // Core rejects the complete proposal atomically.
}
/** Change presentation only; convert valid pending values without changing their intent. */
export function setLengthUnit(next: LengthUnit) {
  const live = [...fields].map((ref) => ({ ref, input: ref.deref() }));
  const values = live
    .filter((v): v is { ref: WeakRef<Field>; input: Field } => !!v.input)
    .map((v) => ({ ...v, value: readDimension(v.input) }));
  if (values.some((v) => v.input.isConnected && v.value.includes('[invalid length]')))
    throw new Error('Correct the unfinished length before changing units.');
  unit = next;
  for (const summary of document.querySelectorAll<HTMLElement>('[data-dimension-summary]'))
    refreshSummary(summary);
  for (const picker of document.querySelectorAll<HTMLSelectElement>('[data-unit-picker]'))
    picker.value = next;
  for (const v of values) writeDimension(v.input, v.value);
  for (const v of live) if (!v.input) fields.delete(v.ref);
  try {
    localStorage.setItem('ecad.length-unit', unit);
  } catch {
    /* Session-only fallback. */
  }
}

function parsePoints(value: string, selected: LengthUnit) {
  return value
    .split(/\r\n?|\n/)
    .map((line) => {
      const coordinates = line.split(',');
      if (coordinates.length !== 2) throw new Error('Enter two coordinates per line.');
      return coordinates.map((v) => parseLength(v, selected)).join(',');
    })
    .join('\n');
}
export function dimensionPoints(input: Field, name: string, label: HTMLElement) {
  const original = input.value;
  dimension(input, name, label);
  states.get(input)!.points = true;
  writeDimension(input, original);
}

/** A read-only coordinate summary follows the same user preference as editable fields. */
export function dimensionSummary(values: string[], prefix: string, className = 'inspector-note') {
  const summary = document.createElement('p');
  summary.className = className;
  summary.dataset.dimensionSummary = JSON.stringify(values);
  summary.dataset.prefix = prefix;
  refreshSummary(summary);
  return summary;
}
function refreshSummary(summary: HTMLElement) {
  const values = JSON.parse(summary.dataset.dimensionSummary!) as string[];
  summary.textContent = `${summary.dataset.prefix}${values.map((v) => formatLength(v)).join(', ')} ${unitLabel()}`;
}
