/** Real canvas gestures against the same Rust fixture as the editor workflows. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { resolve } from 'node:path';
export function registerControlsTests({
  page,
  tree,
  button,
  until,
  pressAction,
  root,
  route,
  label,
}) {
  const anchor = (n) => `anchor:${n.toString(16).padStart(32, '0')}`;
  async function screen(p, point) {
    return p.locator('svg.drawing').evaluate((svg, [x, y]) => {
      const q = new DOMPoint(x, y).matrixTransform(svg.getScreenCTM());
      return { x: q.x, y: q.y };
    }, point);
  }
  async function world(p, point) {
    return p.locator('svg.drawing').evaluate((svg, { x, y }) => {
      const q = new DOMPoint(x, y).matrixTransform(svg.getScreenCTM().inverse());
      return [q.x, q.y];
    }, point);
  }
  async function wheel(p, point, fields) {
    await p.locator('svg.drawing').evaluate(
      (svg, { point, fields }) => {
        svg.dispatchEvent(
          new WheelEvent('wheel', {
            bubbles: true,
            cancelable: true,
            clientX: point.x,
            clientY: point.y,
            ...fields,
          }),
        );
      },
      { point, fields },
    );
  }
  const box = (p) =>
    p
      .locator('svg.drawing')
      .getAttribute('viewBox')
      .then((v) => v.split(' ').map(Number));
  const selected = (p) =>
    p
      .locator('svg.drawing g.selected')
      .evaluateAll((nodes) => nodes.map((n) => n.dataset.object).sort());
  async function marquee(p, from, to, add = false, release = true) {
    const a = await screen(p, from),
      b = await screen(p, to);
    if (add) await p.keyboard.down('Shift');
    await p.mouse.move(a.x, a.y);
    await p.mouse.down();
    await p.mouse.move(b.x, b.y, { steps: 4 });
    if (release) await p.mouse.up();
    if (add) await p.keyboard.up('Shift');
  }
  test('cursor zoom and modifier wheel pans preserve the intended axis across wheel units', async () => {
    const p = await page();
    const cursor = await screen(p, [27, -18]);
    const location = { x: Math.round(cursor.x), y: Math.round(cursor.y) };
    const before = await world(p, location),
      initial = await box(p);
    await p.mouse.move(location.x, location.y);
    await p.mouse.wheel(0, -240);
    await until(async () => (await box(p))[2] !== initial[2]);
    const zoomed = await box(p),
      after = await world(p, location);
    assert.ok(zoomed[2] < initial[2]);
    assert.ok(Math.abs(after[0] - before[0]) < 1e-4 && Math.abs(after[1] - before[1]) < 1e-4);
    await wheel(p, location, { deltaY: 80, shiftKey: true });
    const shifted = await box(p);
    assert.ok(shifted[0] > zoomed[0]);
    assert.equal(shifted[1], zoomed[1]);
    assert.equal(shifted[2], zoomed[2]);
    await wheel(p, location, { deltaY: -80, ctrlKey: true });
    const controlled = await box(p);
    assert.equal(controlled[0], shifted[0]);
    assert.ok(controlled[1] < shifted[1]);
    assert.equal(controlled[2], shifted[2]);
    await wheel(p, location, { deltaX: 2, deltaY: 0, shiftKey: true, deltaMode: 1 });
    const horizontal = await box(p);
    assert.ok(horizontal[0] > controlled[0]);
    assert.equal(horizontal[1], controlled[1]);
    await wheel(p, location, { deltaY: 0 });
    assert.deepEqual(await box(p), horizontal);
    await wheel(p, location, { deltaY: 0.1, ctrlKey: true, deltaMode: 2 });
    assert.ok((await box(p))[1] > horizontal[1]);
    // Middle-button pan remains available independently of the default marquee tool.
    await p.mouse.move(location.x, location.y);
    await p.mouse.down({ button: 'middle' });
    const previous = await box(p);
    await p.mouse.move(location.x + 25, location.y + 15);
    await p.mouse.up({ button: 'middle' });
    assert.ok((await box(p))[0] < previous[0]);
    assert.equal((await box(p))[2], previous[2]);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('directional marquee distinguishes complete geometry from crossings and empty route bounds', async () => {
    const p = await page();
    await marquee(p, [10, 8], [40, 18]);
    assert.deepEqual(await selected(p), []);
    await marquee(p, [40, 18], [10, 8]);
    assert.deepEqual(await selected(p), [route]);
    await marquee(p, [40, 10], [30, 3]);
    assert.deepEqual(await selected(p), []);
    await marquee(p, [-3, -3], [53, 18]);
    assert.deepEqual(await selected(p), [anchor(1), anchor(2), route].sort());
    await marquee(p, [97, -3], [103, 3], true);
    assert.deepEqual(await selected(p), [anchor(1), anchor(2), anchor(3), route].sort());
    const text = await p.locator(`svg [data-object="${label}"] text.ink`).evaluate((n) => {
      const b = n.getBBox();
      return [b.x - 2, b.y - 2, b.x + b.width + 2, b.y + b.height + 2];
    });
    await marquee(p, text.slice(0, 2), text.slice(2));
    assert.deepEqual(await selected(p), [label]);
    await marquee(p, [-5, -40], [110, 20], false, false);
    assert.equal(await p.locator('.selection-marquee.window').count(), 1);
    await p.screenshot({ path: resolve(root, 'target/ui/marquee-selection.png') });
    await p.keyboard.press('Escape');
    await p.mouse.up();
    assert.equal(await p.locator('.selection-marquee').count(), 0);
    assert.deepEqual(await selected(p), [label]);
    await marquee(p, [110, 20], [-5, -40], false, false);
    assert.equal(await p.locator('.selection-marquee.crossing').count(), 1);
    await p.evaluate(() => window.dispatchEvent(new Event('blur')));
    await p.mouse.up();
    assert.deepEqual(await selected(p), [label]);
    assert.equal(await p.locator('.selection-marquee').count(), 0);
    // Independently authored cap/width cases exercise actual SVG bounds as well as segment math.
    const capCases = await p.evaluate(async () => {
      const { marqueeTargets } = await import('/src/marquee.ts');
      const ns = 'http://www.w3.org/2000/svg';
      const svg = document.createElementNS(ns, 'svg'),
        group = document.createElementNS(ns, 'g'),
        ink = document.createElementNS(ns, 'polyline');
      const target = { kind: 'polyline', id: 'f'.repeat(32) };
      group.setAttribute('data-object', `polyline:${target.id}`);
      ink.setAttribute('class', 'ink');
      group.append(ink);
      svg.append(group);
      document.body.append(svg);
      const pick = (points, from, to) => {
        ink.setAttribute('points', points.map((p) => p.join(',')).join(' '));
        return marqueeTargets(svg, [{ target, shape: 'path', points, width: 2 }], from, to).length;
      };
      const result = [
        pick(
          [
            [0, 0],
            [10, 0],
          ],
          [-0.8, -0.9],
          [-0.9, -0.8],
        ),
        pick(
          [
            [0, 0],
            [10, 0],
          ],
          [4, 0.9],
          [3, 0.95],
        ),
        pick(
          [
            [0, 0],
            [10, 0],
          ],
          [4, 1.1],
          [3, 1.2],
        ),
        pick(
          [
            [0, 0],
            [10, 10],
          ],
          [-1.35, -0.01],
          [-1.4, 0.01],
        ),
        pick(
          [
            [0, 0],
            [10, 10],
          ],
          [-1.1, -1.1],
          [11.1, 11.1],
        ),
      ];
      svg.remove();
      return result;
    });
    assert.deepEqual(capCases, [1, 1, 0, 1, 0]);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('object drag preserves selected group and grab offset, waits for final Rust preview and undoes once', async () => {
    const p = await page();
    const ink = () => p.locator(`svg [data-object="${route}"] polyline.ink`).getAttribute('points');
    const original = await ink();
    await tree(p, anchor(1)).click();
    await tree(p, anchor(2)).click({ modifiers: ['Shift'] });
    const start = await screen(p, [0.5, 0.5]),
      end = await screen(p, [10.5, 5.5]);
    // A small click wobble is selection, not an accidental move.
    await p.mouse.move(start.x, start.y);
    await p.mouse.down();
    await p.mouse.move(start.x + 1, start.y + 1);
    await p.mouse.up();
    assert.equal(await ink(), original);
    assert.deepEqual(await selected(p), [anchor(1), anchor(2)].sort());
    await p.route('**/bridge', async (route) => {
      if (route.request().postDataJSON()?.type === 'preview_move') {
        const response = await route.fetch();
        await new Promise((r) => setTimeout(r, 80));
        await route.fulfill({ response });
      } else await route.continue();
    });
    await p.mouse.move(start.x, start.y);
    await p.mouse.down();
    await p.mouse.move(end.x, end.y, { steps: 3 });
    await p.mouse.up();
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Selection moved · one undo operation',
    );
    assert.equal(await ink(), '10,5 30,5 30,20 60,20 60,5');
    assert.deepEqual(await selected(p), [anchor(1), anchor(2)].sort());
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await ink(), original);
    assert.doesNotMatch(await p.title(), /^●/);
    await pressAction(p, 'Redo', 'Redo complete');
    assert.equal(await ink(), '10,5 30,5 30,20 60,20 60,5');
    await pressAction(p, 'Undo', 'Undo complete');
    await p.unroute('**/bridge');
    await p.route('**/bridge', async (route) => {
      if (route.request().postDataJSON()?.type === 'begin_move') {
        const response = await route.fetch();
        await new Promise((resolve) => setTimeout(resolve, 100));
        await route.fulfill({ response });
      } else await route.continue();
    });
    // Escape before/while the asynchronous tool starts must never leave a delayed commit.
    await p.mouse.move(start.x, start.y);
    await p.mouse.down();
    await p.mouse.move(end.x, end.y);
    await p.keyboard.press('Escape');
    await p.mouse.up();
    await until(
      async () => !(await p.getByRole('textbox', { name: 'Move X (µm)', exact: true }).isVisible()),
    );
    assert.equal(await ink(), original);
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'Move cancelled',
    );
    await p.mouse.move(start.x, start.y);
    await p.mouse.down();
    await p.mouse.move(end.x, end.y);
    await p.evaluate(() => window.dispatchEvent(new Event('blur')));
    await p.mouse.up();
    await until(
      async () => !(await p.getByRole('textbox', { name: 'Move X (µm)', exact: true }).isVisible()),
    );
    assert.equal(await ink(), original);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
