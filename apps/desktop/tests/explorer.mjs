/** Page/electrical navigation uses real loaded documents, shared actions and sheet guards. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
export function registerExplorerTests({
  page,
  button,
  until,
  bounds,
  root,
  project,
  pressAction,
  replaceProject,
}) {
  const pages = (p) => p.locator('.tree[data-mode="pages"]');
  const electrical = (p) => p.locator('.tree[data-mode="electrical"]');
  const sheet = (p, id) => pages(p).locator(`button[data-sheet="${id}"]`);
  const display = (p) => p.getByRole('combobox', { name: 'Sheet display', exact: true });
  const dialog = (p) => p.getByRole('dialog', { name: 'Sheets and paper', exact: true });
  const control = (p, name) => dialog(p).getByRole('button', { name, exact: true });
  const field = (p, name) => dialog(p).getByRole('textbox', { name, exact: true });
  async function mode(p, name) {
    await button(p, name === 'pages' ? 'Project pages' : 'Electrical explorer').click();
    await p.waitForFunction((name) => document.querySelector('.tree')?.dataset.mode === name, name);
    await until(async () => !(await button(p, 'Save project').isDisabled()));
  }
  async function cover(p) {
    await button(p, 'Sheets and paper').click();
    await dialog(p).waitFor();
    await control(p, 'New sheet').click();
    await until(async () => !(await field(p, 'Sheet title').isDisabled()));
    await field(p, 'Drawing number').fill('COVER');
    await field(p, 'Sheet title').fill('Cover sheet');
    await dialog(p).getByRole('combobox', { name: 'Placed drawing view' }).selectOption('');
    await control(p, 'Preview sheet').click();
    await until(async () => !(await control(p, 'Apply sheet').isDisabled()));
    await control(p, 'Apply sheet').click();
    await dialog(p).waitFor({ state: 'hidden' });
    await until(async () => !(await button(p, 'Save project').isDisabled()));
    return pages(p)
      .getByRole('button', { name: /COVER · Cover sheet/ })
      .getAttribute('data-sheet');
  }
  test('Pages is the default; mode actions preserve selection, history, expansion and fixed panes', async () => {
    const p = await page({ electrical: false }),
      before = await bounds(p);
    const original = await readFile(project(), 'utf8'),
      id = await display(p).inputValue();
    assert.equal(await button(p, 'Project pages').getAttribute('aria-pressed'), 'true');
    assert.equal(
      await p.getByRole('navigation', { name: 'Project pages', exact: true }).count(),
      1,
    );
    assert.equal(await pages(p).locator('.project-root').count(), 1);
    assert.equal(await pages(p).locator('[data-document]').count(), 0);
    assert.equal(await sheet(p, id).getAttribute('aria-current'), 'page');
    const branch = pages(p).locator('.project-root');
    await branch.locator(':scope > summary').click();
    assert.equal(await branch.evaluate((n) => n.open), false);
    await mode(p, 'electrical');
    const anchor = 'anchor:00000000000000000000000000000001';
    await electrical(p).locator(`[data-object="${anchor}"]`).click();
    assert.equal(
      await electrical(p)
        .locator(`button[data-object="${anchor}"]`)
        .evaluate((n) => n === document.activeElement),
      true,
    );
    assert.equal(await p.locator(`svg.drawing [data-object="${anchor}"].selected`).count(), 1);
    const source = electrical(p).locator(
      'details[data-object="subsystem:0000000000000000000000000000000a"]',
    );
    await source.locator(':scope > summary').click({ position: { x: 3, y: 12 } });
    const sourceOpen = await source.evaluate((n) => n.open);
    await mode(p, 'pages');
    assert.equal(await branch.evaluate((n) => n.open), false);
    assert.equal(await p.locator(`svg.drawing [data-object="${anchor}"].selected`).count(), 1);
    await mode(p, 'electrical');
    assert.equal(await source.evaluate((n) => n.open), sourceOpen);
    await p.locator('.canvas').focus();
    await p.keyboard.press(`${process.platform === 'darwin' ? 'Meta' : 'Control'}+Shift+p`);
    await p
      .getByRole('textbox', { name: 'Search commands', exact: true })
      .fill('workspace.explorer.pages');
    await p.locator('.palette .palette-item').click();
    await pages(p).waitFor();
    await mode(p, 'electrical');
    await p
      .locator('.menu')
      .filter({ has: p.locator('summary', { hasText: /^View$/ }) })
      .locator('summary')
      .click();
    await p.locator('.menu-popup button[data-action="workspace.explorer.pages"]').click();
    await pages(p).waitFor();
    await branch.locator(':scope > summary').click();
    await sheet(p, id).focus();
    await p.keyboard.press('Enter');
    assert.equal(await display(p).inputValue(), id);
    assert.deepEqual(await bounds(p), before);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.equal(await readFile(project(), 'utf8'), original);
    await p.screenshot({ path: resolve(root, 'target/ui/page-explorer.png') });
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('page clicks and context menus share sheet identity; draft guards prevent retarget and undo updates the tree', async () => {
    const p = await page({ electrical: false }),
      original = await display(p).inputValue();
    const id = await cover(p);
    await sheet(p, id).click();
    assert.equal(await display(p).inputValue(), id);
    assert.equal(await sheet(p, id).getAttribute('aria-current'), 'page');
    assert.equal(await button(p, 'Create annotation · T').isDisabled(), true);
    await sheet(p, id).click({ button: 'right' });
    await dialog(p).waitFor();
    assert.equal(await dialog(p).getByRole('combobox', { name: 'Edit sheet' }).inputValue(), id);
    await control(p, 'Close').click();
    await dialog(p).waitFor({ state: 'hidden' });
    await sheet(p, original).click();
    await p.locator('.canvas').focus();
    await p.keyboard.press('t');
    const text = p.getByRole('textbox', { name: 'Annotation text', exact: true });
    await text.waitFor();
    await text.fill('Unfinished sheet draft');
    await until(
      async () =>
        (await p.locator('svg.drawing .move-preview[data-object^="label:"]').count()) === 1,
    );
    assert.equal(await sheet(p, id).isDisabled(), true);
    assert.equal(await button(p, 'Electrical explorer').isDisabled(), true);
    await sheet(p, id).dispatchEvent('click');
    await sheet(p, id).dispatchEvent('contextmenu');
    assert.equal(await display(p).inputValue(), original);
    assert.equal(await dialog(p).isVisible(), false);
    assert.equal(await text.inputValue(), 'Unfinished sheet draft');
    await text.press('Escape');
    await until(async () => !(await sheet(p, id).isDisabled()));
    await pressAction(p, 'Undo', 'Undo complete');
    assert.equal(await sheet(p, id).count(), 0);
    assert.doesNotMatch(await p.title(), /^●/);
    await pressAction(p, 'Redo', 'Redo complete');
    assert.equal(await sheet(p, id).count(), 1);
    await pressAction(p, 'Undo', 'Undo complete');
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('real document groups preserve explicit membership order, empty packages and literal long titles across refresh', async () => {
    const data = JSON.parse(await readFile(project(), 'utf8'));
    const source = data.sheets[0],
      doc = data.drawing_documents[0];
    const clone = {
      ...source,
      id: 'a'.repeat(32),
      number: 'COVER',
      title: 'Cover <literal>\nSecond line',
      placement: null,
    };
    data.sheets.push(clone);
    doc.title = 'Main package';
    data.drawing_documents.push({
      ...doc,
      id: 'b'.repeat(32),
      number: 'AUX',
      title: 'Service\n' + 'Long title '.repeat(30),
      sheets: [clone.id],
    });
    data.drawing_documents.push({
      ...doc,
      id: 'c'.repeat(32),
      number: 'EMPTY',
      title: 'Empty package',
      sheets: [],
    });
    await replaceProject(data);
    const p = await page({ electrical: false }),
      before = await bounds(p);
    assert.equal(await pages(p).locator('[data-document]').count(), 3);
    assert.equal(
      await pages(p).locator(`[data-document="${doc.id}"] [data-sheet="${source.id}"]`).count(),
      1,
    );
    assert.equal(
      await pages(p)
        .locator(`[data-document="${'b'.repeat(32)}"] [data-sheet="${clone.id}"]`)
        .count(),
      1,
    );
    assert.equal(
      await pages(p)
        .locator(`[data-document="${'c'.repeat(32)}"]`)
        .getByText('No sheets in this document.', { exact: true })
        .count(),
      1,
    );
    assert.equal(await pages(p).locator('literal').count(), 0);
    await sheet(p, clone.id).focus();
    await p.keyboard.press('Enter');
    assert.equal(await display(p).inputValue(), clone.id);
    await mode(p, 'electrical');
    assert.equal(await electrical(p).getByText('FUNCTIONAL STRUCTURE', { exact: true }).count(), 1);
    await mode(p, 'pages');
    assert.equal(await sheet(p, clone.id).getAttribute('aria-current'), 'page');
    assert.deepEqual(await bounds(p), before);
    assert.doesNotMatch(await p.title(), /^●/);
    await p.setViewportSize({ width: 960, height: 640 });
    const rect = await bounds(p);
    assert.ok(rect.width > 0 && rect.x + rect.width <= 960);
    assert.equal(
      await p.locator('.workspace').evaluate((n) => n.scrollWidth > n.clientWidth),
      false,
    );
    await p.screenshot({ path: resolve(root, 'target/ui/document-explorer.png') });
    assert.deepEqual(p.errors, []);
    await p.close();
    const sole = {
      ...data,
      drawing_documents: [{ ...data.drawing_documents[1], sheets: [clone.id, source.id] }],
    };
    await replaceProject(sole);
    const single = await page({ electrical: false });
    assert.equal(await pages(single).locator('[data-document]').count(), 1);
    assert.deepEqual(
      await pages(single)
        .locator('button[data-sheet]')
        .evaluateAll((nodes) => nodes.map((n) => n.dataset.sheet)),
      [clone.id, source.id],
    );
    assert.deepEqual(single.errors, []);
    await single.close();
  });
}
