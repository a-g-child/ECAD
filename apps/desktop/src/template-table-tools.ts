/** Captured structural table edits; the Rust core owns geometry, identity allocation and history. */
import { TableLayoutFields } from './template-table-layout';
import { dimension, readDimension } from './dimensions';
import type { Context, Reply, TemplateVariant, TemplateTable, TemplateCell } from './types';
import { element } from './ui';
import { send, inputId, errorText } from './transport';
const actions: Record<string, [string, string]> = {
  layout: ['Table layout', 'template.table.layout'],
  insert_row: ['Insert title-block row', 'template.table.insert_row'],
  insert_column: ['Insert title-block column', 'template.table.insert_column'],
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
  private layoutFields: TableLayoutFields | null = null;
  private operation = '';
  private pending = false;
  private dirty = false;
  private controls = new Map<string, HTMLInputElement | HTMLSelectElement>();
  private selectedCells: TemplateCell[] = [];
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
    selectedCells: string[] = [],
  ) {
    if (this.active || !actions[operation]) return;
    if (['split', 'merge'].includes(operation) && !selection) return;
    this.selectedCells = selection
      ? selection.table.cells
          .filter((c) => selectedCells.includes(c.id))
          .sort((a, b) => {
            for (const i of [1, 0]) {
              const delta = BigInt(a.bounds[i]) - BigInt(b.bounds[i]);
              if (delta) return delta < 0n ? -1 : 1;
            }
            return a.id.localeCompare(b.id);
          })
      : [];
    this.tableSelect.hidden = ['split', 'merge'].includes(operation);
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
    this.populate();
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
    const result = this.field(name, input);
    if (name.includes('(µm)'))
      dimension(input, name, input.closest('label')?.querySelector('span') ?? undefined);
    return result;
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
  private populate() {
    this.body.replaceChildren();
    this.controls.clear();
    this.layoutFields = null;
    const table = this.table();
    this.applyButton.disabled = !table;
    if (!table) {
      this.body.append(element('p', '', 'This variant has no title-block table.'));
      return;
    }
    if (this.operation === 'layout') {
      this.layoutFields = new TableLayoutFields(table);
      this.body.append(this.layoutFields.root);
    } else if (this.operation.startsWith('insert_')) {
      const axis = this.operation === 'insert_row' ? 1 : 0;
      this.number('Insert at (µm)', table.bounds[axis + 2]);
      this.number('Band size (µm)', table.cells[0].bounds[axis + 2]);
      this.select('Growth anchor', [
        ['end', 'Keep bottom/right edge'],
        ['start', 'Keep top/left edge'],
      ]);
      this.body.append(
        element(
          'p',
          'settings-note',
          'Blank cells follow adjacent divisions. Cells spanning the insertion boundary extend across it. The complete table must fit within the paper margins.',
        ),
      );
    } else if (this.operation === 'resize') {
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
      for (const [name, value] of [
        ['Columns', '2'],
        ['Rows', '1'],
      ]) {
        const input = this.number(name, value) as HTMLInputElement;
        input.type = 'number';
        input.min = '1';
        input.max = '4096';
        input.step = '1';
      }
      this.body.append(
        element(
          'p',
          'settings-note',
          `${this.selectedCells.length} selected cell(s). Each original upper-left cell retains its contents; new cells are blank. The complete split is one undo operation.`,
        ),
      );
    } else if (this.operation === 'merge') {
      const otherContents = this.selectedCells
        .slice(1)
        .some((c) => c.caption.length || c.contents.length);
      if (otherContents)
        this.select('Merged contents', [
          ['keep_primary', 'Keep only the first cell’s contents'],
          ['combine', 'Combine all captions and contents in reading order'],
        ]);
      this.body.append(
        element(
          'p',
          'settings-note',
          `${this.selectedCells.length} selected cells form one rectangle. The top-left identity survives. The first (top-left) cell’s caption and contents are kept by default, even if blank. Undo restores the original cells exactly.`,
        ),
      );
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
    const field = this.controls.get(name)!;
    return field instanceof HTMLInputElement ? readDimension(field) : field.value;
  }
  private proposal() {
    const value = (name: string) => this.value(name);
    let edit: object;
    switch (this.operation) {
      case 'layout':
        edit = this.layoutFields!.proposal();
        break;
      case 'insert_row':
      case 'insert_column':
        edit = {
          operation: 'insert',
          axis: this.operation === 'insert_row' ? 'y' : 'x',
          at: value('Insert at (µm)'),
          size: value('Band size (µm)'),
          anchor: value('Growth anchor'),
        };
        break;
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
          cells: this.selectedCells.map((c) => c.id),
          columns: value('Columns'),
          rows: value('Rows'),
        };
        break;
      case 'merge':
        edit = {
          operation: 'merge',
          cells: this.selectedCells.map((c) => c.id),
          primary: this.selectedCells[0].id,
          contents: this.controls.has('Merged contents')
            ? value('Merged contents')
            : 'keep_primary',
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
