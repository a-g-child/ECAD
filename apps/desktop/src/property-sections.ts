/** Compact inspector groups; expanded state belongs to the workspace, never the document. */
import { element } from './ui';
export class PropertySections {
  private expanded = new Map<string, boolean>();
  section(id: string, title: string) {
    const root = element('details', 'property-section');
    root.dataset.section = id;
    root.setAttribute('aria-label', title);
    root.open = this.expanded.get(id) ?? true;
    const summary = element('summary', '', title);
    // Record native click/keyboard activation before a property completion can rebuild the form.
    summary.addEventListener('click', () => this.expanded.set(id, !root.open));
    root.addEventListener('toggle', () => {
      if (root.isConnected) this.expanded.set(id, root.open);
    });
    const body = element('div', 'property-section-body');
    root.append(summary, body);
    return { root, body };
  }
  focus(control: HTMLElement | undefined) {
    if (!control) return;
    for (let p: HTMLElement | null = control.parentElement; p; p = p.parentElement) {
      if (p instanceof HTMLDetailsElement && p.classList.contains('property-section')) {
        this.expanded.set(p.dataset.section!, true);
        p.open = true;
      }
    }
    control.focus();
  }
}
