/** Transient placement controls; the engine owns snapping, geometry and commit proposals. */
import { dimension, readDimension, writeDimension } from './dimensions';
import type { Context, MoveRow, Reply } from './types';
import type { Shell } from './shell';
import { DrawingCanvas } from './canvas';
import { element } from './ui';
import type { DragPlacement, PointerPosition } from './canvas-input';
import type { Target } from './types';
import { keyOf } from './types';
import { errorText, inputId, send } from './transport';
export class MovementTool {
  active = false;
  private generation = 0;
  private row: MoveRow | null = null;
  private pending: Record<string, unknown> | null = null;
  private pumping = false;
  private idle: (() => void)[] = [];
  private valid = false;
  private committing = false;
  private suppressed = false;
  private lastPointer: { x: string; y: string; tolerance: number } | null = null;
  private bar = element('div', 'move-bar');
  private x = element('input', 'move-coordinate');
  private y = element('input', 'move-coordinate');
  private reference = element('select', 'move-reference');
  private applyButton = element('button', 'plain', 'Apply');
  private cancelButton = element('button', 'plain', 'Cancel');
  private note = element(
    'span',
    'move-note',
    'M · Move selected labels, paths and terminal anchors',
  );
  private captured: Context | null = null;
  constructor(
    private shell: Shell,
    private drawing: DrawingCanvas,
    private apply: (reply: Reply) => Promise<void>,
    private status: (text: string, error?: boolean) => void,
    private changed: () => void,
    private cancelAction: () => void,
  ) {
    dimension(this.x, 'Move X (µm)');
    dimension(this.y, 'Move Y (µm)');
    this.reference.setAttribute('aria-label', 'Move reference');
    for (const input of [this.x, this.y]) {
      input.type = 'text';
      input.inputMode = 'numeric';
      input.addEventListener('input', () => {
        this.lastPointer = null;
        this.queue({
          x: readDimension(this.x),
          y: readDimension(this.y),
          exact: true,
          suppressed: false,
          candidate: null,
          tolerance_um: 0,
        });
      });
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
    this.reference.addEventListener('change', () => {
      if (this.captured) void this.begin(this.captured, Number(this.reference.value));
    });
    this.applyButton.addEventListener('click', () => void this.commit());
    this.cancelButton.addEventListener('click', () => {
      if (!this.committing) this.cancelAction();
    });
    this.bar.append(
      this.reference,
      element('span', '', 'X'),
      this.x,
      element('span', '', 'Y'),
      this.y,
      this.applyButton,
      this.cancelButton,
      this.note,
    );
    shell.placement.append(this.bar);
    this.controls();
  }
  private controls() {
    this.bar.classList.toggle('moving', this.active);
    this.shell.app.querySelector('[data-tool="select"]')?.classList.toggle('active', !this.active);
    this.shell.app
      .querySelector('.tool[data-action="drawing.selection.move"]')
      ?.classList.toggle('active', this.active);
    for (const node of [this.x, this.y, this.reference, this.applyButton, this.cancelButton])
      node.hidden = !this.active;
    for (const node of [this.x, this.y, this.reference, this.cancelButton])
      node.disabled = this.committing || !this.row;
    this.applyButton.disabled = !this.valid || this.pumping || !!this.pending;
    this.shell.sheets.disabled = this.active;
    this.shell.inspector.inert = this.active;
    this.changed();
  }
  async begin(context: Context, index = 0) {
    if (this.committing) return;
    this.clear();
    this.active = true;
    this.captured = context;
    const generation = this.generation;
    this.reference.replaceChildren(
      ...context.selection.map((target, i) => {
        const option = element('option', '', `${target.kind} ${target.id.slice(-4)}`);
        option.value = String(i);
        option.selected = i === index;
        return option;
      }),
    );
    this.note.textContent =
      'Move · Exact X/Y in the displayed units · Tab cycles snaps · Hold Alt to suppress';
    this.controls();
    try {
      const reply = await send({
        type: 'begin_move',
        context,
        reference: context.selection[index],
      });
      if (generation !== this.generation) return;
      this.accept(reply);
      this.shell.canvas.focus();
      this.drawing.movement = {
        point: (x, y, tolerance) => {
          if (
            this.lastPointer?.x === x &&
            this.lastPointer.y === y &&
            this.lastPointer.tolerance === tolerance
          )
            return;
          this.lastPointer = { x, y, tolerance };
          this.pointer();
        },
        commit: () => void this.commit(),
      };
    } catch (error) {
      if (generation !== this.generation) return;
      this.clear();
      this.status(errorText(error), true);
    }
  }
  /** Drag preserves the grabbed offset; release waits for the final closed preview. */
  async beginDrag(
    context: Context,
    target: Target,
    start: PointerPosition,
  ): Promise<DragPlacement | null> {
    const generation = this.generation;
    this.active = true;
    this.controls();
    try {
      const requested = await send({
        type: 'action',
        action: 'drawing.selection.move',
        context,
        input: inputId(),
      });
      if (generation !== this.generation || requested.request !== 'move') return null;
      await this.begin(
        context,
        context.selection.findIndex((t) => keyOf(t) === keyOf(target)),
      );
      if (!this.active || !this.row) return null;
      const lifetime = this.generation;
      const dx = BigInt(this.row.x) - BigInt(start.x),
        dy = BigInt(this.row.y) - BigInt(start.y);
      const point = (p: PointerPosition) => {
        if (lifetime !== this.generation) return;
        this.drawing.movement?.point(
          String(BigInt(p.x) + dx),
          String(BigInt(p.y) + dy),
          p.tolerance,
        );
      };
      return {
        point,
        finish: async (p) => {
          if (lifetime !== this.generation) return;
          point(p);
          if (this.pumping || this.pending)
            await new Promise<void>((resolve) => this.idle.push(resolve));
          if (lifetime === this.generation) await this.commit();
        },
        cancel: () => {
          if (lifetime === this.generation) this.cancelAction();
        },
      };
    } catch (error) {
      this.clear();
      this.status(errorText(error), true);
      return null;
    }
  }
  private pointer(candidate: number | null = null) {
    if (!this.lastPointer) return;
    const { x, y, tolerance } = this.lastPointer;
    this.queue({
      x,
      y,
      exact: false,
      suppressed: this.suppressed,
      candidate,
      tolerance_um: tolerance,
    });
  }
  private queue(fields: Record<string, unknown>) {
    if (!this.active || !this.row) return;
    this.valid = false;
    this.pending = { type: 'preview_move', session: this.row.session, ...fields };
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
      else {
        for (const resolve of this.idle.splice(0)) resolve();
      }
    }
  }
  private accept(reply: Reply) {
    if (!reply.movement) throw new Error('Missing movement preview');
    this.row = reply.movement;
    this.valid = true;
    // Preserve the user's raw exact input while typing, including large integers.
    if (document.activeElement !== this.x) writeDimension(this.x, this.row.x);
    if (document.activeElement !== this.y) writeDimension(this.y, this.row.y);
    this.drawing.preview(
      this.row.scene,
      this.row.snap ? [Number(this.row.x) / 1000, Number(this.row.y) / 1000] : undefined,
    );
    this.status(
      this.row.snap
        ? `${this.row.snap} snap · ${(this.row.chosen ?? 0) + 1}/${this.row.candidates}`
        : 'Exact placement · Apply or Escape to cancel',
    );
    this.controls();
  }
  async commit() {
    if (!this.active || !this.valid || this.pumping || this.pending || !this.row) return;
    const token = this.row.token;
    this.committing = true;
    this.valid = false;
    this.drawing.movement = null;
    this.controls();
    try {
      const reply = await send({ type: 'commit_move', token, input: inputId() });
      this.clear();
      await this.apply(reply);
      this.status('Selection moved · one undo operation');
    } catch (error) {
      this.clear();
      this.status(errorText(error), true);
    }
    this.shell.canvas.focus();
  }
  /** Called by the central key handler, after protected-input/composition guards. */
  key(event: KeyboardEvent): boolean {
    if (!this.active) return false;
    if (this.committing) {
      event.preventDefault();
      return true;
    }
    if (event.key === 'Escape') {
      event.preventDefault();
      this.cancelAction();
      return true;
    }
    if (event.key === 'Enter') {
      event.preventDefault();
      void this.commit();
      return true;
    }
    if (event.key === 'Tab' && this.lastPointer) {
      event.preventDefault();
      if (this.valid && this.row?.candidates)
        this.pointer(((this.row.chosen ?? -1) + 1) % this.row.candidates);
      return true;
    }
    if (event.key === 'Alt' && !this.suppressed) {
      this.suppressed = true;
      this.pointer();
    }
    return true;
  }
  releaseAlt() {
    if (this.suppressed) {
      this.suppressed = false;
      this.pointer();
    }
  }
  clear() {
    this.generation++;
    this.committing = false;
    this.active = false;
    this.row = null;
    this.valid = false;
    this.pending = null;
    this.captured = null;
    this.lastPointer = null;
    this.suppressed = false;
    this.drawing.movement = null;
    this.drawing.clearPreview();
    this.note.textContent = 'M · Move selected labels, paths and terminal anchors';
    this.controls();
  }
}
