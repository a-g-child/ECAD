/** Transient new-text placement; the bridge owns identity, validation, defaults and snapping. */
import type { AnnotationRow, Context, Reply, Target } from './types';
import type { Shell } from './shell';
import type { DrawingCanvas } from './canvas';
import { element } from './ui';
import { send, inputId, errorText } from './transport';
export class AnnotationTool {
  active = false;
  private generation = 0;
  private row: AnnotationRow | null = null;
  private pending: object | null = null;
  private pumping = false;
  private committing = false;
  private valid = false;
  private suppressed = false;
  private pointer: { x: string; y: string; tolerance: number } | null = null;
  private bar = element('div', 'move-bar annotation-bar moving');
  private text = element('input', 'annotation-text');
  private x = element('input', 'move-coordinate');
  private y = element('input', 'move-coordinate');
  private applyButton = element('button', 'plain', 'Apply');
  private cancelButton = element('button', 'plain', 'Cancel');
  constructor(
    private shell: Shell,
    private drawing: DrawingCanvas,
    private apply: (reply: Reply) => Promise<void>,
    private status: (text: string, error?: boolean) => void,
    private changed: () => void,
    private select: (target: Target) => void,
    private cancelAction: () => void,
  ) {
    this.text.setAttribute('aria-label', 'Annotation text');
    this.text.placeholder = 'Annotation text';
    this.x.setAttribute('aria-label', 'Annotation X (µm)');
    this.y.setAttribute('aria-label', 'Annotation Y (µm)');
    for (const input of [this.text, this.x, this.y]) {
      if (input !== this.text) input.inputMode = 'numeric';
      input.addEventListener('input', (e) => {
        if ((e as InputEvent).isComposing) return;
        if (input !== this.text) this.pointer = null;
        this.queue();
      });
      input.addEventListener('compositionend', () => this.queue());
      input.addEventListener('keydown', (e) => {
        if (e.isComposing) return;
        if (e.key === 'Enter') {
          e.preventDefault();
          void this.commit();
        }
        if (e.key === 'Escape' && !this.committing) {
          e.preventDefault();
          this.cancelAction();
        }
      });
    }
    this.applyButton.addEventListener('click', () => void this.commit());
    this.cancelButton.addEventListener('click', () => this.cancelAction());
    this.bar.append(
      this.text,
      element('span', '', 'X'),
      this.x,
      element('span', '', 'Y'),
      this.y,
      this.applyButton,
      this.cancelButton,
      element('span', 'move-note', 'µm · Tab cycles snaps · Alt suppresses'),
    );
    shell.placement.append(this.bar);
    this.controls();
  }
  private controls() {
    this.bar.hidden = !this.active;
    this.shell.placement.classList.toggle('annotation-active', this.active);
    this.shell.app
      .querySelector('.tool[data-action="drawing.label.create"]')
      ?.classList.toggle('active', this.active);
    this.shell.app.querySelector('[data-tool="select"]')?.classList.toggle('active', !this.active);
    for (const node of [this.text, this.x, this.y, this.cancelButton])
      node.disabled = this.committing || !this.row;
    this.applyButton.disabled = !this.valid || this.pumping || !!this.pending || this.committing;
    if (this.active) {
      this.shell.views.disabled = true;
      this.shell.inspector.inert = true;
    }
    this.changed();
  }
  async begin(context: Context) {
    this.clear();
    this.active = true;
    this.text.value = '';
    this.x.value = '0';
    this.y.value = '0';
    const generation = this.generation;
    this.controls();
    try {
      const reply = await send({ type: 'begin_annotation', context });
      if (generation !== this.generation) return;
      this.accept(reply);
      this.text.focus();
      this.drawing.movement = {
        point: (x, y, tolerance) => {
          if (this.pointer?.x === x && this.pointer.y === y && this.pointer.tolerance === tolerance)
            return;
          this.pointer = { x, y, tolerance };
          this.queue();
        },
        commit: () => void this.commit(),
      };
    } catch (error) {
      if (generation === this.generation) {
        this.clear();
        this.status(errorText(error), true);
      }
    }
  }
  private queue(candidate: number | null = null) {
    if (!this.active || !this.row) return;
    this.valid = false;
    this.pending = {
      type: 'preview_annotation',
      session: this.row.session,
      placement: {
        text: this.text.value,
        x: this.pointer?.x ?? this.x.value,
        y: this.pointer?.y ?? this.y.value,
        exact: !this.pointer,
        suppressed: this.suppressed,
        candidate,
        tolerance_um: this.pointer?.tolerance ?? 0,
      },
    };
    this.controls();
    void this.pump();
  }
  private async pump() {
    if (this.pumping) return;
    this.pumping = true;
    const generation = this.generation;
    try {
      while (this.pending && generation === this.generation) {
        const request = this.pending;
        this.pending = null;
        try {
          const reply = await send(request);
          if (generation === this.generation && !this.pending) this.accept(reply);
        } catch (error) {
          if (generation === this.generation && !this.pending) {
            this.valid = false;
            this.drawing.clearPreview();
            this.status(errorText(error), true);
          }
        }
      }
    } finally {
      this.pumping = false;
      this.controls();
      if (this.pending) void this.pump();
    }
  }
  private accept(reply: Reply) {
    if (!reply.annotation) throw new Error('Missing annotation proposal');
    this.row = reply.annotation;
    this.valid = this.row.valid;
    if (document.activeElement !== this.x) this.x.value = this.row.x;
    if (document.activeElement !== this.y) this.y.value = this.row.y;
    this.drawing.preview(
      this.row.scene,
      this.row.snap ? [Number(this.row.x) / 1000, Number(this.row.y) / 1000] : undefined,
    );
    this.status(
      !this.row.valid
        ? 'Enter annotation text, then position it'
        : this.row.snap
          ? `${this.row.snap} snap · ${(this.row.chosen ?? 0) + 1}/${this.row.candidates}`
          : 'Annotation preview · Apply or Escape to cancel',
    );
    this.controls();
  }
  async commit() {
    if (!this.active || !this.valid || this.pumping || this.pending || !this.row || this.committing)
      return;
    const target = this.row.scene[0].target;
    this.committing = true;
    this.controls();
    try {
      const reply = await send({
        type: 'commit_annotation',
        token: this.row.token,
        input: inputId(),
      });
      this.clear();
      await this.apply(reply);
      this.select(target);
      this.status('Annotation created · one undo operation');
    } catch (error) {
      this.clear();
      this.status(errorText(error), true);
    }
    this.shell.canvas.focus();
  }
  key(event: KeyboardEvent): boolean {
    if (!this.active) return false;
    if (this.committing) {
      event.preventDefault();
      return true;
    }
    if (event.key === 'Escape') {
      event.preventDefault();
      this.cancelAction();
    }
    if (event.key === 'Enter') {
      event.preventDefault();
      void this.commit();
    }
    if (event.key === 'Tab' && this.pointer) {
      event.preventDefault();
      if (this.valid && this.row?.candidates)
        this.queue(((this.row.chosen ?? -1) + 1) % this.row.candidates);
    }
    if (event.key === 'Alt' && !this.suppressed) {
      this.suppressed = true;
      this.queue();
    }
    return true;
  }
  releaseAlt() {
    if (this.suppressed) {
      this.suppressed = false;
      this.queue();
    }
  }
  clear() {
    const wasActive = this.active;
    this.generation++;
    this.active = false;
    this.row = null;
    this.pending = null;
    this.committing = false;
    this.valid = false;
    this.suppressed = false;
    this.pointer = null;
    if (wasActive) {
      this.drawing.movement = null;
      this.drawing.clearPreview();
      this.shell.views.disabled = false;
      this.shell.inspector.inert = false;
    }
    this.controls();
  }
}
