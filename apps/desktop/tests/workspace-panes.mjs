/** Pane state changes only shell tracks; mounted controls, selections and history survive. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { resolve } from 'node:path';
const primary = process.platform === 'darwin' ? 'Meta' : 'Control';
export function registerWorkspacePaneTests({ page, button, until, bounds, root }) {
  const ready = (p) => until(async () => !(await button(p, 'Save project').isDisabled()));
  test('pane headers, rails, View menu and palette share collapsible workspace state without losing draft text', async () => {
    const p = await page();
    await p.locator('.tree-item[data-object="label:00000000000000000000000000000001"]').click();
    const font = p.locator('[data-property="font_family"] textarea');
    await font.waitFor();
    await font.fill('Uncommitted family');
    const before = await bounds(p);
    const originalCanvas = await p.locator('.canvas').boundingBox();
    const title = await p.title();
    const undo = await button(p, 'Undo').isDisabled();
    await button(p, 'Collapse properties pane').click();
    await ready(p);
    assert.equal(await font.isVisible(), false);
    assert.equal(await button(p, 'Expand properties pane').getAttribute('aria-expanded'), 'false');
    assert.ok((await p.locator('.canvas').boundingBox()).width > originalCanvas.width);
    await button(p, 'Collapse explorer pane').click();
    await ready(p);
    assert.equal(await p.locator('.tree').isVisible(), false);
    assert.equal(await font.isVisible(), false);
    await p.screenshot({ path: resolve(root, 'target/ui/collapsed-panes.png') });
    await button(p, 'Expand explorer pane').click();
    await ready(p);
    assert.equal(await font.isVisible(), false);
    await button(p, 'Expand properties pane').focus();
    await p.keyboard.press('Enter');
    await ready(p);
    assert.equal(await font.inputValue(), 'Uncommitted family');
    await p.locator('.menu').filter({ hasText: 'View' }).locator('summary').click();
    await p.getByRole('button', { name: 'Toggle explorer pane', exact: true }).click();
    await ready(p);
    assert.equal(await p.locator('.tree').isVisible(), false);
    await p.locator('.canvas').focus();
    await p.keyboard.press(`${primary}+Shift+p`);
    await p.getByRole('textbox', { name: 'Search commands' }).fill('Toggle explorer pane');
    await p.locator('.palette-item').filter({ hasText: 'Toggle explorer pane' }).click();
    await ready(p);
    assert.equal(await p.locator('.tree').isVisible(), true);
    // Palette opening/closing is existing command behavior; layout itself preserves history.
    assert.equal(await p.title(), title);
    assert.equal(await button(p, 'Undo').isDisabled(), undo);
    assert.deepEqual(await bounds(p), before);
    await button(p, 'Collapse properties pane').click();
    await ready(p);
    await p.locator('.canvas').focus();
    await p.keyboard.press('e');
    await until(async () => await font.isVisible());
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('template panes inherit workspace visibility, guard pending fields and reopen without moving paper geometry', async () => {
    const p = await page({ electrical: false });
    await button(p, 'Collapse explorer pane').click();
    await ready(p);
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    const editor = p.getByRole('region', { name: 'Drawing template editor', exact: true });
    await editor.waitFor();
    const ribbon = p.getByRole('tabpanel', { name: 'Template', exact: true });
    const idle = () =>
      until(
        async () =>
          !(await ribbon
            .getByRole('button', { name: 'Draw template text', exact: true })
            .isDisabled()),
      );
    await idle();
    const camera = await editor.locator('.template-preview').getAttribute('viewBox');
    assert.equal(
      await editor.getByRole('button', { name: 'Expand explorer pane' }).isVisible(),
      true,
    );
    await editor.getByRole('button', { name: 'Expand explorer pane' }).click();
    await idle();
    await editor.getByRole('button', { name: 'Collapse properties pane' }).click();
    await idle();
    await p.setViewportSize({ width: 960, height: 640 });
    assert.equal(await editor.locator('.template-preview').getAttribute('viewBox'), camera);
    await editor.getByRole('button', { name: 'Expand properties pane' }).click();
    await idle();
    const name = editor.getByRole('textbox', { name: 'Template name', exact: true });
    const original = await name.inputValue();
    await name.fill('');
    assert.equal(
      await editor.getByRole('button', { name: 'Collapse properties pane' }).isDisabled(),
      true,
    );
    await name.press('Escape');
    await idle();
    assert.equal(await name.inputValue(), original);
    await editor.getByRole('button', { name: 'Collapse explorer pane' }).click();
    await idle();
    await ribbon.getByRole('button', { name: 'Close template editor', exact: true }).click();
    await editor.waitFor({ state: 'hidden' });
    await ready(p);
    assert.equal(await button(p, 'Expand explorer pane').isVisible(), true);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
