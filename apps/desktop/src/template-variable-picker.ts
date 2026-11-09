/** Grouped variable selection returns a typed field key; it never resolves project values. */
import { element } from './ui';
const groups: [string, [string, string][]][] = [
  [
    'Company',
    [
      ['company.name', 'Name'],
      ['company.address', 'Address'],
    ],
  ],
  ['Engineer', [['engineer.name', 'Name']]],
  [
    'Project',
    [
      ['project.title', 'Title'],
      ['project.product', 'Product'],
      ['project.client', 'Client'],
      ['project.contract', 'Contract'],
      ['project.issue', 'Issue'],
    ],
  ],
  [
    'Document',
    [
      ['document.title', 'Title'],
      ['document.number', 'Number'],
      ['document.revision', 'Revision'],
      ['document.sheet_count', 'Sheet count'],
    ],
  ],
  [
    'Sheet',
    [
      ['sheet.title', 'Title'],
      ['sheet.number', 'Number'],
    ],
  ],
];
export class TemplateVariablePicker {
  readonly dialog = element('dialog', 'variable-picker');
  private complete: ((key: string | null) => void) | null = null;
  constructor() {
    this.dialog.setAttribute('aria-label', 'Insert variable');
    const heading = element('strong', '', 'Insert variable');
    const tree = element('div', 'variable-tree');
    tree.setAttribute('aria-label', 'Template variables');
    for (const [group, fields] of groups) {
      const branch = element('details');
      branch.open = true;
      branch.append(element('summary', '', group));
      for (const [key, label] of fields) {
        const leaf = element('button', 'variable-leaf', label);
        leaf.type = 'button';
        leaf.title = key;
        leaf.setAttribute('aria-label', key);
        leaf.addEventListener('click', () => this.finish(key));
        branch.append(leaf);
      }
      tree.append(branch);
    }
    const cancel = element('button', 'plain', 'Cancel variable insertion');
    cancel.type = 'button';
    cancel.addEventListener('click', () => this.finish(null));
    this.dialog.append(heading, tree, cancel);
    this.dialog.addEventListener('cancel', (e) => {
      e.preventDefault();
      e.stopPropagation();
      this.finish(null);
    });
    // Picker keys must never complete fields or cancel an enclosing placement tool.
    this.dialog.addEventListener('keydown', (e) => e.stopPropagation());
  }
  open(complete: (key: string | null) => void) {
    if (this.dialog.open) return;
    this.complete = complete;
    this.dialog.showModal();
    this.dialog.querySelector('summary')?.focus();
  }
  private finish(key: string | null) {
    const complete = this.complete;
    this.complete = null;
    this.dialog.close();
    complete?.(key);
  }
}
