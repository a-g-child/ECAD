/** Template editor is a real separate Rust document and publishes through the local library store. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
const primary = process.platform === 'darwin' ? 'Meta' : 'Control';
export function registerTemplateEditorTests({ page, button, until, project, root, pressAction }) {
  const editor = (p) => p.getByRole('region', { name: 'Drawing template editor', exact: true });
  const ribbon = (p) => p.getByRole('tabpanel', { name: 'Template', exact: true });
  const field = (p, name) => editor(p).getByRole('textbox', { name, exact: true });
  async function open(p) {
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor(p).waitFor();
    await until(
      async () =>
        !(await ribbon(p).getByRole('button', { name: 'New template', exact: true }).isDisabled()),
    );
  }
  async function ready(p) {
    await until(
      async () =>
        !(await editor(p)
          .getByRole('button', { name: 'Apply properties', exact: true })
          .isDisabled()),
    );
  }
  async function apply(p) {
    await editor(p).getByRole('button', { name: 'Apply properties', exact: true }).click();
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'Template updated',
    );
    await ready(p);
  }
  async function command(p, name, message) {
    await ribbon(p).getByRole('button', { name, exact: true }).click();
    if (message)
      await p.waitForFunction(
        (s) => document.querySelector('.status-message')?.textContent === s,
        message,
      );
    await ready(p);
  }
  test('library template view has contextual ribbon/properties, independent history and explicit saved project import', async () => {
    const p = await page({ electrical: false });
    await pressAction(p, 'Save project', 'Project saved');
    const original = JSON.parse(await readFile(project(), 'utf8'));
    const workspace = await p.locator('.workspace').boundingBox();
    await open(p);
    assert.equal((await p.locator('.ribbon').boundingBox()).height, 96);
    assert.deepEqual(await p.locator('.workspace').boundingBox(), workspace);
    await command(p, 'New template', 'New template draft');
    assert.match(await p.title(), /^●/);
    await field(p, 'Template name').fill('Browser template\nCompany sheet');
    await apply(p);
    const first = editor(p).locator('[data-template-cell]').first();
    await first.click({ button: 'right' });
    await field(p, 'Cell caption').fill('Organization\nOffice');
    await editor(p)
      .getByRole('combobox', { name: 'Content 1 type', exact: true })
      .selectOption('literal');
    await field(p, 'Content 1 text').fill('Literal company\nSecond line');
    await apply(p);
    assert.equal(
      await editor(p)
        .locator('.frame-cell')
        .first()
        .locator('text.frame-value')
        .getAttribute('aria-label'),
      'Literal company\nSecond line',
    );
    await command(p, 'Save template revision', 'Template revision saved');
    assert.doesNotMatch(await p.title(), /^●/);
    let stored = JSON.parse(await readFile(project(), 'utf8'));
    assert.deepEqual(stored, original);
    await editor(p).getByRole('img', { name: 'Template paper preview' }).focus();
    await p.keyboard.press(`${primary}+z`);
    await until(async () => /^●/.test(await p.title()));
    await editor(p).getByRole('img', { name: 'Template paper preview' }).focus();
    await p.keyboard.press(`${primary}+Shift+z`);
    await until(async () => !/^●/.test(await p.title()));
    await command(
      p,
      'Add saved revision to project',
      'Saved template added to project; existing sheets retain their revision',
    );
    await p.setViewportSize({ width: 960, height: 640 });
    assert.equal(
      await p.locator('.workspace').evaluate((n) => n.scrollWidth > n.clientWidth),
      false,
    );
    assert.equal((await p.locator('.ribbon').boundingBox()).height, 96);
    await p.screenshot({ path: resolve(root, 'target/ui/template-editor.png') });
    await ribbon(p).getByRole('button', { name: 'Close template editor', exact: true }).click();
    await editor(p).waitFor({ state: 'hidden' });
    assert.equal(
      await p.getByRole('tab', { name: 'Drawing', exact: true }).getAttribute('aria-selected'),
      'true',
    );
    assert.match(await p.title(), /^●/);
    await p.setViewportSize({ width: 1360, height: 880 });
    await pressAction(p, 'Undo', 'Undo complete');
    assert.doesNotMatch(await p.title(), /^●/);
    const library = JSON.parse(
      await readFile(resolve(dirname(project()), 'drawing-templates.json'), 'utf8'),
    );
    assert.ok(
      library.revisions.some((r) =>
        r.variants.some((v) => v.name === 'Browser template\nCompany sheet'),
      ),
    );
    await pressAction(p, 'Save project', 'Project saved');
    stored = JSON.parse(await readFile(project(), 'utf8'));
    assert.deepEqual(stored, original);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('template invalid properties retain fields, text owns shortcuts and explicit discard returns safely', async () => {
    const p = await page({ electrical: false });
    const original = await readFile(project(), 'utf8');
    await open(p);
    await command(p, 'New template', 'New template draft');
    await editor(p).getByText('Advanced paper settings', { exact: true }).click();
    await field(p, 'Paper width (µm)').fill('0');
    await editor(p).getByRole('button', { name: 'Apply properties', exact: true }).click();
    await until(async () => (await editor(p).getByRole('alert').innerText()).includes('positive'));
    assert.equal(await field(p, 'Paper width (µm)').inputValue(), '0');
    await editor(p).getByRole('button', { name: 'Revert fields', exact: true }).click();
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'Template fields reverted',
    );
    assert.equal(await field(p, 'Paper width (µm)').inputValue(), '297000');
    await field(p, 'Template name').fill('Uncommitted name');
    await field(p, 'Template name').press(`${primary}+z`);
    assert.equal(
      await ribbon(p).getByRole('button', { name: 'Undo', exact: true }).isDisabled(),
      true,
    );
    await ribbon(p).getByRole('button', { name: 'Discard template draft', exact: true }).click();
    await editor(p).waitFor({ state: 'hidden' });
    assert.doesNotMatch(await p.title(), /^●/);
    assert.equal(await readFile(project(), 'utf8'), original);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
