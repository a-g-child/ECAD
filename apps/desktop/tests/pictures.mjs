/** Portable image import/property/placement workflows use the real Rust bridge. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile, copyFile, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
export function registerPictureTests({
  page,
  button,
  until,
  root,
  project,
  replaceProject,
  baseline,
}) {
  const editor = (p) => p.getByRole('region', { name: 'Drawing template editor', exact: true });
  const ribbon = (p) => p.getByRole('tabpanel', { name: 'Template', exact: true });
  async function load(dialog, file) {
    await dialog.getByLabel('Picture file', { exact: true }).setInputFiles(file);
    await until(async () => !!(await dialog.locator('.picture-preview').getAttribute('src')));
  }
  async function apply(dialog) {
    await dialog.getByRole('button', { name: 'Apply picture', exact: true }).click();
    await dialog.waitFor({ state: 'hidden' });
  }
  async function command(p, name) {
    await ribbon(p).getByRole('button', { name, exact: true }).click();
    if (
      !['Close template editor', 'Discard template draft', 'Split title-block cell'].includes(name)
    )
      await until(
        async () =>
          !(await ribbon(p)
            .getByRole('button', { name: 'Draw template text', exact: true })
            .isDisabled()),
      );
  }
  test('schematic pictures copy imported PNG, expose shared properties, move with exact history and reopen without the source', async () => {
    const before = JSON.parse(await readFile(project(), 'utf8'));
    const clean = baseline();
    await replaceProject(clean);
    const sheet = clean.sheets.find((s) => s.kind === 'schematic' && s.placement)?.id;
    assert.ok(sheet, 'picture test needs an editable schematic');
    const dir = await mkdtemp(resolve(tmpdir(), 'ecad-picture-'));
    const source = resolve(dir, 'downloaded-logo.png');
    await copyFile(resolve(root, 'apps/desktop/tests/fixtures/picture.png'), source);
    const p = await page({ electrical: false });
    await p.getByRole('combobox', { name: 'Sheet display', exact: true }).selectOption(sheet);
    await p.getByRole('tab', { name: 'Drawing', exact: true }).click();
    await button(p, 'Insert picture').click();
    let d = p.getByRole('dialog', { name: 'Insert picture', exact: true });
    await d.waitFor();
    await load(d, source);
    await rm(source);
    await d.getByRole('textbox', { name: 'Picture X (µm)', exact: true }).fill('70000');
    await d.getByRole('textbox', { name: 'Picture Y (µm)', exact: true }).fill('-30000');
    await apply(d);
    const image = p.locator('.drawing [data-object^="picture:"]');
    await image.waitFor();
    const href = await image.locator('image').getAttribute('href');
    assert.match(href, /^data:image\/png;base64,/);
    await image.locator('.picture-bounds').click();
    const width = p.getByRole('textbox', { name: 'Picture Width (µm)', exact: true });
    await width.waitFor();
    await width.fill('50000');
    await width.press('Enter');
    await until(async () => (await image.locator('image').getAttribute('width')) === '50');
    const old = await image.locator('image').getAttribute('x');
    const box = await image.locator('.picture-bounds').boundingBox();
    await p.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await p.mouse.down();
    await p.mouse.move(box.x + box.width / 2 + 25, box.y + box.height / 2 + 10, { steps: 5 });
    await p.mouse.up();
    await until(async () => (await image.locator('image').getAttribute('x')) !== old);
    await p.getByRole('tab', { name: 'File', exact: true }).click();
    await p
      .getByRole('toolbar', { name: 'Quick access' })
      .getByRole('button', { name: 'Undo', exact: true })
      .click();
    await until(async () => (await image.locator('image').getAttribute('x')) === old);
    await p
      .getByRole('toolbar', { name: 'Quick access' })
      .getByRole('button', { name: 'Save project', exact: true })
      .click();
    await until(async () => !(await p.title()).startsWith('●'));
    const saved = JSON.parse(await readFile(project(), 'utf8'));
    assert.equal(saved.schema_version, 14);
    assert.equal(Object.keys(saved.pictures.assets).length, 1);
    assert.ok(!JSON.stringify(saved).includes('downloaded-logo'));
    await p.close();
    await replaceProject(saved);
    const reopened = await page({ electrical: false });
    await reopened
      .getByRole('combobox', { name: 'Sheet display', exact: true })
      .selectOption(sheet);
    assert.equal(
      await reopened.locator('.drawing [data-object^="picture:"] image').getAttribute('href'),
      href,
    );
    assert.deepEqual(reopened.errors, []);
    await reopened.close();
    await replaceProject(before);
    await rm(dir, { recursive: true, force: true });
  });
  test('template and cell pictures share rendering, retain source-independent assets through edits, undo and publication', async () => {
    const p = await page({ electrical: false });
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor(p).waitFor();
    await command(p, 'New template');
    await until(
      async () => (await p.locator('.status-message').textContent()) === 'New template draft',
    );
    await editor(p)
      .getByRole('textbox', { name: 'Template name', exact: true })
      .fill('Portable pictures regression');
    await editor(p).getByRole('button', { name: 'Apply properties', exact: true }).click();
    await until(
      async () => (await p.locator('.status-message').textContent()) === 'Template updated',
    );
    await command(p, 'Insert template picture');
    let d = p.getByRole('dialog', { name: 'Insert picture', exact: true });
    await d.waitFor();
    await load(d, resolve(root, 'apps/desktop/tests/fixtures/picture.png'));
    await apply(d);
    const graphic = editor(p).locator('[data-template-graphic] image');
    await graphic.waitFor();
    const href = await graphic.getAttribute('href');
    await editor(p)
      .getByRole('combobox', { name: 'Picture fit', exact: true })
      .selectOption('cover');
    await until(
      async () => (await graphic.getAttribute('preserveAspectRatio')) === 'xMidYMid slice',
    );
    await command(p, 'Undo');
    assert.equal(await graphic.getAttribute('preserveAspectRatio'), 'xMidYMid meet');
    const first = editor(p).locator('.template-structure [data-template-cell]').first();
    await first.click();
    await command(p, 'Insert template picture');
    d = p.getByRole('dialog', { name: 'Insert picture', exact: true });
    await d.waitFor();
    assert.equal(
      await d.getByRole('textbox', { name: 'Picture Width (µm)', exact: true }).count(),
      0,
    );
    await load(d, resolve(root, 'apps/desktop/tests/fixtures/picture.png'));
    await apply(d);
    const cell = editor(p).locator('.frame-cell').first();
    assert.equal(await cell.locator('image').getAttribute('href'), href);
    const id = await cell.getAttribute('data-field');
    await command(p, 'Split title-block cell');
    const split = p.getByRole('dialog', { name: 'Split title-block cell', exact: true });
    await split.getByRole('button', { name: 'Apply table edit', exact: true }).click();
    await split.waitFor({ state: 'hidden' });
    assert.equal(await editor(p).locator('.frame-cell image').count(), 1);
    assert.equal(
      await editor(p).locator(`.frame-cell[data-field="${id}"] image`).getAttribute('href'),
      href,
    );
    await command(p, 'Undo');
    await command(p, 'Save template revision');
    await until(async () => !(await p.title()).startsWith('●'));
    await command(p, 'Close template editor');
    await editor(p).waitFor({ state: 'hidden' });
    await p.getByRole('tab', { name: 'Library', exact: true }).click();
    await button(p, 'Drawing templates').click();
    await editor(p).waitFor();
    const catalog = editor(p).getByRole('combobox', {
      name: 'Saved template variants',
      exact: true,
    });
    const key = await catalog
      .locator('option')
      .evaluateAll(
        (os) => os.find((o) => o.textContent.includes('Portable pictures regression'))?.value,
      );
    assert.ok(key);
    await catalog.selectOption(key);
    await editor(p).getByRole('button', { name: 'Open selected template', exact: true }).click();
    await editor(p).locator('.frame-cell image').waitFor();
    assert.equal(await editor(p).locator('.frame-cell image').getAttribute('href'), href);
    assert.equal(
      await editor(p).locator('[data-template-graphic] image').getAttribute('href'),
      href,
    );
    await command(p, 'Close template editor');
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
