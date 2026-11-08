/** Paper sheets use real retained Rust previews and never edit their source drawing geometry. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
export function registerSheetTests({ page, button, until, bounds, root, project, pressAction }) {
  const dialog = (p) => p.getByRole('dialog', { name: 'Sheets and paper', exact: true });
  const field = (p, name) => dialog(p).getByRole('textbox', { name, exact: true });
  const control = (p, name) => dialog(p).getByRole('button', { name, exact: true });
  const display = (p) => p.getByRole('combobox', { name: 'Sheet display', exact: true });
  async function open(p) {
    await p.getByRole('tab', { name: 'Drawing', exact: true }).click();
    await button(p, 'Sheets and paper').click();
    await dialog(p).waitFor();
    await until(async () => !(await control(p, 'Preview sheet').isDisabled()));
  }
  async function preview(p) {
    await control(p, 'Preview sheet').click();
    await until(async () => !(await control(p, 'Apply sheet').isDisabled()));
  }
  async function apply(p) {
    await control(p, 'Apply sheet').click();
    await dialog(p).waitFor({ state: 'hidden' });
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Sheet layout updated · one undo operation',
    );
  }
  async function save(p) {
    await pressAction(p, 'Save project', 'Project saved');
  }
  test('sheet paper and exact view placement preview, validate, save and render independently from the editable model', async () => {
    const p = await page(),
      original = JSON.parse(await readFile(project(), 'utf8')),
      before = await bounds(p),
      modelBox = await p.locator('svg.drawing').getAttribute('viewBox');
    await open(p);
    await field(p, 'Drawing number').fill('DWG-001');
    await field(p, 'Sheet title').fill('Harness\nDetail');
    await field(p, 'Paper width (µm)').fill('0');
    await control(p, 'Preview sheet').click();
    await until(async () => (await dialog(p).getByRole('alert').innerText()).includes('positive'));
    assert.equal(await control(p, 'Apply sheet').isDisabled(), true);
    assert.doesNotMatch(await p.title(), /^●/);
    await dialog(p).getByRole('combobox', { name: 'Paper orientation' }).selectOption('portrait');
    await dialog(p).getByRole('combobox', { name: 'Paper preset' }).selectOption('a3');
    assert.equal(await field(p, 'Paper width (µm)').inputValue(), '297000');
    assert.equal(await field(p, 'Paper height (µm)').inputValue(), '420000');
    for (const [name, value] of [
      ['Left margin (µm)', '15000'],
      ['Scale denominator', '2'],
      ['Paper origin X (µm)', '30000'],
      ['Paper origin Y (µm)', '70000'],
      ['View origin X (µm)', '10000'],
      ['View origin Y (µm)', '-5000'],
    ])
      await field(p, name).fill(value);
    await preview(p);
    assert.equal(
      await dialog(p).locator('.sheet-placement').getAttribute('transform'),
      'translate(30 70) scale(0.5) translate(-10 5)',
    );
    assert.equal(await dialog(p).locator('.sheet-draft-preview polyline').count(), 2);
    assert.deepEqual(await bounds(p), before);
    await p.setViewportSize({ width: 960, height: 640 });
    const rect = await dialog(p).boundingBox(),
      applyBounds = await control(p, 'Apply sheet').boundingBox();
    assert.ok(
      rect.x >= 0 && rect.x + rect.width <= 960 && rect.y >= 0 && rect.y + rect.height <= 640,
    );
    assert.ok(applyBounds.y + applyBounds.height <= rect.y + rect.height);
    await p.screenshot({ path: resolve(root, 'target/ui/sheet-layout.png') });
    await apply(p);
    await p.setViewportSize({ width: 1360, height: 880 });
    await save(p);
    const saved = JSON.parse(await readFile(project(), 'utf8')),
      sheet = saved.sheets[0];
    assert.equal(saved.schema_version, 9);
    assert.deepEqual(sheet.paper, {
      width_um: 297000,
      height_um: 420000,
      margins_um: [15000, 10000, 10000, 10000],
    });
    assert.deepEqual(sheet.placement.scale, [1, 2]);
    assert.deepEqual(sheet.placement.paper_origin_um, [30000, 70000]);
    assert.deepEqual(sheet.placement.view_origin_um, [10000, -5000]);
    saved.sheets = [];
    saved.drawing_documents = original.drawing_documents;
    assert.deepEqual(saved, original);
    await display(p).selectOption(sheet.id);
    await p.getByRole('img', { name: 'Paper sheet preview' }).waitFor();
    assert.equal(
      await p.getByRole('combobox', { name: 'Drawing view', exact: true }).isDisabled(),
      true,
    );
    assert.equal(await button(p, 'Create annotation · T').isDisabled(), true);
    await p.locator('.canvas').focus();
    await p.keyboard.press('t');
    assert.equal(
      await p.getByRole('textbox', { name: 'Annotation text', exact: true }).isVisible(),
      false,
    );
    const paper = p.locator('.sheet-surface svg'),
      box = await paper.getAttribute('viewBox'),
      area = await paper.boundingBox();
    await p.mouse.move(area.x + area.width / 2, area.y + area.height / 2);
    await p.mouse.wheel(0, -120);
    await until(async () => (await paper.getAttribute('viewBox')) !== box);
    await p.keyboard.press('f');
    await until(async () => (await paper.getAttribute('viewBox')) === box);
    await p.screenshot({ path: resolve(root, 'target/ui/paper-view.png') });
    await display(p).selectOption('');
    assert.equal(await p.locator('svg.drawing').getAttribute('viewBox'), modelBox);
    assert.equal(
      await p.getByRole('combobox', { name: 'Drawing view', exact: true }).isDisabled(),
      false,
    );
    assert.equal(await button(p, 'Create annotation · T').isDisabled(), false);
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await display(p).locator('option').count(), 1);
    await pressAction(p, 'Redo', 'Redo complete');
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(await bounds(p), before);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('blank sheets retain explicit drawing numbers across reorder/remove/undo and no-op preview preserves redo', async () => {
    const p = await page(),
      source = JSON.parse(await readFile(project(), 'utf8')).drawing;
    await open(p);
    await control(p, 'New sheet').click();
    await until(async () => !(await field(p, 'Sheet title').isDisabled()));
    await field(p, 'Sheet title').fill('Cover');
    await dialog(p).getByRole('combobox', { name: 'Placed drawing view' }).selectOption('');
    await preview(p);
    assert.equal(await dialog(p).locator('.sheet-placement').count(), 0);
    await apply(p);
    await save(p);
    let sheets = JSON.parse(await readFile(project(), 'utf8')).sheets;
    const cover = sheets.find((s) => s.title === 'Cover'),
      drawing = sheets.find((s) => s.title !== 'Cover');
    assert.equal(cover.number, '1');
    await open(p);
    await dialog(p).getByRole('combobox', { name: 'Edit sheet' }).selectOption(cover.id);
    await control(p, 'Move up').click();
    await dialog(p).waitFor({ state: 'hidden' });
    await save(p);
    const reordered = JSON.parse(await readFile(project(), 'utf8'));
    sheets = reordered.drawing_documents.flatMap((d) =>
      d.sheets.map((id) => reordered.sheets.find((s) => s.id === id)),
    );
    assert.deepEqual(
      sheets.map((s) => s.number),
      ['1', 'DWG-001'],
    );
    await display(p).selectOption(cover.id);
    await p.locator('.sheet-surface svg').click({ button: 'right' });
    await dialog(p).waitFor();
    assert.equal(
      await dialog(p).getByRole('combobox', { name: 'Edit sheet' }).inputValue(),
      cover.id,
    );
    await control(p, 'Remove sheet').click();
    await dialog(p).waitFor({ state: 'hidden' });
    assert.equal(await display(p).inputValue(), '');
    await pressAction(p, 'Undo', 'Undo complete');
    assert.doesNotMatch(await p.title(), /^●/);
    await open(p);
    await preview(p);
    await control(p, 'Apply sheet').click();
    await dialog(p).waitFor({ state: 'hidden' });
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Sheet layout already matches · no document changes',
    );
    assert.equal(await button(p, 'Redo').isDisabled(), false);
    await pressAction(p, 'Redo', 'Redo complete');
    await pressAction(p, 'Undo', 'Undo complete');
    for (const id of [cover.id, drawing.id]) {
      await open(p);
      await dialog(p).getByRole('combobox', { name: 'Edit sheet' }).selectOption(id);
      await control(p, 'Remove sheet').click();
      await dialog(p).waitFor({ state: 'hidden' });
    }
    await save(p);
    const final = JSON.parse(await readFile(project(), 'utf8'));
    assert.deepEqual(final.sheets, []);
    assert.deepEqual(final.drawing, source);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
