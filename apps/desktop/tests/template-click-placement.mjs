/** Click placement shares coalesced retained previews with drag placement and schematic text. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
export function registerTemplateClickPlacement({ page, button, until }) {
  test('template start/finish clicks preserve anchors and final delayed coordinates with exact history and cancellation', async () => {
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
          const q = new DOMPoint(x, y).matrixTransform(s.getScreenCTM());
          return { x: q.x, y: q.y };
        },
        [x, y],
      );
    const click = async (x, y) => {
      const q = await point(x, y);
      await p.mouse.click(q.x, q.y);
    };
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    await command('New template');
    await ready();
    for (const kind of ['line', 'rectangle', 'ellipse']) {
      await command(`Draw template ${kind}`);
      await until(async () => (await svg.getAttribute('data-artwork-tool')) === kind);
      await click(90, 60);
      assert.equal(await svg.getAttribute('data-artwork-phase'), 'finish');
      assert.equal(await svg.locator('[data-artwork-anchor]').count(), 1);
      let delayed = false;
      await p.route('**/bridge', async (route) => {
        const request = route.request().postDataJSON();
        if (request.type === 'template_geometry_preview' && request.values.edit.end && !delayed) {
          delayed = true;
          await new Promise((resolve) => setTimeout(resolve, 200));
        }
        await route.continue();
      });
      const hover = await point(50, 30);
      await p.mouse.move(hover.x, hover.y);
      await click(20, 20);
      await ready();
      await p.unrouteAll({ behavior: 'wait' });
      assert.ok(delayed);
      const graphic = art.last();
      const id = await graphic.getAttribute('data-template-graphic');
      // Inspector values prove the final click won over the delayed intermediate preview.
      const value = (name) =>
        editor.getByRole('textbox', { name: `Artwork ${name} (µm)`, exact: true }).inputValue();
      assert.ok(Math.abs(Number(await value('X')) - 20000) < 500);
      assert.ok(Math.abs(Number(await value('Y')) - 20000) < 500);
      assert.ok(Math.abs(Number(await value('Width')) - 70000) < 500);
      assert.ok(Math.abs(Number(await value('Height')) - 40000) < 500);
      await command('Undo');
      await ready();
      await command('Redo');
      await ready();
      assert.equal(await art.last().getAttribute('data-template-graphic'), id);
    }
    await command('Undo');
    await ready();
    const count = await art.count();
    await command('Draw template rectangle');
    await until(async () => (await svg.getAttribute('data-artwork-tool')) === 'rectangle');
    await click(10, 80);
    await p.route('**/bridge', async (route) => {
      if (route.request().postDataJSON().type === 'template_geometry_preview')
        await new Promise((resolve) => setTimeout(resolve, 200));
      await route.continue();
    });
    const hover = await point(40, 100);
    await p.mouse.move(hover.x, hover.y);
    await p.keyboard.press('Escape');
    await ready();
    await p.unrouteAll({ behavior: 'wait' });
    assert.equal(await art.count(), count);
    assert.equal(await svg.locator('[data-artwork-anchor]').count(), 0);
    await command('Redo');
    await ready();
    assert.equal(await art.count(), count + 1);
    await command('Draw template rectangle');
    await until(async () => (await svg.getAttribute('data-artwork-tool')) === 'rectangle');
    await click(10, 80);
    await p.mouse.move(hover.x, hover.y);
    await click(-5, 100);
    await ready();
    assert.equal(await art.count(), count + 1); // An invalid finish never commits the preceding valid preview.
    await command('Discard template draft');
    await editor.waitFor({ state: 'hidden' });
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
