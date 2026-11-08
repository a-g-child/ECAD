/** Captured project context menu. Entries dispatch registered actions after closing the popup. */
import type { Context } from './types';
import type { Shell } from './shell';
import { element } from './ui';
export class ProjectMenu {
  private dialog = element('dialog', 'project-menu');
  private captured: Context | null = null;
  get active() {
    return this.dialog.open;
  }
  constructor(shell: Shell, dispatch: (id: string, context: Context) => void) {
    this.dialog.setAttribute('aria-label', 'Project menu');
    const entry = (label: string, id: string) => {
      const button = element('button', 'menu-entry', label);
      button.type = 'button';
      button.dataset.action = id;
      button.addEventListener('click', () => {
        const context = this.captured;
        this.dialog.close();
        if (context) dispatch(id, context);
      });
      return button;
    };
    const add = element('details', 'project-add');
    add.open = true;
    add.append(
      element('summary', '', 'Add new…'),
      entry('Schematic sheet…', 'drawing.sheet.new_schematic'),
      entry('Cover sheet…', 'drawing.sheet.new_cover'),
      entry('Drawing list sheet…', 'drawing.sheet.new_list'),
    );
    this.dialog.append(entry('Properties…', 'document.metadata.edit'), add);
    this.dialog.addEventListener('click', (event) => {
      if (event.target === this.dialog) this.dialog.close();
    });
    shell.app.append(this.dialog);
  }
  open(event: MouseEvent, context: Context) {
    this.captured = { ...context, selection: [], sheet: null, view: null, focus: 'tree' };
    this.dialog.showModal();
    const b = this.dialog.getBoundingClientRect();
    this.dialog.style.left = `${Math.max(8, Math.min(event.clientX, window.innerWidth - b.width - 8))}px`;
    this.dialog.style.top = `${Math.max(8, Math.min(event.clientY, window.innerHeight - b.height - 8))}px`;
    this.dialog.querySelector<HTMLButtonElement>('button')?.focus();
  }
}
