/** Pointer edits on scaled sheets and reviewed attachment of retained unplaced views. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
export function registerSheetEditingTests({ page, button, until, bounds, project, pressAction }) {
  const dialog = (p) => p.getByRole('dialog', { name: 'Sheets and paper', exact: true });
  const field = (p, name) => dialog(p).getByRole('textbox', { name, exact: true });
  const control = (p, name) => dialog(p).getByRole('button', { name, exact: true });
  const selector = (p) => p.getByRole('combobox', { name: 'Sheet display', exact: true });
  const stored = async () => JSON.parse(await readFile(project(), 'utf8'));
  async function open(p) {
    await p.getByRole('tab', { name: 'Drawing', exact: true }).click();
    await button(p, 'Sheets and paper').click();
    await dialog(p).waitFor();
    await until(async () => !(await control(p, 'Preview sheet').isDisabled()));
  }
  async function apply(p) {
    await control(p, 'Preview sheet').click();
    await until(async () => !(await control(p, 'Apply sheet').isDisabled()));
    await control(p, 'Apply sheet').click();
    await dialog(p).waitFor({ state: 'hidden' });
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Sheet layout updated · one undo operation',
    );
  }
  async function screen(p, x, y) {
    return p.locator('svg.drawing').evaluate(
      (svg, [x, y]) => {
        const q = new DOMPoint(x, y).matrixTransform(svg.getScreenCTM());
        return { x: q.x, y: q.y };
      },
      [x, y],
    );
  }
  test('scaled sheet pointer drag snaps in source units, preserves connectivity and undoes exact routes; numeric input stays exact', async () => {
    const p = await page(),
      original = await stored(),
      before = await bounds(p);
    await open(p);
    for (const [name, value] of [
      ['Scale denominator', '2'],
      ['Paper origin X (µm)', '40000'],
      ['Paper origin Y (µm)', '70000'],
      ['View origin X (µm)', '10000'],
      ['View origin Y (µm)', '-5000'],
    ])
      await field(p, name).fill(value);
    await apply(p);
    const mapped = await p.locator('svg.drawing .sheet-paper').evaluate((g) => {
      const q = new DOMPoint(40, 77.5).matrixTransform(g.getScreenCTM());
      return { x: q.x, y: q.y };
    });
    const end = await screen(p, 10, 10);
    assert.ok(Math.abs(mapped.x - end.x) < 1e-6 && Math.abs(mapped.y - end.y) < 1e-6);
    const anchor = (id) => `anchor:${id.toString(16).padStart(32, '0')}`;
    for (const id of [1, 2])
      await p
        .locator(`.tree-item[data-object="${anchor(id)}"]`)
        .click({ modifiers: id === 2 ? ['Shift'] : [] });
    const start = await screen(p, 0, 0);
    await p.mouse.move(start.x, start.y);
    await p.mouse.down();
    await p.mouse.move(end.x, end.y, { steps: 5 });
    await p.mouse.up();
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Selection moved · one undo operation',
    );
    await pressAction(p, 'Save project', 'Project saved');
    const moved = await stored();
    assert.deepEqual(moved.drawing.anchors[0].position_um, [10000, 10000]);
    assert.deepEqual(moved.drawing.anchors[1].position_um, [60000, 10000]);
    assert.deepEqual(moved.conductors, original.conductors);
    assert.deepEqual(moved.devices, original.devices);
    assert.deepEqual(
      moved.drawing.routes[0].points_um,
      original.drawing.routes[0].points_um.map(([x, y]) => [x + 10000, y + 10000]),
    );
    assert.deepEqual(await bounds(p), before);
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Save project', 'Project saved');
    assert.deepEqual((await stored()).drawing, original.drawing);
    await p.locator(`.tree-item[data-object="${anchor(1)}"]`).click();
    await p.locator('.canvas').focus();
    await p.keyboard.press('m');
    const x = p.getByRole('textbox', { name: 'Move X (µm)', exact: true });
    await x.waitFor();
    await x.fill('12345');
    await p.getByRole('textbox', { name: 'Move Y (µm)', exact: true }).fill('6789');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    await button(p, 'Apply').click();
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Selection moved · one undo operation',
    );
    await pressAction(p, 'Save project', 'Project saved');
    assert.deepEqual((await stored()).drawing.anchors[0].position_um, [12345, 6789]);
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Save project', 'Project saved');
    assert.deepEqual(await stored(), original);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('removing the last sheet retains unplaced geometry; explicit attachment previews, undoes and reopens with stable identity', async () => {
    const p = await page(),
      original = await stored();
    await open(p);
    await control(p, 'Remove sheet').click();
    await dialog(p).waitFor({ state: 'hidden' });
    await pressAction(p, 'Save project', 'Project saved');
    assert.equal(await selector(p).locator('option').count(), 0);
    await p.reload();
    await p.waitForFunction(() =>
      document.querySelector('.status-message')?.textContent?.startsWith('Ready'),
    );
    assert.match(await p.locator('.canvas-empty').textContent(), /No drawing sheets/);
    assert.equal(await button(p, 'Create annotation · T').isDisabled(), true);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual((await stored()).drawing, original.drawing);
    await p
      .locator('.canvas-empty')
      .getByRole('button', { name: 'Sheets and paper', exact: true })
      .click();
    await dialog(p).waitFor();
    await until(async () => !(await field(p, 'Sheet title').isDisabled()));
    await field(p, 'Drawing number').fill('ATTACHED');
    await field(p, 'Sheet title').fill('Reviewed legacy drawing');
    await dialog(p)
      .getByRole('combobox', { name: 'Placed drawing view' })
      .selectOption(original.drawing.views[0].id);
    await control(p, 'Preview sheet').click();
    await until(async () => !(await control(p, 'Apply sheet').isDisabled()));
    assert.equal(await selector(p).locator('option').count(), 0);
    assert.deepEqual((await stored()).sheets, []);
    await control(p, 'Close').click();
    await dialog(p).waitFor({ state: 'hidden' });
    assert.doesNotMatch(await p.title(), /^●/);
    await p
      .locator('.canvas-empty')
      .getByRole('button', { name: 'Sheets and paper', exact: true })
      .click();
    await dialog(p).waitFor();
    await until(async () => !(await field(p, 'Sheet title').isDisabled()));
    await field(p, 'Drawing number').fill('ATTACHED');
    await field(p, 'Sheet title').fill('Reviewed legacy drawing');
    await dialog(p)
      .getByRole('combobox', { name: 'Placed drawing view' })
      .selectOption(original.drawing.views[0].id);
    await apply(p);
    const id = await selector(p).inputValue();
    assert.match(id, /^[a-f0-9]{32}$/);
    assert.equal(await button(p, 'Create annotation · T').isDisabled(), false);
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await selector(p).locator('option').count(), 0);
    await pressAction(p, 'Redo', 'Redo complete');
    assert.equal(await selector(p).inputValue(), id);
    await pressAction(p, 'Save project', 'Project saved');
    await p.reload();
    await p.waitForFunction(() =>
      document.querySelector('.status-message')?.textContent?.startsWith('Ready'),
    );
    assert.equal(await selector(p).inputValue(), id);
    assert.deepEqual((await stored()).drawing, original.drawing);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
