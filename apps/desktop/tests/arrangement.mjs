/** Exact origin layout integrates ribbon/palette controls with reviewed connected-route geometry. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
const primary = process.platform === 'darwin' ? 'Meta' : 'Control';
export function registerArrangementTests({
  page,
  tree,
  button,
  until,
  pressAction,
  bounds,
  root,
  project,
}) {
  const anchor = (n) => `anchor:${n.toString(16).padStart(32, '0')}`;
  const route = 'route:00000000000000000000000000000001';
  const routePoints = (p) => p.locator(`svg [data-object="${route}"] .ink`).getAttribute('points');
  const cx = (p, n) => p.locator(`svg [data-object="${anchor(n)}"] circle.ink`).getAttribute('cx');
  const coordinate = (p) =>
    p.getByRole('textbox', { name: 'Alignment coordinate (µm)', exact: true });
  async function group(p) {
    await tree(p, anchor(1)).click();
    await tree(p, anchor(2)).click({ modifiers: ['Shift'] });
    await tree(p, anchor(3)).click({ modifiers: ['Shift'] });
  }
  test('origin alignment previews exact X/Y coordinates and connected routes, rejects invalid input and saves one undoable edit', async () => {
    const p = await page();
    const original = await routePoints(p),
      before = await bounds(p);
    await group(p);
    await button(p, 'Align drawing origins').click();
    await coordinate(p).waitFor();
    await p.getByRole('combobox', { name: 'Alignment axis', exact: true }).selectOption('align_y');
    await coordinate(p).fill('-10000');
    await until(async () => (await routePoints(p)) === '0,-10 20,-10 20,5 50,5 50,-10');
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(await bounds(p), before);
    await coordinate(p).fill('9223372036854775808');
    await p.waitForFunction(() =>
      document.querySelector('.status-message')?.classList.contains('error'),
    );
    assert.equal(await button(p, 'Apply').isDisabled(), true);
    assert.equal(await routePoints(p), original);
    await coordinate(p).fill('-10000');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    await p.setViewportSize({ width: 960, height: 680 });
    const narrow = await bounds(p),
      cancel = await button(p, 'Cancel').boundingBox();
    assert.ok(cancel.x + cancel.width <= narrow.x + narrow.width);
    assert.ok((await coordinate(p).boundingBox()).width >= 70);
    await p.setViewportSize({ width: 1360, height: 880 });
    await p.screenshot({ path: resolve(root, 'target/ui/origin-alignment.png') });
    await button(p, 'Apply').click();
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Origins aligned · one undo operation',
    );
    await pressAction(p, 'Save project', 'Project saved');
    const saved = JSON.parse(await readFile(project(), 'utf8'));
    assert.deepEqual(saved.drawing.routes[0].points_um, [
      [0, -10000],
      [20000, -10000],
      [20000, 5000],
      [50000, 5000],
      [50000, -10000],
    ]);
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await routePoints(p), original);
    await pressAction(p, 'Redo', 'Redo complete');
    assert.equal(await routePoints(p), '0,-10 20,-10 20,5 50,5 50,-10');
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Save project', 'Project saved');
    assert.deepEqual(await bounds(p), before);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('distribution needs three origins across ribbon/palette, preserves no-op redo and spaces between existing extrema', async () => {
    const p = await page();
    await tree(p, anchor(1)).click();
    await tree(p, anchor(2)).click({ modifiers: ['Shift'] });
    assert.equal(await button(p, 'Distribute drawing origins').isDisabled(), true);
    await p.locator('.canvas').focus();
    await p.keyboard.press(`${primary}+Shift+p`);
    await p.getByRole('textbox', { name: 'Search commands', exact: true }).fill('Distribute');
    assert.equal(
      await p.locator('.palette-results button').filter({ hasText: 'Distribute' }).isDisabled(),
      true,
    );
    await p.keyboard.press('Escape');
    await tree(p, anchor(3)).click({ modifiers: ['Shift'] });
    assert.equal(await button(p, 'Distribute drawing origins').isDisabled(), false);
    const redo = await button(p, 'Redo').isDisabled();
    await button(p, 'Distribute drawing origins').click();
    await p.getByRole('combobox', { name: 'Distribution axis', exact: true }).waitFor();
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    await button(p, 'Apply').click();
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Layout already matches · no document changes',
    );
    assert.equal(await button(p, 'Redo').isDisabled(), redo);
    assert.doesNotMatch(await p.title(), /^●/);
    await tree(p, anchor(2)).click();
    await p.locator('.canvas').focus();
    await p.keyboard.press('m');
    await p.getByRole('textbox', { name: 'Move X (µm)', exact: true }).fill('30000');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    await button(p, 'Apply').click();
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Selection moved · one undo operation',
    );
    assert.equal(await cx(p, 2), '30');
    await group(p);
    await p.locator('.canvas').focus();
    await p.keyboard.press(`${primary}+Shift+p`);
    await p.getByRole('textbox', { name: 'Search commands', exact: true }).fill('Distribute');
    await p.locator('.palette-results button').filter({ hasText: 'Distribute' }).click();
    const axis = p.getByRole('combobox', { name: 'Distribution axis', exact: true });
    await axis.waitFor();
    await until(async () => (await cx(p, 2)) === '50');
    assert.equal(await cx(p, 1), '0');
    assert.equal(await cx(p, 3), '100');
    await axis.selectOption('distribute_y');
    await until(async () => (await cx(p, 2)) === '30');
    await axis.selectOption('distribute_x');
    await until(async () => (await cx(p, 2)) === '50');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    await p.screenshot({ path: resolve(root, 'target/ui/origin-distribution.png') });
    await button(p, 'Apply').click();
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Origins distributed · one undo operation',
    );
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await cx(p, 2), '30');
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await cx(p, 2), '50');
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
