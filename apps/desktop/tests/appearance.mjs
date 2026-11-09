/** Shared choices preserve owner-specific commits, exact custom values and fixed pane bounds. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
export function registerAppearanceTests({ page, button, until, bounds, pressAction }) {
  test('shared appearance choices preserve schematic inheritance, mixed edits and default cancellation', async () => {
    const p = await page();
    const before = await bounds(p);
    const route = 'route:00000000000000000000000000000001';
    const label = 'label:00000000000000000000000000000001';
    await p.locator(`.tree-item[data-object="${label}"]`).click();
    const font = p.locator('[data-property="font_family"]');
    await font.locator('select').selectOption('monospace');
    await until(async () =>
      (await font.locator('textarea').getAttribute('title')).includes('Object override'),
    );
    assert.match(await font.locator('textarea').getAttribute('title'), /Object override/);
    await p
      .getByRole('combobox', { name: 'Use default text formatting', exact: true })
      .selectOption('font');
    await until(async () =>
      (await font.locator('textarea').getAttribute('title')).includes('Project defaults'),
    );
    await pressAction(p, 'Undo', 'Undo complete');
    assert.match(await font.locator('textarea').getAttribute('title'), /Object override/);
    await pressAction(p, 'Undo', 'Undo complete');
    assert.match(await font.locator('textarea').getAttribute('title'), /Project defaults/);
    await p.locator(`.tree-item[data-object="${route}"]`).click({ modifiers: ['Shift'] });
    const colour = p.locator('[data-property="drawing_colour"]');
    await until(
      async () =>
        (await colour.locator('input:not([type="color"])').getAttribute('placeholder')) === 'Mixed',
    );
    assert.equal(
      await colour.locator('input:not([type="color"])').getAttribute('placeholder'),
      'Mixed',
    );
    const picker = colour.locator('input[type="color"]');
    await picker.evaluate((n) => {
      n.value = '#123456';
      n.dispatchEvent(new Event('input', { bubbles: true }));
    });
    assert.equal(await colour.locator('input:not([type="color"])').inputValue(), '');
    await picker.evaluate((n) => {
      n.value = '#0000ff';
      n.dispatchEvent(new Event('change', { bubbles: true }));
    });
    await until(
      async () =>
        (await p.locator(`svg [data-object="${route}"] .ink`).getAttribute('stroke')) === '#0000ff',
    );
    for (const [id, attr] of [
      [route, 'stroke'],
      [label, 'fill'],
    ])
      assert.equal(await p.locator(`svg [data-object="${id}"] .ink`).getAttribute(attr), '#0000ff');
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(
      await colour.locator('input:not([type="color"])').getAttribute('placeholder'),
      'Mixed',
    );
    await button(p, 'Drawing defaults and styles').click();
    const dialog = p.getByRole('dialog', { name: 'Drawing defaults', exact: true });
    const exact = dialog.getByRole('textbox', { name: 'Default font family', exact: true });
    await exact.fill('Custom\nFamily');
    assert.equal(
      await dialog.getByRole('combobox', { name: 'Default font family choices' }).inputValue(),
      '',
    );
    await dialog
      .getByRole('combobox', { name: 'Default font family choices' })
      .selectOption('serif');
    assert.equal(await exact.inputValue(), 'serif');
    const defaultColour = dialog.getByRole('textbox', {
      name: 'Default output colour',
      exact: true,
    });
    const defaultPicker = dialog.getByLabel('Default output colour picker', { exact: true });
    await defaultColour.fill('#aBcD12');
    assert.equal(await defaultPicker.inputValue(), '#abcd12');
    await defaultPicker.evaluate((n) => {
      n.value = '#654321';
      n.dispatchEvent(new Event('change', { bubbles: true }));
    });
    assert.equal(await defaultColour.inputValue(), '#654321');
    await dialog.getByRole('button', { name: 'Cancel', exact: true }).click();
    assert.equal(await button(p, 'Redo').isDisabled(), false);
    assert.deepEqual(await bounds(p), before);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
export function registerTemplateAppearanceTests({ page, button, until }) {
  test('template appearance composite fields repair invalid values and commit one history step', async () => {
    const p = await page({ electrical: false });
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    const editor = p.getByRole('region', { name: 'Drawing template editor', exact: true });
    const ribbon = p.getByRole('tabpanel', { name: 'Template', exact: true });
    const command = (name) => ribbon.getByRole('button', { name, exact: true }).click();
    const ready = () =>
      until(
        async () =>
          !(await ribbon
            .getByRole('button', { name: 'Draw template text', exact: true })
            .isDisabled()),
      );
    await command('New template');
    await ready();
    await command('Draw template text');
    const svg = editor.locator('.template-preview');
    await until(async () => (await svg.getAttribute('data-artwork-tool')) === 'text');
    const q = await svg.evaluate((s) => {
      const p = new DOMPoint(30, 40).matrixTransform(s.getScreenCTM());
      return { x: p.x, y: p.y };
    });
    await p.mouse.click(q.x, q.y);
    await editor
      .getByRole('region', { name: 'Place template text', exact: true })
      .getByRole('textbox', { name: 'Text and variables', exact: true })
      .fill('Appearance');
    await editor.getByRole('button', { name: 'Place text', exact: true }).click();
    await ready();
    const ink = editor.locator('[data-template-graphic] text');
    const preset = editor.getByLabel('Artwork colour picker', { exact: true });
    const original = await ink.getAttribute('fill');
    const pane = await editor.locator('.template-canvas').boundingBox();
    await preset.evaluate((n) => {
      n.value = '#abcdef';
      n.dispatchEvent(new Event('input', { bubbles: true }));
    });
    assert.equal(await ink.getAttribute('fill'), original);
    await preset.evaluate((n) => {
      n.value = '#ff0000';
      n.dispatchEvent(new Event('change', { bubbles: true }));
    });
    await ready();
    await until(async () => (await ink.getAttribute('fill')) === '#ff0000');
    await command('Undo');
    await ready();
    assert.equal(await ink.getAttribute('fill'), original);
    const font = editor.getByRole('textbox', { name: 'Artwork font', exact: true });
    const choices = editor.getByRole('combobox', { name: 'Artwork font choices', exact: true });
    await font.fill('');
    await choices.focus();
    assert.equal(await font.inputValue(), '');
    await choices.selectOption('serif');
    await ready();
    assert.equal(await font.inputValue(), 'serif');

    await font.fill('Custom\nFamily');
    await font.press('Control+Enter');
    await ready();
    assert.equal(await font.inputValue(), 'Custom\nFamily');
    await editor
      .getByRole('combobox', { name: 'Artwork font choices', exact: true })
      .selectOption('monospace');
    await ready();
    await command('Undo');
    await ready();
    assert.equal(await font.inputValue(), 'Custom\nFamily');
    assert.deepEqual(await editor.locator('.template-canvas').boundingBox(), pane);
    await command('Discard template draft');
    await editor.waitFor({ state: 'hidden' });
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
