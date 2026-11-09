/** Read-only Rust eligibility queries; stale responses never enable a newer selection. */
import type { Context } from './types';
import { send } from './transport';
export class TemplateActionState {
  private key = '';
  private states: Record<string, boolean> = {};
  constructor(private changed: () => void) {}
  enabled(id: string, fallback: boolean) {
    return ['template.cell.split', 'template.cells.merge'].includes(id)
      ? (this.states[id] ?? false)
      : fallback;
  }
  refresh(context: Context, pending: boolean) {
    const query = { ...context, focus: 'canvas' };
    const key = JSON.stringify([query, pending]);
    if (this.key === key) return;
    this.key = key;
    this.states = {};
    this.changed();
    if (pending) return;
    void send({ type: 'template_actions', context: query })
      .then((reply) => {
        if (this.key !== key) return;
        this.states = reply.template_actions ?? {};
        this.changed();
      })
      .catch(() => {
        /* Keep unavailable; a new revision/selection will query again. */
      });
  }
}
