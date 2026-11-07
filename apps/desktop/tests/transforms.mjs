/** End-to-end pivot controls and immutable previews backed by the Rust controller. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
export function registerTransformTests({
  page,
  tree,
  button,
  until,
  pressAction,
  bounds,
  root,
  route,
  project,
}) {
  const anchor = (n) => `anchor:${n.toString(16).padStart(32, '0')}`;
  const x = (p) => p.getByRole('textbox', { name: 'Pivot X (µm)', exact: true });
  const y = (p) => p.getByRole('textbox', { name: 'Pivot Y (µm)', exact: true });
  const operation = (p) => p.getByRole('combobox', { name: 'Transform operation', exact: true });
  const ink = (p) => p.locator(`svg [data-object="${route}"] polyline.ink`).getAttribute('points');
  async function group(p) {
    await tree(p, anchor(1)).click();
    await tree(p, anchor(2)).click({ modifiers: ['Shift'] });
    await p.locator('.canvas').focus();
  }
  test('R previews an explicit pivot without moving panes; invalid input cancels safely and committed rotation round trips', async () => {
    const p = await page();
    await group(p);
    const original = await ink(p),
      before = await bounds(p);
    await p.keyboard.press('r');
    await x(p).waitFor();
    await until(async () => (await ink(p)) === '0,0 0,20 -15,20 -15,50 0,50');
    assert.deepEqual(await bounds(p), before);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.equal(await x(p).inputValue(), '0');
    assert.equal(await y(p).inputValue(), '0');
    await x(p).fill('r');
    await until(async () => await button(p, 'Apply').isDisabled());
    await p.waitForFunction(() =>
      document.querySelector('.status-message')?.classList.contains('error'),
    );
    assert.equal(await ink(p), original);
    assert.deepEqual(await bounds(p), before);
    await x(p).press('Escape');
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'Transform cancelled',
    );
    await until(async () => !(await x(p).isVisible()));
    assert.equal(await ink(p), original);
    await p.locator('.canvas').focus();
    await p.keyboard.press('r');
    await x(p).waitFor();
    await x(p).fill('10000');
    await y(p).fill('5000');
    await operation(p).selectOption('rotate_180');
    await until(async () => (await ink(p)) === '20,10 0,10 0,-5 -30,-5 -30,10');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    await p.screenshot({ path: resolve(root, 'target/ui/rotation-preview.png') });
    await button(p, 'Apply').click();
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Selection transformed · one undo operation',
    );
    assert.deepEqual(await bounds(p), before);
    assert.equal(await ink(p), '20,10 0,10 0,-5 -30,-5 -30,10');
    await pressAction(p, 'Save project', 'Project saved');
    const saved = JSON.parse(await readFile(project(), 'utf8'));
    assert.deepEqual(saved.drawing.routes[0].points_um, [
      [20000, 10000],
      [0, 10000],
      [0, -5000],
      [-30000, -5000],
      [-30000, 10000],
    ]);
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await ink(p), original);
    await pressAction(p, 'Redo', 'Redo complete');
    assert.equal(await ink(p), '20,10 0,10 0,-5 -30,-5 -30,10');
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Save project', 'Project saved');
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('mirror controls preserve route identity and support both pivot-defined axes', async () => {
    const p = await page();
    await group(p);
    const original = await ink(p),
      before = await bounds(p);
    await button(p, 'Mirror selection').click();
    await x(p).waitFor();
    await x(p).fill('25000');
    await until(async () => (await ink(p)) === '50,0 30,0 30,15 0,15 0,0');
    assert.deepEqual(await bounds(p), before);
    await operation(p).selectOption('mirror_horizontal');
    await until(async () => (await ink(p)) === '0,0 20,0 20,-15 50,-15 50,0');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    await button(p, 'Apply').click();
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Selection transformed · one undo operation',
    );
    assert.equal(await ink(p), '0,0 20,0 20,-15 50,-15 50,0');
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await ink(p), original);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
