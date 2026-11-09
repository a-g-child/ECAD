import { insertVariable } from './template-variable-helpers.mjs';
/** Real Rust table partitions, exact history and local immutable publication through Chromium. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
export function registerTemplateTableTests({ page, button, until, project, root }) {
  const editor = (p) => p.getByRole('region', { name: 'Drawing template editor', exact: true });
  const ribbon = (p) => p.getByRole('tabpanel', { name: 'Template', exact: true });
  const cells = (p) => editor(p).locator('.template-structure [data-template-cell]');
  const dialog = (p, name) => p.getByRole('dialog', { name, exact: true });
  async function open(p) {
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor(p).waitFor();
    await ribbon(p).getByRole('button', { name: 'New template', exact: true }).click();
    // An already-open saved template can also have eight cells. Wait for the new draft,
    // not merely the old projection's count, before capturing its stable identities.
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'New template draft',
    );
    await until(async () => (await cells(p).count()) === 8);
  }
  async function range(p, first, last) {
    const rects = editor(p).locator('.frame-cell > rect');
    const point = (i) =>
      rects.nth(i).evaluate((n) => {
        const r = n.getBoundingClientRect();
        return { x: r.x + r.width / 2, y: r.y + r.height / 2 };
      });
    const a = await point(first),
      b = await point(last);
    await p.mouse.move(a.x, a.y);
    await p.mouse.down();
    await p.mouse.move(b.x, b.y, { steps: 4 });
    await p.mouse.up();
  }
  async function tool(p, name) {
    await ribbon(p).getByRole('button', { name, exact: true }).click();
    const d = dialog(p, name);
    await d.waitFor();
    return d;
  }
  async function apply(d) {
    await d.getByRole('button', { name: 'Apply table edit', exact: true }).click();
    await d.waitFor({ state: 'hidden' });
  }
  async function command(p, name) {
    await ribbon(p).getByRole('button', { name, exact: true }).click();
    if (name === 'Close template editor' || name === 'Discard template draft') return;
    await until(
      async () =>
        !(await ribbon(p)
          .getByRole('button', { name: 'Draw template text', exact: true })
          .isDisabled()),
    );
  }
  test('structured table tools split and merge real cells, resize shared edges and preserve exact undo through publication', async () => {
    const p = await page({ electrical: false });
    const original = await readFile(project(), 'utf8');
    await open(p);
    const ids = await cells(p).evaluateAll((nodes) => nodes.map((n) => n.dataset.templateCell));
    const firstContents = await editor(p)
      .locator('.frame-value')
      .first()
      .getAttribute('aria-label');
    await cells(p).first().click();
    let d = await tool(p, 'Split title-block cell');
    await d.getByRole('spinbutton', { name: 'Columns', exact: true }).fill('0');
    await d.getByRole('button', { name: 'Apply table edit', exact: true }).click();
    await until(async () =>
      (await d.getByRole('alert').innerText()).includes('between 2 and 4096'),
    );
    assert.equal(
      await d.getByRole('spinbutton', { name: 'Columns', exact: true }).inputValue(),
      '0',
    );
    await d.getByRole('spinbutton', { name: 'Columns', exact: true }).fill('3');
    await d.getByRole('spinbutton', { name: 'Rows', exact: true }).fill('2');
    await apply(d);
    assert.equal(await cells(p).count(), 13);
    const splitIds = await cells(p).evaluateAll((nodes) =>
      nodes.map((n) => n.dataset.templateCell),
    );
    assert.equal(splitIds[0], ids[0]);
    await command(p, 'Undo');
    assert.deepEqual(
      await cells(p).evaluateAll((nodes) => nodes.map((n) => n.dataset.templateCell)),
      ids,
    );
    await command(p, 'Redo');
    assert.deepEqual(
      await cells(p).evaluateAll((nodes) => nodes.map((n) => n.dataset.templateCell)),
      splitIds,
    );
    await cells(p).nth(1).click();
    await editor(p)
      .getByRole('textbox', { name: 'Cell caption', exact: true })
      .fill('Custom field');
    await editor(p).getByRole('button', { name: 'Apply properties', exact: true }).click();
    await until(
      async () =>
        !(await ribbon(p)
          .getByRole('button', { name: 'Draw template text', exact: true })
          .isDisabled()),
    );
    await editor(p)
      .getByRole('textbox', { name: 'Text and variables', exact: true })
      .fill('New cell\nMultiline text');
    await insertVariable(editor(p), 'sheet.title');
    await editor(p).getByRole('button', { name: 'Apply properties', exact: true }).click();
    await until(
      async () =>
        !(await ribbon(p)
          .getByRole('button', { name: 'Draw template text', exact: true })
          .isDisabled()),
    );
    // Keep the first cell as survivor while reviewing newly populated split cells.
    await range(p, 0, 5);
    d = await tool(p, 'Merge title-block cells');
    assert.equal(await d.getByRole('checkbox').count(), 0);
    assert.equal(
      await d.getByRole('combobox', { name: 'Merged contents', exact: true }).inputValue(),
      'keep_primary',
    );
    await apply(d);
    assert.equal(
      await editor(p).locator('.frame-value').first().getAttribute('aria-label'),
      firstContents,
    );
    await command(p, 'Undo');
    assert.deepEqual(
      await cells(p).evaluateAll((nodes) => nodes.map((n) => n.dataset.templateCell)),
      splitIds,
    );
    await range(p, 5, 0); // Reverse selection still keeps spatial reading order.
    d = await tool(p, 'Merge title-block cells');
    await d.getByRole('combobox', { name: 'Merged contents', exact: true }).selectOption('combine');
    await apply(d);
    assert.deepEqual(
      await cells(p).evaluateAll((nodes) => nodes.map((n) => n.dataset.templateCell)),
      ids,
    );
    assert.match(
      await editor(p).locator('.frame-value').first().getAttribute('aria-label'),
      /New cell\nMultiline text/,
    );
    await command(p, 'Undo');
    assert.equal(await cells(p).count(), 13);
    await command(p, 'Redo');
    d = await tool(p, 'Resize title-block table');
    for (const [name, value] of [
      ['Table X (µm)', '100000'],
      ['Table Y (µm)', '150000'],
      ['Table width (µm)', '180001'],
      ['Table height (µm)', '40001'],
    ])
      await d.getByRole('textbox', { name, exact: true }).fill(value);
    await apply(d);
    const resized = await editor(p).locator('.frame-cell').first().getAttribute('data-field');
    d = await tool(p, 'Move table boundary');
    await d.getByRole('textbox', { name: 'Boundary to (µm)', exact: true }).fill('80000');
    await apply(d);
    assert.equal(
      await editor(p).locator('.frame-cell').first().getAttribute('data-field'),
      resized,
    );
    await p.setViewportSize({ width: 960, height: 640 });
    assert.equal((await p.locator('.ribbon').boundingBox()).height, 96);
    await p.screenshot({ path: resolve(root, 'target/ui/template-table-tools.png') });
    await command(p, 'Save template revision');
    await until(async () => !/^●/.test(await p.title()));
    await command(p, 'Close template editor');
    await editor(p).waitFor({ state: 'hidden' });
    assert.equal(await readFile(project(), 'utf8'), original);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('table review cancellation preserves redo and merge requires a cell range', async () => {
    const p = await page({ electrical: false });
    await open(p);
    assert.equal(
      await ribbon(p)
        .getByRole('button', { name: 'Merge title-block cells', exact: true })
        .isDisabled(),
      true,
    );
    await cells(p).first().click();
    assert.equal(
      await ribbon(p)
        .getByRole('button', { name: 'Merge title-block cells', exact: true })
        .isDisabled(),
      true,
    );
    let d = await tool(p, 'Split title-block cell');
    await apply(d);
    assert.equal(await cells(p).count(), 9);
    await command(p, 'Undo');
    d = await tool(p, 'Resize title-block table');
    await d.getByRole('textbox', { name: 'Table width (µm)', exact: true }).fill('1');
    await d.getByRole('button', { name: 'Cancel', exact: true }).click();
    await d.waitFor({ state: 'hidden' });
    await command(p, 'Redo');
    assert.equal(await cells(p).count(), 9);
    await command(p, 'Discard template draft');
    await editor(p).waitFor({ state: 'hidden' });
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
