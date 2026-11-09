/** Shared property rows and report tables stay inside shell-owned bounds. */
import { dimension, dimensionSummary, readDimension, writeDimension } from './dimensions';
import type { PropertyRow } from './types';
import { element } from './ui';
import { appearanceControl } from './appearance-controls';
import type { Shell } from './shell';
import { textEditor, applyTextKey } from './text';
export function renderProperties(
  shell: Shell,
  rows: PropertyRow[],
  count: number,
  edit: (key: string, value: string) => Promise<void>,
) {
  const scroll = shell.inspector.scrollTop;
  shell.inspector.replaceChildren();
  shell.inspectorCount.textContent = count ? `${count} SELECTED` : 'NO SELECTION';
  if (!count) {
    const note = element('div', 'inspector-empty');
    note.append(
      element('strong', '', 'Inspect your design'),
      element(
        'p',
        '',
        'Select an object in the drawing or project tree. Shift-select to inspect and edit shared properties.',
      ),
    );
    shell.inspector.append(note);
    return;
  }
  const summary = dimensionSummary(
    [],
    `${rows.length} properties · lengths in`,
    'inspector-summary',
  );
  shell.inspector.append(summary);
  for (const row of rows) {
    const group = element('div', 'property-row');
    group.dataset.property = row.key;
    const label = element('label', 'property-label', row.label);
    label.title = `${row.label}\n${row.applicable} applicable · ${row.unsupported} unsupported · ${row.restricted} restricted${row.origin ? '\nSource: ' + row.origin : ''}`;
    if (row.applicable < count)
      label.append(element('span', 'property-count', `${row.applicable}/${count}`));
    if (row.editable) {
      const multiline = row.kind === 'text';
      const input = multiline
        ? textEditor('property-input property-text')
        : element('input', 'property-input');
      if (input instanceof HTMLInputElement) input.type = 'text';
      input.value = row.value ?? '';
      input.placeholder = row.mixed ? 'Mixed' : '';
      input.id = `property-${row.key}`;
      input.dataset.original = input.value;
      input.setAttribute('aria-label', row.label);
      label.htmlFor = input.id;
      if (row.kind === 'length') {
        dimension(input);
        input.dataset.original = input.value;
      }
      input.title = [
        row.origin,
        multiline
          ? 'Enter inserts a line break; Ctrl/Cmd+Enter applies; Escape reverts'
          : 'Press Enter to apply; Escape to revert',
      ]
        .filter(Boolean)
        .join('\n');
      input.addEventListener('keydown', (raw) => {
        const event = raw as KeyboardEvent;
        if (event.isComposing) return;
        if (multiline ? applyTextKey(event) : event.key === 'Enter') {
          event.preventDefault();
          event.stopPropagation();
          void edit(row.key, readDimension(input));
        }
        if (event.key === 'Escape') {
          event.stopPropagation();
          writeDimension(input, row.value ?? '');
          input.dispatchEvent(new Event('input'));
          input.blur();
        }
      });
      const editor = element('div', 'property-editor');
      editor.append(
        row.key === 'font_family' || row.kind === 'colour'
          ? appearanceControl(input, row.kind === 'colour' ? 'colour' : 'font', () => {
              void edit(row.key, readDimension(input));
            })
          : input,
      );
      if (multiline) {
        const apply = element('button', 'plain property-apply', 'Apply');
        apply.setAttribute('aria-label', `Apply ${row.label}`);
        apply.addEventListener('click', () => void edit(row.key, readDimension(input)));
        editor.append(apply);
      }
      group.append(label, editor);
    } else {
      const value = element(
        'div',
        `property-value${row.kind === 'text' ? ' property-text-value' : ''}`,
        row.display,
      );
      value.title = `${row.display}\n${row.reason}`;
      value.setAttribute('aria-label', `${row.label}: ${row.display}`);
      group.append(label, value);
    }
    shell.inspector.append(group);
  }
  const hint = element(
    'p',
    'inspector-note',
    'Text: Enter adds a line; Ctrl/Cmd+Enter or Apply commits one edit. Other fields use Enter.',
  );
  shell.inspector.append(hint);
  shell.inspector.scrollTop = scroll;
}
export function renderReport(shell: Shell, rows: string[][]) {
  shell.reportBody.replaceChildren();
  const table = element('table');
  for (const [index, row] of rows.entries()) {
    const tr = element('tr');
    for (const value of row) {
      const cell = element(index === 0 ? 'th' : 'td', '', value);
      cell.title = value;
      tr.append(cell);
    }
    table.append(tr);
  }
  shell.reportBody.append(table);
  shell.reportPane.hidden = false;
}
