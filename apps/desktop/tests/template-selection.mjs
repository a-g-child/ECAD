/** Reproducer: artwork beneath unfilled cells must not steal table selection or inspector focus. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
export function registerTemplateSelectionTests({ page, button, until }) {
  test('template contextual selection switches between overlapping artwork, cells and paper without losing fields', async () => {
    const p = await page({ electrical: false });
    const editor = p.getByRole('region', { name: 'Drawing template editor', exact: true });
    const ribbon = p.getByRole('tabpanel', { name: 'Template', exact: true });
    const svg = editor.locator('.template-preview');
    const field = (name) => editor.getByRole('textbox', { name, exact: true });
    const ready = () =>
      until(
        async () =>
          !(await ribbon
            .getByRole('button', { name: 'Draw template text', exact: true })
            .isDisabled()),
      );
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
    const click = async (x, y, options) => {
      const pnt = await point(x, y);
      await p.mouse.click(pnt.x, pnt.y, options);
      await ready();
    };
    const command = async (name) => {
      await ribbon.getByRole('button', { name, exact: true }).click();
    };
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    await command('New template');
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'New template draft',
    );
    await ready();
    await command('Draw template rectangle');
    await until(async () => (await svg.getAttribute('data-artwork-tool')) === 'rectangle');
    const a = await point(100, 140),
      b = await point(290, 205);
    await p.mouse.move(a.x, a.y);
    await p.mouse.down();
    await p.mouse.move(b.x, b.y);
    await p.mouse.up();
    await ready();
    assert.equal(await field('Artwork Width (µm)').count(), 1);
    assert.equal(await field('Template name').count(), 0);
    assert.equal(await editor.locator('[data-artwork-selected]').count(), 1);
    const artBefore = await editor.locator('[data-template-graphic] > rect').getAttribute('x');
    const cell = editor.locator('.frame-cell').first();
    const cellPoint = await cell.locator(':scope > rect').evaluate((n) => {
      const p = n.ownerSVGElement.createSVGPoint();
      p.x = Number(n.getAttribute('x')) + Number(n.getAttribute('width')) * 0.8;
      p.y = Number(n.getAttribute('y')) + Number(n.getAttribute('height')) * 0.8;
      const r = p.matrixTransform(n.getScreenCTM());
      return { x: r.x, y: r.y };
    });
    // Cancelling a range gesture restores the previous artwork selection, not stale table state.
    await p.mouse.move(cellPoint.x, cellPoint.y);
    await p.mouse.down();
    await p.mouse.move(cellPoint.x + 5, cellPoint.y + 5);
    await p.keyboard.press('Escape');
    await p.mouse.up();
    await ready();
    assert.equal(await field('Artwork Width (µm)').count(), 1);
    assert.equal(await editor.locator('.table-handles').count(), 0);
    // Empty cell interior overlies the rectangle: this selected artwork before the fix.
    await p.mouse.click(cellPoint.x, cellPoint.y);
    await ready();
    assert.equal(await field('Cell caption').count(), 1);
    assert.equal(await field('Artwork Width (µm)').count(), 0);
    assert.equal(await editor.locator('[data-artwork-selected]').count(), 0);
    assert.equal(await editor.locator('[data-selected-cell]').count(), 1);
    await field('Cell caption').fill('Contextual cell');
    await click(40, 60); // Complete the original cell edit before selecting paper.
    await until(async () => (await field('Template name').count()) === 1);
    await editor
      .locator('.template-structure')
      .getByRole('button', { name: 'Artwork 1 · rectangle', exact: true })
      .click();
    assert.equal(await field('Artwork Width (µm)').count(), 1);
    assert.equal(await field('Cell caption').count(), 0);
    assert.equal(await editor.locator('.table-handles').count(), 0);
    await p.mouse.click(cellPoint.x, cellPoint.y, { button: 'right' });
    await ready();
    assert.equal(await field('Cell caption').inputValue(), 'Contextual cell');
    await click(105, 145, { button: 'right' });
    assert.equal(await field('Artwork Width (µm)').count(), 1);
    await click(40, 60);
    assert.equal(await field('Template name').count(), 1);
    assert.equal(await field('Cell caption').count(), 0);
    assert.equal(await editor.locator('[data-artwork-selected],.table-handles').count(), 0);
    await p.mouse.click(cellPoint.x, cellPoint.y);
    await ready();
    await p.keyboard.press('Escape');
    assert.equal(await field('Template name').count(), 1);
    assert.equal(
      await editor.locator('[data-template-graphic] > rect').getAttribute('x'),
      artBefore,
    );
    await command('Undo');
    await ready(); // Selection-only operations did not create undo entries.
    await p.mouse.click(cellPoint.x, cellPoint.y);
    await ready();
    assert.notEqual(await field('Cell caption').inputValue(), 'Contextual cell');
    await command('Redo');
    await ready();
    assert.equal(await field('Cell caption').inputValue(), 'Contextual cell');
    await command('Save template revision');
    await ready();
    await command('Close template editor');
    await editor.waitFor({ state: 'hidden' });
    await p.close();
  });
}
