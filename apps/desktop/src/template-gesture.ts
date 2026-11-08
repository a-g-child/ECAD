/** One coalesced Rust preview stream; pointer release commits the latest closed proposal only. */
import type { Context, Reply, TemplateTable } from './types';
import { send, inputId, errorText } from './transport';
export class TemplateGesture {
  private state: {
    context: Context;
    variant: string;
    table: string;
    action: string;
    latest: object | null;
    token: string | null;
    attempted: boolean;
  } | null = null;
  private running: Promise<void> | null = null;
  private ending = false;
  get active() {
    return this.state !== null;
  }
  constructor(
    private context: () => Context,
    private display: (tables: TemplateTable[] | null, invalid: boolean) => void,
    private apply: (r: Reply) => Promise<void>,
    private changed: () => void,
    private status: (s: string, error?: boolean) => void,
  ) {}
  begin(variant: string, table: string, action: string) {
    if (this.active) return false;
    this.state = {
      context: structuredClone(this.context()),
      variant,
      table,
      action,
      latest: null,
      token: null,
      attempted: false,
    };
    this.ending = false;
    this.changed();
    return true;
  }
  propose(edit: object) {
    if (!this.state || this.ending) return;
    this.state.latest = edit;
    this.state.token = null;
    if (!this.running)
      this.running = this.pump().finally(() => {
        this.running = null;
      });
  }
  private async pump() {
    const state = this.state;
    if (!state) return;
    while (state.latest) {
      const edit = state.latest;
      state.latest = null;
      state.attempted = true;
      try {
        const reply = await send({
          type: 'template_table_preview',
          context: state.context,
          action: state.action,
          values: { variant: state.variant, table: state.table, edit },
          input: inputId(),
        });
        if (!reply.table_preview) throw new Error('Missing table preview');
        // A newer queued pointer coordinate owns the next preview; do not flash an obsolete reply.
        if (!state.latest) {
          state.token = reply.table_preview.token;
          this.display(reply.table_preview.tables, false);
        }
      } catch (e) {
        state.token = null;
        if (!state.latest) {
          this.display(null, true);
          this.status(errorText(e), true);
        }
      }
    }
  }
  async finish(commit: boolean) {
    const state = this.state;
    if (!state || this.ending) return;
    this.ending = true;
    try {
      await this.running;
      if (commit && state.token) {
        await this.apply(
          await send({
            type: 'template_table_commit',
            context: state.context,
            preview: state.token,
            input: inputId(),
          }),
        );
        this.status('Table updated · one undo operation');
      } else if (state.attempted) {
        await this.cancel(state.context);
        if (!commit) this.status('Table gesture cancelled');
      }
    } catch (e) {
      this.status(errorText(e), true);
      if (state.attempted) await this.cancel(state.context).catch(() => undefined);
    } finally {
      this.state = null;
      this.ending = false;
      this.display(null, false);
      this.changed();
    }
  }
  private async cancel(context: Context) {
    await this.apply(
      await send({ type: 'action', context, action: 'placement.cancel', input: inputId() }),
    );
  }
}
