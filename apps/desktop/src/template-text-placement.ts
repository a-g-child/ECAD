import { controlHelp } from './control-help';
/** Canvas-local text draft; shared typed run controls never resolve project values here. */
import type { TemplateRun } from './types';
import { TemplateRuns } from './template-runs';
import { element } from './ui';
import { applyTextKey } from './text';
export class TemplateTextPlacement {
  private root = element('section', 'template-text-placement');
  private runs: TemplateRun[] = [{ kind: 'literal', value: '' }];
  private note = element('p', 'settings-error');
  private busy = false;
  private composing = false;
  private closed = false;
  private resize: ResizeObserver;
  constructor(
    canvas: HTMLElement,
    svg: SVGSVGElement,
    anchor: [number, number],
    private preview: (runs: TemplateRun[]) => void,
    private finish: () => Promise<boolean>,
    private cancel: () => Promise<void>,
  ) {
    this.root.dataset.templateTextEditor = '';
    this.root.dataset.focus = 'text';
    this.root.setAttribute('role', 'region');
    this.root.setAttribute('aria-label', 'Place template text');
    this.note.setAttribute('role', 'alert');
    const controls = new TemplateRuns(this.runs, () => this.queue());
    const place = element('button', 'plain', 'Place text');
    const cancelButton = element('button', 'plain', 'Cancel text');
    place.type = cancelButton.type = 'button';
    controlHelp(place, 'Enter adds a line; Ctrl/Cmd+Enter places text.');
    controlHelp(cancelButton, 'Escape cancels this text placement.');
    place.addEventListener('click', () => void this.commit());
    cancelButton.addEventListener('click', () => {
      if (!this.busy) void this.cancel();
    });
    const actions = element('div', 'settings-actions');
    actions.append(place, cancelButton);
    this.root.append(
      element('strong', '', 'New template text'),
      controls.runFields,
      this.note,
      actions,
    );
    this.root.addEventListener('compositionstart', () => {
      this.composing = true;
    });
    this.root.addEventListener('compositionend', () => {
      this.composing = false;
      this.queue();
    });
    this.root.addEventListener('keydown', (e) => {
      if (e.isComposing || this.composing || this.busy) return;
      if (e.key === 'Escape') {
        e.preventDefault();
        e.stopPropagation();
        void this.cancel();
      } else if (applyTextKey(e)) {
        e.preventDefault();
        e.stopPropagation();
        void this.commit();
      }
    });
    canvas.append(this.root);
    // The floating editor stays inside the existing canvas; no pane/ribbon dimensions change.
    const position = () => {
      const box = canvas.getBoundingClientRect();
      const matrix = svg.getScreenCTM();
      if (!matrix) return;
      const at = new DOMPoint(anchor[0] / 1000, anchor[1] / 1000).matrixTransform(matrix);
      this.root.style.left = `${Math.max(0, Math.min(at.x - box.x + 12, box.width - this.root.offsetWidth))}px`;
      this.root.style.top = `${Math.max(0, Math.min(at.y - box.y + 12, box.height - Math.min(360, box.height)))}px`;
    };
    this.resize = new ResizeObserver(position);
    this.resize.observe(canvas);
    position();
    this.root.querySelector('textarea')?.focus();
    this.queue();
  }
  private queue() {
    // Coalesce synchronous draft changes and defer preview until composition has finished.
    queueMicrotask(() => {
      if (this.closed || this.busy || this.composing) return;
      this.preview(structuredClone(this.runs));
    });
  }
  private async commit() {
    if (this.busy || this.composing || this.closed) return;
    const focused = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    this.preview(structuredClone(this.runs));
    this.busy = true;
    this.lock(true);
    try {
      if (!(await this.finish()))
        this.note.textContent =
          'Enter text or choose a variable, and check the text box fits the paper.';
    } finally {
      this.busy = false;
      if (!this.closed) {
        this.lock(false);
        if (focused && this.root.contains(focused)) focused.focus({ preventScroll: true });
      }
    }
  }
  private lock(locked: boolean) {
    for (const input of this.root.querySelectorAll<
      HTMLInputElement | HTMLButtonElement | HTMLSelectElement | HTMLTextAreaElement
    >('input,button,select,textarea'))
      input.disabled = locked;
  }
  close() {
    this.closed = true;
    this.resize.disconnect();
    this.root.remove();
  }
}
