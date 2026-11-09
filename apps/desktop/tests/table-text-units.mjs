/** Real browser unit boundaries and independent caption/content style transactions. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
export function registerTableTextUnits({ page, button, until }) {
  test('length conversion is exact, defaults to mm and preserves rounded inch displays without edits', async () => {
    const p = await page({ electrical: false, units: null });
    const result = await p.evaluate(async () => {
      const d = await import('/src/dimensions.ts');
      const input = document.createElement('input');
      input.value = '1';
      document.body.append(input);
      d.dimension(input);
      const initial = d.lengthUnit();
      d.setLengthUnit('in');
      const shown = input.value,
        retained = d.readDimension(input);
      const exact = d.parseLength('0.125 in');
      const signed = d.parseLength('-2.5 mm');
      const huge = d.parseLength('9007199254740.991 mm');
      let rejected = false;
      try {
        d.parseLength('0.0001 mm');
      } catch {
        rejected = true;
      }
      input.value = '0.0001 mm';
      let switchRejected = false;
      try {
        d.setLengthUnit('mm');
      } catch {
        switchRejected = true;
      }
      input.remove();
      return { initial, shown, retained, exact, signed, huge, rejected, switchRejected };
    });
    assert.equal(result.initial, 'mm');
    assert.equal(result.shown, '0.000039');
    assert.equal(result.retained, '1');
    assert.equal(result.exact, '3175');
    assert.equal(result.signed, '-2500');
    assert.equal(result.huge, '9007199254740991');
    assert.ok(result.rejected && result.switchRejected);
    await p.reload();
    assert.equal(
      await p.evaluate(async () => (await import('/src/dimensions.ts')).lengthUnit()),
      'in',
    );
    await p.close();
  });
  test('template cell caption and content formats inherit, bulk edit, reset, undo and reopen independently in mm', async () => {
    const p = await page({ electrical: false, units: 'mm' });
    const editor = p.getByRole('region', { name: 'Drawing template editor', exact: true });
    const ribbon = p.getByRole('tabpanel', { name: 'Template', exact: true });
    const ready = () =>
      until(
        async () =>
          !(await ribbon
            .getByRole('button', { name: 'Draw template text', exact: true })
            .isDisabled()),
      );
    const command = async (name) => {
      await ribbon.getByRole('button', { name, exact: true }).click();
      if (name === 'Close template editor') await editor.waitFor({ state: 'hidden' });
      else await ready();
    };
    const field = (name) => editor.getByRole('textbox', { name, exact: true });
    const edit = async (name, value) => {
      await field(name).fill(value);
      await field(name).press('Control+Enter');
      await ready();
    };
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    await command('New template');
    await edit('Template name', 'Cell fonts and mm');
    assert.equal(await field('General margin (mm)').inputValue(), '10');
    await edit('Caption default height (mm)', '2');
    await edit('Content default font', 'monospace');
    const cells = editor.locator('.frame-cell');
    const value = cells.first().locator('.frame-value');
    assert.equal(await value.getAttribute('font-family'), 'monospace');
    const points = await cells.evaluateAll((nodes) =>
      nodes.slice(0, 2).map((n) => {
        const r = n.querySelector('rect').getBoundingClientRect();
        return { x: r.x + r.width / 2, y: r.y + r.height / 2 };
      }),
    );
    await p.mouse.click(points[0].x, points[0].y);
    await ready();
    await edit('Caption font', 'serif');
    await editor.getByRole('combobox', { name: 'Caption bold', exact: true }).selectOption('true');
    await ready();
    assert.equal(await cells.first().locator('text').first().getAttribute('font-weight'), 'bold');
    assert.equal(await value.getAttribute('font-weight'), 'normal');
    await editor
      .getByRole('combobox', { name: 'Content horizontal alignment', exact: true })
      .selectOption('right');
    await ready();
    assert.equal(await value.getAttribute('text-anchor'), 'end');
    const rightAligned = await cells.first().evaluate((cell) => {
      const box = cell.querySelector('rect');
      return (
        Number(cell.querySelector('.frame-value tspan').getAttribute('x')) ===
        Number(box.getAttribute('x')) + Number(box.getAttribute('width')) - 1
      );
    });
    assert.ok(rightAligned);

    await edit('Content height (mm)', '0.125 in');
    assert.equal(await value.getAttribute('font-size'), '3.175');
    assert.equal(await cells.first().locator('text').first().getAttribute('font-family'), 'serif');
    await p.mouse.move(points[0].x, points[0].y);
    await p.mouse.down();
    await p.mouse.move(points[1].x, points[1].y, { steps: 5 });
    await p.mouse.up();
    await ready();
    assert.equal(await field('Content height (mm)').getAttribute('placeholder'), 'Mixed');
    await edit('Content height (mm)', '3');
    for (const n of [0, 1])
      assert.equal(await cells.nth(n).locator('.frame-value').getAttribute('font-size'), '3');
    await command('Undo');
    assert.equal(await cells.first().locator('.frame-value').getAttribute('font-size'), '3.175');
    assert.equal(await cells.nth(1).locator('.frame-value').getAttribute('font-size'), '2.5');
    await command('Redo');
    await editor.getByRole('button', { name: 'Reset content height', exact: true }).click();
    await ready();
    for (const n of [0, 1])
      assert.equal(await cells.nth(n).locator('.frame-value').getAttribute('font-size'), '2.5');
    await command('Save template revision');
    await command('Close template editor');
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor
      .getByRole('combobox', { name: 'Saved template variants', exact: true })
      .selectOption({ label: 'Cell fonts and mm' });
    await editor.getByRole('button', { name: 'Open selected template', exact: true }).click();
    await ready();
    assert.equal(await cells.first().locator('text').first().getAttribute('font-family'), 'serif');
    assert.equal(
      await cells.first().locator('.frame-value').getAttribute('font-family'),
      'monospace',
    );
    await editor
      .getByRole('combobox', { name: 'Measurement units', exact: true })
      .selectOption('in');
    assert.equal(await field('General margin (in)').inputValue(), '0.393701');
    await command('Close template editor');
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
