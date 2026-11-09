/** Shared modal browsing/placement transaction; cancellation never changes document data. */
import { element } from './ui';
import { errorText } from './transport';
import { PictureFields, type PicturePlacement } from './picture-controls';
export interface PictureTarget {
  value: PicturePlacement;
  cell?: boolean;
  existing?: boolean;
  save: (value: PicturePlacement | null) => Promise<void>;
}
export class PictureDialog {
  private dialog = element('dialog', 'settings-dialog');
  private pending = false;
  get active() {
    return this.dialog.open;
  }
  constructor(app: HTMLElement) {
    app.append(this.dialog);
    this.dialog.dataset.focus = 'modal';
    this.dialog.addEventListener('cancel', (e) => {
      if (this.pending) e.preventDefault();
    });
  }
  open(target: PictureTarget) {
    if (this.active) return;
    const heading = element('h2', '', target.existing ? 'Picture properties' : 'Insert picture');
    heading.id = 'picture-dialog-title';
    this.dialog.setAttribute('aria-labelledby', heading.id);
    const form = element('form', 'settings-form'),
      error = element('p', 'settings-error');
    error.setAttribute('role', 'alert');
    const fields = new PictureFields(
      structuredClone(target.value),
      !target.cell,
      () => {},
      () => {},
    );
    const apply = element('button', 'plain', 'Apply picture'),
      cancel = element('button', 'plain', 'Cancel'),
      remove = element('button', 'plain', 'Remove picture');
    for (const b of [apply, cancel, remove]) b.type = 'button';
    const finish = async (value: PicturePlacement | null) => {
      if (this.pending || fields.pending) return;
      if (value && !value.picture.png) {
        error.textContent = 'Choose a picture first.';
        return;
      }
      this.pending = true;
      for (const n of form.querySelectorAll<HTMLInputElement>('input,select,button'))
        n.disabled = true;
      try {
        await target.save(value);
        this.dialog.close();
      } catch (e) {
        error.textContent = errorText(e);
      } finally {
        this.pending = false;
        for (const n of form.querySelectorAll<HTMLInputElement>('input,select,button'))
          n.disabled = false;
      }
    };
    apply.addEventListener('click', () => void finish(fields.value));
    remove.addEventListener('click', () => void finish(null));
    cancel.addEventListener('click', () => {
      if (!this.pending) this.dialog.close();
    });
    const actions = element('div', 'settings-actions');
    if (target.existing) actions.append(remove);
    actions.append(cancel, apply);
    form.addEventListener('submit', (e) => {
      e.preventDefault();
      void finish(fields.value);
    });
    form.append(heading, fields.root, error, actions);
    this.dialog.replaceChildren(form);
    this.dialog.showModal();
  }
}
