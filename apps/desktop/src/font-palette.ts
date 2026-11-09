/** Compact font controls over explicit style references; the owner validates and commits edits. */
import type { TemplateTextStyle } from './types';
import { element } from './ui';
import { dimension, readDimension } from './dimensions';
import { colourPicker } from './colour-picker';
let fontListId = 0;
const fontNames = [
  'Arial',
  'Calibri',
  'Helvetica',
  'Verdana',
  'Times New Roman',
  'Georgia',
  'Courier New',
  'sans-serif',
  'serif',
  'monospace',
];
type Key = keyof TemplateTextStyle;
export class FontPalette {
  readonly root = element('section', 'font-palette');
  constructor(
    private styles: TemplateTextStyle[],
    private title: string,
    private changed: () => void,
    private complete: () => void,
  ) {
    this.root.setAttribute('aria-label', `${title} formatting`);
    this.root.append(element('h4', 'font-palette-heading', `${title} formatting`));
    const familyRow = element('div', 'font-palette-family');
    const family = this.input('font', `${title} font`);
    family.title = 'Choose a font family or type its name. Availability depends on this computer.';
    const list = element('datalist');
    list.id = `font-families-${++fontListId}`;
    for (const name of fontNames) {
      const option = element('option');
      option.value = name;
      list.append(option);
    }
    family.setAttribute('list', list.id);
    familyRow.append(family, list, this.style());
    const tools = element('div', 'font-palette-tools');
    const height = this.input('height', `${title} height (µm)`);
    const size = element('label', 'font-palette-size');
    const sizeLabel = element('span', '', 'Height (µm)');
    dimension(height, `${title} height (µm)`, sizeLabel);
    // The control's accessible name follows units; a visible label uses the same shared conversion.
    size.append(sizeLabel, height);
    tools.append(size, this.toggle('bold', 'B'), this.toggle('italic', 'I'));
    const colour = this.input('colour', `${title} colour`);
    colour.type = 'hidden';
    const picker = colourPicker(colour);
    picker.classList.add('font-palette-colour');
    picker.querySelector('.colour-picker-hint')!.textContent = 'A';
    tools.append(picker, colour);
    const alignment = element('div', 'font-palette-alignment');
    alignment.append(
      this.choice('horizontal', 'horizontal alignment', ['left', 'center', 'right']),
      this.choice('vertical', 'vertical alignment', ['top', 'middle', 'bottom']),
      this.reset(),
    );
    this.root.append(familyRow, tools, alignment);
  }
  private state(key: Key) {
    const values = this.styles.map((s) => s[key]);
    return { value: values[0], mixed: values.some((v) => v !== values[0]) };
  }
  private set(key: Key, value: string | boolean | null) {
    for (const style of this.styles) (style as Record<Key, unknown>)[key] = value;
    this.changed();
  }
  private input(key: 'font' | 'height' | 'colour', name: string) {
    const field = element('input', 'property-input');
    const state = this.state(key);
    field.value = state.mixed || state.value == null ? '' : String(state.value);
    field.placeholder = state.mixed ? 'Mixed' : 'Inherited';
    field.setAttribute('aria-label', name);
    field.addEventListener('input', () => {
      const value = key === 'height' ? readDimension(field) : field.value;
      this.set(key, value === '' ? null : value);
    });
    return field;
  }
  private style() {
    const select = element('select', 'property-input');
    select.setAttribute('aria-label', `${this.title} font style`);
    for (const [value, label] of [
      ['inherit', 'Inherited'],
      ['regular', 'Regular'],
      ['bold', 'Bold'],
      ['italic', 'Italic'],
      ['bold-italic', 'Bold italic'],
      ['mixed', 'Mixed'],
    ]) {
      const option = element('option', '', label);
      option.value = value;
      option.disabled = value === 'mixed';
      select.append(option);
    }
    const bold = this.state('bold'),
      italic = this.state('italic');
    select.value =
      bold.mixed || italic.mixed
        ? 'mixed'
        : bold.value == null && italic.value == null
          ? 'inherit'
          : bold.value
            ? italic.value
              ? 'bold-italic'
              : 'bold'
            : italic.value
              ? 'italic'
              : 'regular';
    select.addEventListener('change', () => {
      this.set('bold', select.value === 'inherit' ? null : select.value.includes('bold'));
      this.set('italic', select.value === 'inherit' ? null : select.value.includes('italic'));
    });
    return select;
  }
  private toggle(key: 'bold' | 'italic', glyph: string) {
    const button = element('button', `plain font-toggle font-${key}`, glyph);
    button.type = 'button';
    button.setAttribute('aria-label', `${this.title} ${key}`);
    const state = this.state(key);
    button.setAttribute('aria-pressed', state.mixed ? 'mixed' : String(state.value === true));
    button.title = `${this.title} ${key}${state.mixed ? ' · Mixed' : state.value == null ? ' · Inherited' : ''}`;
    button.addEventListener('click', () => {
      const current = this.state(key);
      const next = current.mixed || current.value !== true;
      this.set(key, next);
      button.setAttribute('aria-pressed', String(next));
      this.complete();
    });
    return button;
  }
  private choice(key: 'horizontal' | 'vertical', name: string, choices: string[]) {
    const select = element('select', 'property-input');
    select.setAttribute('aria-label', `${this.title} ${name}`);
    select.title = `${this.title} ${name}`;
    const state = this.state(key);
    const inherited = element('option', '', state.mixed ? 'Mixed' : 'Inherited');
    inherited.value = '';
    select.append(inherited);
    for (const value of choices) {
      const option = element('option', '', value[0].toUpperCase() + value.slice(1));
      option.value = value;
      select.append(option);
    }
    select.value = state.mixed || state.value == null ? '' : String(state.value);
    select.addEventListener('change', () => this.set(key, select.value || null));
    return select;
  }
  private reset() {
    const select = element('select', 'property-input font-reset');
    select.setAttribute('aria-label', `Reset ${this.title.toLowerCase()} formatting`);
    const prompt = element('option', '', 'Reset…');
    prompt.value = '';
    select.append(prompt);
    for (const [value, label] of [
      ['all', 'All formatting'],
      ['font', 'Font'],
      ['height', 'Height'],
      ['bold', 'Bold'],
      ['italic', 'Italic'],
      ['colour', 'Colour'],
      ['horizontal', 'Horizontal alignment'],
      ['vertical', 'Vertical alignment'],
    ]) {
      const option = element('option', '', label);
      option.value = value;
      select.append(option);
    }
    select.addEventListener('change', () => {
      if (!select.value) return;
      const keys: Key[] =
        select.value === 'all'
          ? ['font', 'height', 'bold', 'italic', 'colour', 'horizontal', 'vertical']
          : [select.value as Key];
      for (const key of keys) this.set(key, null);
    });
    return select;
  }
}
