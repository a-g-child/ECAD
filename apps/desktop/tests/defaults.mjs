/** Real default inheritance, persistence, modal input ownership and saved history. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
const primary = process.platform === 'darwin' ? 'Meta' : 'Control';
export function registerDefaultsTests({ page, button, until, bounds, root, project, pressAction }) {
  const dialog = (p) => p.getByRole('dialog', { name: 'Drawing defaults', exact: true });
  const field = (p, name) => dialog(p).getByRole('textbox', { name, exact: true });
  const apply = (p) => dialog(p).getByRole('button', { name: 'Apply defaults', exact: true });
  async function open(p) {
    await button(p, 'Drawing defaults and styles').click();
    await dialog(p).waitFor();
  }
  async function ready(p) {
    await until(async () => !(await button(p, 'Save project').isDisabled()));
  }
  test('drawing defaults validate atomically, retain overrides and panes, and save with one-step history', async () => {
    const p = await page();
    const original = JSON.parse(await readFile(project(), 'utf8'));
    const before = await bounds(p);
    await open(p);
    assert.deepEqual(await bounds(p), before);
    const originalFont = await field(p, 'Default font family').inputValue();
    await field(p, 'Default font family').fill('monospace\nExample family');
    await field(p, 'Default text height (µm)').fill('0');
    await apply(p).click();
    await until(async () => (await dialog(p).getByRole('alert').innerText()).length > 0);
    assert.equal(await dialog(p).isVisible(), true);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(await bounds(p), before);
    assert.equal(await field(p, 'Default font family').inputValue(), 'monospace\nExample family');
    await field(p, 'Default font family').fill('monospace');
    await field(p, 'Default text height (µm)').fill('4500');
    await field(p, 'Default stroke width (µm)').fill('1250');
    await field(p, 'Default output colour').fill('#abCD12');
    await p.setViewportSize({ width: 960, height: 640 });
    const rect = await dialog(p).boundingBox();
    assert.ok(
      rect.x >= 0 && rect.x + rect.width <= 960 && rect.y >= 0 && rect.y + rect.height <= 640,
    );
    await p.screenshot({ path: resolve(root, 'target/ui/drawing-defaults.png') });
    await apply(p).click();
    await dialog(p).waitFor({ state: 'hidden' });
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Drawing defaults updated · one undo operation',
    );
    await p.setViewportSize({ width: 1360, height: 880 });
    assert.deepEqual(await bounds(p), before);
    const annotation = p.locator('svg [data-object="label:00000000000000000000000000000001"] text');
    assert.equal(await annotation.getAttribute('font-family'), 'monospace');
    assert.equal(await annotation.getAttribute('fill'), '#abcd12');
    assert.equal(await annotation.getAttribute('font-size'), '3.5');
    await pressAction(p, 'Save project', 'Project saved');
    const saved = JSON.parse(await readFile(project(), 'utf8'));
    assert.deepEqual(saved.drawing.settings.base, {
      font: 'monospace',
      text_height_um: 4500,
      stroke_width_um: 1250,
      colour: [171, 205, 18],
    });
    saved.drawing.settings.base = original.drawing.settings.base;
    assert.deepEqual(saved, original);
    await pressAction(p, 'Undo', 'Undo complete');
    await open(p);
    assert.equal(await field(p, 'Default font family').inputValue(), originalFont);
    await dialog(p).getByRole('button', { name: 'Cancel', exact: true }).click();
    await pressAction(p, 'Redo', 'Redo complete');
    assert.doesNotMatch(await p.title(), /^●/);
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Save project', 'Project saved');
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('defaults shortcut and palette share the dialog; cancel and unchanged Apply retain redo and protect typing', async () => {
    const p = await page();
    const before = await bounds(p);
    const original = await readFile(project(), 'utf8');
    await p.locator('.canvas').focus();
    await p.keyboard.press(`${primary}+k`);
    await p.waitForFunction(() =>
      document.querySelector('.status-message')?.textContent?.includes('next shortcut'),
    );
    await p.keyboard.press('s');
    await dialog(p).waitFor();
    await field(p, 'Default font family').fill('Temporary');
    await field(p, 'Default font family').press('Enter');
    assert.equal(await dialog(p).isVisible(), true);
    assert.match(await field(p, 'Default font family').inputValue(), /\n/);
    await p.keyboard.press('Escape');
    await dialog(p).waitFor({ state: 'hidden' });
    assert.deepEqual(await bounds(p), before);
    assert.doesNotMatch(await p.title(), /^●/);
    await p.keyboard.press(`${primary}+Shift+p`);
    await p.getByRole('textbox', { name: 'Search commands' }).fill('Drawing defaults');
    await p.locator('.palette-item').filter({ hasText: 'Drawing defaults and styles' }).click();
    await dialog(p).waitFor();
    await apply(p).click();
    await dialog(p).waitFor({ state: 'hidden' });
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Drawing defaults already match · no document changes',
    );
    await ready(p);
    assert.equal(await button(p, 'Redo').isDisabled(), false);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.equal(await readFile(project(), 'utf8'), original);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
