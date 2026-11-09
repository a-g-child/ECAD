/** One coalesced Rust preview stream; pointer release commits the latest closed proposal only. */
import type { Context, Reply, TemplateTable, TemplateGraphic } from './types';
import { PreviewQueue } from './preview-queue';
import { send, inputId, errorText } from './transport';
export class TemplateGesture {
  private state: {
    context: Context;
    variant: string;
    table: string;
    action: string;
    token: string | null;
    attempted: boolean;
  } | null = null;
  private queue: PreviewQueue<object, Reply>;
  private ending = false;
  get active() {
    return this.state !== null;
  }
  constructor(
    private context: () => Context,
    private display: (
      tables: TemplateTable[] | null,
      invalid: boolean,
      snapped: string[],
      graphics?: TemplateGraphic[] | null,
    ) => void,
    private apply: (r: Reply) => Promise<void>,
    private changed: () => void,
    private status: (s: string, error?: boolean) => void,
    private artwork = false,
  ) {
    this.queue = new PreviewQueue(
      (request) => send({ ...request, input: inputId() }),
      (reply) => {
        if (!reply.table_preview) throw new Error('Missing table preview');
        if (!this.state) return;
        this.state.token = reply.table_preview.token;
        this.display(
          reply.table_preview.tables,
          false,
          reply.table_preview.snapped,
          reply.table_preview.token ? reply.table_preview.graphics : null,
        );
      },
      (error) => {
        if (this.state) this.state.token = null;
        this.display(null, true, []);
        this.status(errorText(error), true);
      },
    );
  }
  begin(variant: string, table: string, action: string) {
    if (this.active) return false;
    this.state = {
      context: structuredClone(this.context()),
      variant,
      table,
      action,
      token: null,
      attempted: false,
    };
    this.ending = false;
    this.changed();
    return true;
  }
  propose(edit: object) {
    if (!this.state || this.ending) return;
    const state = this.state;
    state.token = null;
    state.attempted = true;
    this.queue.propose({
      type: this.artwork ? 'template_geometry_preview' : 'template_table_preview',
      context: state.context,
      action: state.action,
      values: { variant: state.variant, [this.artwork ? 'graphic' : 'table']: state.table, edit },
    });
  }
  async finish(commit: boolean) {
    const state = this.state;
    if (!state || this.ending) return;
    this.ending = true;
    try {
      await this.queue.drain();
      if (commit && state.token) {
        await this.apply(
          await send({
            type: this.artwork ? 'template_geometry_commit' : 'template_table_commit',
            context: state.context,
            preview: state.token,
            input: inputId(),
          }),
        );
        this.status(`${this.artwork ? 'Artwork' : 'Table'} updated · one undo operation`);
      } else if (state.attempted) {
        await this.cancel(state.context);
        if (!commit) this.status('Template gesture cancelled');
      }
    } catch (e) {
      this.status(errorText(e), true);
      if (state.attempted) await this.cancel(state.context).catch(() => undefined);
    } finally {
      this.queue.reset();
      this.state = null;
      this.ending = false;
      this.display(null, false, []);
      this.changed();
    }
  }
  private async cancel(context: Context) {
    await this.apply(
      await send({ type: 'action', context, action: 'placement.cancel', input: inputId() }),
    );
  }
}
