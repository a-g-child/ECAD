/** Field completion policy shared by contextual inspectors; validation/commands stay with the owner. */
type Field = HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement;
function field(node: EventTarget | null): node is Field {
  return (
    node instanceof HTMLInputElement ||
    node instanceof HTMLTextAreaElement ||
    node instanceof HTMLSelectElement
  );
}
interface Address {
  label: string;
  artwork: string | undefined;
}
export class PropertyCompletion {
  private composing = false;
  private deferred = false;
  constructor(
    private root: HTMLElement,
    private enabled: () => boolean,
    private pending: () => boolean,
    private busy: () => boolean,
    private commit: () => Promise<boolean>,
    private revert: () => Promise<void>,
  ) {
    root.addEventListener('compositionstart', () => {
      this.composing = true;
    });
    root.addEventListener('compositionend', () => {
      this.composing = false;
      if (this.deferred) {
        this.deferred = false;
        queueMicrotask(() => void this.finish());
      }
    });
    root.addEventListener('focusout', (e) => {
      if (!field(e.target) || !this.enabled() || this.busy() || !this.pending()) return;
      // Moving between an exact field and its choices is one composite field edit.
      if (
        e.relatedTarget instanceof Node &&
        e.target.closest('.appearance-control')?.contains(e.relatedTarget)
      )
        return;
      // Buttons own explicit commands/structural edits; do not race their click handlers.
      if (e.relatedTarget instanceof Element && e.relatedTarget.closest('button')) return;
      if (this.composing) {
        this.deferred = true;
        return;
      }
      const next = this.address(e.relatedTarget);
      queueMicrotask(() => void this.finish(next));
    });
    root.addEventListener('change', (e) => {
      if (
        e.target instanceof HTMLSelectElement ||
        (e.target instanceof HTMLInputElement && e.target.type === 'checkbox')
      ) {
        const next = this.address(e.target);
        queueMicrotask(() => void this.finish(next));
      }
    });
    root.addEventListener('keydown', (e) => {
      if (!field(e.target) || !this.enabled() || e.isComposing || this.composing || this.busy())
        return;
      if (e.key === 'Escape' && this.pending()) {
        e.preventDefault();
        e.stopPropagation();
        const at = this.address(e.target);
        void this.revert().then(() => this.focus(at));
      } else if (
        e.key === 'Enter' &&
        (!(e.target instanceof HTMLTextAreaElement) || e.ctrlKey || e.metaKey)
      ) {
        e.preventDefault();
        e.stopPropagation();
        void this.finish(this.address(e.target));
      }
    });
  }
  get isComposing() {
    return this.composing;
  }
  private address(node: EventTarget | null): Address | undefined {
    if (!field(node) || !this.root.contains(node)) return;
    return {
      label: node.getAttribute('aria-label') ?? '',
      artwork: node.closest<HTMLElement>('[data-artwork-id]')?.dataset.artworkId,
    };
  }
  private focus(address?: Address) {
    if (!address) return;
    const target = Array.from(this.root.querySelectorAll<Field>('input,textarea,select')).find(
      (n) =>
        n.getAttribute('aria-label') === address.label &&
        n.closest<HTMLElement>('[data-artwork-id]')?.dataset.artworkId === address.artwork,
    );
    target?.focus({ preventScroll: true });
  }
  private async finish(next?: Address) {
    if (!this.enabled() || this.composing || this.busy() || !this.pending()) return;
    await this.commit();
    this.focus(next);
  }
}
