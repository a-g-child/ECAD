/** Clicks are ordered draft intents; pointer-only previews coalesce without losing vertices. */
import { dimension, readDimension, writeDimension } from './dimensions';
import type { Context, PathRow, Reply, Target } from './types';
import type { Shell } from './shell';
import type { DrawingCanvas } from './canvas';
import { pathCommand } from './path-keys';
import { element } from './ui';
import { send, inputId, errorText } from './transport';
interface Placement {
  x: string;
  y: string;
  exact: boolean;
  suppressed: boolean;
  candidate: number | null;
  tolerance_um: number;
}
type Intent = { kind: 'add'; placement: Placement } | { kind: 'back' } | { kind: 'finish' };
export class PathTool {
  active = false;
  private generation = 0;
  private row: PathRow | null = null;
  private valid = false;
  private pumping = false;
  private finishing = false;
  private committing = false;
  private suppressed = false;
  private choice: number | null = null;
  private pointer: { x: string; y: string; tolerance: number } | null = null;
  private pending: Placement | null = null;
  private intents: Intent[] = [];
  private bar = element('div', 'move-bar path-bar moving');
  private x = element('input', 'move-coordinate');
  private y = element('input', 'move-coordinate');
  private count = element('span', 'path-count', '0 points');
  private addButton = element('button', 'plain', 'Add');
  private backButton = element('button', 'plain', 'Back');
  private finishButton = element('button', 'plain', 'Finish');
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
    dimension(this.x, 'Path X (µm)');
    dimension(this.y, 'Path Y (µm)');
    this.addButton.setAttribute('aria-label', 'Add point');
    this.addButton.title = 'Add point · click canvas or press Enter in a coordinate field';
    this.backButton.setAttribute('aria-label', 'Remove last point');
    this.backButton.title = 'Remove last accepted point · Backspace';
    for (const input of [this.x, this.y]) {
      input.inputMode = 'numeric';
      input.addEventListener('input', () => {
        this.pointer = null;
        this.choice = null;
        this.queue();
      });
      input.addEventListener('keydown', (e) => {
        if (e.isComposing) return;
        if (e.key === 'Enter') {
          e.preventDefault();
          this.add();
        }
        if (e.key === 'Escape' && !this.committing) {
          e.preventDefault();
          this.cancelAction();
        }
      });
    }
    this.addButton.addEventListener('click', () => this.add());
    this.backButton.addEventListener('click', () => this.back());
    this.finishButton.addEventListener('click', () => this.finish());
    this.cancelButton.addEventListener('click', () => this.cancelAction());
    this.bar.append(
      this.count,
      element('span', '', 'X'),
      this.x,
      element('span', '', 'Y'),
      this.y,
      this.addButton,
      this.backButton,
      this.finishButton,
      this.cancelButton,
    );
    shell.placement.append(this.bar);
    this.controls();
  }
  private controls() {
    this.bar.hidden = !this.active;
    this.shell.placement.classList.toggle('path-active', this.active);
    this.shell.app
      .querySelector('.tool[data-action="drawing.polyline.create"]')
      ?.classList.toggle('active', this.active);
    this.shell.app.querySelector('[data-tool="select"]')?.classList.toggle('active', !this.active);
    this.count.textContent = `${this.row?.vertices.length ?? 0} points`;
    for (const node of [this.x, this.y, this.addButton, this.backButton, this.finishButton])
      node.disabled = !this.row || this.finishing || this.committing;
    this.finishButton.disabled ||= !this.valid || !this.row?.can_finish;
    this.backButton.disabled ||= !this.row?.vertices.length;
    this.cancelButton.disabled = !this.row || this.committing;
    if (this.active) {
      this.shell.sheets.disabled = true;
      this.shell.inspector.inert = true;
    }
    this.changed();
  }
  async begin(context: Context) {
    this.clear();
    this.active = true;
    writeDimension(this.x, '0');
    writeDimension(this.y, '0');
    const generation = this.generation;
    this.controls();
    try {
      const reply = await send({ type: 'begin_path', context });
      if (generation !== this.generation) return;
      this.accept(reply);
      this.shell.canvas.focus();
      this.drawing.movement = {
        point: (x, y, tolerance) => {
          if (this.pointer?.x === x && this.pointer.y === y && this.pointer.tolerance === tolerance)
            return;
          this.pointer = { x, y, tolerance };
          this.choice = null;
          this.queue();
        },
        commit: () => this.add(),
      };
    } catch (error) {
      if (generation === this.generation) {
        this.clear();
        this.status(errorText(error), true);
      }
    }
  }
  private placement(): Placement {
    return {
      x: this.pointer?.x ?? readDimension(this.x),
      y: this.pointer?.y ?? readDimension(this.y),
      exact: !this.pointer,
      suppressed: this.suppressed,
      candidate: this.choice,
      tolerance_um: this.pointer?.tolerance ?? 0,
    };
  }
  private queue() {
    if (!this.active || !this.row || this.finishing) return;
    this.pending = this.placement();
    this.valid = false;
    this.controls();
    void this.pump();
  }
  private add() {
    if (!this.active || !this.row || this.finishing) return;
    this.intents.push({ kind: 'add', placement: this.placement() });
    this.controls();
    void this.pump();
  }
  private back() {
    if (!this.active || !this.row || this.finishing) return;
    this.intents.push({ kind: 'back' });
    void this.pump();
  }
  private finish() {
    if (!this.active || !this.row || this.finishing) return;
    this.finishing = true;
    this.intents.push({ kind: 'finish' });
    this.controls();
    void this.pump();
  }
  private async preview(placement: Placement, generation: number) {
    const reply = await send({ type: 'preview_path', session: this.row!.session, placement });
    if (generation !== this.generation) return false;
    this.accept(reply);
    return true;
  }
  private async execute(intent: Intent, generation: number) {
    if (intent.kind === 'add') {
      if (!(await this.preview(intent.placement, generation))) return;
      if (!this.row?.can_add) {
        this.status('Choose a different point before adding a vertex', true);
        return;
      }
      const reply = await send({ type: 'add_path_vertex', token: this.row.token });
      if (generation === this.generation) this.accept(reply);
    } else if (intent.kind === 'back') {
      const point = this.row?.vertices.at(-1);
      if (!point) return;
      if (
        !(await this.preview(
          {
            x: point[0],
            y: point[1],
            exact: true,
            suppressed: false,
            candidate: null,
            tolerance_um: 0,
          },
          generation,
        ))
      )
        return;
      const reply = await send({ type: 'back_path_vertex', token: this.row!.token });
      if (generation === this.generation) {
        this.accept(reply);
        if (this.pointer) this.pending = this.placement();
      }
    } else {
      if (!this.valid || !this.row?.can_finish) {
        this.finishing = false;
        this.status('Accept at least two points with valid coordinates before finishing', true);
        this.controls();
        return;
      }
      const target = this.row.scene[0].target;
      this.committing = true;
      this.controls();
      const reply = await send({ type: 'commit_path', token: this.row.token, input: inputId() });
      if (generation !== this.generation) return;
      this.clear();
      await this.apply(reply);
      this.select(target);
      this.status('Graphical path created · one undo operation');
      this.shell.canvas.focus();
    }
  }
  private async pump() {
    if (this.pumping) return;
    this.pumping = true;
    const generation = this.generation;
    try {
      while (
        this.active &&
        generation === this.generation &&
        (this.pending || this.intents.length)
      ) {
        // Before Finish, resolve any remaining pointer proposal so malformed numeric
        // input cannot leave an older accepted-path token silently committable.
        const intent = this.intents[0];
        const resolvePending = this.pending && (!intent || intent.kind === 'finish');
        try {
          if (resolvePending) {
            const placement = this.pending!;
            this.pending = null;
            await this.preview(placement, generation);
          } else {
            const next = this.intents.shift();
            if (next) await this.execute(next, generation);
          }
        } catch (error) {
          if (generation === this.generation) {
            const failedCommit = this.committing;
            this.valid = false;
            this.committing = false;
            this.finishing = false;
            this.drawing.clearPreview();
            if (failedCommit) this.clear();
            this.status(errorText(error), true);
          }
        }
      }
    } finally {
      this.pumping = false;
      this.controls();
      if (this.active && (this.pending || this.intents.length)) void this.pump();
    }
  }
  private accept(reply: Reply) {
    if (!reply.path) throw new Error('Missing path draft');
    this.row = reply.path;
    this.valid = true;
    if (document.activeElement !== this.x) writeDimension(this.x, this.row.x);
    if (document.activeElement !== this.y) writeDimension(this.y, this.row.y);
    this.drawing.pathPreview(
      this.row.scene,
      this.row.rubber,
      this.row.vertices,
      [Number(this.row.x) / 1000, Number(this.row.y) / 1000],
      this.row.snap !== null,
    );
    this.status(
      `${this.row.vertices.length} accepted points${this.row.snap ? ` · ${this.row.snap} snap ${(this.row.chosen ?? 0) + 1}/${this.row.candidates}` : ''} · Click/Add point · Enter finishes · Backspace removes · Escape cancels`,
    );
    this.controls();
  }
  key(event: KeyboardEvent): boolean {
    if (!this.active) return false;
    if (this.committing) {
      event.preventDefault();
      return true;
    }
    const command = pathCommand(event);
    if (command === 'cancel') this.cancelAction();
    if (command === 'finish') this.finish();
    if (command === 'back') this.back();
    if (event.key === 'Tab' && this.pointer) {
      event.preventDefault();
      if (this.valid && this.row?.candidates) {
        this.choice = ((this.row.chosen ?? -1) + 1) % this.row.candidates;
        this.queue();
      }
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
      this.choice = null;
      this.queue();
    }
  }
  clear() {
    const wasActive = this.active;
    this.generation++;
    this.active = false;
    this.row = null;
    this.valid = false;
    this.pending = null;
    this.intents = [];
    this.finishing = false;
    this.committing = false;
    this.suppressed = false;
    this.choice = null;
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
