/** Latest pointer preview wins; requests stay serial and accepted clicks can await the final result. */
export class PreviewQueue<Request, Result> {
  private generation = 0;
  private pending: { request: Request; sequence: number; generation: number } | null = null;
  private sequence = 0;
  private running: Promise<void> | null = null;
  get busy() {
    return this.running !== null || this.pending !== null;
  }
  constructor(
    private request: (request: Request) => Promise<Result>,
    private accept: (result: Result) => void,
    private reject: (error: unknown) => void,
    private changed: () => void = () => {},
  ) {}
  propose(request: Request) {
    this.pending = { request, sequence: ++this.sequence, generation: this.generation };
    this.start();
  }
  /** Discard queued work and suppress every response belonging to the previous tool session. */
  reset() {
    this.generation++;
    this.pending = null;
  }
  /** Drain the latest proposal, including one queued while an older preview was in flight. */
  async drain() {
    while (this.running) await this.running;
  }
  private start() {
    if (this.running) return;
    // Install ownership before invoking callbacks, which may themselves refresh controls.
    this.running = Promise.resolve()
      .then(() => this.pump())
      .finally(() => {
        this.running = null;
        if (this.pending) this.start();
        this.changed();
      });
    this.changed();
  }
  private async pump() {
    while (this.pending) {
      const next = this.pending;
      this.pending = null;
      const current = () => next.generation === this.generation && next.sequence === this.sequence;
      try {
        const result = await this.request(next.request);
        if (current()) this.accept(result);
      } catch (error) {
        if (current()) this.reject(error);
      }
    }
  }
}
