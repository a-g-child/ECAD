/** Reviewed pivot-based rotation/reflection; Rust owns every proposed geometry record. */
import type { Context, Reply, TransformRow } from './types';
import type { Shell } from './shell';
import type { DrawingCanvas } from './canvas';
import { element } from './ui';
import { send, inputId, errorText } from './transport';
export class TransformTool {
  active = false;
  private generation = 0;
  private row: TransformRow | null = null;
  private kind = 'rotate';
  private valid = false;
  private pumping = false;
  private committing = false;
  private pending: object | null = null;
  private bar = element('div', 'move-bar transform-bar moving');
  private x = element('input', 'move-coordinate');
  private y = element('input', 'move-coordinate');
  private operation = element('select', 'transform-operation');
  private applyButton = element('button', 'plain', 'Apply');
  private cancelButton = element('button', 'plain', 'Cancel');
  constructor(
    private shell: Shell,
    private drawing: DrawingCanvas,
    private apply: (reply: Reply) => Promise<void>,
    private status: (text: string, error?: boolean) => void,
    private changed: () => void,
    private cancelAction: () => void,
  ) {
    this.x.setAttribute('aria-label', 'Pivot X (µm)');
    this.y.setAttribute('aria-label', 'Pivot Y (µm)');
    this.operation.setAttribute('aria-label', 'Transform operation');
    for (const input of [this.x, this.y]) {
      input.inputMode = 'numeric';
      input.addEventListener('input', () => this.queue());
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
    this.operation.addEventListener('change', () => this.queue());
    this.applyButton.addEventListener('click', () => void this.commit());
    this.cancelButton.addEventListener('click', () => this.cancelAction());
    this.bar.append(
      this.operation,
      element('span', '', 'Pivot X'),
      this.x,
      element('span', '', 'Y'),
      this.y,
      this.applyButton,
      this.cancelButton,
      element('span', 'move-note', 'µm · explicit pivot'),
    );
    shell.placement.append(this.bar);
    this.controls();
  }
  private controls() {
    this.bar.hidden = !this.active;
    this.shell.placement.classList.toggle('transform-active', this.active);
    this.shell.app.querySelector('[data-tool="select"]')?.classList.toggle('active', !this.active);
    for (const name of ['rotate', 'mirror'])
      this.shell.app
        .querySelector(`.tool[data-action="drawing.selection.${name}"]`)
        ?.classList.toggle('active', this.active && this.kind === name);
    for (const node of [this.x, this.y, this.operation, this.cancelButton])
      node.disabled = this.committing || !this.row;
    this.applyButton.disabled = !this.valid || this.pumping || !!this.pending || this.committing;
    if (this.active) {
      this.shell.views.disabled = true;
      this.shell.inspector.inert = true;
    }
    this.changed();
  }
  async begin(context: Context, kind: string) {
    this.clear();
    this.active = true;
    this.kind = kind;
    const generation = this.generation;
    const options =
      kind === 'rotate'
        ? [
            ['rotate_90', '90° clockwise'],
            ['rotate_180', '180°'],
            ['rotate_270', '90° anticlockwise'],
          ]
        : [
            ['mirror_vertical', 'Vertical axis'],
            ['mirror_horizontal', 'Horizontal axis'],
          ];
    this.operation.replaceChildren(
      ...options.map(([value, text]) => {
        const option = element('option', '', text);
        option.value = value;
        return option;
      }),
    );
    this.controls();
    try {
      const reply = await send({
        type: 'begin_transform',
        context,
        reference: context.selection[0],
        kind,
      });
      if (generation !== this.generation) return;
      this.accept(reply);
      this.shell.canvas.focus();
      this.drawing.movement = { point: () => {}, commit: () => void this.commit() };
    } catch (error) {
      if (generation === this.generation) {
        this.clear();
        this.status(errorText(error), true);
      }
    }
  }
  private queue() {
    if (!this.active || !this.row) return;
    this.valid = false;
    this.pending = {
      type: 'preview_transform',
      session: this.row.session,
      x: this.x.value,
      y: this.y.value,
      operation: this.operation.value,
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
    if (!reply.transform) throw new Error('Missing transform preview');
    this.row = reply.transform;
    this.valid = true;
    if (document.activeElement !== this.x) this.x.value = this.row.x;
    if (document.activeElement !== this.y) this.y.value = this.row.y;
    this.drawing.preview(this.row.scene, [Number(this.row.x) / 1000, Number(this.row.y) / 1000]);
    this.status(
      `${this.kind === 'rotate' ? 'Rotation' : 'Mirror'} preview · Apply or Escape to cancel`,
    );
    this.controls();
  }
  async commit() {
    if (!this.active || !this.valid || this.pumping || this.pending || !this.row || this.committing)
      return;
    this.committing = true;
    this.controls();
    try {
      const reply = await send({
        type: 'commit_transform',
        token: this.row.token,
        input: inputId(),
      });
      this.clear();
      await this.apply(reply);
      this.status('Selection transformed · one undo operation');
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
    return true;
  }
  clear() {
    const wasActive = this.active;
    this.generation++;
    this.active = false;
    this.row = null;
    this.valid = false;
    this.committing = false;
    this.pending = null;
    if (wasActive) {
      this.drawing.movement = null;
      this.drawing.clearPreview();
      this.shell.views.disabled = false;
      this.shell.inspector.inert = false;
    }
    this.controls();
  }
}
