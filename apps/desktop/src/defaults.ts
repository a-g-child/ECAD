/** Project fallbacks are reviewed locally; Apply submits one guarded settings command. */
import { unitPicker } from './unit-picker';
import { dimension, readDimension, writeDimension } from './dimensions';
import type { Context, DrawingDefaults, Reply } from './types';
import type { Shell } from './shell';
import { element } from './ui';
import { appearanceControl } from './appearance-controls';
import { applyTextKey } from './text';
import { FontPalette } from './font-palette';
import type { FontProperties } from './font-properties';
import { send, inputId, errorText } from './transport';
export class DefaultsDialog {
  private dialog = element('dialog', 'settings-dialog');
  private form = element('form', 'settings-form');
  private error = element('div', 'settings-error');
  private applyButton = element('button', 'plain', 'Apply defaults');
  private cancel = element('button', 'plain', 'Cancel');
  private fields = new Map<keyof DrawingDefaults, HTMLInputElement | HTMLTextAreaElement>();
  private fontHost = element('div');
  private fontValues: FontProperties = {};
  private captured: Context | null = null;
  private pending = false;
  get active() {
    return this.dialog.open;
  }
  constructor(
    private shell: Shell,
    private apply: (reply: Reply) => Promise<void>,
    private status: (message: string, error?: boolean) => void,
  ) {
    const heading = element('h2', '', 'Drawing defaults');
    heading.id = 'drawing-defaults-heading';
    this.dialog.setAttribute('aria-labelledby', heading.id);
    this.form.append(
      heading,
      element(
        'p',
        'settings-note',
        'Project defaults update drawing properties that use the default. Object, view and named-style overrides take precedence.',
      ),
    );
    this.form.append(this.fontHost);
    const definitions: [keyof DrawingDefaults, string][] = [
      ['stroke_width', 'Default stroke width (µm)'],
      ['colour', 'Default output colour'],
    ];
    for (const [key, caption] of definitions) {
      const label = element('label', 'settings-field');
      const field = element('input', 'property-input');
      field.setAttribute('aria-label', caption);
      if (key === 'text_height' || key === 'stroke_width') field.inputMode = 'numeric';
      this.fields.set(key, field);
      label.append(
        element('span', '', caption),
        key === 'colour' ? appearanceControl(field, key) : field,
      );
      this.form.append(label);
      if (key === 'text_height' || key === 'stroke_width')
        dimension(field, caption, label.firstElementChild as HTMLElement);
    }
    this.form.append(
      element(
        'p',
        'settings-note',
        'Lengths use the selected units (0.001 mm internal precision); colour uses #rrggbb. Font availability depends on the system. Creation presets and named styles are retained; their editors are planned.',
      ),
    );
    this.form.append(unitPicker());
    this.error.setAttribute('role', 'alert');
    this.error.setAttribute('aria-live', 'polite');
    const actions = element('div', 'settings-actions');
    this.applyButton.type = 'submit';
    this.cancel.type = 'button';
    actions.append(this.cancel, this.applyButton);
    this.form.append(this.error, actions);
    this.dialog.append(this.form);
    shell.app.append(this.dialog);
    this.form.addEventListener('submit', (e) => {
      e.preventDefault();
      void this.commit();
    });
    this.form.addEventListener('keydown', (e) => {
      if (applyTextKey(e)) {
        e.preventDefault();
        void this.commit();
      }
    });
    this.cancel.addEventListener('click', () => this.close());
    this.dialog.addEventListener('cancel', (e) => {
      e.preventDefault();
      if (!this.pending) this.close();
    });
  }
  open(context: Context, values: DrawingDefaults) {
    this.captured = context;
    this.error.textContent = '';
    for (const [key, field] of this.fields) {
      writeDimension(field, values[key]);
      field.dispatchEvent(new Event('input'));
    }
    this.fontValues = { font: values.font, height: values.text_height };
    const palette = new FontPalette(
      [this.fontValues],
      'Default',
      () => {},
      () => {},
      {
        defaults: [],
        labels: { font: 'Default font family', height: 'Default text height (µm)' },
      },
    );
    palette.root.addEventListener('keydown', (e) => {
      if (
        e.key === 'Enter' &&
        !e.ctrlKey &&
        !e.metaKey &&
        !(e.target instanceof HTMLTextAreaElement)
      )
        e.preventDefault();
    });
    this.fontHost.replaceChildren(palette.root);
    this.dialog.showModal();
    palette.root.querySelector('textarea')!.focus();
  }
  private close() {
    this.dialog.close();
    this.captured = null;
    this.shell.canvas.focus();
  }
  private async commit() {
    if (this.pending || !this.captured) return;
    this.pending = true;
    this.error.textContent = '';
    for (const field of this.form.querySelectorAll<HTMLInputElement>('input,textarea,select'))
      field.disabled = true;
    this.applyButton.disabled = this.cancel.disabled = true;
    try {
      const defaults = Object.fromEntries(
        [...this.fields].map(([key, field]) => [key, readDimension(field)]),
      );
      defaults.font = this.fontValues.font ?? '';
      defaults.text_height = this.fontValues.height ?? '';
      const reply = await send({
        type: 'update_defaults',
        context: this.captured,
        defaults,
        input: inputId(),
      });
      this.close();
      await this.apply(reply);
      this.status(reply.message ?? 'Drawing defaults updated');
    } catch (error) {
      this.error.textContent = errorText(error);
    } finally {
      this.pending = false;
      for (const field of this.form.querySelectorAll<HTMLInputElement>('input,textarea,select'))
        field.disabled = false;
      this.applyButton.disabled = this.cancel.disabled = false;
    }
  }
}
