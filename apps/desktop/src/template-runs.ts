/** One multiline editor for cell/free text; inline placeholders map to typed stored runs. */
import type { TemplateRun } from './types';
import { element } from './ui';
import { textEditor } from './text';
import { formatTemplateText, parseTemplateText } from './template-text-codec';
import { TemplateVariablePicker } from './template-variable-picker';
export class TemplateRuns {
  readonly runFields = element('div', 'template-run-fields');
  constructor(
    readonly runs: TemplateRun[],
    changed: () => void,
  ) {
    this.runFields.dataset.variableEditor = '';
    const text = textEditor('property-input property-text template-text-input', 5);
    text.setAttribute('aria-label', 'Text and variables');
    text.title = 'Multiline text with {{field}} variables. Right-click or use Insert variable.';
    text.value = formatTemplateText(runs);
    const initialText = text.value;
    const initialRuns = structuredClone(runs);
    const insert = element('button', 'plain', 'Insert variable');
    insert.type = 'button';
    const label = element('label', 'template-text-label');
    label.append(element('span', '', 'Text and variables'), text);
    const toolbar = element('div', 'template-text-toolbar');
    toolbar.append(insert);
    const error = element('div', 'variable-error');
    error.setAttribute('role', 'alert');
    const picker = new TemplateVariablePicker();
    let composing = false;
    text.addEventListener('compositionstart', () => {
      composing = true;
    });
    text.addEventListener('compositionend', () => {
      composing = false;
    });
    text.addEventListener('input', () => {
      // An unchanged editor must not normalize legacy run boundaries or empty literal runs.
      const next =
        text.value === initialText ? structuredClone(initialRuns) : parseTemplateText(text.value);
      runs.splice(0, runs.length, ...next);
      error.textContent = '';
      changed();
    });
    const open = () => {
      if (text.disabled || composing) return;
      const [start, end, scroll] = [text.selectionStart, text.selectionEnd, text.scrollTop];
      picker.open((key) => {
        if (!text.isConnected || text.disabled) return;
        text.focus({ preventScroll: true });
        text.setSelectionRange(start, end);
        text.scrollTop = scroll;
        if (key === null) return;
        // Native insertion retains the textarea's undo/redo stack, selection and IME behavior.
        // Keep this legacy editing API isolated until webviews offer an equivalent undoable edit.
        if (!document.execCommand('insertText', false, `{{${key}}}`))
          error.textContent = `Could not insert automatically. Type {{${key}}} at the cursor.`;
      });
    };
    insert.addEventListener('click', open);
    text.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      e.stopPropagation();
      open();
    });
    const hint = element(
      'small',
      'template-text-hint',
      'Variables use {{field}}. Use \\{{ for literal braces.',
    );
    this.runFields.append(label, toolbar, hint, error, picker.dialog);
  }
}
