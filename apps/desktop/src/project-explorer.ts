/** Explorer mode/expansion/focus belong to the open workspace, outside document history. */
import type { Shell } from './shell';
import type { Snapshot, Target } from './types';
import { element } from './ui';
import { renderElectricalTree } from './electrical-tree';
import { renderPageTree } from './page-tree';
type Mode = 'pages' | 'electrical';
export class ProjectExplorer {
  private mode: Mode = 'pages';
  private snapshot: Snapshot | null = null;
  private selection: Target[] = [];
  private sheet = '';
  private locked = false;
  private expansion = { pages: new Map<string, boolean>(), electrical: new Map<string, boolean>() };
  constructor(
    private shell: Shell,
    private choose: (t: Target, add: boolean) => void,
    private showSheet: (id: string) => boolean,
    private sheetSettings: (id: string) => void,
    private action: (id: string) => void,
  ) {
    for (const [mode, label, name] of [
      ['pages', 'Pages', 'Project pages'],
      ['electrical', 'Electrical', 'Electrical explorer'],
    ] as const) {
      const button = element('button', 'plain explorer-mode', label);
      button.type = 'button';
      button.dataset.mode = mode;
      button.dataset.action = `workspace.explorer.${mode}`;
      button.setAttribute('aria-label', name);
      button.addEventListener('click', () => action(button.dataset.action!));
      shell.explorerModes.append(button);
    }
    shell.tree.addEventListener('contextmenu', (event) => {
      event.preventDefault();
      if (this.mode !== 'electrical' || this.locked) return;
      const key = (event.target as Element).closest<HTMLElement>('[data-object]')?.dataset.object;
      if (!key) return;
      [...shell.tree.querySelectorAll<HTMLButtonElement>('button[data-object]')]
        .find((b) => b.dataset.object === key)
        ?.focus();
      action('selection.properties');
    });
    this.paintMode();
  }
  update(snapshot: Snapshot, selection: Target[], sheet: string, replaced = false) {
    if (replaced) {
      this.expansion = { pages: new Map(), electrical: new Map() };
      this.shell.tree.replaceChildren();
    }
    this.snapshot = snapshot;
    this.selection = selection;
    this.sheet = sheet;
    this.render();
  }
  show(mode: Mode) {
    this.captureExpansion();
    this.mode = mode;
    this.shell.tree.replaceChildren();
    this.paintMode();
    this.render();
    this.shell.explorerModes.querySelector<HTMLButtonElement>(`[data-mode="${mode}"]`)?.focus();
  }
  setLocked(locked: boolean) {
    this.locked = locked;
    for (const button of this.shell.tree.querySelectorAll<HTMLButtonElement>(
      'button[data-sheet],button[data-object]',
    ))
      button.disabled = locked;
  }
  private paintMode() {
    this.shell.tree.setAttribute(
      'aria-label',
      this.mode === 'pages' ? 'Project pages' : 'Project electrical structure',
    );
    this.shell.tree.dataset.mode = this.mode;
    for (const button of this.shell.explorerModes.querySelectorAll<HTMLButtonElement>(
      'button[data-mode]',
    )) {
      button.setAttribute('aria-pressed', String(button.dataset.mode === this.mode));
    }
  }
  private captureExpansion() {
    for (const details of this.shell.tree.querySelectorAll<HTMLDetailsElement>(
      'details[data-node]',
    ))
      this.expansion[this.mode].set(details.dataset.node!, details.open);
  }
  private render() {
    if (!this.snapshot) return;
    const body = this.shell.tree;
    const scroll = body.scrollTop;
    const focused = body.contains(document.activeElement)
      ? (document.activeElement as HTMLElement)
      : null;
    const focusSelector = focused?.matches('button[data-sheet]')
      ? 'button[data-sheet]'
      : focused?.matches('button[data-object]')
        ? 'button[data-object]'
        : 'details[data-node]';
    const focusKey =
      focused?.dataset.sheet ??
      focused?.dataset.object ??
      focused?.closest<HTMLElement>('details[data-node]')?.dataset.node;
    this.captureExpansion();
    if (this.mode === 'pages')
      renderPageTree(
        body,
        this.snapshot,
        this.sheet,
        this.expansion.pages,
        this.locked,
        this.showSheet,
        this.sheetSettings,
      );
    else
      renderElectricalTree(
        body,
        this.snapshot,
        this.selection,
        this.choose,
        this.expansion.electrical,
      );
    this.setLocked(this.locked);
    body.scrollTop = scroll;
    if (focusKey) {
      const target = [...body.querySelectorAll<HTMLElement>(focusSelector)].find(
        (node) => (node.dataset.sheet ?? node.dataset.object ?? node.dataset.node) === focusKey,
      );
      (target?.matches('details') ? target.querySelector<HTMLElement>('summary') : target)?.focus({
        preventScroll: true,
      });
    }
  }
}
