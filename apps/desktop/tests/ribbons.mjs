/** Ribbons reorganize existing action surfaces without becoming document state. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
const primary = process.platform === 'darwin' ? 'Meta' : 'Control';
const label = 'label:00000000000000000000000000000001';
export function registerRibbonTests({ page, tree, button, input, until, bounds, root, project }) {
  const tab = (p, name) => p.getByRole('tab', { name, exact: true });
  const panel = (p, name) => p.getByRole('tabpanel', { name, exact: true });
  async function ready(p) {
    await until(
      async () =>
        !(await p.locator('.quick-access button[data-action="document.save"]').isDisabled()),
    );
  }
  async function fileAction(p, name, message) {
    await panel(p, 'File').getByRole('button', { name, exact: true }).click();
    await p.waitForFunction(
      (message) => document.querySelector('.status-message')?.textContent === message,
      message,
    );
    await ready(p);
  }
  test('File/Drawing/Library tabs retain bounds and selection, support keyboard navigation, and mark library tools as planned', async () => {
    const p = await page();
    await tree(p, label).click();
    await input(p, 'drawing_text').waitFor();
    const original = await readFile(project(), 'utf8'),
      before = await bounds(p),
      viewBox = await p.locator('svg.drawing').getAttribute('viewBox');
    assert.equal(await tab(p, 'Drawing').getAttribute('aria-selected'), 'true');
    assert.equal((await p.locator('.ribbon').boundingBox()).height, 96);
    for (const name of ['File', 'Library', 'Drawing']) {
      await tab(p, name).click();
      assert.equal(await panel(p, name).isVisible(), true);
      assert.equal(await p.locator('.ribbon-panel:not([hidden])').count(), 1);
      assert.deepEqual(await bounds(p), before);
      assert.equal(await p.locator('svg.drawing').getAttribute('viewBox'), viewBox);
      assert.equal(await p.locator(`svg [data-object="${label}"].selected`).count(), 1);
      assert.equal(await input(p, 'drawing_text').inputValue(), 'CONVEYOR / SIGNAL HARNESS');
    }
    await tab(p, 'Library').click();
    const library = panel(p, 'Library');
    assert.match(await library.innerText(), /planned/);
    assert.equal(await library.locator('button:not([disabled])').count(), 0);
    assert.equal(await library.locator('button[data-action]').count(), 0);
    assert.equal(
      await library.getByRole('button', { name: 'Symbols (planned)', exact: true }).isDisabled(),
      true,
    );
    await p.screenshot({ path: resolve(root, 'target/ui/ribbon-library.png') });
    await tab(p, 'Library').press('ArrowRight');
    assert.equal(await tab(p, 'File').getAttribute('aria-selected'), 'true');
    await tab(p, 'File').press('End');
    assert.equal(await tab(p, 'Library').getAttribute('aria-selected'), 'true');
    await tab(p, 'Library').press('Home');
    assert.equal(await tab(p, 'File').getAttribute('aria-selected'), 'true');
    await tab(p, 'File').press('ArrowRight');
    assert.equal(await tab(p, 'Drawing').getAttribute('aria-selected'), 'true');
    assert.equal(await tab(p, 'Drawing').evaluate((n) => n === document.activeElement), true);
    assert.equal(await p.locator('[role="tab"][tabindex="0"]').count(), 1);
    await tab(p, 'Drawing').press('p');
    assert.equal(
      await p.getByRole('textbox', { name: 'Path X (µm)', exact: true }).isVisible(),
      false,
    );
    await p.setViewportSize({ width: 960, height: 680 });
    const narrow = await bounds(p);
    for (const name of ['File', 'Library', 'Drawing']) {
      await tab(p, name).click();
      assert.deepEqual(await bounds(p), narrow);
      assert.equal(
        await p.evaluate(() => document.documentElement.scrollWidth > innerWidth),
        false,
      );
    }
    await p.setViewportSize({ width: 1360, height: 880 });
    await p.screenshot({ path: resolve(root, 'target/ui/ribbon-drawing.png') });
    assert.equal(await readFile(project(), 'utf8'), original);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('ribbon commands use real document history, preserve active drafts across tabs and target drawing actions from palette', async () => {
    const p = await page();
    await tree(p, label).click();
    await input(p, 'drawing_text').waitFor();
    const before = await bounds(p);
    // Switching tabs gives the ribbon focus; explicit drawing commands still target the canvas.
    await tab(p, 'File').click();
    await tab(p, 'Drawing').click();
    await panel(p, 'Drawing')
      .getByRole('button', { name: 'Rotate selection · R', exact: true })
      .click();
    const pivot = p.getByRole('textbox', { name: 'Pivot X (µm)', exact: true });
    await pivot.waitFor();
    await pivot.fill('10000');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    const preview = await p.locator(`svg [data-object="${label}"] text.ink`).getAttribute('x');
    await tab(p, 'Library').click();
    assert.equal(await pivot.inputValue(), '10000');
    assert.equal(
      await p.locator(`svg [data-object="${label}"] text.ink`).getAttribute('x'),
      preview,
    );
    assert.deepEqual(await bounds(p), before);
    await tab(p, 'File').click();
    assert.equal(
      await panel(p, 'File')
        .getByRole('button', { name: 'Save project', exact: true })
        .isDisabled(),
      true,
    );
    await pivot.press('Escape');
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'Transform cancelled',
    );
    await ready(p);
    await fileAction(p, 'Save project', 'Project saved');
    await p.screenshot({ path: resolve(root, 'target/ui/ribbon-file.png') });
    await panel(p, 'File').getByRole('button', { name: 'Command palette', exact: true }).click();
    await p.getByRole('textbox', { name: 'Search commands', exact: true }).fill('Mirror');
    await p.locator('.palette-results button').filter({ hasText: 'Mirror' }).click();
    await pivot.waitFor();
    await pivot.press('Escape');
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'Transform cancelled',
    );
    await ready(p);
    await tab(p, 'Drawing').click();
    await input(p, 'drawing_text').fill('Ribbon edit');
    await input(p, 'drawing_text').press(`${primary}+Enter`);
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Property updated · one undo operation',
    );
    await tab(p, 'File').click();
    await fileAction(p, 'Undo', 'Undo complete');
    assert.equal(await input(p, 'drawing_text').inputValue(), 'CONVEYOR / SIGNAL HARNESS');
    await fileAction(p, 'Redo', 'Redo complete');
    assert.equal(await input(p, 'drawing_text').inputValue(), 'Ribbon edit');
    await fileAction(p, 'Undo', 'Undo complete');
    assert.deepEqual(await bounds(p), before);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
