/** Compact font controls over explicit style references; the owner validates and commits edits. */
import type { FontProperties, FontProperty, FontPaletteOptions } from './font-properties';
import { element } from './ui';
import { dimension, readDimension } from './dimensions';
import { textEditor } from './text';
import { colourPicker } from './colour-picker';
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
type Key = FontProperty;
export class FontPalette {
  readonly root = element('section', 'font-palette');
  constructor(
    private styles: FontProperties[],
    private title: string,
    private changed: (key: Key) => void,
    private complete: () => void,
    private options: FontPaletteOptions = {},
  ) {
    this.root.setAttribute('aria-label', `${title} formatting`);
    this.root.append(element('h4', 'font-palette-heading', `${title} formatting`));
    const familyRow = element('div', 'font-palette-family');
    const family = this.input('font', `${title} font`);
    family.title +=
      '\nChoose a font family or type its name. Availability depends on this computer.';
    const familyControl = element('div', 'font-family-control appearance-control');
    const choices = element('select', 'property-input font-family-choices');
    choices.setAttribute('aria-label', `${family.getAttribute('aria-label')} choices`);
    choices.title = 'Choose font family';
    const custom = element('option', '', 'Custom');
    custom.value = '';
    choices.append(custom);
    if (this.defaultKeys().includes('font')) {
      const option = element('option', '', 'Default');
      option.value = '@default';
      choices.append(option);
    }
    for (const name of fontNames) {
      const option = element('option', '', name);
      option.value = name;
      choices.append(option);
    }
    const sync = () => {
      choices.value = fontNames.includes(family.value) ? family.value : '';
    };
    family.addEventListener('input', sync);
    choices.addEventListener('change', () => {
      if (!choices.value) return;
      family.value = choices.value === '@default' ? '' : choices.value;
      family.dispatchEvent(new Event('input', { bubbles: true }));
    });
    sync();
    familyControl.append(family, choices);
    familyRow.append(familyControl);
    if (this.supports('bold') && this.supports('italic')) familyRow.append(this.style());
    else familyRow.classList.add('font-palette-family-only');
    const tools = element('div', 'font-palette-tools');
    if (this.supports('height')) {
      const height = this.input('height', `${title} height (µm)`);
      const size = element('label', 'font-palette-size');
      const sizeLabel = element('span', '', 'Height (µm)');
      dimension(height, height.getAttribute('aria-label')!, sizeLabel);
      size.append(sizeLabel, height);
      tools.append(size);
    }
    for (const [key, glyph] of [
      ['bold', 'B'],
      ['italic', 'I'],
    ] as const)
      if (this.supports(key)) tools.append(this.toggle(key, glyph));
    if (this.supports('colour')) {
      const colour = this.input('colour', `${title} colour`);
      colour.type = 'hidden';
      const picker = colourPicker(colour);
      picker.classList.add('font-palette-colour');
      picker.querySelector('.colour-picker-hint')!.textContent = 'A';
      tools.append(picker, colour);
    }
    const alignment = element('div', 'font-palette-alignment');
    if (this.supports('horizontal'))
      alignment.append(
        this.choice('horizontal', 'horizontal alignment', ['left', 'center', 'right']),
      );
    if (this.supports('vertical'))
      alignment.append(this.choice('vertical', 'vertical alignment', ['top', 'middle', 'bottom']));
    if (this.defaultKeys().length) alignment.append(this.defaults());
    if (!this.supports('bold')) tools.classList.add('font-palette-basic');
    if (!this.supports('horizontal')) alignment.classList.add('font-palette-basic');
    this.root.append(familyRow, tools, alignment);
  }
  private supports(key: Key) {
    return !this.options.exclude?.includes(key) && this.styles.some((s) => key in s);
  }
  private defaultKeys() {
    return (this.options.defaults ?? (Object.keys(this.styles[0]) as Key[])).filter((key) =>
      this.supports(key),
    );
  }
  private state(key: Key) {
    const values = this.styles.filter((s) => key in s).map((s) => s[key]);
    return {
      value: values[0],
      mixed: this.options.mixed?.includes(key) || values.some((v) => v !== values[0]),
    };
  }
  private set(key: Key, value: string | boolean | null) {
    for (const style of this.styles) if (key in style) (style as Record<Key, unknown>)[key] = value;
    this.options.mixed = this.options.mixed?.filter((k) => k !== key);
    this.changed(key);
  }
  private input(key: 'font', name: string): HTMLTextAreaElement;
  private input(key: 'height' | 'colour', name: string): HTMLInputElement;
  private input(key: 'font' | 'height' | 'colour', name: string) {
    const field =
      key === 'font'
        ? textEditor('property-input font-family-value', 1)
        : element('input', 'property-input');
    const state = this.state(key);
    field.value = state.mixed || state.value == null ? '' : String(state.value);
    field.placeholder = state.mixed ? 'Mixed' : this.defaultKeys().includes(key) ? 'Default' : '';
    field.setAttribute('aria-label', this.options.labels?.[key] ?? name);
    this.options.field?.(key, field);
    field.addEventListener('input', () => {
      const value = key === 'height' ? readDimension(field) : field.value;
      this.set(key, value === '' && this.defaultKeys().includes(key) ? null : value);
    });
    return field;
  }
  private style() {
    const select = element('select', 'property-input');
    select.setAttribute('aria-label', `${this.title} font style`);
    for (const [value, label] of [
      ['inherit', 'Default'],
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
    button.title = `${this.title} ${key}${state.mixed ? ' · Mixed' : state.value == null ? ' · Default' : ''}`;
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
    if (state.mixed) {
      const mixed = element('option', '', 'Mixed');
      mixed.value = 'mixed';
      mixed.disabled = true;
      select.append(mixed);
    }
    const fallback = element('option', '', 'Default');
    fallback.value = '';
    select.append(fallback);
    for (const value of choices) {
      const option = element('option', '', value[0].toUpperCase() + value.slice(1));
      option.value = value;
      select.append(option);
    }
    select.value = state.mixed ? 'mixed' : state.value == null ? '' : String(state.value);
    select.addEventListener('change', () => this.set(key, select.value || null));
    return select;
  }
  private defaults() {
    const select = element('select', 'property-input font-reset');
    select.setAttribute('aria-label', `Use default ${this.title.toLowerCase()} formatting`);
    select.title = 'Choose a property to use its default; other properties stay unchanged.';
    const prompt = element('option', '', 'Default…');
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
      if (value !== 'all' && !this.defaultKeys().includes(value as Key)) continue;
      const option = element('option', '', label);
      option.value = value;
      select.append(option);
    }
    select.addEventListener('change', () => {
      if (!select.value) return;
      const keys: Key[] = select.value === 'all' ? this.defaultKeys() : [select.value as Key];
      for (const key of keys) this.set(key, null);
    });
    return select;
  }
}
