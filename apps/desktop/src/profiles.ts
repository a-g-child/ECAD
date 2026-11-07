/** Profile saves are personal settings; reviewed copies alone enter project history. */
import type { Context, ProfileCatalog, ProfileImport, Reply } from './types';
import type { Shell } from './shell';
import { element } from './ui';
import { textEditor } from './text';
import { send, inputId, errorText } from './transport';
export class ProfilesDialog {
  private dialog = element('dialog', 'settings-dialog metadata-dialog profiles-dialog');
  private form = element('form', 'settings-form');
  private select = element('select', 'property-input');
  private name = textEditor('property-input property-text', 2);
  private body = element('div', 'metadata-fields');
  private fields = new Map<string, HTMLTextAreaElement>();
  private choices = new Map<string, HTMLInputElement>();
  private review = element('div', 'profile-review');
  private message = element('div', 'settings-error');
  private reload = element('button', 'plain', 'Reload profiles');
  private save = element('button', 'plain', 'Save profile');
  private preview = element('button', 'plain', 'Preview import');
  private applyButton = element('button', 'plain', 'Apply import');
  private closeButton = element('button', 'plain', 'Close');
  private catalog: ProfileCatalog | null = null;
  private proposal: ProfileImport | null = null;
  private context: Context | null = null;
  private pending = false;
  private dirty = false;
  get active() {
    return this.dialog.open;
  }
  constructor(
    private shell: Shell,
    private apply: (reply: Reply) => Promise<void>,
    private status: (text: string) => void,
  ) {
    const title = element('h2', '', 'Company / contact profiles');
    title.id = 'company-profiles-heading';
    this.dialog.setAttribute('aria-labelledby', title.id);
    this.select.setAttribute('aria-label', 'Saved profile');
    this.name.setAttribute('aria-label', 'Profile name');
    this.name.placeholder = 'Name this reusable profile';
    this.name.title = 'Enter adds a line; Save profile stores these personal defaults.';
    this.message.setAttribute('role', 'status');
    const actions = element('div', 'settings-actions');
    for (const node of [this.reload, this.save, this.preview, this.applyButton, this.closeButton]) {
      node.type = 'button';
      actions.append(node);
    }
    this.body.append(this.name);
    this.form.append(
      title,
      element(
        'p',
        'settings-note',
        'Saving a profile changes personal defaults only. Select fields and preview their old/new project values before applying. Empty profile values clear project fields only when selected.',
      ),
      this.select,
      this.body,
      this.message,
      actions,
    );
    this.dialog.append(this.form);
    shell.app.append(this.dialog);
    this.form.addEventListener('submit', (e) => e.preventDefault());
    this.select.addEventListener('change', () => this.populate());
    this.name.addEventListener('input', () => this.edited());
    this.reload.addEventListener(
      'click',
      () =>
        void this.work(async () => this.accept((await send({ type: 'load_profiles' })).profiles!)),
    );
    this.save.addEventListener(
      'click',
      () =>
        void this.work(async () => {
          const reply = await send({
            type: 'save_profile',
            token: this.catalog!.token,
            id: this.select.value || null,
            name: this.name.value,
            values: Object.fromEntries([...this.fields].map(([key, field]) => [key, field.value])),
            input: inputId(),
          });
          const id = this.select.value;
          this.accept(reply.profiles!, id || undefined, this.name.value);
          this.message.textContent = reply.message;
        }),
    );
    this.preview.addEventListener(
      'click',
      () =>
        void this.work(async () => {
          const reply = await send({
            type: 'preview_profile_import',
            context: this.context,
            token: this.catalog!.token,
            id: this.select.value,
            fields: [...this.choices].filter(([, field]) => field.checked).map(([key]) => key),
          });
          this.proposal = reply.profile_import;
          this.renderReview();
        }),
    );
    this.applyButton.addEventListener(
      'click',
      () =>
        void this.work(async () => {
          const reply = await send({
            type: 'commit_profile_import',
            token: this.proposal!.token,
            input: inputId(),
          });
          this.dialog.close();
          await this.apply(reply);
          this.status(reply.message ?? 'Profile values imported');
          this.shell.canvas.focus();
        }),
    );
    const close = () =>
      void this.work(async () => {
        await send({ type: 'cancel_profile_import' });
        this.dialog.close();
        this.shell.canvas.focus();
      });
    this.closeButton.addEventListener('click', close);
    this.dialog.addEventListener('cancel', (e) => {
      e.preventDefault();
      if (!this.pending) close();
    });
  }
  open(context: Context, catalog: ProfileCatalog) {
    this.context = context;
    this.accept(catalog);
    this.dialog.showModal();
    this.select.focus();
  }
  private accept(catalog: ProfileCatalog, id?: string, name?: string) {
    this.catalog = catalog;
    this.select.replaceChildren();
    for (const row of [{ id: '', name: 'New profile…' }, ...catalog.entries]) {
      const option = element('option', '', row.name);
      option.value = row.id;
      this.select.append(option);
    }
    this.select.value = id ?? catalog.entries.find((p) => p.name === name)?.id ?? '';
    this.populate();
  }
  private populate() {
    const selected = this.catalog!.entries.find((p) => p.id === this.select.value);
    this.name.value = selected?.name ?? '';
    this.body.replaceChildren(element('label', '', 'Profile name'), this.name);
    this.fields.clear();
    this.choices.clear();
    for (const row of this.catalog!.fields) {
      const line = element('label', 'settings-field');
      const caption = element('span', '', row.label);
      const check = element('input');
      check.type = 'checkbox';
      check.setAttribute('aria-label', `Import ${row.label}`);
      check.checked = !!selected?.values[row.key];
      check.addEventListener('change', () => {
        this.invalidate();
        this.controls();
      });
      caption.prepend(check);
      const field = textEditor('property-input property-text', 2);
      field.setAttribute('aria-label', row.label);
      field.title = 'Enter adds a line; Save profile stores these personal defaults.';
      field.value = selected?.values[row.key] ?? '';
      field.addEventListener('input', () => this.edited());
      line.append(caption, field);
      this.body.append(line);
      this.fields.set(row.key, field);
      this.choices.set(row.key, check);
    }
    this.body.append(this.review);
    this.dirty = false;
    this.message.textContent = '';
    this.invalidate();
    this.controls();
  }
  private invalidate() {
    this.proposal = null;
    this.review.replaceChildren();
  }
  private edited() {
    this.dirty = true;
    this.message.classList.remove('error');
    this.invalidate();
    this.message.textContent =
      'Save before previewing an import, or Reload profiles to discard edits.';
    this.controls();
  }
  private controls() {
    for (const field of [
      this.select,
      this.name,
      ...this.fields.values(),
      ...this.choices.values(),
      this.reload,
      this.save,
      this.closeButton,
    ])
      field.disabled = this.pending;
    this.select.disabled = this.pending || this.dirty;
    this.preview.disabled = this.pending || this.dirty || !this.select.value;
    this.applyButton.disabled = this.pending || !this.proposal;
  }
  private renderReview() {
    this.review.replaceChildren(element('strong', '', 'Project changes'));
    if (!this.proposal?.changes.length)
      this.review.append(
        element(
          'p',
          'settings-note',
          'Selected values already match. Apply will leave history unchanged.',
        ),
      );
    for (const change of this.proposal?.changes ?? []) {
      const row = element('div', 'profile-change');
      row.append(
        element('strong', '', change.label),
        element('div', '', `Current: ${change.before ?? '(empty)'}`),
        element('div', '', `Proposed: ${change.after ?? '(clear value)'}`),
      );
      this.review.append(row);
    }
    this.review.scrollIntoView({ block: 'nearest' });
  }
  private async work(operation: () => Promise<void>) {
    if (this.pending) return;
    this.pending = true;
    this.message.classList.remove('error');
    this.message.textContent = '';
    this.controls();
    try {
      await operation();
    } catch (error) {
      this.invalidate();
      this.message.classList.add('error');
      this.message.textContent = errorText(error);
    } finally {
      this.pending = false;
      this.controls();
    }
  }
}
