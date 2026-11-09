/** Contextual structural actions and exact preset/custom paper changes use the real engine. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
export function registerTemplateContextualTools({ page, button, until }) {
  async function setup() {
    const p = await page({ electrical: false });
    const editor = p.getByRole('region', { name: 'Drawing template editor', exact: true });
    const ribbon = p.getByRole('tabpanel', { name: 'Template', exact: true });
    const control = (name) => ribbon.getByRole('button', { name, exact: true });
    const ready = () => until(async () => !(await control('Draw template text').isDisabled()));
    const command = async (name) => {
      await control(name).click();
      if (!['Close template editor', 'Discard template draft'].includes(name)) await ready();
    };
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    await command('New template');
    return { p, editor, ribbon, control, ready, command };
  }
  test('contextual column split commits once and ribbon/palette share rectangular selection eligibility', async () => {
    const { p, editor, control, ready, command } = await setup();
    const cells = editor.locator('.frame-cell');
    const range = async (a, b) => {
      const point = (i) =>
        cells
          .nth(i)
          .locator(':scope > rect')
          .evaluate((n) => {
            const r = n.getBoundingClientRect();
            return { x: r.x + r.width / 2, y: r.y + r.height / 2 };
          });
      const from = await point(a),
        to = await point(b);
      await p.mouse.move(from.x, from.y);
      await p.mouse.down();
      await p.mouse.move(to.x, to.y, { steps: 4 });
      await p.mouse.up();
      await ready();
    };
    assert.equal(await control('Split title-block cell').isDisabled(), true);
    await range(0, 2);
    await control('Split title-block cell').click();
    const d = p.getByRole('dialog', { name: 'Split title-block cell', exact: true });
    await d.waitFor();
    assert.equal(await d.getByRole('combobox').count(), 0);
    assert.equal(await d.getByRole('spinbutton').count(), 2);
    await d.getByRole('button', { name: 'Apply table edit', exact: true }).click();
    await d.waitFor({ state: 'hidden' });
    await ready();
    assert.equal(await cells.count(), 10);
    const splitIds = await cells.evaluateAll((nodes) => nodes.map((n) => n.dataset.field));
    await command('Undo');
    assert.equal(await cells.count(), 8);
    await command('Redo');
    assert.deepEqual(
      await cells.evaluateAll((nodes) => nodes.map((n) => n.dataset.field)),
      splitIds,
    );
    await range(0, 1);
    await until(async () => !(await control('Merge title-block cells').isDisabled()));
    assert.equal(await control('Split title-block cell').isDisabled(), true);
    await editor.locator('svg').focus();
    await p.keyboard.press(`${process.platform === 'darwin' ? 'Meta' : 'Control'}+Shift+p`);
    const search = p.getByRole('textbox', { name: 'Search commands', exact: true });
    await search.fill('template.cell.split');
    assert.equal(await p.locator('.palette-item').first().isDisabled(), true);
    await p.keyboard.press('Escape');
    await command('Discard template draft');
    await editor.waitFor({ state: 'hidden' });
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('paper presets preserve exact custom values, never scale cells and survive library reopen', async () => {
    const { p, editor, ready, command } = await setup();
    const field = (name) => editor.getByRole('textbox', { name, exact: true });
    const choice = (name) => editor.getByRole('combobox', { name, exact: true });
    const extents = () =>
      editor
        .locator('.frame-cell > rect')
        .evaluateAll((nodes) =>
          nodes.map((n) => [n.getAttribute('width'), n.getAttribute('height')]),
        );
    assert.equal(await choice('Paper family').inputValue(), 'ISO A');
    assert.equal(await choice('Paper size').inputValue(), 'A4');
    assert.equal(await field('Paper width (µm)').isVisible(), false);
    const original = await extents();
    await choice('Paper size').selectOption('A3');
    await ready();
    assert.deepEqual(await extents(), original);
    await choice('Orientation').selectOption('Portrait');
    await ready();
    assert.deepEqual(await extents(), original);
    await choice('Paper family').selectOption('US');
    await ready();
    assert.equal(await choice('Paper size').inputValue(), 'Letter / ANSI A');
    await choice('Orientation').selectOption('Landscape');
    await ready();
    await choice('Paper family').selectOption('Custom');
    await field('Paper width (µm)').fill('300001');
    await field('Paper width (µm)').press('Enter');
    await ready();
    await field('Paper height (µm)').fill('210007');
    await field('Paper height (µm)').press('Enter');
    await ready();
    await choice('Paper family').selectOption('ISO A');
    await ready();
    await command('Undo');
    assert.equal(await choice('Paper family').inputValue(), 'Custom');
    assert.equal(await field('Paper width (µm)').inputValue(), '300001');
    assert.equal(await field('Paper height (µm)').inputValue(), '210007');
    await field('Template name').fill('Custom paper regression');
    await field('Template name').press('Control+Enter');
    await ready();
    await command('Save template revision');
    await command('Close template editor');
    await editor.waitFor({ state: 'hidden' });
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    await editor
      .getByRole('combobox', { name: 'Saved template variants', exact: true })
      .selectOption({ label: 'Custom paper regression' });
    await editor.getByRole('button', { name: 'Open selected template', exact: true }).click();
    await ready();
    assert.equal(await choice('Paper family').inputValue(), 'Custom');
    assert.equal(await field('Paper width (µm)').inputValue(), '300001');
    assert.equal(await field('Paper height (µm)').inputValue(), '210007');
    assert.deepEqual(await extents(), original);
    await command('Close template editor');
    await editor.waitFor({ state: 'hidden' });
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
