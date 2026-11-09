/** Sheet drafts are local strings; Rust validates and retains the exact reviewed paper edit. */
import { dimension, readDimension, writeDimension } from './dimensions';
import type { Context, Reply, SheetRow, SheetPreview, Snapshot } from './types';
import type { Shell } from './shell';
import { element } from './ui';
import { textEditor } from './text';
import { send, inputId, errorText } from './transport';
import { renderPaper, paperBox } from './paper';
export class SheetsDialog {
  private dialog = element('dialog', 'settings-dialog sheet-dialog');
  private form = element('form', 'settings-form');
  private select = element('select', 'property-input');
  private body = element('div', 'sheet-fields');
  private previewSvg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  private message = element('div', 'settings-error');
  private fields = new Map<keyof SheetRow, HTMLInputElement | HTMLTextAreaElement>();
  private view = element('select', 'property-input');
  private inheritedVariant: string | null = null;
  private controlsMap = new Map<string, HTMLButtonElement>();
  private context: Context | null = null;
  private snapshot: Snapshot | null = null;
  private row: SheetRow | null = null;
  private owner = element('select', 'property-input');
  private template = element('select', 'property-input');
  private kindLabel = element('p', 'settings-note');
  private create = false;
  private dirty = false;
  private pending = false;
  private proposal: SheetPreview | null = null;
  get active() {
    return this.dialog.open;
  }
  constructor(
    private shell: Shell,
    private apply: (reply: Reply) => Promise<void>,
    private status: (text: string) => void,
  ) {
    const title = element('h2', '', 'Sheets and paper');
    title.id = 'sheet-manager-heading';
    this.dialog.setAttribute('aria-labelledby', title.id);
    this.select.setAttribute('aria-label', 'Edit sheet');
    this.view.setAttribute('aria-label', 'Placed drawing view');
    for (const [key, label] of [
      ['number', 'Drawing number'],
      ['title', 'Sheet title'],
      ['width', 'Paper width (µm)'],
      ['height', 'Paper height (µm)'],
      ['left', 'Left margin (µm)'],
      ['right', 'Right margin (µm)'],
      ['top', 'Top margin (µm)'],
      ['bottom', 'Bottom margin (µm)'],
    ] as [keyof SheetRow, string][])
      this.field(key, label);
    this.body.append(this.label('Drawing view', this.view));
    for (const [key, label] of [
      ['numerator', 'Scale numerator'],
      ['denominator', 'Scale denominator'],
      ['paper_x', 'Paper origin X (µm)'],
      ['paper_y', 'Paper origin Y (µm)'],
      ['source_x', 'View origin X (µm)'],
      ['source_y', 'View origin Y (µm)'],
    ] as [keyof SheetRow, string][])
      this.field(key, label);
    const layout = element('div', 'sheet-editor');
    const preview = element('div', 'sheet-draft-preview');
    this.previewSvg.setAttribute('aria-label', 'Draft paper preview');
    preview.append(this.previewSvg);
    layout.append(this.body, preview);
    const actions = element('div', 'settings-actions');
    for (const name of [
      'New sheet',
      'Move up',
      'Move down',
      'Remove sheet',
      'Preview sheet',
      'Apply sheet',
      'Close',
    ]) {
      const b = element('button', 'plain', name);
      b.type = 'button';
      this.controlsMap.set(name, b);
      actions.append(b);
    }
    this.owner.setAttribute('aria-label', 'Drawing document');
    this.template.setAttribute('aria-label', 'Sheet template');
    for (const input of [this.owner, this.template])
      input.addEventListener('change', () => {
        this.inheritedVariant = null;
        this.templatePaper();
        this.edited();
      });
    this.body.prepend(
      this.kindLabel,
      this.label('Drawing document', this.owner),
      this.label('Sheet template', this.template),
    );
    this.form.append(
      title,
      element(
        'p',
        'settings-note',
        'Select a saved template. Paper and margins come from that layout. Scale is paper/model (1/2 is half size); placement changes do not resize source geometry.',
      ),
      this.select,
      layout,
      this.message,
      actions,
    );
    this.dialog.append(this.form);
    shell.app.append(this.dialog);
    this.message.setAttribute('role', 'alert');
    this.form.addEventListener('submit', (e) => e.preventDefault());
    this.select.addEventListener('change', () => {
      const row = this.snapshot!.sheets.find((s) => s.id === this.select.value);
      if (row) this.populate(row, false);
    });
    this.view.addEventListener('change', () => this.edited());
    this.button('New sheet', () => this.newSheet());
    this.button('Preview sheet', () => this.preview());
    this.button('Apply sheet', () => this.commit());
    this.button('Move up', () => this.change('move', -1));
    this.button('Move down', () => this.change('move', 1));
    this.button('Remove sheet', () => this.change('remove', 0));
    this.button('Close', () => this.close());
    this.dialog.addEventListener('cancel', (e) => {
      e.preventDefault();
      if (!this.pending) void this.work(() => this.close());
    });
  }
  private options(select: HTMLSelectElement, rows: [string, string][]) {
    select.replaceChildren(
      ...rows.map(([value, label]) => {
        const option = element('option', '', label);
        option.value = value;
        return option;
      }),
    );
  }
  private label(caption: string, field: HTMLElement) {
    const label = element('label', 'settings-field');
    label.append(element('span', '', caption), field);
    return label;
  }
  private field(key: keyof SheetRow, caption: string) {
    const text = key === 'title' || key === 'number';
    const input = text
      ? textEditor('property-input property-text', 2)
      : element('input', 'property-input');
    input.setAttribute('aria-label', caption);
    if (!text) input.inputMode = 'numeric';
    if (['width', 'height', 'left', 'right', 'top', 'bottom'].includes(key)) input.readOnly = true;
    input.addEventListener('input', () => {
      this.edited();
    });
    this.fields.set(key, input);
    const label = this.label(caption, input);
    this.body.append(label);
    if (caption.includes('(µm)')) dimension(input, caption, label.firstElementChild as HTMLElement);
  }
  private button(name: string, operation: () => Promise<void>) {
    this.controlsMap.get(name)!.addEventListener('click', () => void this.work(operation));
  }
  async open(context: Context, snapshot: Snapshot, id: string, kind?: string) {
    this.context = context;
    this.snapshot = snapshot;
    this.options(
      this.select,
      snapshot.sheets.map((s) => [s.id, `${s.number} · ${s.title}`]),
    );
    this.options(this.view, [
      ['', 'No placed view'],
      ...snapshot.views.map((v) => [v.id, v.name] as [string, string]),
    ]);
    this.options(
      this.owner,
      snapshot.drawing_documents.map((d) => [d.id, `${d.number} · ${d.title}`]),
    );
    this.dialog.showModal();
    if (kind) {
      await this.work(() => this.newSheet(kind));
      return;
    }
    const row = snapshot.sheets.find((s) => s.id === id) ?? snapshot.sheets[0];
    if (row) this.populate(row, false);
    else await this.work(() => this.newSheet());
  }
  private async newSheet(kind?: string) {
    const reply = await send({
      type: 'new_sheet',
      context: this.context,
      ...(kind ? { kind } : {}),
    });
    if (reply.sheet_draft) this.populate(reply.sheet_draft, true);
  }
  private populate(row: SheetRow, create: boolean) {
    this.row = row;
    this.inheritedVariant = row.inherited_variant;
    this.view.querySelector('option[data-new-view]')?.remove();
    if (row.view && !this.snapshot?.views.some((v) => v.id === row.view)) {
      const option = element('option', '', 'New independent drawing view');
      option.value = row.view;
      option.dataset.newView = 'true';
      this.view.append(option);
    }
    this.owner.value = row.document ?? this.snapshot?.drawing_documents[0]?.id ?? '';
    const choices = this.snapshot!.template_choices.filter(
      (v) => v.new_assignments || v.key === row.template,
    );
    this.options(this.template, [
      ['', 'Inherit document template'],
      ...choices.map((v) => [v.key, v.label] as [string, string]),
    ]);
    for (const v of choices)
      if (!v.new_assignments)
        this.template.querySelector<HTMLOptionElement>(`option[value="${v.key}"]`)!.disabled = true;
    this.template.value = row.template ?? '';
    this.kindLabel.textContent =
      row.kind === 'cover'
        ? 'Cover sheet'
        : row.kind === 'drawing-list'
          ? 'Drawing list · generated number/title navigation'
          : 'Schematic sheet';
    this.create = create;
    this.select.querySelector('option[data-draft]')?.remove();
    if (create) {
      const option = element('option', '', 'New sheet (unsaved)');
      option.value = '';
      option.dataset.draft = 'true';
      this.select.append(option);
    }
    this.dirty = false;
    for (const [key, input] of this.fields) writeDimension(input, String(row[key] ?? ''));
    this.view.value = row.view ?? '';
    this.select.value = create ? '' : row.id;
    this.invalidate();
    this.message.textContent = '';
    this.controls();
  }
  private templatePaper() {
    const key =
      this.template.value ||
      this.snapshot?.drawing_documents.find((d) => d.id === this.owner.value)?.default_template;
    const variant = this.snapshot?.template_choices.find((v) => v.key === key);
    if (variant)
      for (const key of ['width', 'height', 'left', 'right', 'top', 'bottom'] as const)
        writeDimension(this.fields.get(key)!, variant[key]);
  }
  private draft(): SheetRow {
    return {
      ...this.row!,
      ...Object.fromEntries([...this.fields].map(([key, input]) => [key, readDimension(input)])),
      view: this.view.value || null,
      document: this.owner.value,
      template: this.template.value || null,
      inherited_variant: this.template.value ? null : this.inheritedVariant,
    };
  }
  private invalidate() {
    this.proposal = null;
    this.previewSvg.replaceChildren();
  }
  private edited() {
    this.dirty = true;
    this.message.classList.remove('error');
    this.invalidate();
    this.message.textContent = 'Preview the updated sheet before applying.';
    this.controls();
  }
  private controls() {
    for (const input of [...this.fields.values(), this.view]) input.disabled = this.pending;
    this.owner.disabled = this.pending || !this.create;
    this.template.disabled = this.pending;
    this.view.disabled = this.pending || this.row?.kind !== 'schematic';
    this.select.disabled =
      this.pending || this.dirty || this.create || !this.snapshot?.sheets.length;
    for (const b of this.controlsMap.values()) b.disabled = this.pending;
    this.controlsMap.get('Apply sheet')!.disabled = this.pending || !this.proposal;
    this.controlsMap.get('New sheet')!.disabled = this.pending || this.dirty || this.create;
    const index = this.snapshot?.sheets.findIndex((s) => s.id === this.row?.id) ?? -1;
    for (const name of ['Move up', 'Move down', 'Remove sheet'])
      this.controlsMap.get(name)!.disabled =
        this.pending ||
        this.dirty ||
        this.create ||
        index < 0 ||
        (name === 'Move up' && index === 0) ||
        (name === 'Move down' && index === (this.snapshot?.sheets.length ?? 0) - 1);
  }
  private async preview() {
    const reply = await send({
      type: 'preview_sheet',
      context: this.context,
      sheet: this.draft(),
      create: this.create,
    });
    if (!reply.sheet_preview) throw new Error('Missing sheet preview');
    this.proposal = reply.sheet_preview;
    renderPaper(
      this.previewSvg,
      this.proposal.sheet,
      this.proposal.scene,
      this.proposal.frame,
      this.proposal.list,
    );
    this.previewSvg.setAttribute('viewBox', paperBox(this.proposal.sheet, 1.25).join(' '));
    this.message.textContent = this.proposal.frame.title_block_too_small
      ? 'Title block cannot fit within the paper margins. The border is retained.'
      : 'Reviewed paper layout · Apply sheet commits one edit.';
  }
  private async commit() {
    if (!this.proposal) return;
    const reply = await send({
      type: 'commit_sheet',
      token: this.proposal.token,
      input: inputId(),
    });
    this.dialog.close();
    await this.apply(reply);
    this.status(reply.message ?? 'Sheet updated');
    this.shell.canvas.focus();
  }
  private async change(operation: string, delta: number) {
    const index = this.snapshot!.sheets.findIndex((s) => s.id === this.row!.id) + delta;
    const reply = await send({
      type: 'change_sheet',
      context: this.context,
      id: this.row!.id,
      operation,
      index,
      input: inputId(),
    });
    this.dialog.close();
    await this.apply(reply);
    this.status(reply.message ?? 'Sheet updated');
    this.shell.canvas.focus();
  }
  private async close() {
    await send({ type: 'cancel_sheet' });
    this.dialog.close();
    this.shell.canvas.focus();
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

/** Typed creation requests share the same reviewed paper editor. */
export function sheetKindForRequest(request: string | null) {
  return (
    { new_schematic: 'schematic', new_cover: 'cover', new_drawing_list: 'drawing-list' } as Record<
      string,
      string
    >
  )[request ?? ''];
}
