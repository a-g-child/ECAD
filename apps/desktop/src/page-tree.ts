/** Project/document/sheet navigation uses stable identities and never electrical object selection. */
import type { Snapshot } from './types';
import { element } from './ui';
export function renderPageTree(
  container: HTMLElement,
  snapshot: Snapshot,
  active: string,
  opened: Map<string, boolean>,
  locked: boolean,
  show: (id: string) => boolean,
  settings: (id: string) => void,
) {
  container.replaceChildren();
  const branch = (key: string, label: string, depth: number) => {
    const details = element('details', 'tree-branch page-branch');
    details.dataset.node = key;
    details.open = opened.get(key) ?? true;
    const summary = element('summary');
    summary.style.paddingInlineStart = `${4 + depth * 12}px`;
    summary.append(element('span', 'tree-label', label));
    summary.title = label;
    details.append(summary);
    return details;
  };
  const root = branch(`project:${snapshot.revision.epoch}`, snapshot.name, 0);
  root.classList.add('project-root');
  root.querySelector('summary')!.title = snapshot.path ?? snapshot.name;
  container.append(root);
  const sheets = new Map(snapshot.sheets.map((s) => [s.id, s]));
  for (const document of snapshot.drawing_documents) {
    const elide = snapshot.drawing_documents.length === 1 && document.is_default;
    const parent = elide
      ? root
      : branch(`document:${document.id}`, `${document.number} · ${document.title}`, 1);
    if (!elide) {
      parent.dataset.document = document.id;
      root.append(parent);
    }
    for (const id of document.sheets) {
      const sheet = sheets.get(id);
      if (!sheet) throw new Error('The page explorer references an unavailable sheet');
      const button = element('button', 'tree-item page-item');
      button.type = 'button';
      button.dataset.sheet = id;
      button.style.paddingInlineStart = `${10 + (elide ? 1 : 2) * 12}px`;
      button.title = `${sheet.number}\n${sheet.title}${sheet.view ? '' : '\nBlank sheet'}`;
      button.classList.toggle('selected', id === active);
      if (id === active) button.setAttribute('aria-current', 'page');
      button.disabled = locked;
      button.append(
        element('span', 'object-kind', '▧'),
        element('span', 'tree-label', `${sheet.number} · ${sheet.title}`),
      );
      button.addEventListener('click', () => {
        if (!button.disabled) show(id);
      });
      button.addEventListener('contextmenu', (event) => {
        event.preventDefault();
        event.stopPropagation();
        if (!button.disabled) settings(id);
      });
      parent.append(button);
    }
    if (!document.sheets.length) {
      const note = element(
        'p',
        'tree-empty',
        elide
          ? 'No sheets. Use Sheets and paper to create or attach one.'
          : 'No sheets in this document.',
      );
      parent.append(note);
    }
  }
}
