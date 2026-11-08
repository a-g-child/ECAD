/** Captured structural table edits; the Rust core owns geometry, identity allocation and history. */
import type { Context, Reply, TemplateVariant, TemplateTable, TemplateCell } from './types';
import { element } from './ui';
import { send, inputId, errorText } from './transport';
const actions: Record<string, [string, string]> = {
  resize: ['Resize title-block table', 'template.table.resize'],
  split: ['Split title-block cell', 'template.cell.split'],
  merge: ['Merge title-block cells', 'template.cells.merge'],
  boundary: ['Move table boundary', 'template.table.boundary'],
};
export class TemplateTableTools {
  private dialog = element('dialog', 'settings-dialog');
  private heading = element('h2');
  private tableSelect = element('select', 'property-input');
  private body = element('div', 'metadata-fields');
  private note = element('div', 'settings-error');
  private applyButton = element('button', 'plain', 'Apply table edit');
  private cancelButton = element('button', 'plain', 'Cancel');
  private context: Context | null = null;
  private variant: TemplateVariant | null = null;
  private operation = '';
  private pending = false;
  private dirty = false;
  private controls = new Map<string, HTMLInputElement | HTMLSelectElement>();
  private choices: HTMLInputElement[] = [];
  get active() {
    return this.dialog.open;
  }
  constructor(
    app: HTMLElement,
    private apply: (r: Reply) => Promise<void>,
    private changed: () => void,
  ) {
    this.heading.id = 'template-table-tools-heading';
    this.dialog.setAttribute('aria-labelledby', this.heading.id);
    this.dialog.dataset.focus = 'modal';
    this.tableSelect.setAttribute('aria-label', 'Title-block table');
    this.note.setAttribute('role', 'alert');
    const form = element('form', 'settings-form');
    const buttons = element('div', 'settings-actions');
    this.applyButton.type = this.cancelButton.type = 'button';
    buttons.append(this.applyButton, this.cancelButton);
    form.append(this.heading, this.tableSelect, this.body, this.note, buttons);
    form.addEventListener('submit', (e) => e.preventDefault());
    this.tableSelect.addEventListener('change', () => this.populate());
    this.body.addEventListener('input', () => this.edited());
    this.body.addEventListener('change', () => this.edited());
    this.applyButton.addEventListener('click', () => void this.commit());
    this.cancelButton.addEventListener('click', () => void this.cancel());
    this.dialog.addEventListener('cancel', (e) => {
      e.preventDefault();
      void this.cancel();
    });
    this.dialog.append(form);
    app.append(this.dialog);
  }
  open(
    context: Context,
    variant: TemplateVariant,
    selection: { table: TemplateTable; cell: TemplateCell } | null,
    operation: string,
  ) {
    if (this.active || !actions[operation]) return;
    this.context = structuredClone(context);
    this.variant = variant;
    this.operation = operation;
    this.dirty = false;
    this.pending = false;
    this.heading.textContent = actions[operation][0];
    this.note.textContent = '';
    this.tableSelect.replaceChildren(
      ...variant.tables.map((t, i) => {
        const o = element('option', '', `Table ${i + 1} · ${t.cells.length} cells`);
        o.value = t.id;
        return o;
      }),
    );
    if (selection) this.tableSelect.value = selection.table.id;
    this.populate(selection?.cell.id);
    this.dialog.showModal();
    this.changed();
  }
  private table() {
    return this.variant?.tables.find((t) => t.id === this.tableSelect.value);
  }
  private field(name: string, input: HTMLInputElement | HTMLSelectElement) {
    input.setAttribute('aria-label', name);
    this.controls.set(name, input);
    const label = element('label', 'settings-field');
    label.append(element('span', '', name), input);
    this.body.append(label);
    return input;
  }
  private number(name: string, value: string) {
    const input = element('input', 'property-input');
    input.inputMode = 'numeric';
    input.value = value;
    return this.field(name, input);
  }
  private select(name: string, options: [string, string][], selected?: string) {
    const select = element('select', 'property-input');
    select.append(
      ...options.map(([value, label]) => {
        const o = element('option', '', label);
        o.value = value;
        return o;
      }),
    );
    if (selected) select.value = selected;
    return this.field(name, select);
  }
  private populate(selected?: string) {
    this.body.replaceChildren();
    this.controls.clear();
    this.choices = [];
    const table = this.table();
    this.applyButton.disabled = !table;
    if (!table) {
      this.body.append(element('p', '', 'This variant has no title-block table.'));
      return;
    }
    if (this.operation === 'resize') {
      ['Table X (µm)', 'Table Y (µm)', 'Table width (µm)', 'Table height (µm)'].forEach((name, i) =>
        this.number(name, table.bounds[i]),
      );
      this.body.append(
        element(
          'p',
          'settings-note',
          'Shared edges scale together. Cell identities, captions and variables are retained. The table must fit inside the paper margins.',
        ),
      );
    } else if (this.operation === 'split') {
      this.select(
        'Cell to split',
        table.cells.map((c) => [c.id, c.caption || 'Blank cell']),
        selected,
      );
      this.number('Columns', '2');
      this.number('Rows', '1');
      this.body.append(
        element(
          'p',
          'settings-note',
          'The upper-left cell keeps its identity and contents. Additional cells are blank. Split a different row separately for unequal subdivisions.',
        ),
      );
    } else if (this.operation === 'merge') {
      this.select(
        'Surviving cell',
        table.cells.map((c) => [c.id, c.caption || 'Blank cell']),
        selected,
      );
      this.select('Merged contents', [
        ['combine', 'Combine all captions and contents in reading order'],
        ['keep_primary', 'Keep only the surviving cell’s contents'],
      ]);
      this.body.append(
        element(
          'p',
          'settings-note',
          'Select cells forming one rectangle. The surviving identity remains; other identities are removed in this draft. Undo restores them exactly.',
        ),
      );
      const list = element('div', 'template-merge-cells');
      for (const cell of table.cells) {
        const input = element('input');
        input.type = 'checkbox';
        input.value = cell.id;
        input.checked = cell.id === selected;
        input.setAttribute('aria-label', `Merge ${cell.caption || 'blank cell'} ${cell.id}`);
        this.choices.push(input);
        const label = element('label');
        label.append(
          input,
          element(
            'span',
            '',
            `${cell.caption || 'Blank cell'}\n${cell.contents.map((r) => (r.kind === 'literal' ? r.value : `{${r.value}}`)).join('')}`,
          ),
        );
        list.append(label);
      }
      this.body.append(list);
    } else {
      this.select('Boundary axis', [
        ['x', 'Vertical boundary (X)'],
        ['y', 'Horizontal boundary (Y)'],
      ]);
      const first = table.cells.find(
        (c) => BigInt(c.bounds[0]) + BigInt(c.bounds[2]) < BigInt(table.bounds[2]),
      );
      const x = first ? String(BigInt(first.bounds[0]) + BigInt(first.bounds[2])) : '0';
      this.number('Boundary from (µm)', x);
      this.number('Boundary to (µm)', x);
      this.body.append(
        element(
          'p',
          'settings-note',
          'Coordinates are local to the table. All cell edges aligned at this coordinate move together; spanning cells stay intact. Boundaries cannot cross or collapse other coordinates.',
        ),
      );
    }
  }
  private edited() {
    if (this.dirty || !this.context) return;
    this.dirty = true;
    // Freeze the target table once fields are edited; cancellation explicitly releases the guard.
    this.tableSelect.disabled = true;
    void send({
      type: 'template_pending',
      context: this.context,
      table_action: actions[this.operation][1],
      input: inputId(),
    }).catch((e) => (this.note.textContent = errorText(e)));
  }
  private value(name: string) {
    return this.controls.get(name)!.value;
  }
  private proposal() {
    const value = (name: string) => this.value(name);
    let edit: object;
    switch (this.operation) {
      case 'resize':
        edit = {
          operation: 'resize',
          bounds: ['Table X (µm)', 'Table Y (µm)', 'Table width (µm)', 'Table height (µm)'].map(
            value,
          ),
        };
        break;
      case 'split':
        edit = {
          operation: 'split',
          cell: value('Cell to split'),
          columns: value('Columns'),
          rows: value('Rows'),
        };
        break;
      case 'merge':
        edit = {
          operation: 'merge',
          cells: this.choices.filter((c) => c.checked).map((c) => c.value),
          primary: value('Surviving cell'),
          contents: value('Merged contents'),
        };
        break;
      default:
        edit = {
          operation: 'boundary',
          axis: value('Boundary axis'),
          from: value('Boundary from (µm)'),
          to: value('Boundary to (µm)'),
        };
    }
    return { variant: this.variant!.id, table: this.tableSelect.value, edit };
  }
  private busy(value: boolean) {
    this.pending = value;
    for (const node of this.dialog.querySelectorAll<
      HTMLInputElement | HTMLSelectElement | HTMLButtonElement
    >('input,select,button'))
      node.disabled = value;
    this.tableSelect.disabled = value || this.dirty;
    this.applyButton.disabled = value || !this.table();
  }
  private async commit() {
    if (this.pending || !this.context || !this.table()) return;
    const values = this.proposal();
    this.busy(true);
    try {
      const reply = await send({
        type: 'template_operation',
        context: this.context,
        action: actions[this.operation][1],
        values,
        source: null,
        input: inputId(),
      });
      this.dialog.close();
      this.dirty = false;
      await this.apply(reply);
    } catch (e) {
      this.note.textContent = errorText(e);
    } finally {
      this.busy(false);
      this.changed();
    }
  }
  private async cancel() {
    if (this.pending) return;
    this.busy(true);
    try {
      if (this.dirty)
        await this.apply(
          await send({
            type: 'template_operation',
            context: this.context,
            action: 'template.properties.revert',
            values: null,
            source: null,
            input: inputId(),
          }),
        );
      this.dirty = false;
      this.dialog.close();
    } catch (e) {
      this.note.textContent = errorText(e);
    } finally {
      this.busy(false);
      this.changed();
    }
  }
}
