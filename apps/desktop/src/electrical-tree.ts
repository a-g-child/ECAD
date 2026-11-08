/** Electrical objects and drawing representations retain real property/selection identities. */
import type { Snapshot, Target } from './types';
import { keyOf } from './types';
import { element } from './ui';
export function renderElectricalTree(
  container: HTMLElement,
  snapshot: Snapshot,
  selection: Target[],
  choose: (target: Target, add: boolean) => void,
  opened: Map<string, boolean>,
) {
  const selected = new Set(selection.map(keyOf));
  container.replaceChildren();
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
    details.dataset.node = key;
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
    container.append(element('div', 'tree-section', label));
    for (const row of roots.filter(filter)) container.append(render(row, 0));
  }
}
