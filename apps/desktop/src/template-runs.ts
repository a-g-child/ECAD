/** Shared literal/variable controls for cell and free template text. */
import type { TemplateRun } from './types';
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
export class TemplateRuns {
  readonly runFields = element('div', 'template-run-fields');
  constructor(
    readonly runs: TemplateRun[],
    private changed: () => void,
  ) {
    this.render();
  }
  private label(name: string, input: HTMLElement, parent: HTMLElement) {
    const label = element('label', 'settings-field');
    label.append(element('span', '', name), input);
    parent.append(label);
    input.setAttribute('aria-label', name);
    input.addEventListener('input', this.changed);
    input.addEventListener('change', this.changed);
    return label;
  }
  render() {
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
        this.render();
        this.changed();
      });
      this.runFields.append(remove);
    });
    const add = element('button', 'plain', 'Add text content');
    add.type = 'button';
    add.addEventListener('click', () => {
      this.runs.push({ kind: 'literal', value: '' });
      this.render();
      this.changed();
    });
    this.runFields.append(add);
  }
}
