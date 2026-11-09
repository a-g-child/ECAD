import { insertVariable } from './template-variable-helpers.mjs';
/** Template artwork runs through real Rust history, validation and library saves. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
export function registerTemplateArtworkTests({ page, button, until }) {
  test('template artwork keeps multiline variables and coherent vectors through edit, history and save', async () => {
    const p = await page({ electrical: false });
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
      await ready();
    };
    const apply = async () => {
      await editor.getByRole('button', { name: 'Apply properties', exact: true }).click();
      await p.waitForFunction(
        () => document.querySelector('.status-message')?.textContent === 'Template updated',
      );
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
    await editor
      .getByRole('textbox', { name: 'Template name', exact: true })
      .fill('Artwork regression');
    await apply();
    await command('Template text and graphics');
    await editor.getByRole('button', { name: 'Add text', exact: true }).click();
    let detail = editor.locator('.template-artwork > details').first();
    await detail
      .getByRole('textbox', { name: 'Text and variables', exact: true })
      .fill('Company\n');
    await insertVariable(detail, 'company.name');
    await apply();
    const art = editor.locator('[data-template-graphic]');
    assert.equal(await art.count(), 1);
    assert.equal(await art.locator('text').textContent(), 'Company{company.name}');
    assert.equal(await art.locator('text tspan').count(), 2);
    assert.equal(
      await detail.getByRole('textbox', { name: 'Artwork Width (µm)', exact: true }).count(),
      0,
    );
    assert.equal(await art.locator('clipPath').count(), 0);
    // Exercise the shared renderer with a tiny legacy box and resolved, multiline values.
    const measured = await p.evaluate(async () => {
      const { renderTemplateGraphics } = await import('/src/template-graphic-render.ts');
      const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
      document.body.append(svg);
      const graphic = {
        id: 'bounds-probe',
        kind: 'text',
        bounds: ['10000', '10000', '1', '1'],
        contents: [{ kind: 'literal', value: '\nLong resolved company name\n' }],
        font: 'sans-serif',
        text_height: '3500',
        stroke_width: '250',
        colour: '#000000',
        points: [],
      };
      const measure = () => {
        svg.replaceChildren();
        renderTemplateGraphics(svg, [graphic]);
        const box = svg.querySelector('.text-bounds').getBBox();
        const glyph = svg.querySelector('text').getBBox();
        return {
          width: box.width,
          height: box.height,
          encloses:
            box.x <= glyph.x &&
            box.y <= glyph.y &&
            box.x + box.width >= glyph.x + glyph.width &&
            box.y + box.height >= glyph.y + glyph.height,
          clipped: svg.querySelector('text').hasAttribute('clip-path'),
        };
      };
      const first = measure();
      graphic.text_height = '7000';
      const larger = measure();
      graphic.contents = [{ kind: 'literal', value: '' }];
      const empty = measure();
      svg.remove();
      return { first, larger, empty };
    });
    assert.ok(measured.first.encloses && !measured.first.clipped);
    assert.ok(measured.first.width > 20 && measured.first.height > 10);
    assert.ok(measured.larger.width > measured.first.width * 1.9);
    assert.ok(measured.larger.height > measured.first.height * 1.9);
    assert.ok(measured.empty.width > 0 && measured.empty.height > 0);
    await command('Undo');
    assert.equal(await art.count(), 0);
    await command('Redo');
    assert.equal(await art.count(), 1);
    for (const name of ['Add rectangle', 'Add ellipse', 'Add line / polyline']) {
      await editor.getByRole('button', { name, exact: true }).click();
      await apply();
    }
    assert.equal(await art.count(), 4);
    assert.equal(await art.locator('ellipse').count(), 1);
    assert.equal(await art.locator('polyline').count(), 1);
    detail = editor.locator('.template-artwork > details').last();
    await detail.locator(':scope > summary').click();
    await detail
      .getByRole('textbox', { name: 'Local points (µm, x,y per line)', exact: true })
      .fill('0,0\n40000,0\n40000,15000');
    await apply();
    assert.equal(await art.locator('polyline').getAttribute('points'), '10,10 50,10 50,25');
    detail = editor.locator('.template-artwork > details').nth(1);
    await detail.locator(':scope > summary').click();
    await detail
      .getByRole('textbox', { name: 'Artwork Width (µm)', exact: true })
      .fill('999999999');
    await editor.getByRole('button', { name: 'Apply properties', exact: true }).click();
    await until(async () => (await editor.getByRole('alert').innerText()).includes('artwork'));
    assert.equal(await art.count(), 4);
    await editor.getByRole('button', { name: 'Revert fields', exact: true }).click();
    await ready();
    await command('Save template revision');
    await ribbon.getByRole('button', { name: 'Close template editor', exact: true }).click();
    await editor.waitFor({ state: 'hidden' });
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    const select = editor.getByRole('combobox', { name: 'Saved template variants', exact: true });
    const value = await select
      .locator('option')
      .evaluateAll((os) => os.find((o) => o.textContent.includes('Artwork regression'))?.value);
    assert.ok(value);
    await select.selectOption(value);
    await editor.getByRole('button', { name: 'Open selected template', exact: true }).click();
    await ready();
    assert.equal(await art.count(), 4);
    await ribbon.getByRole('button', { name: 'Close template editor', exact: true }).click();
    await editor.waitFor({ state: 'hidden' });
    await p.close();
  });
}
