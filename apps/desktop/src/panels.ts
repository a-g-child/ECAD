/** Explorer, shared property rows, and report table stay inside shell-owned bounds. */
import type { PropertyRow, Snapshot, Target } from './types';
import { keyOf } from './types';
import { element } from './ui';
import type { Shell } from './shell';
export function renderTree(
  shell: Shell,
  snapshot: Snapshot,
  selection: Target[],
  choose: (target: Target, add: boolean) => void,
) {
  const selected = new Set(selection.map(keyOf));
  const opened = new Map(
    [...shell.tree.querySelectorAll<HTMLDetailsElement>('details[data-object]')].map((d) => [
      d.dataset.object!,
      d.open,
    ]),
  );
  shell.tree.replaceChildren();
  const children = new Map<string, typeof snapshot.objects>();
  for (const row of snapshot.objects) {
    const key = row.parent ? keyOf(row.parent) : 'root';
    if (!children.has(key)) children.set(key, []);
    children.get(key)!.push(row);
  }
  const render = (row: Snapshot['objects'][number], depth: number): HTMLElement => {
    const key = keyOf(row.target);
    const descendants = children.get(key) ?? [];
    const button = element('button', 'tree-item');
    button.type = 'button';
    button.dataset.object = key;
    button.title = `${row.label}\n${row.target.kind} · ${row.target.id}`;
    button.classList.toggle('selected', selected.has(key));
    button.setAttribute('aria-pressed', String(selected.has(key)));
    const kind = element(
      'span',
      'object-kind',
      (
        {
          subsystem: 'S',
          device: 'D',
          terminal: 'T',
          conductor: 'W',
          cable: 'C',
          view: '▧',
          route: '↔',
          anchor: '○',
          label: 'T',
          polyline: '⌁',
          port: 'P',
        } as Record<string, string>
      )[row.target.kind] ?? '·',
    );
    button.append(kind, element('span', 'tree-label', row.label));
    button.addEventListener('click', (event) =>
      choose(row.target, event.shiftKey || event.metaKey || event.ctrlKey),
    );
    button.addEventListener('contextmenu', (event) => {
      event.preventDefault();
      if (!selected.has(key)) choose(row.target, false);
      button.focus();
    });
    if (!descendants.length) {
      button.style.paddingInlineStart = `${10 + depth * 12}px`;
      return button;
    }
    const details = element('details', 'tree-branch');
    details.dataset.object = key;
    details.open = opened.get(key) ?? depth < 2;
    const summary = element('summary');
    summary.style.paddingInlineStart = `${4 + depth * 12}px`;
    summary.append(button);
    details.append(summary);
    for (const child of descendants) details.append(render(child, depth + 1));
    return details;
  };
  const roots = children.get('root') ?? [];
  for (const [label, filter] of [
    ['FUNCTIONAL STRUCTURE', (r: (typeof roots)[number]) => r.target.kind !== 'view'],
    ['DRAWING VIEWS', (r: (typeof roots)[number]) => r.target.kind === 'view'],
  ] as const) {
    shell.tree.append(element('div', 'tree-section', label));
    for (const row of roots.filter(filter)) shell.tree.append(render(row, 0));
  }
}
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
  const summary = element('div', 'inspector-summary', `${rows.length} properties · values in µm`);
  shell.inspector.append(summary);
  for (const row of rows) {
    const group = element('div', 'property-row');
    group.dataset.property = row.key;
    const label = element('label', 'property-label', row.label);
    label.title = `${row.label}\n${row.applicable} applicable · ${row.unsupported} unsupported · ${row.restricted} restricted${row.origin ? '\nSource: ' + row.origin : ''}`;
    if (row.applicable < count)
      label.append(element('span', 'property-count', `${row.applicable}/${count}`));
    if (row.editable) {
      const input = element('input', 'property-input');
      input.type = 'text';
      input.value = row.value ?? '';
      input.placeholder = row.mixed ? 'Mixed' : '';
      input.id = `property-${row.key}`;
      input.dataset.original = input.value;
      input.setAttribute('aria-label', row.label);
      label.htmlFor = input.id;
      if (row.kind === 'length') input.inputMode = 'numeric';
      input.title = [row.origin, 'Press Enter to apply; Escape to revert']
        .filter(Boolean)
        .join('\n');
      input.addEventListener('keydown', (event) => {
        if (event.key === 'Enter') {
          event.preventDefault();
          event.stopPropagation();
          void edit(row.key, input.value);
        }
        if (event.key === 'Escape') {
          event.stopPropagation();
          input.value = input.dataset.original ?? '';
          input.blur();
        }
      });
      group.append(label, input);
    } else {
      const value = element('div', 'property-value', row.display);
      value.title = `${row.display}\n${row.reason}`;
      value.setAttribute('aria-label', `${row.label}: ${row.display}`);
      group.append(label, value);
    }
    shell.inspector.append(group);
  }
  const hint = element(
    'p',
    'inspector-note',
    'Enter applies a value as one undoable edit. Graphical fields are distinct from physical wire attributes.',
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
