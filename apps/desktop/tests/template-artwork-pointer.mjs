/** Pointer artwork uses retained Rust proposals, including delayed release and cancellation. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
export function registerTemplateArtworkPointerTests({ page, button, until }) {
  test('pointer template primitives and whole artwork drags commit once and cancel cleanly', async () => {
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
          const p = s.createSVGPoint();
          p.x = x;
          p.y = y;
          const r = p.matrixTransform(s.getScreenCTM());
          return { x: r.x, y: r.y };
        },
        [x, y],
      );
    const drag = async (a, b) => {
      a = await point(...a);
      b = await point(...b);
      await p.mouse.move(a.x, a.y);
      await p.mouse.down();
      await p.mouse.move(b.x, b.y, { steps: 5 });
      await p.mouse.up();
      const inline = editor.getByRole('region', { name: 'Place template text', exact: true });
      if (await inline.count()) {
        await inline.getByRole('textbox', { name: 'Content 1 text', exact: true }).fill('Text');
        await inline.getByRole('button', { name: 'Place text', exact: true }).click();
      }
      await ready();
    };
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    await command('New template');
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'New template draft',
    );
    await ready();
    for (const [name, a, b] of [
      ['Draw template text', [20, 20], [70, 40]],
      ['Draw template rectangle', [130, 50], [90, 20]],
      ['Draw template ellipse', [150, 20], [190, 50]],
      ['Draw template line', [200, 40], [240, 40]],
    ]) {
      await command(name);
      await svg.locator(':scope').waitFor();
      await until(async () => !!(await svg.getAttribute('data-artwork-tool')));
      await drag(a, b);
    }
    assert.equal(await art.count(), 4);
    assert.equal(await art.locator('polyline').count(), 1);
    assert.equal(await art.locator('ellipse').count(), 1);
    const before = await art.first().getAttribute('data-template-graphic');
    const original = await art
      .first()
      .locator('rect')
      .last()
      .evaluate((n) => ['x', 'y', 'width', 'height'].map((k) => Number(n.getAttribute(k))));
    let delayed = false;
    await p.route('**/bridge', async (route) => {
      const r = route.request().postDataJSON();
      if (
        r.type === 'template_geometry_preview' &&
        r.action === 'template.artwork.move' &&
        !delayed
      ) {
        delayed = true;
        await new Promise((resolve) => setTimeout(resolve, 180));
      }
      await route.continue();
    });
    await drag([30, 30], [45, 45]);
    await p.unroute('**/bridge');
    assert.ok(delayed);
    assert.equal(await art.first().getAttribute('data-template-graphic'), before);
    const moved = await art
      .first()
      .locator('rect')
      .last()
      .evaluate((n) => ['x', 'y', 'width', 'height'].map((k) => Number(n.getAttribute(k))));
    assert.ok(Math.abs(moved[0] - original[0] - 15) < 0.5);
    assert.ok(Math.abs(moved[1] - original[1] - 15) < 0.5);
    assert.deepEqual(moved.slice(2), original.slice(2));
    await command('Undo');
    await ready();
    assert.deepEqual(
      await art
        .first()
        .locator('rect')
        .last()
        .evaluate((n) => ['x', 'y', 'width', 'height'].map((k) => Number(n.getAttribute(k)))),
      original,
    );
    const click = await point(30, 30);
    await p.mouse.click(click.x, click.y);
    await ready();
    await command('Draw template ellipse');
    await until(async () => (await svg.getAttribute('data-artwork-tool')) === 'ellipse');
    await p.keyboard.press('Escape');
    await ready();
    assert.equal(await art.count(), 4);
    await command('Redo');
    await ready();
    assert.deepEqual(
      await art
        .first()
        .locator('rect')
        .last()
        .evaluate((n) => ['x', 'y', 'width', 'height'].map((k) => Number(n.getAttribute(k)))),
      moved,
    );
    // Out-of-paper releases do not apply the last successful in-paper preview.
    await command('Draw template rectangle');
    await until(async () => (await svg.getAttribute('data-artwork-tool')) === 'rectangle');
    await drag([15, 80], [-5, 100]);
    assert.equal(await art.count(), 4);
    await command('Save template revision');
    await ready();
    await command('Close template editor');
    await editor.waitFor({ state: 'hidden' });
    await p.close();
  });
}
