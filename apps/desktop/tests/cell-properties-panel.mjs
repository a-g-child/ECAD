/** Cell inspector groups preserve edit targets, workspace state and compact layout. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { resolve } from 'node:path';
import { mkdir } from 'node:fs/promises';
export function registerCellPropertiesPanelTests({ page, button, until, root }) {
  test('cell inspector shows applicable compact sections and retains collapsed groups through edits and selection', async () => {
    const p = await page({ electrical: false });
    const editor = p.getByRole('region', { name: 'Drawing template editor', exact: true });
    const ribbon = p.getByRole('tabpanel', { name: 'Template', exact: true });
    const form = editor.locator('.template-properties');
    const section = (id) => form.locator(`.property-section[data-section="${id}"]`);
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
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    await command('New template');
    const cells = editor.locator('.template-structure [data-template-cell]');
    await cells.first().click();
    await ready();
    assert.deepEqual(
      await form
        .locator('.property-section')
        .evaluateAll((nodes) =>
          nodes.filter((n) => n.checkVisibility()).map((n) => n.dataset.section),
        ),
      ['cell-text', 'cell-layout', 'cell-style'],
    );
    assert.equal(await form.locator('.picture-properties').count(), 0);
    const caption = editor.getByRole('textbox', { name: 'Cell caption', exact: true });
    const metrics = await form.evaluate((form) => {
      const caption = form.querySelector('[aria-label="Cell caption"]').getBoundingClientRect();
      const text = form.querySelector('[data-section="cell-text"]').getBoundingClientRect();
      const layout = form.querySelector('[data-section="cell-layout"]').getBoundingClientRect();
      return {
        captionWidth: caption.width,
        textWidth: text.width,
        captionHeight: caption.height,
        gap: layout.top - text.bottom,
        horizontalOverflow: form.scrollWidth > form.clientWidth + 1,
      };
    });
    assert.ok(metrics.captionWidth > metrics.textWidth * 0.85);
    assert.ok(metrics.captionHeight <= 48);
    assert.ok(metrics.gap >= 0 && metrics.gap <= 10);
    assert.equal(metrics.horizontalOverflow, false);
    await section('cell-style').locator('summary').first().click();
    assert.equal(await section('cell-style').getAttribute('open'), null);
    await caption.fill('Compact cell caption');
    await caption.press('Control+Enter');
    await ready();
    assert.equal(await section('cell-style').getAttribute('open'), null);
    await cells.nth(1).click();
    await ready();
    assert.equal(await section('cell-style').getAttribute('open'), null);
    await cells.first().click();
    await ready();
    assert.equal(await caption.inputValue(), 'Compact cell caption');
    await caption.fill('Pending grouped edit');
    await section('cell-text').locator('summary').first().click();
    await until(async () => (await section('cell-text').getAttribute('open')) === null);
    await section('cell-text').locator('summary').first().click();
    assert.equal(await caption.inputValue(), 'Pending grouped edit');
    await caption.press('Control+Enter');
    await ready();
    await mkdir(resolve(root, 'target/ui'), { recursive: true });
    await p.screenshot({ path: resolve(root, 'target/ui/cell-properties-text.png') });
    // Picture insertion remains the registered ribbon action, and switches the inspector type.
    await command('Insert template picture');
    const dialog = p.getByRole('dialog', { name: 'Insert picture', exact: true });
    await dialog.waitFor();
    await dialog
      .getByLabel('Picture file', { exact: true })
      .setInputFiles(resolve(root, 'apps/desktop/tests/fixtures/picture.svg'));
    await until(async () => !!(await dialog.locator('.picture-preview').getAttribute('src')));
    await dialog.getByRole('button', { name: 'Apply picture', exact: true }).click();
    await dialog.waitFor({ state: 'hidden' });
    await ready();
    assert.equal(await section('cell-picture').count(), 1);
    assert.equal(await section('cell-text').count(), 0);
    assert.equal(await section('cell-layout').count(), 0);
    assert.equal(await section('cell-style').count(), 0);
    await p.screenshot({ path: resolve(root, 'target/ui/cell-properties-picture.png') });
    await command('Undo');
    assert.equal(await caption.inputValue(), 'Pending grouped edit');
    assert.equal(await form.locator('.picture-properties').count(), 0);
    await section('cell-style').locator('summary').first().click();
    const font = editor.getByRole('textbox', { name: 'Content font', exact: true });
    await font.fill('monospace');
    const scroll = await form.evaluate((form) => form.scrollTop);
    await font.press('Control+Enter');
    await ready();
    assert.ok(Math.abs((await form.evaluate((form) => form.scrollTop)) - scroll) <= 2);
    await command('Save template revision');
    await command('Close template editor');
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
