/** Reversible editor notation; persisted content remains typed literal/field runs. */
import type { TemplateRun } from './types';
/** Escape literal opening delimiters/backslashes, including across adjacent literal runs. */
export function formatTemplateText(runs: readonly TemplateRun[]): string {
  let text = '',
    literal = '';
  const flush = () => {
    text += literal.replaceAll('\\', '\\\\').replaceAll('{{', '\\{{');
    literal = '';
  };
  for (const run of runs) {
    if (run.kind === 'literal') literal += run.value;
    else {
      flush();
      text += `{{${run.value}}}`;
    }
  }
  flush();
  return text;
}
/** Unknown complete tokens stay typed fields for core validation, never silently become literals. */
export function parseTemplateText(text: string): TemplateRun[] {
  const runs: TemplateRun[] = [];
  let literal = '';
  const flush = () => {
    if (literal) runs.push({ kind: 'literal', value: literal });
    literal = '';
  };
  for (let i = 0; i < text.length;) {
    if (text.startsWith('\\\\', i)) {
      literal += '\\';
      i += 2;
    } else if (text.startsWith('\\{{', i)) {
      literal += '{{';
      i += 3;
    } else if (text.startsWith('{{', i) && text.indexOf('}}', i + 2) >= 0) {
      const end = text.indexOf('}}', i + 2);
      flush();
      runs.push({ kind: 'field', value: text.slice(i + 2, end) });
      i = end + 2;
    } else {
      literal += text[i];
      i++;
    }
  }
  flush();
  return runs;
}
