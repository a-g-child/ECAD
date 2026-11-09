/** Direct table interactions use real Rust previews, exact history and stable inserted IDs. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
export function registerTemplateTableInteractionTests({ page, button, until, project, root }) {
  const editor = (p) => p.getByRole('region', { name: 'Drawing template editor', exact: true });
  const ribbon = (p) => p.getByRole('tabpanel', { name: 'Template', exact: true });
  const rectangles = (p) => editor(p).locator('.frame-cell > rect');
  const ids = (p) =>
    editor(p)
      .locator('.template-structure [data-template-cell]')
      .evaluateAll((nodes) => nodes.map((n) => n.dataset.templateCell));
  async function point(locator, fx = 0.5, fy = 0.5) {
    return locator.evaluate(
      (n, { fx, fy }) => {
        const r = n.getBoundingClientRect();
        return { x: r.x + r.width * fx, y: r.y + r.height * fy };
      },
      { fx, fy },
    );
  }
  async function ready(p) {
    try {
      await until(
        async () =>
          (await editor(p).locator('.table-control[tabindex="0"]').count()) > 0 &&
          !(await ribbon(p)
            .getByRole('button', { name: 'Draw template text', exact: true })
            .isDisabled()),
      );
    } catch (e) {
      throw new Error(
        `${e.message}; status=${await p.locator('.status-message').innerText()}; controls=${await editor(p).locator('.table-control').count()}; selected=${await editor(p).locator('[data-selected-cell]').count()}`,
      );
    }
  }

  async function open(p) {
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor(p).waitFor();
    await ribbon(p).getByRole('button', { name: 'New template', exact: true }).click();
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'New template draft',
    );
    await until(
      async () =>
        !(await ribbon(p)
          .getByRole('button', { name: 'Draw template text', exact: true })
          .isDisabled()),
    );
    const at = await point(rectangles(p).first());
    await p.mouse.click(at.x, at.y);
    await ready(p);
  }
  async function drag(p, from, to) {
    await p.mouse.move(from.x, from.y);
    await p.mouse.down();
    await p.mouse.move(to.x, to.y, { steps: 5 });
    await p.mouse.up();
    await ready(p);
  }
  async function command(p, name) {
    await ribbon(p).getByRole('button', { name, exact: true }).click();
    if (!['Close template editor', 'Discard template draft'].includes(name)) await ready(p);
  }
  async function bounds(p) {
    return rectangles(p)
      .first()
      .evaluate((n) => ['x', 'y', 'width', 'height'].map((k) => Number(n.getAttribute(k))));
  }
  async function scale(p) {
    return editor(p)
      .locator('svg.template-preview')
      .evaluate((n) => n.getScreenCTM().a);
  }
  test('table pointer ranges preselect merge cells and drag resizing/moving commit exact single-step history', async () => {
    const p = await page({ electrical: false });
    await open(p);
    const original = await bounds(p);
    const gripBefore = await editor(p).locator('[data-control="move"]').boundingBox();
    const zoomAt = await point(rectangles(p).first());
    const previousScale = await scale(p);
    await p.mouse.move(zoomAt.x, zoomAt.y);
    await p.mouse.wheel(0, -200);
    await until(async () => (await scale(p)) > previousScale);
    const gripAfter = await editor(p).locator('[data-control="move"]').boundingBox();
    assert.ok(Math.abs(gripBefore.width - gripAfter.width) < 0.1);
    // Restore the fitted camera before dragging the far table edge, which zoom may clip.
    await p.mouse.wheel(0, 200);
    await until(async () => Math.abs((await scale(p)) - previousScale) < 0.0001);
    await drag(
      p,
      await point(rectangles(p).nth(0), 0.25, 0.5),
      await point(rectangles(p).nth(3), 0.75, 0.5),
    );
    assert.equal(await editor(p).locator('[data-selected-cell]').count(), 4);
    await ribbon(p).getByRole('button', { name: 'Merge title-block cells', exact: true }).click();
    const d = p.getByRole('dialog', { name: 'Merge title-block cells', exact: true });
    await d.waitFor();
    assert.equal(await d.getByRole('checkbox').count(), 0);
    assert.match(await d.innerText(), /4 selected cells/);
    await d.getByRole('button', { name: 'Apply table edit', exact: true }).click();
    await d.waitFor({ state: 'hidden' });
    await ready(p);
    assert.equal(await rectangles(p).count(), 5);
    // A range touching a merged span expands to complete cells and closes the rectangle.
    await drag(
      p,
      await point(rectangles(p).nth(3), 0.25, 0.5),
      await point(rectangles(p).first(), 0.25, 0.5),
    );
    assert.equal(await editor(p).locator('[data-selected-cell]').count(), 5);
    await command(p, 'Undo');
    assert.deepEqual(await bounds(p), original);
    const seam = editor(p).locator('.table-edge[data-axis="x"][data-from="90000"]').first();
    const from = await point(seam);
    const s = await scale(p);
    await drag(p, from, { x: from.x - 10 * s, y: from.y });
    assert.ok(Math.abs((await bounds(p))[2] - 80) < 0.002);
    const resized = await bounds(p);
    await command(p, 'Undo');
    assert.deepEqual(await bounds(p), original);
    await command(p, 'Redo');
    assert.deepEqual(await bounds(p), resized);
    const move = await point(editor(p).locator('[data-control="move"]'));
    await drag(p, move, { x: move.x - 5 * s, y: move.y - 5 * s });
    const moved = await bounds(p);
    assert.ok(Math.abs(moved[0] - (resized[0] - 5)) < 0.002);
    assert.ok(Math.abs(moved[1] - (resized[1] - 5)) < 0.002);
    await command(p, 'Undo');
    assert.deepEqual(await bounds(p), resized);
    await command(p, 'Redo');
    assert.deepEqual(await bounds(p), moved);
    const edge = editor(p).locator('.table-edge[data-axis="x"][data-outer="end"]');
    const end = await point(edge);
    await drag(p, end, { x: end.x + 3 * s, y: end.y });
    assert.ok((await bounds(p))[2] > moved[2]);
    await command(p, 'Undo');
    assert.deepEqual(await bounds(p), moved);
    await p.screenshot({ path: resolve(root, 'target/ui/template-table-interaction.png') });
    await command(p, 'Discard template draft');
    await editor(p).waitFor({ state: 'hidden' });
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('context plus controls insert coherent rows and columns with pinned far edges and exact undo/redo identities', async () => {
    const p = await page({ electrical: false });
    const originalProject = await readFile(project(), 'utf8');
    await open(p);
    const originalIds = await ids(p);
    await editor(p)
      .getByRole('button', { name: 'Insert row below selection; keep bottom edge', exact: true })
      .focus();
    await p.keyboard.press('Space');
    await ready(p);
    assert.equal(await rectangles(p).count(), 10);
    const rowIds = await ids(p);
    assert.deepEqual(rowIds.slice(0, 8), originalIds);
    await command(p, 'Undo');
    assert.deepEqual(await ids(p), originalIds);
    await command(p, 'Redo');
    assert.deepEqual(await ids(p), rowIds);
    await editor(p).locator('[data-control="column"]').click();
    await ready(p);
    assert.equal(await rectangles(p).count(), 15);
    const columnIds = await ids(p);
    assert.deepEqual(columnIds.slice(0, 10), rowIds);
    assert.equal((await bounds(p))[0], 17);
    assert.equal((await bounds(p))[1], 150);
    await command(p, 'Undo');
    assert.deepEqual(await ids(p), rowIds);
    await command(p, 'Redo');
    assert.deepEqual(await ids(p), columnIds);
    await command(p, 'Save template revision');
    await until(async () => !/^●/.test(await p.title()));
    await command(p, 'Close template editor');
    await editor(p).waitFor({ state: 'hidden' });
    assert.equal(await readFile(project(), 'utf8'), originalProject);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('delayed table previews commit the release position while Escape and invalid boundary drags preserve redo', async () => {
    const p = await page({ electrical: false });
    await open(p);
    const original = await bounds(p);
    const s = await scale(p);
    let delayed = false;
    await p.route('**/bridge', async (route) => {
      const request = route.request().postDataJSON();
      if (request.type === 'template_table_preview' && !delayed) {
        delayed = true;
        await new Promise((r) => setTimeout(r, 250));
      }
      await route.continue();
    });
    const move = await point(editor(p).locator('[data-control="move"]'));
    await drag(p, move, { x: move.x - 6 * s, y: move.y - 4 * s });
    const moved = await bounds(p);
    assert.equal(delayed, true);
    assert.ok(Math.abs(moved[0] - (original[0] - 6)) < 0.002);
    await command(p, 'Undo');
    assert.deepEqual(await bounds(p), original);
    delayed = false;
    const cancel = await point(editor(p).locator('[data-control="move"]'));
    await p.mouse.move(cancel.x, cancel.y);
    await p.mouse.down();
    await p.mouse.move(cancel.x - 4 * s, cancel.y - 4 * s);
    await p.keyboard.press('Escape');
    await p.mouse.up();
    await ready(p);
    assert.deepEqual(await bounds(p), original);
    const seam = await point(
      editor(p).locator('.table-edge[data-axis="x"][data-from="90000"]').first(),
    );
    await drag(p, seam, { x: seam.x - 100 * s, y: seam.y });
    assert.deepEqual(await bounds(p), original);
    await command(p, 'Redo');
    assert.deepEqual(await bounds(p), moved);
    await command(p, 'Discard template draft');
    await editor(p).waitFor({ state: 'hidden' });
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
