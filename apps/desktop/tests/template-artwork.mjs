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
            .getByRole('button', { name: 'Split title-block cell', exact: true })
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
    let detail = editor.locator('.template-artwork details').first();
    await detail.getByRole('textbox', { name: 'Content 1 text', exact: true }).fill('Company\n');
    await detail.getByRole('button', { name: 'Add text content', exact: true }).click();
    await detail
      .getByRole('combobox', { name: 'Content 2 type', exact: true })
      .selectOption('field');
    await detail
      .getByRole('combobox', { name: 'Content 2 variable', exact: true })
      .selectOption('company.name');
    await apply();
    const art = editor.locator('[data-template-graphic]');
    assert.equal(await art.count(), 1);
    assert.equal(await art.locator('text').textContent(), 'Company{company.name}');
    assert.equal(await art.locator('text tspan').count(), 2);
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
    detail = editor.locator('.template-artwork details').last();
    await detail.locator('summary').click();
    await detail
      .getByRole('textbox', { name: 'Local points (x,y per line)', exact: true })
      .fill('0,0\n40000,0\n40000,15000');
    await apply();
    assert.equal(await art.locator('polyline').getAttribute('points'), '10,10 50,10 50,25');
    detail = editor.locator('.template-artwork details').first();
    await detail.locator('summary').click();
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
