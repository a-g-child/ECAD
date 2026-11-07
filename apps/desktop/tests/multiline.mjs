/** Multiline is the same persistent text value across creation, properties and drawing views. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
const primary = process.platform === 'darwin' ? 'Meta' : 'Control';
const label = 'label:00000000000000000000000000000001';
export function registerMultilineTests({
  page,
  tree,
  button,
  input,
  until,
  pressAction,
  changed,
  bounds,
  root,
  project,
}) {
  const spans = (p, key) =>
    p.locator(`svg [data-object="${key}"] text.ink tspan`).allTextContents();
  test('multiline creation keeps blank lines/indentation and the same object through single-line edits and history', async () => {
    const p = await page();
    const before = await bounds(p);
    await p.keyboard.press('t');
    const text = p.getByRole('textbox', { name: 'Annotation text', exact: true });
    await text.waitFor();
    assert.equal(await text.evaluate((n) => n.tagName), 'TEXTAREA');
    await text.fill('First');
    await text.press('End');
    await text.press('Enter');
    await text.press('w');
    assert.equal(await text.inputValue(), 'First\nw');
    assert.doesNotMatch(await p.title(), /^●/);
    const value = '\n  First Ω\n\nSecond <tag>\n';
    await text.fill(value);
    await p.getByRole('textbox', { name: 'Annotation X (µm)', exact: true }).fill('10000');
    await p.getByRole('textbox', { name: 'Annotation Y (µm)', exact: true }).fill('-20000');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    const key = await p
      .locator('svg .move-preview[data-object^="label:"]')
      .getAttribute('data-object');
    assert.deepEqual(await spans(p, key), value.split('\n'));
    assert.equal(
      await p.locator(`svg [data-object="${key}"] text.ink`).getAttribute('aria-label'),
      value,
    );
    assert.ok(
      (await p
        .locator(`svg [data-object="${key}"] .text-bounds`)
        .evaluate((n) => n.getBBox().height)) >= 16,
    );
    await text.press(`${primary}+Enter`);
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Annotation created · one undo operation',
    );
    await input(p, 'drawing_text').waitFor();
    assert.equal(await input(p, 'drawing_text').inputValue(), value);
    await pressAction(p, 'Save project', 'Project saved');
    const saved = JSON.parse(await readFile(project(), 'utf8'));
    assert.equal(saved.drawing.labels.find((l) => `label:${l.id}` === key).text, value);
    const editor = input(p, 'drawing_text');
    await editor.fill('One line');
    await editor.press('End');
    await editor.press('Enter');
    assert.equal(await editor.inputValue(), 'One line\n');
    assert.deepEqual(await spans(p, key), value.split('\n'));
    await editor.press('Escape');
    assert.equal(await editor.inputValue(), value);
    await editor.fill('One line');
    await changed(p, () => editor.press(`${primary}+Enter`));
    assert.deepEqual(await spans(p, key), ['One line']);
    assert.equal(await p.locator(`svg [data-object="${key}"] .text-bounds`).count(), 0);
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await input(p, 'drawing_text').inputValue(), value);
    await pressAction(p, 'Redo', 'Redo complete');
    assert.deepEqual(await spans(p, key), ['One line']);
    await pressAction(p, 'Undo', 'Undo complete');
    await p.screenshot({ path: resolve(root, 'target/ui/multiline-text.png') });
    assert.deepEqual(await bounds(p), before);
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await p.locator(`svg [data-object="${key}"]`).count(), 0);
    await pressAction(p, 'Save project', 'Project saved');
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('selection and Fit include multiline blanks and the complete annotation block', async () => {
    const p = await page();
    await tree(p, label).click();
    const value = '\n  Left\n\nLast\n';
    await input(p, 'drawing_text').fill(value);
    await changed(p, () => button(p, 'Apply Text').click());
    const geometry = await p.locator(`svg [data-object="${label}"]`).evaluate((n) => {
      const b = n.getBBox();
      return { x: b.x, y: b.y, width: b.width, height: b.height };
    });
    async function screen(point) {
      return p.locator('svg.drawing').evaluate((svg, point) => {
        const q = new DOMPoint(...point).matrixTransform(svg.getScreenCTM());
        return { x: q.x, y: q.y };
      }, point);
    }
    async function drag(a, b) {
      const start = await screen(a),
        end = await screen(b);
      await p.mouse.move(start.x, start.y);
      await p.mouse.down();
      await p.mouse.move(end.x, end.y, { steps: 3 });
      await p.mouse.up();
    }
    const b = geometry;
    await drag([b.x - 1, b.y - 1], [b.x + b.width + 1, b.y + b.height - 1]);
    assert.equal(await p.locator(`svg [data-object="${label}"].selected`).count(), 0);
    await drag([b.x + b.width + 2, b.y + b.height + 1], [b.x + 0.5, b.y + b.height - 0.5]);
    assert.equal(await p.locator(`svg [data-object="${label}"].selected`).count(), 1);
    await p.locator('.canvas').hover();
    const initialView = await p.locator('svg.drawing').getAttribute('viewBox');
    await p.mouse.wheel(0, -450);
    await until(
      async () => (await p.locator('svg.drawing').getAttribute('viewBox')) !== initialView,
    );
    const zoomed = await p.locator('svg.drawing').getAttribute('viewBox');
    await p.locator('.canvas').focus();
    await p.keyboard.press('f');
    await until(async () => (await p.locator('svg.drawing').getAttribute('viewBox')) !== zoomed);
    const box = (await p.locator('svg.drawing').getAttribute('viewBox')).split(' ').map(Number);
    assert.ok(
      box[0] <= b.x &&
        box[1] <= b.y &&
        box[0] + box[2] >= b.x + b.width &&
        box[1] + box[3] >= b.y + b.height,
    );
    await pressAction(p, 'Undo', 'Undo complete');
    assert.deepEqual(await spans(p, label), ['CONVEYOR / SIGNAL HARNESS']);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('shared multiline properties render in derived captions and report cells without changing pane bounds', async () => {
    const p = await page();
    await button(p, 'Connection reports').click();
    await p.locator('.report-pane:not([hidden])').waitFor();
    const before = await bounds(p);
    await tree(p, 'device:00000000000000000000000000000001').click();
    await tree(p, 'cable:00000000000000000000000000000001').click({ modifiers: ['Shift'] });
    const field = input(p, 'designation');
    await field.waitFor();
    assert.equal(await field.evaluate((n) => n.tagName), 'TEXTAREA');
    const value = 'Unit\nA';
    await field.fill(value);
    await changed(p, () => field.press(`${primary}+Enter`));
    assert.deepEqual(
      await p
        .locator(
          'svg [data-object="anchor:00000000000000000000000000000001"] .anchor-caption tspan',
        )
        .allTextContents(),
      ['Unit', 'A : 1'],
    );
    assert.deepEqual(await bounds(p), before);
    await button(p, 'Refresh connection table').click();
    await until(async () =>
      (await p.locator('.report-body td').allTextContents()).some((v) => v.includes(value)),
    );
    assert.equal(
      await p
        .locator('.report-body td')
        .first()
        .evaluate((n) => getComputedStyle(n).whiteSpace),
      'pre-wrap',
    );
    await pressAction(p, 'Save project', 'Project saved');
    const saved = JSON.parse(await readFile(project(), 'utf8'));
    assert.equal(saved.devices.find((d) => d.id.endsWith('01')).designation, value);
    assert.equal(saved.cables[0].designation, value);
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Save project', 'Project saved');
    assert.deepEqual(await bounds(p), before);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
