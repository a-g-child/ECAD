/** Inline typed text stays transient until one closed creation commit. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
export function registerTemplateInlineText({ page, button, until }) {
  test('template click text edits multiline variables inline, protects IME and commits once after delayed validation', async () => {
    const p = await page({ electrical: false });
    const editor = p.getByRole('region', { name: 'Drawing template editor', exact: true });
    const ribbon = p.getByRole('tabpanel', { name: 'Template', exact: true });
    const inline = editor.getByRole('region', { name: 'Place template text', exact: true });
    const svg = editor.locator('.template-preview');
    const art = editor.locator('[data-template-graphic]');
    const ready = () =>
      until(
        async () =>
          !(await ribbon
            .getByRole('button', { name: 'Draw template text', exact: true })
            .isDisabled()),
      );
    const command = async (name) => {
      await ribbon.getByRole('button', { name, exact: true }).click();
    };
    const start = async (x = 30, y = 40) => {
      await command('Draw template text');
      await until(async () => (await svg.getAttribute('data-artwork-tool')) === 'text');
      const q = await svg.evaluate(
        (s, [x, y]) => {
          const p = new DOMPoint(x, y).matrixTransform(s.getScreenCTM());
          return { x: p.x, y: p.y };
        },
        [x, y],
      );
      await p.mouse.click(q.x, q.y);
      await inline.waitFor();
    };
    const text = () => inline.getByRole('textbox', { name: 'Content 1 text', exact: true });
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    await command('New template');
    await ready();
    await editor
      .getByRole('textbox', { name: 'Template name', exact: true })
      .fill('Inline text regression');
    await editor
      .getByRole('textbox', { name: 'Template name', exact: true })
      .press('Control+Enter');
    await ready();
    const pane = await editor.locator('.template-canvas').boundingBox();
    let commits = 0;
    await p.route('**/bridge', async (route) => {
      const request = route.request().postDataJSON();
      if (request.type === 'template_geometry_commit') commits++;
      if (request.type === 'template_geometry_preview' && request.values.edit.contents)
        await new Promise((resolve) => setTimeout(resolve, 120));
      await route.continue();
    });
    await start();
    assert.equal(await text().evaluate((e) => document.activeElement === e), true);
    await text().fill('Office');
    await text().press('End');
    await text().press('Enter');
    await text().press('w');
    assert.equal(await text().inputValue(), 'Office\nw');
    assert.equal(commits, 0);
    await text().fill('Office\n  Site\n');
    await inline.getByRole('button', { name: 'Add text content', exact: true }).click();
    await inline
      .getByRole('combobox', { name: 'Content 2 type', exact: true })
      .selectOption('field');
    await inline
      .getByRole('combobox', { name: 'Content 2 variable', exact: true })
      .selectOption('project.title');
    await text().dispatchEvent('compositionstart');
    await text().press('Control+Enter');
    assert.equal(commits, 0);
    await text().dispatchEvent('compositionend');
    assert.deepEqual(await editor.locator('.template-canvas').boundingBox(), pane);
    await text().press('Control+Enter');
    await inline.waitFor({ state: 'hidden' });
    await ready();
    await p.unrouteAll({ behavior: 'wait' });
    assert.equal(commits, 1);
    assert.equal(await art.count(), 1);
    const id = await art.getAttribute('data-template-graphic');
    assert.equal(
      await editor.getByRole('textbox', { name: 'Content 1 text', exact: true }).inputValue(),
      'Office\n  Site\n',
    );
    assert.equal(
      await editor.getByRole('combobox', { name: 'Content 2 variable', exact: true }).inputValue(),
      'project.title',
    );
    await command('Undo');
    await ready();
    assert.equal(await art.count(), 0);
    await start(280, 190);
    await text().fill('Valid preview');
    await until(async () => (await art.count()) === 1);
    await text().fill('  ');
    await inline.getByRole('button', { name: 'Place text', exact: true }).click();
    await until(async () => (await inline.getByRole('alert').innerText()).includes('Enter text'));
    assert.equal(await inline.isVisible(), true);
    assert.equal(
      await inline
        .getByRole('button', { name: 'Place text', exact: true })
        .evaluate((node) => node === document.activeElement),
      true,
    );
    await inline
      .getByRole('combobox', { name: 'Content 1 type', exact: true })
      .selectOption('field');
    await until(async () => (await art.count()) === 1);
    await p.setViewportSize({ width: 960, height: 640 });
    // ResizeObserver clamps the overlay on the next rendering frame.
    await until(async () => {
      const overlay = await inline.boundingBox();
      const canvas = await editor.locator('.template-canvas').boundingBox();
      return (
        overlay.x >= canvas.x &&
        overlay.y >= canvas.y &&
        overlay.x + overlay.width <= canvas.x + canvas.width + 1 &&
        overlay.y + overlay.height <= canvas.y + canvas.height + 1
      );
    });
    const overlay = await inline.boundingBox(),
      canvas = await editor.locator('.template-canvas').boundingBox();
    assert.ok(
      overlay.x >= canvas.x &&
        overlay.y >= canvas.y &&
        overlay.x + overlay.width <= canvas.x + canvas.width + 1 &&
        overlay.y + overlay.height <= canvas.y + canvas.height + 1,
    );
    await inline.getByRole('combobox', { name: 'Content 1 variable', exact: true }).press('Escape');
    await inline.waitFor({ state: 'hidden' });
    await ready();
    assert.equal(await art.count(), 0);
    await command('Redo');
    await ready();
    assert.equal(await art.getAttribute('data-template-graphic'), id);
    await command('Save template revision');
    await ready();
    await command('Close template editor');
    await editor.waitFor({ state: 'hidden' });
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor.waitFor();
    await editor
      .getByRole('combobox', { name: 'Saved template variants', exact: true })
      .selectOption({ label: 'Inline text regression' });
    await editor.getByRole('button', { name: 'Open selected template', exact: true }).click();
    await ready();
    await editor.locator('[data-template-artwork]').click();
    assert.equal(
      await editor.getByRole('textbox', { name: 'Content 1 text', exact: true }).inputValue(),
      'Office\n  Site\n',
    );
    assert.equal(
      await editor.getByRole('combobox', { name: 'Content 2 variable', exact: true }).inputValue(),
      'project.title',
    );
    await command('Close template editor');
    await editor.waitFor({ state: 'hidden' });
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
