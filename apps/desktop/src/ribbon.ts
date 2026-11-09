/** Fixed-height grouped action surfaces. Tabs only change workspace presentation. */
import { element, tool } from './ui';
interface Command {
  caption: string;
  label: string;
  icon: string;
  action?: string;
  planned?: string;
  select?: boolean;
}
interface Group {
  title: string;
  commands: Command[];
}
const command = (caption: string, label: string, icon: string, action: string): Command => ({
  caption,
  label,
  icon,
  action,
});
const planned = (caption: string, icon: string): Command => ({
  caption,
  label: `${caption} (planned)`,
  icon,
  planned: 'This management tool is planned',
});
const definitions: { id: string; label: string; groups: Group[] }[] = [
  {
    id: 'file',
    label: 'File',
    groups: [
      {
        title: 'Project',
        commands: [
          command('Open', 'Open project', 'open', 'document.open'),
          command('Save', 'Save project', 'save', 'document.save'),
          command('Information', 'Project properties', 'properties', 'document.metadata.edit'),
          command('Profiles', 'Company / contact profiles', 'properties', 'workspace.profiles'),
        ],
      },
      {
        title: 'History',
        commands: [
          command('Undo', 'Undo', 'undo', 'document.undo'),
          command('Redo', 'Redo', 'redo', 'document.redo'),
        ],
      },
      {
        title: 'Outputs',
        commands: [command('Connections', 'Connection reports', 'report', 'reports.connections')],
      },
      {
        title: 'Commands',
        commands: [command('Search', 'Command palette', 'search', 'workspace.palette')],
      },
      { title: 'Version control · planned', commands: [planned('Git', 'git')] },
    ],
  },
  {
    id: 'drawing',
    label: 'Drawing',
    groups: [
      {
        title: 'Selection',
        commands: [
          {
            caption: 'Select',
            label: 'Select objects · Shift to add',
            icon: 'select',
            select: true,
          },
          command('Properties', 'Properties · E', 'properties', 'selection.properties'),
        ],
      },
      {
        title: 'Create',
        commands: [
          command('Text', 'Create annotation · T', 'text', 'drawing.label.create'),
          command('Path', 'Create graphical path · P', 'path', 'drawing.polyline.create'),
          command('Wire', 'Route conductor · W', 'wire', 'drawing.route.create'),
        ],
      },
      {
        title: 'Transform',
        commands: [
          command('Move', 'Move selection · M', 'move', 'drawing.selection.move'),
          command('Rotate', 'Rotate selection · R', 'rotate', 'drawing.selection.rotate'),
          command('Mirror', 'Mirror selection', 'mirror', 'drawing.selection.mirror'),
        ],
      },
      {
        title: 'Arrange',
        commands: [
          command('Align', 'Align drawing origins', 'align', 'drawing.selection.align'),
          command(
            'Distribute',
            'Distribute drawing origins',
            'distribute',
            'drawing.selection.distribute',
          ),
        ],
      },
      {
        title: 'View / outputs',
        commands: [
          command('Fit', 'Fit drawing · F', 'fit', 'drawing.view.fit'),
          command('Connections', 'Connection reports', 'report', 'reports.connections'),
          command('Defaults', 'Drawing defaults and styles', 'properties', 'drawing.styles.edit'),
          command('Sheets', 'Sheets and paper', 'properties', 'drawing.sheets.edit'),
        ],
      },
    ],
  },
  {
    id: 'library',
    label: 'Library',
    groups: [
      {
        title: 'Drawing templates',
        commands: [command('Templates', 'Drawing templates', 'properties', 'library.templates')],
      },
      {
        title: 'Definitions · planned',
        commands: [
          planned('Symbols', 'symbols'),
          planned('Wires / cables', 'wire'),
          planned('Parts', 'parts'),
        ],
      },
      {
        title: 'Reuse · planned',
        commands: [planned('Shared blocks', 'blocks'), planned('Revisions', 'revisions')],
      },
    ],
  },
  {
    id: 'template',
    label: 'Template',
    groups: [
      {
        title: 'Template',
        commands: [
          command('New', 'New template', 'properties', 'template.new'),
          command('Open', 'Open saved template', 'open', 'template.open'),
          command('Save revision', 'Save template revision', 'save', 'document.save'),
        ],
      },
      {
        title: 'History',
        commands: [
          command('Undo', 'Undo', 'undo', 'document.undo'),
          command('Redo', 'Redo', 'redo', 'document.redo'),
        ],
      },
      {
        title: 'Properties',
        commands: [
          command('Properties', 'Template properties', 'properties', 'selection.properties'),
          command('Artwork', 'Template text and graphics', 'properties', 'template.artwork.edit'),
          command('Revert fields', 'Revert property fields', 'undo', 'template.properties.revert'),
          command('Fit', 'Fit template paper', 'fit', 'drawing.view.fit'),
        ],
      },
      {
        title: 'Draw',
        commands: [
          command('Text', 'Draw template text', 'properties', 'template.draw.text'),
          command('Line', 'Draw template line', 'move', 'template.draw.line'),
          command('Path', 'Draw template path', 'move', 'template.draw.polyline'),
          command('Rectangle', 'Draw template rectangle', 'blocks', 'template.draw.rectangle'),
          command('Ellipse', 'Draw template ellipse', 'blocks', 'template.draw.ellipse'),
        ],
      },
      {
        title: 'Tables',
        commands: [
          command('Resize', 'Resize title-block table', 'properties', 'template.table.resize'),
          command('Split', 'Split title-block cell', 'blocks', 'template.cell.split'),
          command('Merge', 'Merge title-block cells', 'blocks', 'template.cells.merge'),
          command('Boundary', 'Move table boundary', 'move', 'template.table.boundary'),
        ],
      },
      {
        title: 'Project',
        commands: [
          command('Reload library', 'Reload template library', 'open', 'template.library.reload'),
          command('Add saved', 'Add saved revision to project', 'blocks', 'template.import'),
        ],
      },
      {
        title: 'Editor',
        commands: [
          command('Close', 'Close template editor', 'properties', 'template.close'),
          command('Discard', 'Discard template draft', 'undo', 'template.discard'),
        ],
      },
    ],
  },
];
export function createQuickAccess(action: (id: string) => void) {
  const quick = element('div', 'quick-access');
  quick.setAttribute('role', 'toolbar');
  quick.setAttribute('aria-label', 'Quick access');
  for (const [label, name, id] of [
    ['Open project', 'open', 'document.open'],
    ['Save project', 'save', 'document.save'],
    ['Undo', 'undo', 'document.undo'],
    ['Redo', 'redo', 'document.redo'],
  ]) {
    const button = tool(label, name, () => action(id));
    button.dataset.action = id;
    quick.append(button);
  }
  return quick;
}
export function createRibbon(action: (id: string) => void, select: () => void) {
  const root = element('section', 'ribbon');
  root.setAttribute('aria-label', 'Tool ribbons');
  root.dataset.focus = 'workspace';
  const tabs = element('div', 'ribbon-tabs');
  const tabList = element('div', 'ribbon-tablist');
  tabList.setAttribute('role', 'tablist');
  tabList.setAttribute('aria-label', 'Tool ribbons');
  const body = element('div', 'ribbon-body');
  const buttons: HTMLButtonElement[] = [];
  const panels: HTMLElement[] = [];
  let templateContext = false;
  function activate(index: number, focus = false) {
    if (definitions[index].id === 'template' && !templateContext) return;
    buttons.forEach((button, i) => {
      const active = i === index;
      button.setAttribute('aria-selected', String(active));
      button.tabIndex = active ? 0 : -1;
      panels[i].hidden = !active;
    });
    root.dataset.ribbon = definitions[index].id;
    if (focus) buttons[index].focus();
  }
  function control(item: Command) {
    const button = tool(item.label, item.icon, () => {
      if (item.action) {
        action(item.action);
      } else if (item.select) select();
    });
    button.classList.add('ribbon-command');
    button.append(element('span', 'ribbon-caption', item.caption));
    if (item.action) button.dataset.action = item.action;
    if (item.select) {
      button.dataset.tool = 'select';
      button.classList.add('active');
    }
    if (item.planned) {
      button.disabled = true;
      button.dataset.planned = 'true';
      button.title = `${item.label}\n${item.planned}`;
    }
    return button;
  }
  definitions.forEach((definition, index) => {
    const tab = element('button', 'ribbon-tab', definition.label);
    tab.type = 'button';
    tab.hidden = definition.id === 'template';
    tab.id = `ribbon-tab-${definition.id}`;
    tab.setAttribute('role', 'tab');
    tab.setAttribute('aria-controls', `ribbon-panel-${definition.id}`);
    tab.addEventListener('click', () => activate(index));
    tab.addEventListener('keydown', (event) => {
      let next: number | undefined;
      const visible = definitions.map((_, i) => i).filter((i) => !buttons[i].hidden);
      const current = visible.indexOf(index);
      if (event.key === 'ArrowRight') next = visible[(current + 1) % visible.length];
      if (event.key === 'ArrowLeft')
        next = visible[(current + visible.length - 1) % visible.length];
      if (event.key === 'Home') next = 0;
      if (event.key === 'End') next = visible[visible.length - 1];
      if (next !== undefined) {
        event.preventDefault();
        activate(next, true);
      }
    });
    const panel = element('div', 'ribbon-panel');
    panel.id = `ribbon-panel-${definition.id}`;
    panel.dataset.ribbon = definition.id;
    panel.setAttribute('role', 'tabpanel');
    panel.setAttribute('aria-labelledby', tab.id);
    for (const group of definition.groups) {
      const section = element('div', 'ribbon-group');
      section.setAttribute('role', 'group');
      section.setAttribute('aria-label', group.title);
      const commands = element('div', 'ribbon-commands');
      commands.append(...group.commands.map(control));
      section.append(commands, element('div', 'ribbon-group-title', group.title));
      panel.append(section);
    }
    if (definition.id === 'library') {
      const note = element(
        'p',
        'ribbon-note',
        'Drawing templates are available. Other library tools are planned.',
      );
      note.id = 'ribbon-library-note';
      panel.append(note);
      panel.setAttribute('aria-describedby', note.id);
    }
    buttons.push(tab);
    panels.push(panel);
    tabList.append(tab);
    body.append(panel);
  });
  const hint = element(
    'span',
    'ribbon-hint',
    'Shift scroll X · Ctrl scroll Y · Scroll zoom · Drag window / crossing selection',
  );
  tabs.append(tabList, hint);
  root.append(tabs, body);
  activate(1);
  return Object.assign(root, {
    setTemplateContext(enabled: boolean) {
      if (templateContext === enabled) return;
      templateContext = enabled;
      buttons[3].hidden = !enabled;
      activate(enabled ? 3 : 1);
    },
  });
}
