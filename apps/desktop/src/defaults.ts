/** Project fallbacks are reviewed locally; Apply submits one guarded settings command. */
import type { Context, DrawingDefaults, Reply } from './types';
import type { Shell } from './shell';
import { element } from './ui';
import { textEditor, applyTextKey } from './text';
import { send, inputId, errorText } from './transport';
export class DefaultsDialog {
  private dialog = element('dialog', 'settings-dialog');
  private form = element('form', 'settings-form');
  private error = element('div', 'settings-error');
  private applyButton = element('button', 'plain', 'Apply defaults');
  private cancel = element('button', 'plain', 'Cancel');
  private fields = new Map<keyof DrawingDefaults, HTMLInputElement | HTMLTextAreaElement>();
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
        'Project fallbacks update inherited drawing properties. Object, view and named-style overrides take precedence.',
      ),
    );
    const definitions: [keyof DrawingDefaults, string][] = [
      ['font', 'Default font family'],
      ['text_height', 'Default text height (µm)'],
      ['stroke_width', 'Default stroke width (µm)'],
      ['colour', 'Default output colour'],
    ];
    for (const [key, caption] of definitions) {
      const label = element('label', 'settings-field');
      const field =
        key === 'font'
          ? textEditor('property-input property-text', 2)
          : element('input', 'property-input');
      field.setAttribute('aria-label', caption);
      if (key === 'text_height' || key === 'stroke_width') field.inputMode = 'numeric';
      this.fields.set(key, field);
      label.append(element('span', '', caption), field);
      this.form.append(label);
    }
    this.form.append(
      element(
        'p',
        'settings-note',
        'Lengths use positive integer micrometres; colour uses #rrggbb. Font availability depends on the system. Creation presets and named styles are retained; their editors are planned.',
      ),
    );
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
    for (const [key, field] of this.fields) field.value = values[key];
    this.dialog.showModal();
    this.fields.get('font')!.focus();
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
    for (const field of this.fields.values()) field.disabled = true;
    this.applyButton.disabled = this.cancel.disabled = true;
    try {
      const defaults = Object.fromEntries(
        [...this.fields].map(([key, field]) => [key, field.value]),
      );
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
      for (const field of this.fields.values()) field.disabled = false;
      this.applyButton.disabled = this.cancel.disabled = false;
    }
  }
}
