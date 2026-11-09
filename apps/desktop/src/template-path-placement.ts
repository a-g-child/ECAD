/** Ordered local click intents submit complete paths; hover is only a visual rubber segment. */
import { pathCommand } from './path-keys';
type Point = { x: number; y: number };
export class TemplatePathPlacement {
  private vertices: Point[] = [];
  private pointer: Point | null = null;
  private pending = false;
  private closed = false;
  constructor(
    private svg: SVGSVGElement,
    private propose: (edit: object) => void,
    private finish: () => Promise<boolean>,
    private cancel: () => Promise<void>,
    private status: (message: string, error?: boolean) => void,
  ) {
    this.note();
  }
  add(point: Point) {
    if (this.pending || this.closed) return;
    if (this.vertices.length >= 4096) {
      this.status('A path supports at most 4096 vertices.', true);
      return;
    }
    const last = this.vertices.at(-1);
    if (last?.x === point.x && last.y === point.y) {
      this.status('Choose a different point before adding a vertex.', true);
      return;
    }
    this.vertices.push(point);
    this.pointer = point;
    this.update();
  }
  hover(point: Point) {
    if (this.pending || this.closed) return;
    this.pointer = point;
    this.paint();
  }
  key(event: KeyboardEvent) {
    const command = pathCommand(event);
    if (this.pending || this.closed) return;
    if (command === 'cancel') void this.cancel();
    else if (command === 'finish') void this.commit();
    else if (command === 'back') {
      this.vertices.pop();
      this.update();
    }
  }
  private update() {
    this.propose(
      this.vertices.length < 2
        ? { start: null, end: null }
        : {
            points: this.vertices.map((p) => [String(p.x), String(p.y)]),
          },
    );
    this.note();
    this.paint();
  }
  private note() {
    this.status(
      `${this.vertices.length} points · Click adds · Enter finishes clicked points · Backspace removes · Escape cancels`,
    );
  }
  private async commit() {
    if (this.vertices.length < 2) {
      this.status('Choose at least two distinct points before finishing.', true);
      return;
    }
    this.update();
    this.pending = true;
    try {
      if (!(await this.finish()))
        this.status('The path is invalid. Backspace removes its last point; Escape cancels.', true);
    } finally {
      this.pending = false;
    }
  }
  paint() {
    this.svg.querySelector('[data-template-path-draft]')?.remove();
    if (this.closed) return;
    const ns = 'http://www.w3.org/2000/svg';
    const group = document.createElementNS(ns, 'g');
    group.setAttribute('data-template-path-draft', String(this.vertices.length));
    group.setAttribute('pointer-events', 'none');
    const radius = 3 / Math.max(0.000001, Math.abs(this.svg.getScreenCTM()?.a ?? 1));
    for (const p of this.vertices) {
      const dot = document.createElementNS(ns, 'circle');
      dot.setAttribute('cx', String(p.x / 1000));
      dot.setAttribute('cy', String(p.y / 1000));
      dot.setAttribute('r', String(radius));
      dot.setAttribute('fill', '#2383bf');
      group.append(dot);
    }
    const last = this.vertices.at(-1);
    if (last && this.pointer) {
      const rubber = document.createElementNS(ns, 'line');
      for (const [key, value] of Object.entries({
        x1: last.x / 1000,
        y1: last.y / 1000,
        x2: this.pointer.x / 1000,
        y2: this.pointer.y / 1000,
      }))
        rubber.setAttribute(key, String(value));
      rubber.setAttribute('stroke', '#2383bf');
      rubber.setAttribute('stroke-width', '1');
      rubber.setAttribute('stroke-dasharray', '4 3');
      rubber.setAttribute('vector-effect', 'non-scaling-stroke');
      group.append(rubber);
    }
    this.svg.append(group);
  }
  close() {
    this.closed = true;
    this.svg.querySelector('[data-template-path-draft]')?.remove();
  }
}
