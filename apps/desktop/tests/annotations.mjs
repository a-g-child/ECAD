/** Annotation creation exercises fresh SVG objects, typed defaults, history and protected controls. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
export function registerAnnotationTests({
  page,
  button,
  until,
  pressAction,
  bounds,
  input,
  root,
  project,
}) {
  const text = (p) => p.getByRole('textbox', { name: 'Annotation text', exact: true });
  const x = (p) => p.getByRole('textbox', { name: 'Annotation X (µm)', exact: true });
  const y = (p) => p.getByRole('textbox', { name: 'Annotation Y (µm)', exact: true });
  const ghosts = (p) => p.locator('svg.drawing .move-preview[data-object^="label:"] text.ink');
  test('T creates a literal annotation with fixed panes, then save/undo/redo preserves identity and clears deleted selection', async () => {
    const p = await page();
    const before = await bounds(p);
    await p.keyboard.press('t');
    await text(p).waitFor();
    assert.equal(await button(p, 'Apply').isDisabled(), true);
    assert.equal(await text(p).evaluate((n) => n === document.activeElement), true);
    await text(p).fill('Service Ω <img src=x>');
    await x(p).fill('10000');
    await y(p).fill('-20000');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    assert.equal(await ghosts(p).textContent(), 'Service Ω <img src=x>');
    assert.equal(await p.locator('img').count(), 0);
    assert.deepEqual(await bounds(p), before);
    assert.doesNotMatch(await p.title(), /^●/);
    await text(p).fill('   ');
    await p.waitForFunction(() =>
      document.querySelector('.status-message')?.classList.contains('error'),
    );
    assert.equal(await ghosts(p).count(), 0);
    assert.equal(await button(p, 'Apply').isDisabled(), true);
    await text(p).fill('Service Ω <img src=x>');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    const key = await ghosts(p).evaluate((n) => n.parentElement.dataset.object);
    assert.match(key, /^label:[0-9a-f]{32}$/);
    assert.notEqual(key, 'label:00000000000000000000000000000001');
    await p.screenshot({ path: resolve(root, 'target/ui/annotation-preview.png') });
    await button(p, 'Apply').click();
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Annotation created · one undo operation',
    );
    await input(p, 'drawing_text').waitFor();
    assert.equal(await input(p, 'drawing_text').inputValue(), 'Service Ω <img src=x>');
    await pressAction(p, 'Save project', 'Project saved');
    const saved = JSON.parse(await readFile(project(), 'utf8'));
    const stored = saved.drawing.labels.find((l) => `label:${l.id}` === key);
    assert.equal(stored.text, 'Service Ω <img src=x>');
    assert.deepEqual(stored.position_um, [10000, -20000]);
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await p.locator(`svg [data-object="${key}"]`).count(), 0);
    assert.equal(await p.locator('.inspector-pane .pane-tag').innerText(), 'NO SELECTION');
    await pressAction(p, 'Redo', 'Redo complete');
    assert.equal(
      await p.locator(`svg [data-object="${key}"] text.ink`).textContent(),
      'Service Ω <img src=x>',
    );
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Save project', 'Project saved');
    assert.deepEqual(await bounds(p), before);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('annotation pointer placement snaps/cycles/suppresses; native control navigation and cancellation preserve redo', async () => {
    const p = await page();
    const redoDisabled = await button(p, 'Redo').isDisabled();
    await button(p, 'Create annotation · T').click();
    await text(p).waitFor();
    await text(p).fill('Temporary annotation');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    const cursor = await p.locator('svg.drawing').evaluate((svg) => {
      const q = new DOMPoint(100, 0).matrixTransform(svg.getScreenCTM());
      return { x: q.x, y: q.y };
    });
    await p.mouse.move(cursor.x, cursor.y);
    await p.waitForFunction(() =>
      document.querySelector('.status-message')?.textContent?.startsWith('Terminal snap'),
    );
    await p.locator('.canvas').focus();
    await p.keyboard.press('Tab');
    await p.waitForFunction(() =>
      document.querySelector('.status-message')?.textContent?.startsWith('Endpoint snap'),
    );
    await p.keyboard.down('Alt');
    await until(async () => (await p.locator('.snap-marker').count()) === 0);
    await p.keyboard.up('Alt');
    await until(async () => (await p.locator('.snap-marker').count()) === 1);
    // A focused Apply/Cancel control owns Tab and Enter even when snap cycling is available.
    await button(p, 'Apply').focus();
    await p.keyboard.press('Tab');
    assert.equal(await button(p, 'Cancel').evaluate((n) => n === document.activeElement), true);
    await p.keyboard.press('Enter');
    await until(async () => !(await text(p).isVisible()));
    await p.waitForFunction(
      () => document.querySelector('.status-message')?.textContent === 'Annotation cancelled',
    );
    assert.equal(await p.locator('svg .move-preview').count(), 0);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.equal(await button(p, 'Redo').isDisabled(), redoDisabled);
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('annotation finish waits for the latest preview, rejects invalid text and cancels delayed drafts', async () => {
    const p = await page();
    const before = await p.locator('svg.drawing [data-object^="label:"]').count();
    await button(p, 'Create annotation · T').click();
    await text(p).waitFor();
    await text(p).fill('Initial text');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    let delayed = 0;
    await p.route('**/bridge', async (route) => {
      const request = route.request().postDataJSON();
      if (request.type === 'preview_annotation') {
        delayed++;
        await new Promise((resolve) => setTimeout(resolve, 220));
      }
      await route.continue();
    });
    await text(p).fill('Final multiline\ntext');
    await x(p).fill('12345');
    await y(p).fill('-23456');
    await text(p).press('Control+Enter');
    await until(async () => !(await text(p).isVisible()));
    assert.ok(delayed > 0);
    await p.unrouteAll({ behavior: 'wait' });
    await input(p, 'drawing_text').waitFor();
    assert.equal(await input(p, 'drawing_text').inputValue(), 'Final multiline\ntext');
    const key = await p
      .locator('svg.drawing [data-object^="label:"]')
      .filter({ hasText: 'Final multiline' })
      .getAttribute('data-object');
    await pressAction(p, 'Save project', 'Project saved');
    const saved = JSON.parse(await readFile(project(), 'utf8'));
    assert.deepEqual(
      saved.drawing.labels.find((l) => `label:${l.id}` === key).position_um,
      [12345, -23456],
    );
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await p.locator('svg.drawing [data-object^="label:"]').count(), before);
    await button(p, 'Create annotation · T').click();
    await text(p).waitFor();
    await text(p).fill('Valid draft');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    await p.route('**/bridge', async (route) => {
      if (route.request().postDataJSON().type === 'preview_annotation')
        await new Promise((resolve) => setTimeout(resolve, 180));
      await route.continue();
    });
    await text(p).fill('   ');
    await text(p).press('Control+Enter');
    await until(async () =>
      (await p.locator('.status-message').getAttribute('class')).includes('error'),
    );
    assert.equal(await text(p).isVisible(), true);
    assert.equal(await button(p, 'Apply').isDisabled(), true);
    await text(p).fill('Cancelled while loading');
    await text(p).press('Escape');
    await until(async () => !(await text(p).isVisible()));
    await p.unrouteAll({ behavior: 'wait' });
    assert.equal(await ghosts(p).count(), 0);
    assert.equal(await p.locator('svg.drawing [data-object^="label:"]').count(), before);
    await pressAction(p, 'Redo', 'Redo complete');
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Save project', 'Project saved');
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('schematic annotation click during a delayed pointer preview commits the clicked position once', async () => {
    const p = await page();
    await button(p, 'Create annotation · T').click();
    await text(p).waitFor();
    await text(p).fill('Delayed pointer click');
    await until(async () => !(await button(p, 'Apply').isDisabled()));
    let placement = null;
    await p.route('**/bridge', async (route) => {
      const request = route.request().postDataJSON();
      if (request.type === 'preview_annotation') {
        placement = request.placement;
        await new Promise((resolve) => setTimeout(resolve, 220));
      }
      await route.continue();
    });
    const cursor = await p.locator('svg.drawing').evaluate((svg) => {
      const point = new DOMPoint(55, 35).matrixTransform(svg.getScreenCTM());
      return { x: point.x, y: point.y };
    });
    await p.mouse.click(cursor.x, cursor.y);
    await until(async () => !(await text(p).isVisible()));
    await p.unrouteAll({ behavior: 'wait' });
    assert.ok(placement);
    await input(p, 'drawing_text').waitFor();
    assert.equal(await input(p, 'drawing_text').inputValue(), 'Delayed pointer click');
    await pressAction(p, 'Save project', 'Project saved');
    const saved = JSON.parse(await readFile(project(), 'utf8'));
    const added = saved.drawing.labels.filter((l) => l.text === 'Delayed pointer click');
    assert.equal(added.length, 1);
    assert.deepEqual(added[0].position_um, [Number(placement.x), Number(placement.y)]);
    await pressAction(p, 'Undo', 'Undo complete');
    await pressAction(p, 'Save project', 'Project saved');
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
