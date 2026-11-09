/** Session-only pane visibility is shared by project/template shells, never document history. */
import { element } from './ui';
type Pane = 'explorer' | 'properties';
export class WorkspacePanes {
  private collapsed = { explorer: false, properties: false };
  private controls: {
    kind: Pane;
    pane: HTMLElement;
    button: HTMLButtonElement;
    rail: HTMLButtonElement;
  }[] = [];
  constructor(
    private workspace: HTMLElement,
    private action: (id: string) => void,
  ) {}
  attach(kind: Pane, pane: HTMLElement, header: HTMLElement) {
    const button = element('button', 'pane-toggle');
    const rail = element('button', 'pane-rail', kind === 'explorer' ? 'Explorer' : 'Properties');
    const id = `workspace-pane-${kind}-${this.controls.length}`;
    pane.id = id;
    for (const control of [button, rail]) {
      control.type = 'button';
      control.dataset.action = `workspace.panes.${kind}`;
      control.setAttribute('aria-controls', id);
      control.addEventListener('click', () => this.action(control.dataset.action!));
    }
    header.append(button);
    pane.append(rail);
    this.controls.push({ kind, pane, button, rail });
    this.render();
  }
  toggle(kind: Pane) {
    this.collapsed[kind] = !this.collapsed[kind];
    this.render();
    // Dispatch temporarily disables buttons; restore focus after the host unlocks them.
    requestAnimationFrame(() => {
      const entry = this.controls.find(
        (c) =>
          c.kind === kind &&
          c.pane.getClientRects().length &&
          getComputedStyle(c.pane).visibility !== 'hidden',
      );
      if (entry) (this.collapsed[kind] ? entry.rail : entry.button).focus({ preventScroll: true });
    });
  }
  show(kind: Pane) {
    this.collapsed[kind] = false;
    this.render();
  }
  private render() {
    this.workspace.dataset.explorerCollapsed = String(this.collapsed.explorer);
    this.workspace.dataset.propertiesCollapsed = String(this.collapsed.properties);
    for (const { kind, pane, button, rail } of this.controls) {
      const collapsed = this.collapsed[kind];
      pane.classList.toggle('pane-collapsed', collapsed);
      button.textContent = kind === 'explorer' ? '‹' : '›';
      button.title = `Collapse ${kind} pane`;
      rail.title = `Expand ${kind} pane`;
      button.setAttribute('aria-label', button.title);
      rail.setAttribute('aria-label', rail.title);
      for (const control of [button, rail])
        control.setAttribute('aria-expanded', String(!collapsed));
    }
  }
}
