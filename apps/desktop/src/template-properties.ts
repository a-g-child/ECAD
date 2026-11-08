/** Exact local fields form one closed property proposal; typing never edits a project. */
import type { TemplateVariant, TemplateCell, TemplateTable, TemplateRun } from './types';
import { element } from './ui';
import { textEditor } from './text';
const fields = [
  'company.name',
  'company.address',
  'engineer.name',
  'project.title',
  'project.product',
  'project.client',
  'project.contract',
  'project.issue',
  'document.title',
  'document.number',
  'document.revision',
  'sheet.title',
  'sheet.number',
  'document.sheet_count',
];
export class TemplateProperties {
  readonly root = element('form', 'template-properties inspector');
  private name = textEditor('property-input property-text', 2);
  private paper: HTMLInputElement[] = [];
  private content: HTMLInputElement[] = [];
  private caption = textEditor('property-input property-text', 2);
  private runs: TemplateRun[] = [];
  private runFields = element('div', 'template-run-fields');
  private selected: { table: TemplateTable; cell: TemplateCell } | null = null;
  private model: TemplateVariant | null = null;
  constructor(
    private changed: () => void,
    apply: () => void,
  ) {
    this.root.addEventListener('submit', (e) => e.preventDefault());
    this.root.addEventListener('keydown', (e) => {
      if (e.isComposing) return;
      if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        apply();
      }
    });
  }
  private label(name: string, input: HTMLElement, parent: HTMLElement = this.root) {
    const label = element('label', 'settings-field');
    label.append(element('span', '', name), input);
    parent.append(label);
    input.setAttribute('aria-label', name);
    input.addEventListener('input', this.changed);
    input.addEventListener('change', this.changed);
    return label;
  }
  private numbers(names: string[], values: string[]) {
    return names.map((label, i) => {
      const input = element('input', 'property-input');
      input.inputMode = 'numeric';
      input.value = values[i];
      this.label(label, input);
      return input;
    });
  }
  render(model: TemplateVariant, selection: { table: TemplateTable; cell: TemplateCell } | null) {
    this.model = model;
    this.selected = selection;
    this.root.replaceChildren();
    this.name = textEditor('property-input property-text', 2);
    this.name.value = model.name;
    this.label('Template name', this.name);
    this.paper = this.numbers(
      [
        'Paper width (µm)',
        'Paper height (µm)',
        'Left margin (µm)',
        'Right margin (µm)',
        'Top margin (µm)',
        'Bottom margin (µm)',
      ],
      model.paper,
    );
    this.content = this.numbers(
      ['Content X (µm)', 'Content Y (µm)', 'Content width (µm)', 'Content height (µm)'],
      model.content,
    );
    this.root.append(
      element(
        'p',
        'inspector-note',
        'Custom paper/layout · border standards and anchoring tools are pending. Preview uses placeholders, never personal or project values.',
      ),
    );
    this.runs = [];
    if (selection) {
      this.root.append(element('h3', '', 'Cell properties'));
      this.caption = textEditor('property-input property-text', 2);
      this.caption.value = selection.cell.caption;
      this.label('Cell caption', this.caption);
      this.root.append(
        element(
          'p',
          'inspector-note',
          `Cell ${selection.cell.id} · bounds ${selection.cell.bounds.join(', ')} µm`,
        ),
      );
      this.runs = selection.cell.contents.map((r) => ({ ...r }));
      this.root.append(this.runFields);
      this.renderRuns();
    }
  }
  private renderRuns() {
    this.runFields.replaceChildren();
    this.runs.forEach((run, i) => {
      const type = element('select', 'property-input');
      for (const v of ['literal', 'field']) {
        const option = element('option', '', v === 'literal' ? 'Literal text' : 'Variable');
        option.value = v;
        type.append(option);
      }
      type.value = run.kind;
      const literal = textEditor('property-input property-text', 3);
      literal.value = run.kind === 'literal' ? run.value : '';
      const binding = element('select', 'property-input');
      for (const key of fields) {
        const option = element('option', '', key);
        option.value = key;
        binding.append(option);
      }
      binding.value = run.kind === 'field' ? run.value : fields[0];
      this.label(`Content ${i + 1} type`, type, this.runFields);
      const literalLabel = this.label(`Content ${i + 1} text`, literal, this.runFields);
      const bindingLabel = this.label(`Content ${i + 1} variable`, binding, this.runFields);
      const show = () => {
        literalLabel.hidden = type.value !== 'literal';
        bindingLabel.hidden = type.value !== 'field';
      };
      show();
      type.addEventListener('change', () => {
        run.kind = type.value as TemplateRun['kind'];
        run.value = run.kind === 'literal' ? literal.value : binding.value;
        show();
      });
      literal.addEventListener('input', () => (run.value = literal.value));
      binding.addEventListener('change', () => (run.value = binding.value));
      const remove = element('button', 'plain', `Remove content ${i + 1}`);
      remove.type = 'button';
      remove.addEventListener('click', () => {
        this.runs.splice(i, 1);
        this.renderRuns();
        this.changed();
      });
      this.runFields.append(remove);
    });
    const add = element('button', 'plain', 'Add text content');
    add.type = 'button';
    add.addEventListener('click', () => {
      this.runs.push({ kind: 'literal', value: '' });
      this.renderRuns();
      this.changed();
    });
    this.runFields.append(add);
  }

  proposal() {
    if (!this.model) throw new Error('Template is closed');
    return {
      variant: this.model.id,
      name: this.name.value,
      paper: this.paper.map((v) => v.value),
      content: this.content.map((v) => v.value),
      cell: this.selected
        ? {
            table: this.selected.table.id,
            id: this.selected.cell.id,
            caption: this.caption.value,
            contents: this.runs,
          }
        : null,
    };
  }
  focus() {
    (this.selected ? this.caption : this.name).focus();
  }
}
