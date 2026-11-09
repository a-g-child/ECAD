/** Shared renderer and real template history for caption flow and block justification. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
export function registerCellTextLayoutTests({ page, button, until }) {
  test('cell block rendering handles justification, inline mixed fonts, multiline blanks and overflow', async () => {
    const p = await page({ electrical: false });
    const cases = await p.evaluate(async () => {
      const { renderFrame } = await import('/src/frame.ts');
      const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
      document.body.append(svg);
      const style = {
        font: null,
        height: null,
        bold: null,
        italic: null,
        colour: null,
        horizontal: null,
        vertical: null,
      };
      function draw(mode, horizontal, vertical, caption = 'Caption', value = 'Value', width = 100) {
        svg.replaceChildren();
        renderFrame(svg, {
          template: 'probe',
          border: { x: 0, y: 0, width: 200, height: 100 },
          graphics: [],
          title_block_too_small: false,
          font: 'sans-serif',
          text_height: 3,
          stroke_width: 0.25,
          colour: '#000',
          cells: [
            {
              key: 'cell',
              caption,
              value,
              bounds: { x: 20, y: 20, width, height: 30 },
              text_styles: {
                caption: { ...style, font: 'serif', bold: true, height: '1500' },
                content: { ...style, font: 'monospace', height: '3000' },
                layout: { caption_position: mode, horizontal, vertical, separator: ' - ' },
              },
            },
          ],
        });
        const block = svg.querySelector('.cell-text-layout'),
          b = block.getBBox(),
          t = block.transform.baseVal.consolidate().matrix;
        const cap = svg.querySelector('.frame-caption'),
          val = svg.querySelector('.frame-value');
        return {
          mode,
          horizontal,
          vertical,
          left: b.x + t.e,
          right: b.x + b.width + t.e,
          top: b.y + t.f,
          bottom: b.y + b.height + t.f,
          caption: cap.textContent,
          value: val.textContent,
          capLines: cap.querySelectorAll('tspan').length,
          valLines: val.querySelectorAll('tspan').length,
          capY: cap.querySelector('tspan:last-child').getAttribute('y'),
          valY: val.querySelector('tspan').getAttribute('y'),
          separator: svg.querySelector('.frame-separator')?.textContent ?? null,
          overflow: !!svg.querySelector('.frame-overflow'),
        };
      }
      const results = [];
      for (const mode of ['above', 'inline'])
        for (const h of ['left', 'center', 'right'])
          for (const v of ['top', 'middle', 'bottom']) results.push(draw(mode, h, v));
      results.push(
        draw('inline', 'center', 'middle', 'First\nCaption', '\nValue\n'),
        draw('inline', 'left', 'top', '', 'Only value'),
        draw('inline', 'left', 'top', 'Only caption', ''),
        draw('inline', 'left', 'top', 'Caption', 'Full overflowing content', 5),
      );
      svg.remove();
      return results;
    });
    for (const c of cases.slice(0, 18)) {
      if (c.horizontal === 'left') assert.ok(Math.abs(c.left - 21) < 0.01);
      if (c.horizontal === 'right') assert.ok(Math.abs(c.right - 119) < 0.01);
      if (c.horizontal === 'center') assert.ok(Math.abs((c.left + c.right) / 2 - 70) < 0.01);
      assert.ok(c.top >= 20.99 && c.bottom <= 49.01);
      assert.equal(c.caption, 'Caption');
      assert.equal(c.value, 'Value');
      assert.equal(c.capLines, 1);
      assert.equal(c.valLines, 1);
      if (c.mode === 'inline') {
        assert.equal(c.separator, ' - ');
        assert.equal(c.capY, c.valY);
      } else {
        assert.equal(c.separator, null);
        assert.ok(Number(c.valY) > Number(c.capY));
      }
    }
    for (const mode of ['above', 'inline']) {
      const rows = cases.filter((c) => c.mode === mode && c.horizontal === 'left');
      assert.ok(rows[0].top < rows[1].top && rows[1].top < rows[2].top);
    }
    assert.equal(cases[18].capLines, 2);
    assert.equal(cases[18].valLines, 3);
    assert.equal(cases[19].separator, null);
    assert.equal(cases[20].separator, null);
    assert.ok(cases[21].overflow);
    assert.equal(cases[21].value, 'Full overflowing content');
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('cell layout defaults and range overrides reset undo and survive publication', async () => {
    const p = await page({ electrical: false });
    const editor = p.getByRole('region', { name: 'Drawing template editor', exact: true }),
      ribbon = p.getByRole('tabpanel', { name: 'Template', exact: true });
    const ready = () =>
      until(
        async () =>
          !(await ribbon
            .getByRole('button', { name: 'Draw template text', exact: true })
            .isDisabled()),
      );
    const command = async (name) => {
      await ribbon.getByRole('button', { name, exact: true }).click();
      if (name !== 'Close template editor') await ready();
    };
    const choose = async (name, value) => {
      await editor.getByRole('combobox', { name, exact: true }).selectOption(value);
      await ready();
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
    await edit('Template name', 'Caption layout regression');
    await choose('Cell default caption position', 'inline');
    await edit('Cell default caption separator', ' - ');
    const cell = editor.locator('.frame-cell').first();
    assert.equal(await cell.locator('.frame-separator').textContent(), ' - ');
    await editor.locator('.template-structure [data-template-cell]').first().click();
    await ready();
    await choose('Cell caption position', 'above');
    assert.equal(await cell.locator('.frame-separator').count(), 0);
    await choose('Cell caption position', 'inline');
    await edit('Cell caption separator', ': ');
    await edit('Cell caption separator', '');
    assert.equal(await field('Cell caption separator').getAttribute('placeholder'), 'No separator');
    assert.equal(await cell.locator('.frame-separator').textContent(), '');
    await command('Undo');
    await choose('Cell horizontal justification', 'right');
    await choose('Cell vertical justification', 'bottom');
    assert.equal(await cell.locator('.frame-separator').textContent(), ': ');
    await editor.getByRole('button', { name: 'Default separator', exact: true }).click();
    await ready();
    assert.equal(await cell.locator('.frame-separator').textContent(), ' - ');
    await command('Undo');
    assert.equal(await cell.locator('.frame-separator').textContent(), ': ');
    await editor.getByRole('button', { name: 'Table 1 · select all cells', exact: true }).click();
    await ready();
    assert.equal(
      await editor
        .getByRole('combobox', { name: 'Cell horizontal justification', exact: true })
        .inputValue(),
      '@mixed',
    );
    await choose('Cell horizontal justification', 'center');
    await choose('Cell vertical justification', 'middle');
    await command('Undo');
    await command('Redo');
    await command('Save template revision');
    await command('Close template editor');
    await editor.waitFor({ state: 'hidden' });
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    await editor
      .getByRole('combobox', { name: 'Saved template variants', exact: true })
      .selectOption({ label: 'Caption layout regression' });
    await editor.getByRole('button', { name: 'Open selected template', exact: true }).click();
    await ready();
    await editor.locator('.template-structure [data-template-cell]').first().click();
    await ready();
    assert.equal(await field('Cell caption separator').inputValue(), ': ');
    for (const [name, value] of [
      ['Cell caption position', 'inline'],
      ['Cell horizontal justification', 'center'],
      ['Cell vertical justification', 'middle'],
    ])
      assert.equal(await editor.getByRole('combobox', { name, exact: true }).inputValue(), value);
    assert.equal(await cell.locator('.frame-separator').textContent(), ': ');
    await command('Close template editor');
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
