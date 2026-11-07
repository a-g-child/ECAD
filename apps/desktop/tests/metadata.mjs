/** Project metadata stays literal, portable and separate from electrical/drawing geometry. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
const primary = process.platform === 'darwin' ? 'Meta' : 'Control';
export function registerMetadataTests({
  page,
  button,
  until,
  bounds,
  root,
  project,
  pressAction: sharedAction,
}) {
  async function pressAction(p, name, message) {
    await p.getByRole('tab', { name: 'Drawing', exact: true }).click();
    await sharedAction(p, name, message);
  }
  const dialog = (p) => p.getByRole('dialog', { name: 'Project information', exact: true });
  const field = (p, name) => dialog(p).getByRole('textbox', { name, exact: true });
  const apply = (p) => dialog(p).getByRole('button', { name: 'Apply information', exact: true });
  async function ribbon(p) {
    await p.getByRole('tab', { name: 'File', exact: true }).click();
    await button(p, 'Project information').click();
    await dialog(p).waitFor();
  }
  test('project information carries multiline company and project fields with unchanged geometry and exact saved history', async () => {
    const p = await page();
    const before = await bounds(p),
      original = JSON.parse(await readFile(project(), 'utf8'));
    await ribbon(p);
    assert.deepEqual(await bounds(p), before);
    assert.match(await dialog(p).innerText(), /company.address/);
    await field(p, 'Company name').fill('Example & Co');
    await field(p, 'Company address').fill(' Unit 2\n\nExample Road\n');
    await field(p, 'Engineer / contact').fill('Engineer Ω');
    await field(p, 'Project title').fill('Vehicle <harness>\nPrototype');
    await field(p, 'Project issue').fill('B');
    assert.doesNotMatch(await p.title(), /^●/);
    await p.setViewportSize({ width: 960, height: 640 });
    const rect = await dialog(p).boundingBox(),
      actions = await apply(p).boundingBox();
    assert.ok(
      rect.x >= 0 && rect.x + rect.width <= 960 && rect.y >= 0 && rect.y + rect.height <= 640,
    );
    assert.ok(actions.y + actions.height <= rect.y + rect.height);
    await dialog(p)
      .locator('.metadata-fields')
      .evaluate((node) => (node.scrollTop = 0));
    await p.screenshot({ path: resolve(root, 'target/ui/project-information.png') });
    await apply(p).click();
    await dialog(p).waitFor({ state: 'hidden' });
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Project information updated · one undo operation',
    );
    await p.setViewportSize({ width: 1360, height: 880 });
    assert.deepEqual(await bounds(p), before);
    await pressAction(p, 'Save project', 'Project saved');
    const saved = JSON.parse(await readFile(project(), 'utf8'));
    assert.deepEqual(saved.metadata, {
      'company.name': 'Example & Co',
      'company.address': ' Unit 2\n\nExample Road\n',
      'engineer.name': 'Engineer Ω',
      'project.title': 'Vehicle <harness>\nPrototype',
      'project.issue': 'B',
    });
    saved.metadata = original.metadata;
    assert.deepEqual(saved, original);
    await pressAction(p, 'Undo', 'Undo complete');
    await ribbon(p);
    assert.equal(await field(p, 'Company name').inputValue(), '');
    await p.keyboard.press('Escape');
    await dialog(p).waitFor({ state: 'hidden' });
    await pressAction(p, 'Redo', 'Redo complete');
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('reopened metadata uses project values; menu/palette cancel and no-op Apply preserve stored values and redo', async () => {
    const p = await page(),
      original = await readFile(project(), 'utf8');
    await p.locator('.canvas').focus();
    await p.keyboard.press(`${primary}+Shift+p`);
    await p.getByRole('textbox', { name: 'Search commands' }).fill('Project information');
    await p.locator('.palette-item').filter({ hasText: 'Project information' }).click();
    await dialog(p).waitFor();
    assert.equal(await field(p, 'Company address').inputValue(), ' Unit 2\n\nExample Road\n');
    assert.equal(await field(p, 'Project title').inputValue(), 'Vehicle <harness>\nPrototype');
    await field(p, 'Company name').fill('Unsaved personal replacement');
    await field(p, 'Company name').press('Enter');
    assert.equal(await dialog(p).isVisible(), true);
    await p.keyboard.press('Escape');
    await dialog(p).waitFor({ state: 'hidden' });
    assert.equal(await readFile(project(), 'utf8'), original);
    await pressAction(p, 'Undo', 'Undo complete');
    await p
      .locator('details.menu')
      .filter({ has: p.locator('summary').filter({ hasText: /^File$/ }) })
      .locator('summary')
      .click();
    await button(p, 'Project information…').click();
    await dialog(p).waitFor();
    assert.equal(await field(p, 'Company name').inputValue(), '');
    await apply(p).click();
    await dialog(p).waitFor({ state: 'hidden' });
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Project information already matches · no document changes',
    );
    await until(async () => !(await button(p, 'Redo').isDisabled()));
    await pressAction(p, 'Redo', 'Redo complete');
    assert.doesNotMatch(await p.title(), /^●/);
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Save project', 'Project saved');
    assert.deepEqual(JSON.parse(await readFile(project(), 'utf8')).metadata, {});
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
