/** Separate library workspace; project panes/camera stay mounted behind the bounded editor surface. */
import type {
  Context,
  Reply,
  TemplateView,
  TemplateVariant,
  TemplateTable,
  TemplateCell,
} from './types';
import type { Shell } from './shell';
import { element } from './ui';
import { send, inputId, errorText } from './transport';
import { TemplatePreview } from './template-preview';
import { TemplateProperties } from './template-properties';
import { TemplateTableTools } from './template-table-tools';
import { TemplateTableInteraction } from './template-table-interaction';
export class TemplateEditor {
  private root = element('section', 'template-workspace');
  private catalog = element('select', 'property-input');
  private structure = element('nav', 'template-structure');
  private header = element('strong', '', 'Template editor');
  private note = element('div', 'settings-error');
  private canvas = element('div', 'template-canvas');
  private form: TemplateProperties;
  private preview: TemplatePreview;
  private interaction: TemplateTableInteraction;
  private selectedCells: TemplateCell[] = [];
  get gestureActive() {
    return this.interaction?.active ?? false;
  }
  cancelGesture() {
    return this.interaction.cancel();
  }
  private tableTools: TemplateTableTools;
  get tableActive() {
    return this.tableTools?.active ?? false;
  }
  private model: TemplateView | null = null;
  private variant: TemplateVariant | null = null;
  private selection: { table: TemplateTable; cell: TemplateCell } | null = null;
  private fieldsPending = false;
  private operating = false;
  private uiBusy = false;
  private captured: Context | null = null;
  get active() {
    return this.model !== null;
  }
  get working() {
    return this.operating || this.tableActive || this.gestureActive;
  }
  get pending() {
    return this.fieldsPending;
  }
  constructor(
    private shell: Shell,
    private context: () => Context,
    private apply: (r: Reply) => Promise<void>,
    private status: (s: string, error?: boolean) => void,
    private action: (id: string) => void,
    private controls: () => void,
  ) {
    this.tableTools = new TemplateTableTools(shell.app, apply, controls);
    this.root.hidden = true;
    this.root.setAttribute('aria-label', 'Drawing template editor');
    const left = element('aside', 'pane project-pane');
    left.dataset.focus = 'tree';
    left.setAttribute('aria-label', 'Template library');
    const lh = element('div', 'pane-header');
    lh.append(element('strong', '', 'Drawing templates'));
    left.append(lh);
    this.catalog.setAttribute('aria-label', 'Saved template variants');
    left.append(this.catalog);
    const open = element('button', 'plain', 'Open selected template');
    open.dataset.action = 'template.open';
    open.addEventListener(
      'click',
      () => void this.work(() => this.operation('template.open', this.catalog.value)),
    );
    left.append(open, this.structure);
    const center = element('section', 'template-center');
    const ch = element('div', 'pane-header');
    ch.append(this.header, element('span', 'pane-tag', 'PLACEHOLDERS · mm'));
    center.append(ch, this.canvas);
    const right = element('aside', 'pane inspector-pane');
    right.dataset.focus = 'inspector';
    right.setAttribute('aria-label', 'Template properties');
    const rh = element('div', 'pane-header');
    rh.append(element('strong', '', 'Template properties'));
    right.append(rh);
    this.form = new TemplateProperties(
      () => this.changed(),
      () => void this.work(() => this.commitProperties()),
    );
    const buttons = element('div', 'template-property-actions');
    for (const [name, id] of [
      ['Apply properties', 'template.properties.apply'],
      ['Revert fields', 'template.properties.revert'],
    ]) {
      const b = element('button', 'plain', name);
      b.addEventListener(
        'click',
        () =>
          void this.work(() =>
            id.endsWith('apply') ? this.commitProperties() : this.operation(id),
          ),
      );
      buttons.append(b);
    }
    this.note.setAttribute('role', 'alert');
    right.append(this.form.root, this.note, buttons);
    this.preview = new TemplatePreview(() => this.action('selection.properties'));
    this.interaction = new TemplateTableInteraction(
      this.preview,
      context,
      apply,
      (table, cells) => this.selectGroup(table, cells),
      () => !this.uiBusy && !this.operating && !this.fieldsPending && !this.tableActive,
      controls,
      status,
    );
    this.canvas.append(this.preview.svg);
    this.root.append(left, center, right);
    shell.workspace.append(this.root);
  }
  private changed() {
    if (this.fieldsPending) return;
    this.fieldsPending = true;
    this.captured = this.context();
    void send({ type: 'template_pending', context: this.captured, input: inputId() }).catch((e) =>
      this.status(errorText(e), true),
    );
    this.header.textContent = `${this.variant?.name ?? 'Template'} •`;
    document.title = `● ${this.variant?.name ?? 'Template'} — Template — ECAD`;
    this.controls();
  }
  private selectGroup(table: TemplateTable, cells: TemplateCell[]) {
    if (this.uiBusy || this.operating || this.fieldsPending || !cells.length) return false;
    this.selection = { table, cell: cells[0] };
    this.selectedCells = cells;
    this.form.render(this.variant!, this.selection);
    if (cells.length > 1)
      this.note.textContent = `${cells.length} cells selected. Properties show the first cell; Merge reviews the complete selection.`;
    else this.note.textContent = '';
    return true;
  }
  private select(table: TemplateTable, cell: TemplateCell) {
    if (!this.selectGroup(table, [cell])) return;
    this.interaction.selection(table, [cell]);
    this.form.focus();
  }
  update(view: TemplateView | null) {
    const previous = this.model;
    this.model = view;
    this.root.hidden = !view;
    if (!view) {
      this.fieldsPending = false;
      this.captured = null;
      this.selection = null;
      this.selectedCells = [];
      delete this.shell.workspace.dataset.editor;
      this.shell.ribbon.setTemplateContext(false);
      return;
    }
    this.shell.workspace.dataset.editor = 'template';
    this.shell.ribbon.setTemplateContext(true);
    const changed =
      previous?.revision.epoch !== view.revision.epoch ||
      previous?.revision.sequence !== view.revision.sequence;
    const reset = previous?.revision.epoch !== view.revision.epoch;
    this.variant = view.variants.find((v) => v.id === view.selected) ?? view.variants[0];
    this.header.textContent = `${this.variant.name}${view.dirty || view.pending || this.fieldsPending ? ' •' : ''}`;
    this.header.title = view.published ?? 'Unsaved template definition';
    const selected = this.catalog.value;
    this.catalog.replaceChildren(
      ...view.catalog.map(([key, label]) => {
        const o = element('option', '', label);
        o.value = key;
        return o;
      }),
    );
    if (view.catalog.some(([key]) => key === selected)) this.catalog.value = selected;
    if (reset) {
      this.selection = null;
      this.selectedCells = [];
    }
    if (changed || !this.fieldsPending) {
      this.fieldsPending = false;
      this.captured = null;
      if (this.selection) {
        const table = this.variant.tables.find((t) => t.id === this.selection?.table.id);
        const cell = table?.cells.find((c) => c.id === this.selection?.cell.id);
        this.selection = table && cell ? { table, cell } : null;
        const ids = new Set(this.selectedCells.map((c) => c.id));
        this.selectedCells = table?.cells.filter((c) => ids.has(c.id)) ?? [];
      }
      this.form.render(this.variant, this.selection);
    }
    this.structure.replaceChildren();
    const props = element('button', 'tree-item', 'Template properties');
    props.addEventListener('click', () => {
      if (!this.fieldsPending) {
        this.selection = null;
        this.selectedCells = [];
        this.interaction.update(this.variant!, null);
        this.form.render(this.variant!, null);
        this.form.focus();
      }
    });
    this.structure.append(props);
    for (const table of this.variant.tables) {
      this.structure.append(
        element('p', 'inspector-note', `Table · ${table.bounds.join(', ')} µm`),
      );
      for (const cell of table.cells) {
        const b = element('button', 'tree-item', cell.caption || 'Blank cell');
        b.dataset.templateCell = cell.id;
        b.addEventListener('click', () => this.select(table, cell));
        b.addEventListener('contextmenu', (e) => {
          e.preventDefault();
          this.select(table, cell);
          this.action('selection.properties');
        });
        this.structure.append(b);
      }
    }
    this.note.textContent = view.catalog_error ?? '';
    this.interaction.update(
      this.variant,
      this.selection ? { table: this.selection.table, cells: this.selectedCells } : null,
    );
    this.preview.render(this.variant, reset);
    document.title = `${view.dirty || view.pending || this.fieldsPending ? '● ' : ''}${this.variant.name} — Template — ECAD`;
  }
  async commitProperties() {
    if (!this.active) return;
    const reply = await send({
      type: 'template_operation',
      context: this.captured ?? this.context(),
      action: 'template.properties.apply',
      source: null,
      values: this.form.proposal(),
      input: inputId(),
    });
    this.fieldsPending = false;
    this.captured = null;
    await this.apply(reply);
  }
  async operation(id: string, source?: string) {
    const reply = await send({
      type: 'template_operation',
      context: this.context(),
      action: id,
      source: source ?? null,
      values: null,
      input: inputId(),
    });
    if (id === 'template.properties.revert') {
      this.fieldsPending = false;
      this.captured = null;
    }
    await this.apply(reply);
  }
  setBusy(busy: boolean) {
    this.uiBusy = busy;
    for (const input of this.root.querySelectorAll<
      HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement | HTMLButtonElement
    >('input,textarea,select,button'))
      input.disabled = busy || this.operating || this.gestureActive;
  }
  focus() {
    this.preview.svg.focus();
  }
  properties() {
    setTimeout(() => {
      if (this.active) this.form.focus();
    }, 0);
  }
  tables(operation: string) {
    if (this.variant)
      this.tableTools.open(
        this.context(),
        this.variant,
        this.selection,
        operation,
        this.selectedCells.map((c) => c.id),
      );
  }
  fit() {
    this.preview.fit();
    this.focus();
  }
  private async work(operation: () => Promise<void>) {
    if (this.operating) return;
    this.operating = true;
    this.controls();
    try {
      await operation();
    } catch (e) {
      this.note.textContent = errorText(e);
      this.status(errorText(e), true);
    } finally {
      this.operating = false;
      this.controls();
    }
  }
}
