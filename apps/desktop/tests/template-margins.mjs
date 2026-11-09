/** Margin editing and screen-distance border snaps use real Rust proposals/history. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
export function registerTemplateMarginTests({ page, button, until, project, root }) {
  const editor = (p) => p.getByRole('region', { name: 'Drawing template editor', exact: true });
  const ribbon = (p) => p.getByRole('tabpanel', { name: 'Template', exact: true });
  const field = (p, name) => editor(p).getByRole('textbox', { name, exact: true });
  async function point(locator) {
    return locator.evaluate((n) => {
      const r = n.getBoundingClientRect();
      return { x: r.x + r.width / 2, y: r.y + r.height / 2 };
    });
  }
  async function ready(p) {
    await until(
      async () =>
        !(await ribbon(p)
          .getByRole('button', { name: 'Draw template text', exact: true })
          .isDisabled()),
    );
  }
  async function open(p) {
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor(p).waitFor();
    await ribbon(p).getByRole('button', { name: 'New template', exact: true }).click();
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'New template draft',
    );
    await ready(p);
  }
  async function command(p, name) {
    await ribbon(p).getByRole('button', { name, exact: true }).click();
    if (!['Close template editor', 'Discard template draft'].includes(name)) await ready(p);
  }
  async function apply(p) {
    await editor(p).getByRole('button', { name: 'Apply properties', exact: true }).click();
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'Template updated',
    );
    await ready(p);
  }
  async function box(p) {
    return editor(p)
      .locator('.frame-cell > rect')
      .evaluateAll((nodes) => {
        const b = nodes.map((n) =>
          ['x', 'y', 'width', 'height'].map((k) => Number(n.getAttribute(k))),
        );
        const x = Math.min(...b.map((r) => r[0])),
          y = Math.min(...b.map((r) => r[1]));
        return [
          x,
          y,
          Math.max(...b.map((r) => r[0] + r[2])) - x,
          Math.max(...b.map((r) => r[1] + r[3])) - y,
        ];
      });
  }
  async function pointer(p, from, to, edges = []) {
    await p.mouse.move(from.x, from.y);
    await p.mouse.down();
    await p.mouse.move(to.x, to.y, { steps: 4 });
    for (const edge of edges)
      await editor(p).locator(`[data-snap-edge="${edge}"]`).waitFor({ state: 'attached' });
    await p.mouse.up();
    await ready(p);
  }
  test('general margin sets all sides, fits layout without scaling and preserves mixed edits, validation and exact history', async () => {
    const p = await page({ electrical: false });
    const originalProject = await readFile(project(), 'utf8');
    await open(p);
    await editor(p).getByText('Advanced paper settings', { exact: true }).click();
    const original = await box(p);
    assert.equal(await field(p, 'General margin (µm)').inputValue(), '10000');
    await field(p, 'General margin (µm)').fill('20000');
    assert.equal(
      await editor(p)
        .getByRole('checkbox', { name: 'Fit layout inside margins', exact: true })
        .isChecked(),
      true,
    );
    for (const side of ['Left', 'Right', 'Top', 'Bottom'])
      assert.equal(await field(p, `${side} margin (µm)`).inputValue(), '20000');
    await apply(p);
    assert.deepEqual(await box(p), [97, 150, 180, 40]);
    assert.equal(await field(p, 'Content X (µm)').inputValue(), '20000');
    assert.equal(await field(p, 'Content width (µm)').inputValue(), '257000');
    await command(p, 'Undo');
    assert.deepEqual(await box(p), original);
    assert.equal(await field(p, 'General margin (µm)').inputValue(), '10000');
    await command(p, 'Redo');
    assert.deepEqual(await box(p), [97, 150, 180, 40]);
    await field(p, 'Left margin (µm)').fill('19000');
    assert.equal(await field(p, 'General margin (µm)').inputValue(), '');
    assert.equal(await field(p, 'General margin (µm)').getAttribute('placeholder'), 'Mixed');
    await apply(p);
    await field(p, 'General margin (µm)').fill('60000');
    await editor(p).getByRole('button', { name: 'Apply properties', exact: true }).click();
    await until(async () => (await editor(p).getByRole('alert').innerText()).includes('geometry'));
    assert.equal(await field(p, 'General margin (µm)').inputValue(), '60000');
    assert.deepEqual(await box(p), [97, 150, 180, 40]);
    await editor(p).getByRole('button', { name: 'Revert fields', exact: true }).click();
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'Template fields reverted',
    );
    await command(p, 'Undo');
    assert.equal(await field(p, 'Left margin (µm)').inputValue(), '20000');
    await command(p, 'Redo');
    assert.equal(await field(p, 'Left margin (µm)').inputValue(), '19000');
    await command(p, 'Save template revision');
    await until(async () => !/^●/.test(await p.title()));
    await command(p, 'Close template editor');
    await editor(p).waitFor({ state: 'hidden' });
    assert.equal(await readFile(project(), 'utf8'), originalProject);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('pointer table movement and resizing snap to margin borders with feedback, Alt bypass and exact numeric alternatives', async () => {
    const p = await page({ electrical: false });
    await open(p);
    await editor(p).locator('[data-template-cell]').first().click();
    const original = await box(p);
    const scale = await editor(p)
      .locator('svg.template-preview')
      .evaluate((n) => n.getScreenCTM().a);
    let grip = await point(editor(p).locator('[data-control="move"]'));
    await pointer(
      p,
      grip,
      { x: grip.x + (10.4 - original[0]) * scale, y: grip.y + (10.6 - original[1]) * scale },
      ['left', 'top'],
    );
    assert.deepEqual(await box(p), [10, 10, 180, 40]);
    await command(p, 'Undo');
    assert.deepEqual(await box(p), original);
    await command(p, 'Redo');
    assert.deepEqual(await box(p), [10, 10, 180, 40]);
    grip = await point(editor(p).locator('[data-control="move"]'));
    await p.keyboard.down('Alt');
    await pointer(p, grip, { x: grip.x + 0.4 * scale, y: grip.y + 0.6 * scale });
    await p.keyboard.up('Alt');
    const bypassed = await box(p);
    assert.ok(Math.abs(bypassed[0] - 10.4) < 0.002);
    assert.ok(Math.abs(bypassed[1] - 10.6) < 0.002);
    let edge = await point(editor(p).locator('.table-edge[data-axis="x"][data-outer="end"]'));
    await pointer(p, edge, { x: edge.x + (286.6 - bypassed[0] - bypassed[2]) * scale, y: edge.y }, [
      'right',
    ]);
    const right = await box(p);
    assert.equal(right[0] + right[2], 287);
    edge = await point(editor(p).locator('.table-edge[data-axis="y"][data-outer="end"]'));
    await pointer(p, edge, { x: edge.x, y: edge.y + (199.5 - right[1] - right[3]) * scale }, [
      'bottom',
    ]);
    const bottom = await box(p);
    assert.equal(bottom[1] + bottom[3], 200);
    await command(p, 'Undo');
    assert.deepEqual(await box(p), right);
    await command(p, 'Redo');
    assert.deepEqual(await box(p), bottom);
    await editor(p).locator('svg.template-preview').focus();
    await p.keyboard.press(`${process.platform === 'darwin' ? 'Meta' : 'Control'}+Shift+p`);
    await p
      .getByRole('textbox', { name: 'Search commands', exact: true })
      .fill('template.table.resize');
    await p.locator('.palette-item').first().click();
    const d = p.getByRole('dialog', { name: 'Resize title-block table', exact: true });
    await d.waitFor();
    await d.getByRole('textbox', { name: 'Table width (µm)', exact: true }).fill('276000');
    await d.getByRole('button', { name: 'Apply table edit', exact: true }).click();
    await d.waitFor({ state: 'hidden' });
    await ready(p);
    const numeric = await box(p);
    assert.ok(Math.abs(numeric[2] - 276) < 0.00001);
    assert.ok(numeric[0] + numeric[2] < 287);
    // Merely pressing/releasing a grip must preserve an exact near-border numeric placement.
    const clickOnly = await point(editor(p).locator('[data-control="move"]'));
    await p.mouse.click(clickOnly.x, clickOnly.y);
    await ready(p);
    assert.deepEqual(await box(p), numeric);
    await p.screenshot({ path: resolve(root, 'target/ui/template-margins.png') });
    await command(p, 'Discard template draft');
    await editor(p).waitFor({ state: 'hidden' });
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
