/** Retained graphical path placement against the real document and temporary saved project. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
export function registerPathTests({
  page,
  button,
  until,
  pressAction,
  bounds,
  input,
  root,
  project,
}) {
  const x = (p) => p.getByRole('textbox', { name: 'Path X (µm)', exact: true });
  const y = (p) => p.getByRole('textbox', { name: 'Path Y (µm)', exact: true });
  const count = async (p, n) =>
    until(async () => (await p.locator('.path-count').innerText()) === `${n} points`);
  async function start(p) {
    await p.locator('.canvas').focus();
    await p.keyboard.press('p');
    await x(p).waitFor();
    await until(async () => !(await button(p, 'Add point').isDisabled()));
  }
  async function point(p, px, py, n) {
    await x(p).fill(String(px));
    await y(p).fill(String(py));
    await y(p).press('Enter');
    await count(p, n);
  }
  async function screen(p, point) {
    return p.locator('svg.drawing').evaluate((svg, p) => {
      const q = new DOMPoint(...p).matrixTransform(svg.getScreenCTM());
      return { x: q.x, y: q.y };
    }, point);
  }
  test('P accepts exact vertices, revises the draft, then saves one whole movable/rotatable path with one undo step', async () => {
    const p = await page();
    const before = await bounds(p);
    await start(p);
    await p.setViewportSize({ width: 960, height: 680 });
    const narrow = await bounds(p),
      cancel = await button(p, 'Cancel').boundingBox();
    assert.ok(cancel.x + cancel.width <= narrow.x + narrow.width);
    assert.ok((await x(p).boundingBox()).width >= 48);
    assert.equal(await p.evaluate(() => document.documentElement.scrollWidth > innerWidth), false);
    await p.setViewportSize({ width: 1360, height: 880 });
    assert.equal(await button(p, 'Finish').isDisabled(), true);
    await point(p, 10000, -25000, 1);
    await point(p, 40000, -25000, 2);
    await point(p, 40000, -5000, 3);
    await p.locator('.canvas').focus();
    await p.keyboard.press('Backspace');
    await count(p, 2);
    await until(async () => (await p.locator('.path-rubber').count()) === 1);
    const ghost = p.locator('svg .move-preview[data-object^="polyline:"]');
    assert.equal(await ghost.locator('.ink').getAttribute('points'), '10,-25 40,-25');
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(await bounds(p), before);
    const key = await ghost.getAttribute('data-object');
    await p.screenshot({ path: resolve(root, 'target/ui/path-placement.png') });
    await p.locator('.canvas').focus();
    await p.keyboard.press('Enter');
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Graphical path created · one undo operation',
    );
    await input(p, 'stroke_width').waitFor();
    assert.equal(await input(p, 'stroke_width').inputValue(), '250');
    assert.equal(
      await p.locator(`svg [data-object="${key}"] .ink`).getAttribute('points'),
      '10,-25 40,-25',
    );
    await pressAction(p, 'Save project', 'Project saved');
    const saved = JSON.parse(await readFile(project(), 'utf8'));
    assert.deepEqual(saved.drawing.polylines.find((v) => `polyline:${v.id}` === key).points_um, [
      [10000, -25000],
      [40000, -25000],
    ]);
    assert.equal(saved.conductors.length, 2);
    await p.locator('.canvas').focus();
    await p.keyboard.press('m');
    await p.getByRole('textbox', { name: 'Move X (µm)', exact: true }).fill('20000');
    await p.getByRole('textbox', { name: 'Move Y (µm)', exact: true }).fill('-15000');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    await button(p, 'Apply').click();
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Selection moved · one undo operation',
    );
    assert.equal(
      await p.locator(`svg [data-object="${key}"] .ink`).getAttribute('points'),
      '20,-15 50,-15',
    );
    await pressAction(p, 'Undo', 'Undo complete');
    await p.locator('.canvas').focus();
    await p.keyboard.press('r');
    await p.getByRole('textbox', { name: 'Pivot X (µm)', exact: true }).waitFor();
    await until(
      async () =>
        (await p.locator(`svg [data-object="${key}"] .ink`).getAttribute('points')) ===
        '10,-25 10,5',
    );
    await p.locator('.canvas').focus();
    await p.keyboard.press('Escape');
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'Transform cancelled',
    );
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await p.locator(`svg [data-object="${key}"]`).count(), 0);
    await pressAction(p, 'Redo', 'Redo complete');
    assert.equal(
      await p.locator(`svg [data-object="${key}"] .ink`).getAttribute('points'),
      '10,-25 40,-25',
    );
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Save project', 'Project saved');
    assert.deepEqual(await bounds(p), before);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('rapid clicks retain their coordinates while preview responses are delayed and Enter finishes the accepted path', async () => {
    const p = await page();
    await start(p);
    await p.route('**/bridge', async (route) => {
      if (route.request().postDataJSON()?.type === 'preview_path') {
        const response = await route.fetch();
        await new Promise((resolve) => setTimeout(resolve, 60));
        await route.fulfill({ response });
      } else await route.continue();
    });
    for (const point of [
      [10, -20],
      [25, -20],
      [25, -5],
    ]) {
      const q = await screen(p, point);
      await p.mouse.click(q.x, q.y);
    }
    await p.keyboard.press('Enter');
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Graphical path created · one undo operation',
    );
    const path = p.locator('svg g.selected[data-object^="polyline:"] .ink');
    assert.equal(await path.getAttribute('points'), '10,-20 25,-20 25,-5');
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await p.locator('svg [data-object^="polyline:"]').count(), 0);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('path snapping/cycling/suppression, empty draft Backspace and in-flight cancellation preserve document and redo', async () => {
    const p = await page();
    const redo = await button(p, 'Redo').isDisabled();
    await start(p);
    const q = await screen(p, [100, 0]);
    await p.mouse.move(q.x, q.y);
    await p.waitForFunction(() =>
      document.querySelector('.status-message')?.textContent?.includes('Terminal snap'),
    );
    await p.keyboard.press('Tab');
    await p.waitForFunction(() =>
      document.querySelector('.status-message')?.textContent?.includes('Endpoint snap'),
    );
    await p.keyboard.down('Alt');
    await until(async () => (await p.locator('.snap-marker').count()) === 0);
    await p.keyboard.up('Alt');
    await until(async () => (await p.locator('.snap-marker').count()) === 1);
    await p.mouse.click(q.x, q.y);
    await count(p, 1);
    await p.keyboard.press('Backspace');
    await count(p, 0);
    await p.keyboard.press('Enter');
    await p.waitForFunction(() =>
      document
        .querySelector('.status-message')
        ?.textContent?.startsWith('Accept at least two points'),
    );
    assert.equal(await x(p).isVisible(), true);
    assert.equal(await button(p, 'Finish').isDisabled(), true);
    await p.keyboard.press('Escape');
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'Path cancelled',
    );
    await start(p);
    await p.route('**/bridge', async (route) => {
      if (route.request().postDataJSON()?.type === 'preview_path') {
        const response = await route.fetch();
        await new Promise((resolve) => setTimeout(resolve, 100));
        await route.fulfill({ response });
      } else await route.continue();
    });
    await x(p).fill('10000');
    await x(p).press('Escape');
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'Path cancelled',
    );
    assert.equal(
      await p.locator('svg .path-rubber, svg .path-vertex, svg .path-cursor').count(),
      0,
    );
    assert.equal(await button(p, 'Redo').isDisabled(), redo);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
