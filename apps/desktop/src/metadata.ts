/** Portable project information; profile import and sheet placement are separate future workflows. */
import type { Context, MetadataRow, Reply } from './types';
import type { Shell } from './shell';
import { element } from './ui';
import { textEditor, applyTextKey } from './text';
import { send, inputId, errorText } from './transport';
export class MetadataDialog {
  private dialog = element('dialog', 'settings-dialog metadata-dialog');
  private form = element('form', 'settings-form');
  private body = element('div', 'metadata-fields');
  private error = element('div', 'settings-error');
  private applyButton = element('button', 'plain', 'Apply information');
  private cancel = element('button', 'plain', 'Cancel');
  private fields = new Map<string, HTMLTextAreaElement>();
  private captured: Context | null = null;
  private pending = false;
  get active() {
    return this.dialog.open;
  }
  constructor(
    private shell: Shell,
    private apply: (reply: Reply) => Promise<void>,
    private status: (message: string) => void,
  ) {
    const title = element('h2', '', 'Project information');
    title.id = 'project-information-heading';
    this.dialog.setAttribute('aria-labelledby', title.id);
    this.error.setAttribute('role', 'alert');
    this.error.setAttribute('aria-live', 'polite');
    const actions = element('div', 'settings-actions');
    this.applyButton.type = 'submit';
    this.cancel.type = 'button';
    actions.append(this.cancel, this.applyButton);
    this.form.append(
      title,
      element(
        'p',
        'settings-note',
        'These values travel with this project. Engineer/contact is separate from drawn, checked or approved responsibilities. Profile import and sheet templates are planned.',
      ),
      this.body,
      this.error,
      actions,
    );
    this.dialog.append(this.form);
    shell.app.append(this.dialog);
    this.form.addEventListener('submit', (event) => {
      event.preventDefault();
      void this.commit();
    });
    this.form.addEventListener('keydown', (event) => {
      if (applyTextKey(event)) {
        event.preventDefault();
        void this.commit();
      }
    });
    this.cancel.addEventListener('click', () => this.close());
    this.dialog.addEventListener('cancel', (event) => {
      event.preventDefault();
      if (!this.pending) this.close();
    });
  }
  open(context: Context, rows: MetadataRow[]) {
    this.captured = context;
    this.error.textContent = '';
    this.fields.clear();
    this.body.replaceChildren();
    for (const [group, project] of [
      ['Company / contact', false],
      ['Project', true],
    ] as const) {
      this.body.append(element('strong', 'metadata-group', group));
      for (const row of rows.filter((row) => row.key.startsWith('project.') === project)) {
        const label = element('label', 'settings-field');
        const field = textEditor('property-input property-text', 2);
        field.setAttribute('aria-label', row.label);
        field.value = row.value;
        const caption = element('span', '', row.label);
        caption.append(element('small', 'metadata-key', row.key));
        label.append(caption, field);
        this.body.append(label);
        this.fields.set(row.key, field);
      }
    }
    this.dialog.showModal();
    this.fields.values().next().value?.focus();
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
      const reply = await send({
        type: 'update_metadata',
        context: this.captured,
        values: Object.fromEntries([...this.fields].map(([key, field]) => [key, field.value])),
        input: inputId(),
      });
      this.close();
      await this.apply(reply);
      this.status(reply.message ?? 'Project information updated');
    } catch (error) {
      this.error.textContent = errorText(error);
    } finally {
      this.pending = false;
      for (const field of this.fields.values()) field.disabled = false;
      this.applyButton.disabled = this.cancel.disabled = false;
    }
  }
}
