/** The shell owns every pane boundary. Content scrolls inside its own grid cell. */
import { element, icon } from './ui';
import { createQuickAccess, createRibbon } from './ribbon';
export function createShell(action: (id: string) => void) {
  const app = document.querySelector<HTMLDivElement>('#app')!;
  const menu = element('header', 'menubar');
  const brand = element('div', 'brand');
  brand.append(icon('properties'), element('strong', '', 'ECAD'));
  menu.append(brand, createQuickAccess(action));
  const menus = [
    [
      'File',
      [
        ['Open project…', 'document.open'],
        ['Save', 'document.save'],
      ],
    ],
    [
      'Edit',
      [
        ['Undo', 'document.undo'],
        ['Redo', 'document.redo'],
        ['Properties', 'selection.properties'],
        ['Create annotation', 'drawing.label.create'],
        ['Create graphical path', 'drawing.polyline.create'],
        ['Move selection', 'drawing.selection.move'],
        ['Rotate selection', 'drawing.selection.rotate'],
        ['Mirror selection', 'drawing.selection.mirror'],
      ],
    ],
    [
      'View',
      [
        ['Fit drawing', 'drawing.view.fit'],
        ['Command palette', 'workspace.palette'],
      ],
    ],
    ['Reports', [['Connection table', 'reports.connections']]],
  ] as const;
  for (const [label, entries] of menus) {
    const details = element('details', 'menu');
    const summary = element('summary', '', label);
    const popup = element('div', 'menu-popup');
    for (const [text, id] of entries) {
      const button = element('button', 'menu-entry', text);
      button.dataset.action = id;
      button.addEventListener('click', () => {
        details.open = false;
        action(id);
      });
      popup.append(button);
    }
    details.append(summary, popup);
    menu.append(details);
  }
  const title = element('div', 'document-title');
  menu.append(title);
  const ribbon = createRibbon(action, () => canvas.focus());
  const workspace = element('main', 'workspace');
  const left = element('aside', 'pane project-pane');
  left.dataset.focus = 'tree';
  left.setAttribute('aria-label', 'Project explorer');
  const leftHeader = element('div', 'pane-header');
  leftHeader.append(element('strong', '', 'Project'), element('span', 'pane-tag', 'ELECTRICAL'));
  const tree = element('div', 'tree');
  left.append(leftHeader, tree);
  const center = element('section', 'drawing-pane');
  const tabs = element('div', 'drawing-tabs');
  const views = element('select', 'view-selector');
  views.setAttribute('aria-label', 'Drawing view');
  tabs.append(views, element('span', 'pane-tag', 'SCHEMATIC'));
  const canvas = element('div', 'canvas');
  canvas.tabIndex = 0;
  canvas.dataset.focus = 'canvas';
  canvas.setAttribute('aria-label', 'Drawing canvas');
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.classList.add('drawing');
  svg.setAttribute('role', 'img');
  svg.setAttribute('aria-label', 'Electrical drawing');
  canvas.append(svg);
  const empty = element(
    'div',
    'canvas-empty',
    'This project has no drawing views. Electrical objects remain available in the explorer.',
  );
  canvas.append(empty);
  const reportPane = element('section', 'report-pane');
  reportPane.hidden = true;
  const reportHeader = element('div', 'pane-header');
  reportHeader.append(element('strong', '', 'Connection table'));
  const close = element('button', 'plain', 'Close');
  close.addEventListener('click', () => {
    reportPane.hidden = true;
  });
  reportHeader.append(close);
  const reportBody = element('div', 'report-body');
  reportPane.append(reportHeader, reportBody);
  const placement = element('div', 'placement-pane');
  center.append(tabs, placement, canvas, reportPane);
  const right = element('aside', 'pane inspector-pane');
  right.dataset.focus = 'inspector';
  right.setAttribute('aria-label', 'Properties');
  const inspectorHeader = element('div', 'pane-header');
  const inspectorCount = element('span', 'pane-tag', 'NO SELECTION');
  inspectorHeader.append(element('strong', '', 'Properties'), inspectorCount);
  const inspector = element('div', 'inspector');
  right.append(inspectorHeader, inspector);
  workspace.append(left, center, right);
  const status = element('footer', 'statusbar');
  const statusText = element('span', 'status-message', 'Connecting to the document engine…');
  statusText.setAttribute('role', 'status');
  const selectionStatus = element('span', 'status-cell', 'No selection');
  const zoom = element('span', 'status-cell', '100%');
  const units = element('span', 'status-cell', 'mm');
  const connection = element('span', 'status-cell', 'Rust engine');
  status.append(statusText, selectionStatus, zoom, units, connection);
  const palette = element('dialog', 'palette');
  const search = element('input', 'palette-search');
  search.placeholder = 'Search commands…';
  search.setAttribute('aria-label', 'Search commands');
  const results = element('div', 'palette-results');
  palette.append(search, results);
  app.append(menu, ribbon, workspace, status, palette);
  return {
    app,
    title,
    tree,
    views,
    canvas,
    placement,
    svg,
    empty,
    inspector,
    inspectorCount,
    statusText,
    selectionStatus,
    zoom,
    connection,
    palette,
    search,
    results,
    reportPane,
    reportBody,
  };
}
export type Shell = ReturnType<typeof createShell>;
