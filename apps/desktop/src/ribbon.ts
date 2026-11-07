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
          command('Information', 'Project information', 'properties', 'document.metadata.edit'),
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
  function activate(index: number, focus = false) {
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
    tab.id = `ribbon-tab-${definition.id}`;
    tab.setAttribute('role', 'tab');
    tab.setAttribute('aria-controls', `ribbon-panel-${definition.id}`);
    tab.addEventListener('click', () => activate(index));
    tab.addEventListener('keydown', (event) => {
      let next: number | undefined;
      if (event.key === 'ArrowRight') next = (index + 1) % definitions.length;
      if (event.key === 'ArrowLeft') next = (index + definitions.length - 1) % definitions.length;
      if (event.key === 'Home') next = 0;
      if (event.key === 'End') next = definitions.length - 1;
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
      const note = element('p', 'ribbon-note', 'Library management tools are planned.');
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
  return root;
}
