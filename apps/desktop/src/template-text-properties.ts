/** Shared caption/content controls edit only the chosen property across a cell range. */
import type { TemplateTextStyle, TemplateTextStyles } from './types';
import { element } from './ui';
import { appearanceControl } from './appearance-controls';
import { dimension, readDimension } from './dimensions';
export class TemplateTextProperties {
  readonly root = element('section', 'template-text-properties');
  constructor(
    readonly styles: TemplateTextStyles[],
    changed: () => void,
    defaults = false,
    complete: () => void = () => {},
  ) {
    for (const channel of ['caption', 'content'] as const) {
      const section = element('details');
      section.open = true;
      const title = `${channel === 'caption' ? 'Caption' : 'Content'}${defaults ? ' default' : ''}`;
      section.append(element('summary', '', `${title} formatting`));
      for (const [key, name, choices] of [
        ['font', 'font', null],
        ['height', 'height (µm)', null],
        ['bold', 'bold', ['Normal', 'Bold']],
        ['italic', 'italic', ['Upright', 'Italic']],
        ['colour', 'colour', null],
        ['horizontal', 'horizontal alignment', ['left', 'center', 'right']],
        ['vertical', 'vertical alignment', ['top', 'middle', 'bottom']],
      ] as const) {
        const values = styles.map((s) => s[channel][key]);
        const mixed = values.some((v) => v !== values[0]);
        const value = mixed ? null : values[0];
        const field = choices
          ? element('select', 'property-input')
          : element('input', 'property-input');
        const labelText = element('span', '', `${title} ${name}`);
        const label = element('label', 'settings-field');
        field.setAttribute('aria-label', `${title} ${name}`);
        if (field instanceof HTMLSelectElement && choices) {
          const inherit = element('option', '', mixed ? 'Mixed' : 'Inherited');
          inherit.value = '';
          field.append(inherit);
          choices.forEach((v, i) => {
            const option = element('option', '', v);
            option.value = key === 'bold' || key === 'italic' ? String(i === 1) : v;
            field.append(option);
          });
          field.value = value == null ? '' : String(value);
        } else if (field instanceof HTMLInputElement) {
          field.value = value == null ? '' : String(value);
          field.placeholder = mixed ? 'Mixed' : 'Inherited';
          if (key === 'height') dimension(field, `${title} ${name}`, labelText);
        }
        const update = () => {
          const raw =
            field instanceof HTMLInputElement && key === 'height'
              ? readDimension(field)
              : field.value;
          const next =
            raw === '' ? null : key === 'bold' || key === 'italic' ? raw === 'true' : raw;
          for (const style of styles)
            (style[channel] as Record<keyof TemplateTextStyle, unknown>)[key] = next;
          changed();
        };
        field.addEventListener(choices ? 'change' : 'input', update);
        label.append(
          labelText,
          field instanceof HTMLInputElement && (key === 'font' || key === 'colour')
            ? appearanceControl(field, key)
            : field,
        );
        const reset = element('button', 'plain', 'Reset');
        reset.type = 'button';
        reset.setAttribute(
          'aria-label',
          `Reset ${title.toLowerCase()} ${name.replace(' (µm)', '')}`,
        );
        reset.addEventListener('click', () => {
          field.value = '';
          field.dispatchEvent(new Event(choices ? 'change' : 'input', { bubbles: true }));
          if (!choices) complete();
        });
        label.append(reset);
        section.append(label);
      }
      this.root.append(section);
    }
  }
}
