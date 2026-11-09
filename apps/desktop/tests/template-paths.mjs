/** Rapid click intents retain every vertex even while replaceable whole-path previews coalesce. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
export function registerTemplatePaths({ page, button, until }) {
  test('template paths preserve rapid clicks, revise invalid points, ignore hover on finish and reopen as whole objects', async () => {
    const p = await page({ electrical: false });
    const editor = p.getByRole('region', { name: 'Drawing template editor', exact: true });
    const ribbon = p.getByRole('tabpanel', { name: 'Template', exact: true });
    const svg = editor.locator('.template-preview');
    const art = editor.locator('[data-template-graphic]');
    const ready = () =>
      until(
        async () =>
          !(await ribbon
            .getByRole('button', { name: 'Draw template text', exact: true })
            .isDisabled()),
      );
    const command = async (name) => {
      await ribbon.getByRole('button', { name, exact: true }).click();
    };
    const point = (x, y) =>
      svg.evaluate(
        (s, [x, y]) => {
          const p = new DOMPoint(x, y).matrixTransform(s.getScreenCTM());
          return { x: p.x, y: p.y };
        },
        [x, y],
      );
    const click = async (x, y) => {
      const q = await point(x, y);
      await p.mouse.click(q.x, q.y);
    };
    const count = () =>
      svg.locator('[data-template-path-draft]').getAttribute('data-template-path-draft');
    const start = async () => {
      await command('Draw template path');
      await until(async () => (await svg.getAttribute('data-artwork-tool')) === 'polyline');
    };
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    await command('New template');
    await ready();
    await editor
      .getByRole('textbox', { name: 'Template name', exact: true })
      .fill('Path regression');
    await editor
      .getByRole('textbox', { name: 'Template name', exact: true })
      .press('Control+Enter');
    await ready();
    await start();
    await p.keyboard.press('Enter');
    assert.equal(await count(), '0');
    await click(70, 60);
    await p.keyboard.press('Enter');
    assert.equal(await count(), '1');
    let delayed = false,
      commits = 0;
    await p.route('**/bridge', async (route) => {
      const r = route.request().postDataJSON();
      if (r.type === 'template_geometry_commit') commits++;
      if (r.type === 'template_geometry_preview' && r.values.edit.points && !delayed) {
        delayed = true;
        await new Promise((resolve) => setTimeout(resolve, 250));
      }
      await route.continue();
    });
    await click(20, 60);
    await click(20, 20);
    await click(70, 20);
    assert.equal(await count(), '4');
    await svg.dispatchEvent('keydown', { key: 'Backspace', repeat: true });
    assert.equal(await count(), '4');
    await p.keyboard.press('Backspace');
    assert.equal(await count(), '3');
    await click(70, 60); // Closing point remains part of the path.
    await click(70, 60);
    assert.equal(await count(), '4'); // Consecutive duplicate is not added.
    const hover = await point(150, 100);
    await p.mouse.move(hover.x, hover.y);
    await p.keyboard.press('Enter');
    await ready();
    await p.unrouteAll({ behavior: 'wait' });
    assert.ok(delayed);
    assert.equal(commits, 1);
    assert.equal(await art.count(), 1);
    const id = await art.getAttribute('data-template-graphic');
    const points = await art.locator('polyline').getAttribute('points');
    const local = await editor
      .getByRole('textbox', { name: 'Local points (x,y per line)', exact: true })
      .inputValue();
    const vertices = points
      .trim()
      .split(/\s+/)
      .map((s) => s.split(',').map(Number));
    assert.equal(vertices.length, 4);
    assert.deepEqual(vertices[0], vertices[3]);
    assert.ok(vertices.every(([x, y]) => x < 71 && y < 61));
    await command('Undo');
    await ready();
    assert.equal(await art.count(), 0);
    await start();
    await click(20, 20);
    await click(-5, 30);
    await p.keyboard.press('Enter');
    await until(async () =>
      (await p.locator('.status-message').innerText()).includes('path is invalid'),
    );
    assert.equal(await count(), '2');
    await p.keyboard.press('Backspace');
    assert.equal(await count(), '1');
    await click(30, 30);
    await p.keyboard.press('Escape');
    await ready();
    assert.equal(await art.count(), 0);
    await command('Redo');
    await ready();
    assert.equal(await art.getAttribute('data-template-graphic'), id);
    assert.equal(await art.locator('polyline').getAttribute('points'), points);
    // Selecting/moving the path treats every segment as one artwork object.
    const a = await point(45, 60),
      b = await point(55, 70);
    await p.mouse.move(a.x, a.y);
    await p.mouse.down();
    await p.mouse.move(b.x, b.y);
    await p.mouse.up();
    await ready();
    assert.equal(await art.getAttribute('data-template-graphic'), id);
    assert.equal(
      await editor
        .getByRole('textbox', { name: 'Local points (x,y per line)', exact: true })
        .inputValue(),
      local,
    );
    const moved = (await art.locator('polyline').getAttribute('points'))
      .trim()
      .split(/\s+/)
      .map((v) => v.split(',').map(Number));
    assert.ok(
      moved.every((v, i) => v.every((n, axis) => Math.abs(n - vertices[i][axis] - 10) < 0.5)),
    );
    await command('Undo');
    await ready();
    await command('Save template revision');
    await ready();
    await command('Close template editor');
    await editor.waitFor({ state: 'hidden' });
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    await editor
      .getByRole('combobox', { name: 'Saved template variants', exact: true })
      .selectOption({ label: 'Path regression' });
    await editor.getByRole('button', { name: 'Open selected template', exact: true }).click();
    await ready();
    assert.equal(await art.getAttribute('data-template-graphic'), id);
    assert.equal(await art.locator('polyline').getAttribute('points'), points);
    await command('Close template editor');
    await editor.waitFor({ state: 'hidden' });
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
