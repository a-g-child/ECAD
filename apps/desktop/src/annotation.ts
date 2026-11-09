/** Transient new-text placement; the bridge owns identity, validation, defaults and snapping. */
import { dimension, readDimension, writeDimension } from './dimensions';
import type { AnnotationRow, Context, Reply, Target } from './types';
import type { Shell } from './shell';
import type { DrawingCanvas } from './canvas';
import { element } from './ui';
import { textEditor, applyTextKey } from './text';
import { PreviewQueue } from './preview-queue';
import { send, inputId, errorText } from './transport';
export class AnnotationTool {
  active = false;
  private generation = 0;
  private row: AnnotationRow | null = null;
  private previews: PreviewQueue<object, Reply>;
  private committing = false;
  private valid = false;
  private suppressed = false;
  private pointer: { x: string; y: string; tolerance: number } | null = null;
  private bar = element('div', 'move-bar annotation-bar moving');
  private text = textEditor('annotation-text', 2);
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
    this.previews = new PreviewQueue(
      send,
      (reply) => this.accept(reply),
      (error) => {
        this.valid = false;
        this.drawing.clearPreview();
        this.status(errorText(error), true);
      },
      () => this.controls(),
    );
    this.text.setAttribute('aria-label', 'Annotation text');
    this.text.placeholder = 'Annotation text';
    dimension(this.x, 'Annotation X (µm)');
    dimension(this.y, 'Annotation Y (µm)');
    for (const input of [this.text, this.x, this.y]) {
      if (input !== this.text) input.inputMode = 'numeric';
      input.addEventListener('input', (e) => {
        if ((e as InputEvent).isComposing) return;
        if (input !== this.text) this.pointer = null;
        this.queue();
      });
      input.addEventListener('compositionend', () => this.queue());
      input.addEventListener('keydown', (raw) => {
        const e = raw as KeyboardEvent;
        if (e.isComposing) return;
        if (
          (input === this.text && applyTextKey(e)) ||
          (input !== this.text && e.key === 'Enter')
        ) {
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
      element('span', 'move-note', 'Enter: newline · Ctrl/Cmd+Enter: Apply'),
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
    this.applyButton.disabled = !this.valid || this.previews.busy || this.committing;
    if (this.active) {
      this.shell.sheets.disabled = true;
      this.shell.inspector.inert = true;
    }
    this.changed();
  }
  async begin(context: Context) {
    this.clear();
    this.active = true;
    this.text.value = '';
    writeDimension(this.x, '0');
    writeDimension(this.y, '0');
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
    if (!this.active || !this.row || this.committing) return;
    this.valid = false;
    this.previews.propose({
      type: 'preview_annotation',
      session: this.row.session,
      placement: {
        text: this.text.value,
        x: this.pointer?.x ?? readDimension(this.x),
        y: this.pointer?.y ?? readDimension(this.y),
        exact: !this.pointer,
        suppressed: this.suppressed,
        candidate,
        tolerance_um: this.pointer?.tolerance ?? 0,
      },
    });
    this.controls();
  }
  private accept(reply: Reply) {
    if (!reply.annotation) throw new Error('Missing annotation proposal');
    this.row = reply.annotation;
    this.valid = this.row.valid;
    if (document.activeElement !== this.x) writeDimension(this.x, this.row.x);
    if (document.activeElement !== this.y) writeDimension(this.y, this.row.y);
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
    if (!this.active || !this.row || this.committing) return;
    const generation = this.generation;
    this.committing = true;
    this.controls();
    await this.previews.drain();
    if (generation !== this.generation || !this.active) return;
    if (!this.valid || !this.row) {
      this.committing = false;
      this.controls();
      return;
    }
    const target = this.row.scene[0].target;
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
    this.previews.reset();
    this.committing = false;
    this.valid = false;
    this.suppressed = false;
    this.pointer = null;
    if (wasActive) {
      this.drawing.movement = null;
      this.drawing.clearPreview();
      this.shell.sheets.disabled = false;
      this.shell.inspector.inert = false;
    }
    this.controls();
  }
}
