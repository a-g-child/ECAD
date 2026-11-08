/** Project context properties, typed page creation and generated sheet navigation use real Rust state. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
export function registerProjectPageTests({
  page,
  button,
  until,
  bounds,
  root,
  project,
  pressAction,
  replaceProject,
}) {
  const sheetDialog = (p) => p.getByRole('dialog', { name: 'Sheets and paper', exact: true });
  const properties = (p) => p.getByRole('dialog', { name: 'Project properties', exact: true });
  const field = (p, name) => sheetDialog(p).getByRole('textbox', { name, exact: true });
  const control = (p, name) => sheetDialog(p).getByRole('button', { name, exact: true });
  const display = (p) => p.getByRole('combobox', { name: 'Sheet display', exact: true });
  const stored = async () => JSON.parse(await readFile(project(), 'utf8'));
  async function rootMenu(p) {
    await p.locator('.tree .project-root > summary').click({ button: 'right' });
    const menu = p.getByRole('dialog', { name: 'Project menu', exact: true });
    await menu.waitFor();
    return menu;
  }
  async function openProperties(p) {
    const menu = await rootMenu(p);
    await menu.getByRole('button', { name: 'Properties…', exact: true }).click();
    await properties(p).waitFor();
  }
  async function applyProperties(p) {
    await properties(p).getByRole('button', { name: 'Apply properties', exact: true }).click();
    await properties(p).waitFor({ state: 'hidden' });
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Project properties updated · one undo operation',
    );
  }
  async function openNew(p, label) {
    const menu = await rootMenu(p);
    await menu.getByRole('button', { name: label, exact: true }).click();
    await sheetDialog(p).waitFor();
    await until(async () => !(await control(p, 'Preview sheet').isDisabled()));
  }
  async function applySheet(p) {
    await control(p, 'Preview sheet').click();
    await until(async () => !(await control(p, 'Apply sheet').isDisabled()));
    await control(p, 'Apply sheet').click();
    await sheetDialog(p).waitFor({ state: 'hidden' });
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Sheet layout updated · one undo operation',
    );
    return display(p).inputValue();
  }
  async function save(p) {
    await pressAction(p, 'Save project', 'Project saved');
  }
  test('project right-click properties carry template defaults; new cover pins its choice and existing pages do not restyle', async () => {
    const p = await page({ electrical: false }),
      original = await stored(),
      before = await bounds(p);
    await openProperties(p);
    assert.equal(
      await properties(p)
        .getByRole('combobox', { name: 'Cover template', exact: true })
        .inputValue(),
      'cover-v1',
    );
    await properties(p)
      .getByRole('textbox', { name: 'Company name', exact: true })
      .fill('Project <company>\nContact');
    await properties(p)
      .getByRole('combobox', { name: 'Cover template', exact: true })
      .selectOption('standard-v1');
    await p.setViewportSize({ width: 960, height: 640 });
    const dialog = await properties(p).boundingBox(),
      apply = await properties(p)
        .getByRole('button', { name: 'Apply properties', exact: true })
        .boundingBox();
    assert.ok(
      dialog.x >= 0 &&
        dialog.x + dialog.width <= 960 &&
        apply.y + apply.height <= dialog.y + dialog.height,
    );
    await p.screenshot({ path: resolve(root, 'target/ui/project-template-properties.png') });
    await applyProperties(p);
    await p.setViewportSize({ width: 1360, height: 880 });
    await save(p);
    assert.equal((await stored()).template_defaults.cover, 'standard-v1');
    await openNew(p, 'Cover sheet…');
    assert.equal(
      await sheetDialog(p)
        .getByRole('combobox', { name: 'Sheet template', exact: true })
        .inputValue(),
      'standard-v1',
    );
    assert.equal(
      await sheetDialog(p)
        .getByRole('combobox', { name: 'Placed drawing view', exact: true })
        .isDisabled(),
      true,
    );
    await field(p, 'Drawing number').fill('TEST-COVER');
    await field(p, 'Sheet title').fill('Project cover\nLiteral');
    const id = await applySheet(p);
    await save(p);
    let data = await stored(),
      cover = data.sheets.find((s) => s.id === id);
    assert.equal(await p.locator('[aria-label="Sheet type"]').textContent(), 'COVER');
    assert.equal(cover.kind, 'cover');
    assert.equal(cover.template, 'standard-v1');
    assert.equal(cover.placement, null);
    assert.deepEqual(data.drawing, original.drawing);
    await openProperties(p);
    await properties(p)
      .getByRole('combobox', { name: 'Cover template', exact: true })
      .selectOption('cover-v1');
    await applyProperties(p);
    await save(p);
    data = await stored();
    assert.equal(data.template_defaults.cover, 'cover-v1');
    assert.equal(data.sheets.find((s) => s.id === id).template, 'standard-v1');
    assert.equal(
      await p.locator('svg.drawing .sheet-frame').getAttribute('data-template'),
      'standard-v1',
    );
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Undo', 'Undo complete');
    await save(p);
    data = await stored();
    assert.deepEqual(data.template_defaults, original.template_defaults);
    assert.deepEqual(data.metadata, original.metadata);
    assert.deepEqual(data.drawing, original.drawing);
    assert.deepEqual(await bounds(p), before);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('new schematic draft creates an independent editable view only on Apply and undoes as one sheet operation', async () => {
    const p = await page({ electrical: false }),
      original = await stored();
    await openNew(p, 'Schematic sheet…');
    const cancelledView = await sheetDialog(p)
      .getByRole('combobox', { name: 'Placed drawing view', exact: true })
      .inputValue();
    assert.ok(!original.drawing.views.some((v) => v.id === cancelledView));
    await control(p, 'Preview sheet').click();
    await until(async () => !(await control(p, 'Apply sheet').isDisabled()));
    await control(p, 'Close').click();
    await sheetDialog(p).waitFor({ state: 'hidden' });
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual((await stored()).drawing, original.drawing);
    await openNew(p, 'Schematic sheet…');
    await field(p, 'Drawing number').fill('2');
    await field(p, 'Sheet title').fill('New harness\nWiring');
    const view = await sheetDialog(p)
      .getByRole('combobox', { name: 'Placed drawing view', exact: true })
      .inputValue();
    const id = await applySheet(p);
    assert.equal(await display(p).inputValue(), id);
    assert.equal(await p.locator('svg.drawing [data-object]').count(), 0);
    await p.locator('.canvas').focus();
    await p.keyboard.press('t');
    await p
      .getByRole('textbox', { name: 'Annotation text', exact: true })
      .fill('Connector\nCallout');
    await p.getByRole('textbox', { name: 'Annotation X (µm)', exact: true }).fill('0');
    await p.getByRole('textbox', { name: 'Annotation Y (µm)', exact: true }).fill('0');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    await button(p, 'Apply').click();
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Annotation created · one undo operation',
    );
    await save(p);
    let data = await stored();
    assert.equal(data.sheets.find((s) => s.id === id).kind, 'schematic');
    assert.ok(data.drawing.views.some((v) => v.id === view));
    assert.ok(data.drawing.labels.some((l) => l.view === view && l.text === 'Connector\nCallout'));
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Undo', 'Undo complete');
    await save(p);
    data = await stored();
    assert.deepEqual(data.drawing, original.drawing);
    assert.ok(!data.sheets.some((s) => s.id === id));
    await pressAction(p, 'Redo', 'Redo complete');
    await save(p);
    assert.ok((await stored()).drawing.views.some((v) => v.id === view));
    await pressAction(p, 'Undo', 'Undo complete');
    await save(p);
    assert.deepEqual((await stored()).drawing, original.drawing);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('drawing-list sheets paginate live number/title links, navigate by stable identity and update after rename/undo', async () => {
    const p = await page({ electrical: false }),
      original = await stored();
    const make = async (label, number, title, small = false) => {
      await openNew(p, label);
      await field(p, 'Drawing number').fill(number);
      await field(p, 'Sheet title').fill(title);
      if (small) await field(p, 'Paper height (µm)').fill('80000');
      return applySheet(p);
    };
    const two = await make('Cover sheet…', '2', 'Two\nLines'),
      ten = await make('Cover sheet…', '10', 'Ten <literal>');
    const first = await make('Drawing list sheet…', 'LIST-1', 'Drawing list', true);
    assert.equal(await p.locator('[aria-label="Sheet type"]').textContent(), 'DRAWING LIST');
    assert.match(await p.locator('.paper-badge').textContent(), /2 drawings need more list pages/);
    assert.deepEqual(
      await p
        .locator('svg.drawing .drawing-list [data-sheet-link]')
        .evaluateAll((nodes) => nodes.map((n) => n.dataset.sheetLink)),
      [two, ten],
    );
    const second = await make('Drawing list sheet…', 'LIST-2', 'Drawing list continued', true);
    assert.doesNotMatch(await p.locator('.paper-badge').textContent(), /need more list pages/);
    await save(p);
    let data = await stored();
    assert.equal(data.sheets.find((s) => s.id === first).kind, 'drawing-list');
    assert.equal(data.sheets.find((s) => s.id === first).template, 'drawing-list-v1');
    assert.deepEqual(data.drawing, original.drawing);
    assert.equal(
      Object.hasOwn(
        data.sheets.find((s) => s.id === first),
        'rows',
      ),
      false,
    );
    await display(p).selectOption(first);
    await p.locator(`svg.drawing .drawing-list a[data-sheet-link="${ten}"]`).click();
    assert.equal(await display(p).inputValue(), ten);
    await display(p).selectOption(first);
    await p.locator(`svg.drawing .drawing-list a[data-sheet-link="${two}"]`).focus();
    await p.keyboard.press('Enter');
    assert.equal(await display(p).inputValue(), two);
    assert.doesNotMatch(await p.title(), /^●/);
    await p.locator(`.tree button[data-sheet="${ten}"]`).click({ button: 'right' });
    await sheetDialog(p).waitFor();
    await field(p, 'Drawing number').fill('1');
    await field(p, 'Sheet title').fill('Renamed\nTarget');
    await applySheet(p);
    await display(p).selectOption(first);
    assert.deepEqual(
      await p
        .locator('svg.drawing .drawing-list [data-sheet-link]')
        .evaluateAll((nodes) => nodes.map((n) => n.dataset.sheetLink)),
      [ten, two],
    );
    assert.match(
      await p
        .locator(`svg.drawing .drawing-list a[data-sheet-link="${ten}"]`)
        .getAttribute('aria-label'),
      /1 · Renamed\nTarget/,
    );
    await p.screenshot({ path: resolve(root, 'target/ui/generated-drawing-list.png') });
    await pressAction(p, 'Undo', 'Undo complete');
    assert.deepEqual(
      await p
        .locator('svg.drawing .drawing-list [data-sheet-link]')
        .evaluateAll((nodes) => nodes.map((n) => n.dataset.sheetLink)),
      [two, ten],
    );
    for (let i = 0; i < 4; i++) await pressAction(p, 'Undo', 'Undo complete');
    await save(p);
    data = await stored();
    assert.deepEqual(data.drawing, original.drawing);
    assert.ok(!data.sheets.some((s) => [two, ten, first, second].includes(s.id)));
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('a project with no drawing views can create its first schematic from the project menu', async () => {
    const data = JSON.parse(
      await readFile(resolve(root, 'crates/ecad-adapters/tests/fixtures/minimal-v11.json'), 'utf8'),
    );
    await replaceProject(data);
    const p = await page({ electrical: false });
    assert.equal(await display(p).locator('option').count(), 0);
    assert.equal(await button(p, 'Create annotation · T').isDisabled(), true);
    await openNew(p, 'Schematic sheet…');
    await field(p, 'Drawing number').fill('FIRST');
    await field(p, 'Sheet title').fill('First schematic');
    assert.equal((await stored()).drawing.views.length, 0);
    const id = await applySheet(p);
    assert.equal(await display(p).inputValue(), id);
    assert.equal(await button(p, 'Create annotation · T').isDisabled(), false);
    await save(p);
    const saved = await stored();
    assert.equal(saved.drawing.views.length, 1);
    assert.deepEqual(saved.devices, data.devices);
    await pressAction(p, 'Undo', 'Undo complete');
    await save(p);
    assert.equal(await display(p).locator('option').count(), 0);
    assert.equal((await stored()).drawing.views.length, 0);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
