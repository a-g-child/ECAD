import { controlHelp } from './control-help';
/** Row-count and per-row column inputs. The core owns repartitioning and identity retention. */
import type { TemplateTable } from './types';
import { element } from './ui';
export class TableLayoutFields {
  readonly root = element('div', 'metadata-fields');
  private rows = element('select', 'property-input');
  private columns = element('div', 'metadata-fields');
  private removal = element('label', 'settings-note');
  private acknowledge = element('input');
  private counts: string[];
  private original: number[];
  constructor(table: TemplateTable) {
    const groups = new Map<string, number>();
    for (const cell of table.cells)
      groups.set(cell.bounds[1], (groups.get(cell.bounds[1]) ?? 0) + 1);
    this.original = [...groups].sort(([a], [b]) => Number(a) - Number(b)).map(([, n]) => n);
    this.counts = this.original.map(String);
    // Normal title blocks rarely need many rows; retain larger imported counts as an option.
    const choices = Array.from({ length: 64 }, (_, i) => i + 1);
    if (this.original.length > 64) choices.push(this.original.length);
    for (const n of choices) {
      const option = element('option', '', String(n));
      option.value = String(n);
      this.rows.append(option);
    }
    this.rows.value = String(this.original.length);
    this.rows.setAttribute('aria-label', 'Number of rows');
    const rowLabel = element('label', 'settings-field');
    rowLabel.append(element('span', '', 'Number of rows'), this.rows);
    this.acknowledge.type = 'checkbox';
    this.acknowledge.setAttribute('aria-label', 'Remove surplus cells and their contents');
    this.removal.append(this.acknowledge, element('span'));
    this.rows.addEventListener('change', () => this.renderColumns());
    controlHelp(
      this.rows,
      'Outer table size stays fixed. Existing cells retain their contents and formatting by row and column. Added cells are blank. Changed rows use equal column widths; changing row count uses equal row heights. Spanning cells become ordinary row cells when the layout changes. Undo restores the complete previous layout.',
    );
    this.root.append(rowLabel, this.columns, this.removal);
    this.renderColumns();
  }
  private renderColumns() {
    this.columns.replaceChildren();
    for (let i = 0; i < Number(this.rows.value); i++) {
      const input = element('input', 'property-input');
      input.type = 'number';
      controlHelp(
        input,
        'Set the number of columns in this row. Outer table bounds stay fixed; removed cells require acknowledgement.',
      );
      input.min = '1';
      input.max = '4096';
      input.step = '1';
      input.value = this.counts[i] ?? '1';
      this.counts[i] = input.value;
      const name = `Row ${i + 1} columns`;
      input.setAttribute('aria-label', name);
      const label = element('label', 'settings-field');
      label.append(element('span', '', name), input);
      this.columns.append(label);
      input.addEventListener('input', () => {
        this.counts[i] = input.value;
        this.reviewRemoval();
      });
    }
    this.reviewRemoval();
  }
  private reviewRemoval() {
    this.acknowledge.checked = false;
    const count = Number(this.rows.value);
    const removed = this.original.reduce(
      (sum, n, i) => sum + Math.max(0, n - (i < count ? Number(this.counts[i]) || 0 : 0)),
      0,
    );
    this.removal.hidden = removed === 0;
    this.removal.querySelector('span')!.textContent =
      `Remove ${removed} surplus cell(s) and their contents (rightmost cells and bottom rows).`;
  }
  proposal() {
    return {
      operation: 'layout',
      columns: this.counts.slice(0, Number(this.rows.value)),
      allow_removal: this.acknowledge.checked,
    };
  }
}
