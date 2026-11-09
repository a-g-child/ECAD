/** Field completion uses real validated Rust commands and template history. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
export function registerPropertyCompletionTests({ page, button, until }) {
  test('template fields complete on Enter and Tab, protect invalid selection and preserve multiline/IME input', async () => {
    const p = await page({ electrical: false });
    const editor = p.getByRole('region', { name: 'Drawing template editor', exact: true });
    const ribbon = p.getByRole('tabpanel', { name: 'Template', exact: true });
    const field = (name) => editor.getByRole('textbox', { name, exact: true });
    const ready = () =>
      until(
        async () =>
          !(await ribbon
            .getByRole('button', { name: 'Draw template text', exact: true })
            .isDisabled()),
      );
    const command = async (name) => {
      await ribbon.getByRole('button', { name, exact: true }).click();
      if (name !== 'Discard template draft') await ready();
    };
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    await command('New template');
    const original = await field('General margin (µm)').inputValue();
    await field('General margin (µm)').fill('15000');
    await field('General margin (µm)').press('Enter');
    await ready();
    assert.equal(await field('Left margin (µm)').inputValue(), '15000');
    await command('Undo');
    assert.equal(await field('General margin (µm)').inputValue(), original);
    // Validation failure keeps the old target and staged input; Escape preserves redo history.
    await field('Paper width (µm)').fill('0');
    await editor.locator('[data-template-cell]').first().click();
    await until(async () => (await editor.getByRole('alert').innerText()).length > 0);
    assert.equal(await field('Paper width (µm)').inputValue(), '0');
    assert.equal(await field('Cell caption').count(), 0);
    await field('Paper width (µm)').press('Escape');
    await ready();
    await command('Redo');
    assert.equal(await field('General margin (µm)').inputValue(), '15000');
    await editor.locator('[data-template-cell]').first().click();
    const oldCaption = await field('Cell caption').inputValue();
    await field('Cell caption').fill('First');
    await field('Cell caption').press('End');
    await field('Cell caption').press('Enter');
    await field('Cell caption').press('x');
    assert.equal(await field('Cell caption').inputValue(), 'First\nx');
    await field('Cell caption').press('Control+Enter');
    await ready();
    await command('Undo');
    assert.equal(await field('Cell caption').inputValue(), oldCaption);
    await command('Redo');
    assert.equal(await field('Cell caption').inputValue(), 'First\nx');
    // Tab completes the field and restores the destination focus after the inspector is rebuilt.
    await field('Cell caption').fill('Tab complete');
    await field('Cell caption').press('Tab');
    await ready();
    assert.equal(
      await editor.locator('.template-structure [data-template-cell]').first().innerText(),
      'Tab complete',
    );
    assert.equal(
      await p.evaluate(() => document.activeElement?.closest('.template-properties') !== null),
      true,
    );
    // Composition Enter must not publish an intermediate value.
    await field('Cell caption').dispatchEvent('compositionstart');
    await field('Cell caption').fill('Composed');
    await field('Cell caption').press('Control+Enter');
    assert.equal(
      await editor.locator('.template-structure [data-template-cell]').first().innerText(),
      'Tab complete',
    );
    await field('Cell caption').dispatchEvent('compositionend');
    await editor.locator('[data-template-cell]').nth(1).click();
    await ready();
    await until(
      async () =>
        (await editor.locator('.template-structure [data-template-cell]').first().innerText()) ===
        'Composed',
    );
    assert.notEqual(await field('Cell caption').inputValue(), 'Composed');
    // Delayed completion commits once to the old cell while the last canvas selection wins.
    const cells = editor.locator('.frame-cell');
    const originalId = await editor
      .locator('[data-selected-cell]')
      .getAttribute('data-selected-cell');
    const destination = await cells.last().getAttribute('data-field');
    const location = async (cell) =>
      cell.locator(':scope > rect').evaluate((n) => {
        const r = n.getBoundingClientRect();
        return { x: r.x + r.width * 0.75, y: r.y + r.height * 0.75 };
      });
    const first = await location(cells.first()),
      last = await location(cells.last());
    let commits = 0;
    await p.route('**/bridge', async (route) => {
      const request = route.request().postDataJSON();
      if (request.type === 'template_operation' && request.action === 'template.properties.apply') {
        commits++;
        await new Promise((resolve) => setTimeout(resolve, 300));
      }
      await route.continue();
    });
    await field('Cell caption').fill('Original target only');
    await p.mouse.click(first.x, first.y);
    await p.mouse.click(last.x, last.y);
    await ready();
    await until(
      async () =>
        (await editor.locator('[data-selected-cell]').getAttribute('data-selected-cell')) ===
        destination,
    );
    await p.unroute('**/bridge');
    assert.equal(commits, 1);
    assert.equal(
      await editor.locator(`[data-template-cell="${originalId}"]`).innerText(),
      'Original target only',
    );
    assert.notEqual(await field('Cell caption').inputValue(), 'Original target only');
    await command('Discard template draft');
    await editor.waitFor({ state: 'hidden' });
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
