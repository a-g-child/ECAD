import './style.css';
import { createShell } from './shell';
import { DrawingCanvas } from './canvas';
import { MovementTool } from './movement';
import { LayoutTool } from './layout';
import { DefaultsDialog } from './defaults';
import { MetadataDialog } from './metadata';
import { ProfilesDialog } from './profiles';
import { SheetsDialog, sheetKindForRequest } from './sheets';
import { SheetWorkspace } from './sheet-workspace';
import { AnnotationTool } from './annotation';
import { PathTool } from './path';
import { renderProperties, renderReport } from './panels';
import { ProjectMenu } from './project-menu';
import { ProjectExplorer } from './project-explorer';
import { element } from './ui';
import { chooseFile, errorText, inputId, native, send, syncInput } from './transport';
import { keyOf, type Context, type Reply, type Snapshot, type Target } from './types';
const shell = createShell((id) => void action(id));
let snapshot: Snapshot | null = null;
let objectViews = new Map<string, string | null>();
let selection: Target[] = [];
let view: string | null = null;
let selectionSequence = 0;
let busy = false;
let pendingSequence = false;
let composing = false;
let focus = 'canvas';
let paletteContext: Context | null = null;
const drawing = new DrawingCanvas(shell, choose, selectMany);
let movement: MovementTool | undefined;
let layout: LayoutTool | undefined;
let annotation: AnnotationTool | undefined;
let path: PathTool | undefined;
movement = new MovementTool(shell, drawing, apply, status, buttons, () => {
  movement?.clear();
  void action('placement.cancel');
});
layout = new LayoutTool(shell, drawing, apply, status, buttons, () => {
  layout?.clear();
  void action('placement.cancel');
});
annotation = new AnnotationTool(
  shell,
  drawing,
  apply,
  status,
  buttons,
  (target) => choose(target, false),
  () => {
    annotation?.clear();
    void action('placement.cancel');
  },
);
path = new PathTool(
  shell,
  drawing,
  apply,
  status,
  buttons,
  (target) => choose(target, false),
  () => {
    path?.clear();
    void action('placement.cancel');
  },
);
const defaults = new DefaultsDialog(shell, apply, status);
const metadata = new MetadataDialog(shell, apply, status);
const profiles = new ProfilesDialog(shell, apply, status);
const sheets = new SheetsDialog(shell, apply, status);
const paper = new SheetWorkspace(
  shell,
  drawing,
  () => !busy && !toolActive() && !drawing.gestureActive,
  () => {
    view = paper.view;
    selection = [];
    selectionChanged();
  },
  () => void action('drawing.sheets.edit'),
);
drawing.navigateSheet = (id) => {
  paper.show(id);
};
const projectMenu = new ProjectMenu(shell, (id, captured) => void action(id, captured));
const explorer = new ProjectExplorer(
  shell,
  choose,
  (id) => paper.show(id),
  (id) => {
    if (paper.show(id)) void action('drawing.sheets.edit');
  },
  (id) => void action(id),
  (event) => projectMenu.open(event, context()),
);
function toolActive() {
  return !!(movement?.active || layout?.active || annotation?.active || path?.active);
}
function clearTools() {
  movement?.clear();
  layout?.clear();
  annotation?.clear();
  path?.clear();
}
drawing.startMove = async (target, point) => {
  if (!snapshot || busy || toolActive()) return null;
  if (selection.some((t) => !['anchor', 'label', 'polyline'].includes(t.kind))) {
    status(
      'Drag terminal anchors to move connected routes. Labels and graphical paths can be dragged directly.',
      true,
    );
    return null;
  }
  return movement?.beginDrag(context(), target, point) ?? null;
};
shell.connection.textContent = native ? 'Desktop · Rust' : 'Browser harness · Rust';
function status(message: string, error = false) {
  shell.statusText.textContent = message;
  shell.statusText.title = message;
  shell.statusText.classList.toggle('error', error);
}
function context(): Context {
  if (!snapshot) throw new Error('The document is not ready');
  return {
    revision: snapshot.revision,
    selection: [...selection],
    view,
    sheet: paper.id || null,
    focus: !paper.id && focus === 'canvas' ? 'workspace' : focus,
  };
}
const layoutActions = new Set([
  'drawing.selection.move',
  'drawing.selection.rotate',
  'drawing.selection.mirror',
  'drawing.selection.align',
  'drawing.selection.distribute',
]);
function availableAction(id: string, enabled: boolean) {
  if (!enabled || (id === 'document.open' && !native)) return false;
  if (
    !view &&
    id.startsWith('drawing.') &&
    !['drawing.sheets.edit', 'drawing.styles.edit', 'drawing.view.fit'].includes(id) &&
    !id.startsWith('drawing.sheet.new_')
  )
    return false;
  if (id === 'drawing.view.fit' && !paper.id) return false;
  if (id.startsWith('workspace.explorer.') && drawing.gestureActive) return false;
  if (id === 'selection.properties') return selection.length > 0;
  if (layoutActions.has(id))
    return (
      selection.length >= (id === 'drawing.selection.distribute' ? 3 : 1) &&
      selection.every(
        (t) =>
          ['anchor', 'label', 'polyline'].includes(t.kind) && objectViews.get(keyOf(t)) === view,
      )
    );
  return true;
}
function buttons() {
  if (snapshot) explorer.setLocked(busy || toolActive() || drawing.gestureActive);
  shell.sheets.disabled = busy || toolActive();
  for (const button of document.querySelectorAll<HTMLButtonElement>('button[data-action]')) {
    const id = button.dataset.action!;
    const definition = snapshot?.actions.find((a) => a.id === id);
    button.disabled = busy || toolActive() || !availableAction(id, definition?.enabled ?? false);
    const label = button.dataset.tooltip ?? button.title;
    button.dataset.tooltip = label;
    const reasons: Record<string, string> = {
      'Tool UI is planned': 'This tool is planned',
      NothingToUndo: 'No changes to undo',
      NothingToRedo: 'No changes to redo',
    };
    button.title = [label, !definition?.enabled ? reasons[definition?.reason ?? ''] : undefined]
      .filter(Boolean)
      .join('\n');
  }
}
async function apply(reply: Reply) {
  if (reply.snapshot) {
    const next = reply.snapshot;
    const changed =
      snapshot &&
      (snapshot.revision.epoch !== next.revision.epoch ||
        snapshot.revision.sequence !== next.revision.sequence);
    if (changed && !reply.report && !shell.reportPane.hidden) {
      const note = element(
        'p',
        'inspector-note',
        'The document changed. Refresh the connection table to show current values.',
      );
      const refresh = element('button', 'plain', 'Refresh connection table');
      refresh.addEventListener('click', () => void action('reports.connections'));
      shell.reportBody.replaceChildren(note, refresh);
    }
    syncInput(next.next_input);
    const replaced = snapshot?.revision.epoch !== next.revision.epoch;
    snapshot = next;
    if (changed) {
      clearTools();
    }
    if (replaced) {
      selection = [];
      selectionSequence++;
    }
    const liveTargets = new Map(next.objects.map((row) => [keyOf(row.target), row.view]));
    objectViews = liveTargets;
    selection = selection.filter((target) => liveTargets.has(keyOf(target)));
    const sheetChanged = paper.update(next, replaced);
    view = paper.view;
    selection = sheetChanged
      ? []
      : selection.filter((t) => !objectViews.get(keyOf(t)) || objectViews.get(keyOf(t)) === view);
    shell.title.textContent = `${next.name}${next.dirty ? '  •' : ''}`;
    shell.title.title = next.path ?? 'Built-in demonstration. Save as to create your own project.';
    document.title = `${next.dirty ? '● ' : ''}${next.name} — ECAD`;
    drawing.setSelection(selection);
    shell.selectionStatus.textContent = selection.length
      ? `${selection.length} selected`
      : 'No selection';
    explorer.update(next, selection, paper.id, replaced);
    buttons();
    await inspect();
  }
  if (reply.properties) renderProperties(shell, reply.properties, selection.length, edit);
  if (reply.report) renderReport(shell, reply.report);
  if (reply.message) status(reply.message);
  if (reply.request === 'path') await path?.begin(context());
  if (reply.request === 'annotation') await annotation?.begin(context());
  if (reply.request === 'move') await movement?.begin(context());
  if (reply.request && ['rotate', 'mirror', 'align', 'distribute'].includes(reply.request))
    await layout?.begin(context(), reply.request);
  if (reply.request === 'cancel_move') {
    clearTools();
    shell.canvas.focus();
  }
  if (reply.request === 'fit') {
    drawing.fit();
  }
  if (reply.activate_sheet) paper.show(reply.activate_sheet);
  const newKind = sheetKindForRequest(reply.request);
  if (newKind && snapshot) await sheets.open(context(), snapshot, '', newKind);
  if (reply.request === 'sheets' && snapshot) await sheets.open(context(), snapshot, paper.id);
  if (reply.request === 'explorer_pages') explorer.show('pages');
  if (reply.request === 'explorer_electrical') explorer.show('electrical');
  if (reply.request === 'palette') openPalette();
  if (reply.request === 'profiles' && reply.profiles) profiles.open(context(), reply.profiles);
  if (reply.request === 'metadata' && reply.metadata && reply.project_templates)
    metadata.open(context(), reply.metadata, reply.project_templates);
  if (reply.request === 'defaults' && reply.defaults) defaults.open(context(), reply.defaults);
  if (reply.request === 'save_as' || reply.request === 'open') {
    const result = await chooseFile(reply.request === 'save_as');
    if (result) await apply(result);
  }
}
async function inspect() {
  if (!snapshot) return;
  const serial = ++selectionSequence;
  const captured = context();
  const reply = await send({ type: 'inspect', context: captured });
  if (serial === selectionSequence && reply.properties)
    renderProperties(shell, reply.properties, selection.length, edit);
}
function choose(target: Target | null, add: boolean) {
  if (!snapshot || busy || toolActive()) return;
  if (!target) {
    if (!add) selection = [];
  } else {
    const row = snapshot.objects.find((r) => keyOf(r.target) === keyOf(target));
    if (row?.view && !paper.showView(row.view)) {
      status('This view has no sheet. Use Sheets and paper to review and attach it.', true);
      return;
    }
    const existing = selection.some((t) => keyOf(t) === keyOf(target));
    selection = add
      ? existing
        ? selection.filter((t) => keyOf(t) !== keyOf(target))
        : [...selection, target]
      : [target];
  }
  selectionChanged();
}
function selectMany(targets: Target[], add: boolean) {
  if (!snapshot || busy || toolActive()) return;
  selection = [
    ...new Map(
      (add ? [...selection, ...targets] : targets).map((target) => [keyOf(target), target]),
    ).values(),
  ];
  selectionChanged();
}
function selectionChanged() {
  if (!snapshot) return;
  drawing.setSelection(selection);
  explorer.update(snapshot, selection, paper.id);
  shell.selectionStatus.textContent = selection.length
    ? `${selection.length} selected`
    : 'No selection';
  buttons();
  void inspect().catch((error) => status(errorText(error), true));
}
async function run(operation: () => Promise<void>) {
  if (busy) return;
  busy = true;
  buttons();
  try {
    await operation();
  } catch (error) {
    if (
      error &&
      typeof error === 'object' &&
      'code' in error &&
      ['stale', 'replayed'].includes(String(error.code))
    ) {
      try {
        await apply(await send({ type: 'snapshot' }));
      } catch {
        /* Preserve the original failure below. */
      }
    }
    status(errorText(error), true);
  } finally {
    busy = false;
    buttons();
  }
}
async function action(id: string, captured?: Context) {
  if (!snapshot || (toolActive() && id !== 'placement.cancel')) return;
  const target = captured ?? context();
  // Explicit menus/ribbons/palette actions target the drawing while retaining
  // the captured revision, view and selection. Keyboard routing keeps its focus rules.
  if (!availableAction(id, true)) return;
  const drawingAction = id.startsWith('drawing.');
  if (drawingAction) shell.canvas.focus();
  const invocationContext = drawingAction ? { ...target, focus: 'canvas' } : target;
  await run(async () => {
    const reply = await send({
      type: 'action',
      context: invocationContext,
      action: id,
      input: inputId(),
    });
    await apply(reply);
    if (!reply.message && !reply.request)
      status(
        id === 'document.undo'
          ? 'Undo complete'
          : id === 'document.redo'
            ? 'Redo complete'
            : 'Ready',
      );
  });
}
async function edit(property: string, value: string) {
  if (!snapshot || toolActive()) return;
  const captured = context();
  await run(async () => {
    const reply = await send({ type: 'edit', context: captured, property, value });
    await apply(reply);
    shell.canvas.focus();
    status('Property updated · one undo operation');
  });
}
function openPalette() {
  paletteContext = context();
  shell.search.value = '';
  paletteResults();
  shell.palette.showModal();
  shell.search.focus();
}
function paletteResults() {
  shell.results.replaceChildren();
  for (const definition of snapshot?.actions.filter((a) =>
    `${a.label} ${a.id}`.toLowerCase().includes(shell.search.value.toLowerCase()),
  ) ?? []) {
    const button = element('button', 'palette-item');
    button.disabled = !availableAction(definition.id, definition.enabled);
    button.append(
      element('span', '', definition.label),
      element('kbd', '', definition.shortcuts.join(' / ')),
    );
    button.title = definition.reason ?? '';
    button.addEventListener('click', () => {
      shell.palette.close();
      void action(definition.id, paletteContext ?? undefined);
    });
    shell.results.append(button);
  }
}
shell.search.addEventListener('input', paletteResults);
document.addEventListener('pointerdown', (event) => {
  const target = event.target as HTMLElement;
  const surface = target.closest<HTMLElement>('[data-focus]');
  if (surface && !target.matches('input,textarea,[contenteditable="true"]'))
    focus = surface.dataset.focus ?? focus;
});
document.addEventListener('focusin', (event) => {
  const target = event.target as HTMLElement;
  focus = target.matches('input,textarea,[contenteditable="true"]')
    ? 'text'
    : (target.closest<HTMLElement>('[data-focus]')?.dataset.focus ?? focus);
});
document.addEventListener('compositionstart', () => (composing = true));
document.addEventListener('compositionend', () => (composing = false));
document.addEventListener('keydown', (event) => {
  const target = event.target as HTMLElement;
  if (
    !snapshot ||
    busy ||
    composing ||
    event.isComposing ||
    target.matches('input,textarea,select,button,[contenteditable="true"]') ||
    shell.palette.open ||
    defaults.active ||
    metadata.active ||
    profiles.active ||
    sheets.active ||
    projectMenu.active
  )
    return;
  if (event.key === 'Escape' && drawing.cancelGesture()) {
    event.preventDefault();
    return;
  }
  if (path?.key(event)) return;
  if (annotation?.key(event)) return;
  if (layout?.key(event)) return;
  if (movement?.key(event)) return;
  const primary = navigator.platform.toLowerCase().includes('mac') ? event.metaKey : event.ctrlKey;
  const key = event.key.toLowerCase();
  const relevant = pendingSequence || event.key.length === 1 || key === 'escape';
  if (!relevant) return;
  if (
    pendingSequence ||
    (primary && ['s', 'z', 'o', 'p', 'k'].includes(key)) ||
    (!event.ctrlKey &&
      !event.metaKey &&
      !event.altKey &&
      ['e', 'f', 'w', 't', 'p', 'm', 'r', 'escape'].includes(key))
  )
    event.preventDefault();
  const captured = context();
  void run(async () => {
    const reply = await send({
      type: 'key',
      context: captured,
      key: event.key,
      control: event.ctrlKey,
      meta: event.metaKey,
      alt: event.altKey,
      shift: event.shiftKey,
      repeat: event.repeat,
      composing: event.isComposing,
      input: inputId(),
      now_ms: Math.floor(performance.now()),
    });
    pendingSequence = reply.request === 'key_prefix';
    await apply(reply);
  });
});
document.addEventListener('keyup', (event) => {
  if (event.key === 'Alt') {
    movement?.releaseAlt();
    annotation?.releaseAlt();
    path?.releaseAlt();
  }
});
window.addEventListener('blur', () => {
  movement?.releaseAlt();
  annotation?.releaseAlt();
  path?.releaseAlt();
});
shell.canvas.addEventListener('contextmenu', (event) => {
  event.preventDefault();
  const target = event.target as Element;
  void action(target.closest('[data-object]') ? 'selection.properties' : 'drawing.sheets.edit');
});
if (native) {
  void import('@tauri-apps/api/event').then(({ listen }) =>
    listen<string>('ecad:close-blocked', (event) => status(event.payload, true)),
  );
}
void run(async () => {
  await apply(await send({ type: 'snapshot' }));
  status('Ready · electrical identities are shared across drawing views');
  shell.canvas.focus();
});
