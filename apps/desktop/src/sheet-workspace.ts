/** Active sheet and per-sheet cameras are workspace state, never document commands. */
import type { SheetRow, Snapshot } from './types';
import type { Shell } from './shell';
import type { DrawingCanvas } from './canvas';
import { element } from './ui';
export class SheetWorkspace {
  private snapshot: Snapshot | null = null;
  private current = '';
  private layout = '';
  private cameras = new Map<string, { layout: string; box: number[] }>();
  private badge = element('div', 'paper-badge');
  get id() {
    return this.current;
  }
  get sheet() {
    return this.snapshot?.sheets.find((s) => s.id === this.current) ?? null;
  }
  get view() {
    return this.sheet?.view ?? null;
  }
  constructor(
    private shell: Shell,
    private drawing: DrawingCanvas,
    private canSwitch: () => boolean,
    private changed: () => void,
    editSheets: () => void,
  ) {
    shell.canvas.append(this.badge);
    const create = element('button', 'plain', 'Sheets and paper');
    create.dataset.action = 'drawing.sheets.edit';
    create.addEventListener('click', editSheets);
    // The empty-state action is dispatched by the same action surface as the ribbon.
    shell.empty.append(document.createElement('br'), create);
    shell.sheets.addEventListener('change', () => this.show(shell.sheets.value));
  }
  update(snapshot: Snapshot, replaced: boolean) {
    const previous = this.current;
    if (replaced) {
      this.cameras.clear();
      this.current = '';
      this.layout = '';
    }
    this.snapshot = snapshot;
    for (const id of this.cameras.keys())
      if (!snapshot.sheets.some((s) => s.id === id)) this.cameras.delete(id);
    if (!snapshot.sheets.some((s) => s.id === this.current))
      this.current = snapshot.sheets[0]?.id ?? '';
    this.shell.sheets.replaceChildren(
      ...snapshot.sheets.map((s) => {
        const option = element('option', '', `${s.number} · ${s.title}`);
        option.value = s.id;
        return option;
      }),
    );
    this.shell.sheets.value = this.current;
    this.render(replaced || previous !== this.current);
    return previous !== this.current || replaced;
  }
  show(id: string) {
    if (!this.canSwitch() || !this.snapshot?.sheets.some((s) => s.id === id)) {
      this.shell.sheets.value = this.current;
      return false;
    }
    if (id === this.current) return true;
    if (this.current)
      this.cameras.set(this.current, { layout: this.layout, box: this.drawing.camera() });
    this.current = id;
    this.shell.sheets.value = id;
    this.render(true);
    const camera = this.cameras.get(id);
    if (camera?.layout === this.layout) this.drawing.restoreCamera(camera.box);
    this.changed();
    return true;
  }
  showView(view: string) {
    if (this.view === view) return true;
    const sheet = this.snapshot?.sheets.find((s) => s.view === view);
    return !!sheet && this.show(sheet.id);
  }
  private render(fit: boolean) {
    const sheet = this.sheet;
    const frame = this.snapshot?.sheet_frames.find((f) => f.sheet === this.current) ?? null;
    const layout = sheet ? layoutKey(sheet) : '';
    const layoutChanged = layout !== this.layout;
    if (layoutChanged || fit) this.drawing.cancelGesture();
    this.layout = layout;
    this.shell.sheetKind.textContent = !sheet
      ? 'SHEETS'
      : sheet.kind === 'cover'
        ? 'COVER'
        : sheet.kind === 'drawing-list'
          ? 'DRAWING LIST'
          : 'SCHEMATIC';
    this.shell.empty.hidden = !!sheet;
    this.badge.hidden = !sheet;
    const list = this.snapshot?.drawing_lists.find((l) => l.sheet === this.current) ?? null;
    this.drawing.setSheet(sheet, frame, list);
    this.drawing.render(
      this.snapshot?.scene.filter((r) => r.view === sheet?.view) ?? [],
      fit || layoutChanged,
    );
    if (!sheet) return;
    const uses = this.snapshot!.sheets.filter((s) => s.view && s.view === sheet.view).length;
    const messages = [`${sheet.number} · ${sheet.title}`];
    if (uses > 1) messages.push(`Shared drawing on ${uses} sheets · edits appear on all`);
    if (!sheet.view && sheet.kind === 'schematic')
      messages.push('Blank sheet · attach a drawing view in Sheets and paper');
    if (sheet.kind === 'cover') messages.push('Cover sheet');
    if (list)
      messages.push(
        `Drawing list · ${list.ordinal}/${list.page_count}${list.remaining !== '0' ? ` · ${list.remaining} drawings need more list pages` : ''}`,
      );
    if (frame?.title_block_too_small) messages.push('Title block cannot fit');
    const unplaced = this.snapshot!.views.filter(
      (v) => !this.snapshot!.sheets.some((s) => s.view === v.id),
    ).length;
    if (unplaced)
      messages.push(`${unplaced} unplaced drawing view(s) · attach using Sheets and paper`);
    this.badge.textContent = messages.join(' · ');
    this.badge.title = this.badge.textContent;
  }
}
function layoutKey(s: SheetRow) {
  return JSON.stringify([
    s.width,
    s.height,
    s.view,
    s.paper_x,
    s.paper_y,
    s.source_x,
    s.source_y,
    s.numerator,
    s.denominator,
  ]);
}
