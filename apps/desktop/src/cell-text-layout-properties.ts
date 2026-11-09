import { controlHelp } from './control-help';
/** Shared cell-block controls over explicit style references, including range/default editing. */
import type { TemplateCellTextLayout, TemplateTextStyles } from './types';
import { element } from './ui';
export class CellTextLayoutProperties {
  readonly root = element('section', 'template-text-properties');
  constructor(
    styles: TemplateTextStyles[],
    changed: () => void,
    defaults: boolean,
    complete: () => void,
  ) {
    const layouts = styles.map((s) => (s.layout ??= {}));
    const prefix = defaults ? 'Cell default' : 'Cell';
    this.root.append(
      element('h4', '', defaults ? 'Cell text layout defaults' : 'Cell text layout'),
    );
    const state = (key: keyof TemplateCellTextLayout) => {
      const values = layouts.map((l) => l[key] ?? null);
      return { value: values[0], mixed: values.some((v) => v !== values[0]) };
    };
    const set = (key: keyof TemplateCellTextLayout, value: string | null) => {
      for (const layout of layouts) layout[key] = value;
      changed();
    };
    const label = (name: string, control: HTMLElement) => {
      const row = element('label', 'settings-field');
      control.setAttribute('aria-label', `${prefix} ${name}`);
      row.append(element('span', '', name[0].toUpperCase() + name.slice(1)), control);
      this.root.append(row);
    };
    for (const [key, name, choices] of [
      [
        'caption_position',
        'caption position',
        [
          ['above', 'Above content'],
          ['inline', 'Inline with content'],
        ],
      ],
      [
        'horizontal',
        'horizontal justification',
        [
          ['left', 'Left'],
          ['center', 'Centre'],
          ['right', 'Right'],
        ],
      ],
      [
        'vertical',
        'vertical justification',
        [
          ['top', 'Top'],
          ['middle', 'Middle'],
          ['bottom', 'Bottom'],
        ],
      ],
    ] as const) {
      const select = element('select', 'property-input');
      controlHelp(
        select,
        key === 'caption_position'
          ? 'Place the caption above the content or inline before it. Default uses the template setting.'
          : 'Position the caption and content together within the cell. Default uses the template setting.',
      );
      const { value, mixed } = state(key);
      for (const [v, text] of [
        ['', 'Default'],
        ...choices,
        ...(mixed ? [['@mixed', 'Mixed']] : []),
      ]) {
        const option = element('option', '', text);
        option.value = v;
        option.disabled = v === '@mixed';
        select.append(option);
      }
      select.value = mixed ? '@mixed' : (value ?? '');
      select.addEventListener('change', () => set(key, select.value || null));
      label(name, select);
    }
    const separator = element('input', 'property-input');
    const current = state('separator');
    separator.value = current.mixed ? '' : (current.value ?? '');
    separator.placeholder = current.mixed
      ? 'Mixed'
      : current.value === ''
        ? 'No separator'
        : defaults
          ? 'Default (: )'
          : 'Default';
    separator.maxLength = 128;
    controlHelp(
      separator,
      'Used exactly as entered, including spaces. Only shown when both caption and content are present.',
    );
    separator.addEventListener('input', () => {
      separator.placeholder = 'No separator';
      set('separator', separator.value);
    });
    label('caption separator', separator);
    const reset = element('button', 'plain', 'Default separator');
    reset.type = 'button';
    reset.addEventListener('click', () => {
      separator.value = '';
      separator.placeholder = defaults ? 'Default (: )' : 'Default';
      set('separator', null);
      complete();
    });
    this.root.append(reset);
  }
}
