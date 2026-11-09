/** Inline notation and variable picking preserve literal identity, caret and document boundaries. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { resolve } from 'node:path';
import { insertVariable } from './template-variable-helpers.mjs';
const primary = process.platform === 'darwin' ? 'Meta' : 'Control';
export function registerInlineVariables({ page, button, until, root }) {
  test('inline variable notation preserves literal lookalikes, adjacent runs and unknown typed bindings', async () => {
    const p = await page({ electrical: false });
    const result = await p.evaluate(async () => {
      const { formatTemplateText, parseTemplateText } = await import('/src/template-text-codec.ts');
      const { TemplateRuns } = await import('/src/template-runs.ts');
      const source = [
        { kind: 'literal', value: '{' },
        { kind: 'literal', value: '{project.title}}\\path\n' },
        { kind: 'field', value: 'sheet.title' },
        { kind: 'literal', value: '' },
      ];
      const before = JSON.stringify(source);
      const editor = new TemplateRuns(source, () => {});
      editor.runFields.querySelector('textarea').dispatchEvent(new Event('input'));
      return {
        encoded: formatTemplateText(source),
        decoded: parseTemplateText(formatTemplateText(source)),
        preserved: JSON.stringify(source) === before,
        unknown: parseTemplateText('A{{unknown.field}}B'),
        partial: parseTemplateText('Text {{unfinished'),
      };
    });
    assert.equal(result.encoded, '\\{{project.title}}\\\\path\n{{sheet.title}}');
    assert.deepEqual(result.decoded, [
      { kind: 'literal', value: '{{project.title}}\\path\n' },
      { kind: 'field', value: 'sheet.title' },
    ]);
    assert.equal(result.preserved, true);
    assert.deepEqual(result.unknown, [
      { kind: 'literal', value: 'A' },
      { kind: 'field', value: 'unknown.field' },
      { kind: 'literal', value: 'B' },
    ]);
    assert.deepEqual(result.partial, [{ kind: 'literal', value: 'Text {{unfinished' }]);
    await p.close();
  });
  test('variable picker inserts at the caret, replaces selection, supports local undo and cancels independently', async () => {
    const p = await page({ electrical: false });
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    const editor = p.getByRole('region', { name: 'Drawing template editor', exact: true });
    const ribbon = p.getByRole('tabpanel', { name: 'Template', exact: true });
    const command = (name) => ribbon.getByRole('button', { name, exact: true }).click();
    const ready = () =>
      until(
        async () =>
          !(await ribbon
            .getByRole('button', { name: 'Draw template text', exact: true })
            .isDisabled()),
      );
    await command('New template');
    await ready();
    await command('Draw template text');
    const svg = editor.locator('.template-preview');
    await until(async () => (await svg.getAttribute('data-artwork-tool')) === 'text');
    const q = await svg.evaluate((s) => {
      const p = new DOMPoint(25, 35).matrixTransform(s.getScreenCTM());
      return { x: p.x, y: p.y };
    });
    await p.mouse.click(q.x, q.y);
    const inline = editor.getByRole('region', { name: 'Place template text', exact: true });
    const text = inline.getByRole('textbox', { name: 'Text and variables', exact: true });
    const picker = inline.getByRole('dialog', { name: 'Insert variable', exact: true });
    let commits = 0,
      lastRuns = null;
    await p.route('**/bridge', async (route) => {
      const request = route.request().postDataJSON();
      if (request.type === 'template_geometry_commit') commits++;
      if (request.type === 'template_geometry_preview' && request.values.edit.contents)
        lastRuns = request.values.edit.contents;
      await route.continue();
    });
    const original = 'Before REPLACE after\nSecond line';
    await text.fill(original);
    await text.evaluate((n) => n.setSelectionRange(7, 14));
    await inline.getByRole('button', { name: 'Insert variable', exact: true }).click();
    await p.screenshot({ path: resolve(root, 'target/ui/inline-variable-picker.png') });
    await picker.locator('summary').first().press('Escape');
    await picker.waitFor({ state: 'hidden' });
    assert.equal(await inline.isVisible(), true);
    assert.equal(await text.inputValue(), original);
    assert.deepEqual(await text.evaluate((n) => [n.selectionStart, n.selectionEnd]), [7, 14]);
    await insertVariable(inline, 'project.title');
    const expected = 'Before {{project.title}} after\nSecond line';
    assert.equal(await text.inputValue(), expected);
    await text.press(`${primary}+z`);
    assert.equal(await text.inputValue(), original);
    await text.press(`${primary}+Shift+z`);
    assert.equal(await text.inputValue(), expected);
    await text.click({ button: 'right' });
    await picker.waitFor();
    await picker.getByRole('button', { name: 'Cancel variable insertion', exact: true }).click();
    assert.equal(await text.inputValue(), expected);
    await text.evaluate((n) => n.setSelectionRange(n.value.length, n.value.length));
    await text.dispatchEvent('contextmenu');
    await picker.getByRole('button', { name: 'sheet.number', exact: true }).click();
    assert.equal(await text.inputValue(), expected + '{{sheet.number}}');
    await until(async () => lastRuns?.length === 4);
    assert.deepEqual(lastRuns, [
      { kind: 'literal', value: 'Before ' },
      { kind: 'field', value: 'project.title' },
      { kind: 'literal', value: ' after\nSecond line' },
      { kind: 'field', value: 'sheet.number' },
    ]);
    assert.equal(commits, 0);
    await text.dispatchEvent('compositionstart');
    await inline.getByRole('button', { name: 'Insert variable', exact: true }).click();
    assert.equal(await picker.isVisible(), false);
    await text.dispatchEvent('compositionend');
    await text.fill('{{unknown.field}}');
    await inline.getByRole('button', { name: 'Place text', exact: true }).click();
    await until(async () => (await inline.getByRole('alert').innerText()).includes('Enter text'));
    assert.equal(commits, 0);
    await text.fill(expected + '{{sheet.number}}');
    await inline.getByRole('button', { name: 'Place text', exact: true }).click();
    await inline.waitFor({ state: 'hidden' });
    await ready();
    assert.equal(commits, 1);
    assert.equal(
      await editor.getByRole('textbox', { name: 'Text and variables', exact: true }).inputValue(),
      expected + '{{sheet.number}}',
    );
    await command('Undo');
    await ready();
    assert.equal(await editor.locator('[data-template-graphic]').count(), 0);
    await command('Discard template draft');
    await editor.waitFor({ state: 'hidden' });
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
