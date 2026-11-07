/** Real personal profile storage and reviewed project copy with isolated temporary files. */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
export function registerProfileTests({
  page,
  button,
  until,
  bounds,
  root,
  project,
  pressAction: sharedAction,
}) {
  const dialog = (p) => p.getByRole('dialog', { name: 'Company / contact profiles', exact: true });
  const field = (p, name) => dialog(p).getByRole('textbox', { name, exact: true });
  const control = (p, name) => dialog(p).getByRole('button', { name, exact: true });
  const check = (p, name) =>
    dialog(p).getByRole('checkbox', { name: `Import ${name}`, exact: true });
  const select = (p) => dialog(p).getByRole('combobox', { name: 'Saved profile' });
  const profiles = () => resolve(dirname(project()), 'company-profiles.json');
  async function open(p) {
    await p.getByRole('tab', { name: 'File', exact: true }).click();
    await button(p, 'Company / contact profiles').click();
    await dialog(p).waitFor();
  }
  async function action(p, name, message) {
    await p.getByRole('tab', { name: 'Drawing', exact: true }).click();
    await sharedAction(p, name, message);
  }
  async function preview(p) {
    await control(p, 'Preview import').click();
    await until(async () => !(await control(p, 'Apply import').isDisabled()));
  }
  test('profiles save independently, review selected literal fields, cancel safely and import one exact undoable project copy', async () => {
    const p = await page(),
      original = await readFile(project(), 'utf8'),
      before = await bounds(p);
    await open(p);
    assert.deepEqual(await bounds(p), before);
    await control(p, 'Save profile').click();
    await until(async () =>
      (await p.locator('.profiles-dialog .settings-error').innerText()).includes('profile name'),
    );
    await field(p, 'Profile name').fill('Workshop');
    await field(p, 'Company name').fill('Example & Co');
    await field(p, 'Company address').fill(' Unit 2\n\nExample Road\n');
    await control(p, 'Save profile').click();
    await until(async () =>
      (await p.locator('.profiles-dialog .settings-error').innerText()).includes('Profile saved'),
    );
    const saved = await readFile(profiles(), 'utf8');
    assert.equal(JSON.parse(saved).profiles[0].name, 'Workshop');
    assert.equal(await readFile(project(), 'utf8'), original);
    assert.doesNotMatch(await p.title(), /^●/);
    assert.equal(await check(p, 'Engineer / contact').isChecked(), false);
    await check(p, 'Company address').uncheck();
    await preview(p);
    assert.match(await p.locator('.profile-review').innerText(), /Example & Co/);
    assert.equal(await readFile(project(), 'utf8'), original);
    await p.setViewportSize({ width: 960, height: 640 });
    const rect = await dialog(p).boundingBox(),
      apply = await control(p, 'Apply import').boundingBox();
    assert.ok(
      rect.x >= 0 && rect.x + rect.width <= 960 && rect.y >= 0 && rect.y + rect.height <= 640,
    );
    assert.ok(apply.y + apply.height <= rect.y + rect.height);
    await p.screenshot({ path: resolve(root, 'target/ui/profile-import.png') });
    await control(p, 'Close').click();
    assert.equal(await readFile(project(), 'utf8'), original);
    await p.setViewportSize({ width: 1360, height: 880 });
    await open(p);
    await select(p).selectOption({ label: 'Workshop' });
    await check(p, 'Company address').uncheck();
    await preview(p);
    await control(p, 'Apply import').click();
    await dialog(p).waitFor({ state: 'hidden' });
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Profile values imported · one undo operation',
    );
    assert.deepEqual(await bounds(p), before);
    await action(p, 'Save project', 'Project saved');
    assert.deepEqual(JSON.parse(await readFile(project(), 'utf8')).metadata, {
      'company.name': 'Example & Co',
    });
    assert.equal(await readFile(profiles(), 'utf8'), saved);
    await action(p, 'Undo', 'Undo complete');
    await action(p, 'Redo', 'Redo complete');
    assert.doesNotMatch(await p.title(), /^●/);
    await action(p, 'Undo', 'Undo complete');
    await action(p, 'Save project', 'Project saved');
    assert.deepEqual(JSON.parse(await readFile(project(), 'utf8')).metadata, {});
    assert.deepEqual(p.errors, []);
    await p.close();
  });
  test('profile reload survives frontend restart, refuses external overwrites and keeps redo through personal saves and unchanged imports', async () => {
    const p = await page();
    await open(p);
    await select(p).selectOption({ label: 'Workshop' });
    assert.equal(await field(p, 'Company address').inputValue(), ' Unit 2\n\nExample Road\n');
    const original = JSON.parse(await readFile(profiles(), 'utf8')),
      id = original.profiles[0].id;
    await field(p, 'Profile name').fill('Workshop updated');
    await control(p, 'Save profile').click();
    await until(async () =>
      (await p.locator('.profiles-dialog .settings-error').innerText()).includes('Profile saved'),
    );
    const external = JSON.parse(await readFile(profiles(), 'utf8'));
    assert.equal(external.profiles[0].id, id);
    external.profiles[0].name = 'External update';
    await writeFile(profiles(), JSON.stringify(external, null, 2) + '\n');
    const bytes = await readFile(profiles(), 'utf8');
    await field(p, 'Company name').fill('Stale replacement');
    await control(p, 'Save profile').click();
    await until(async () =>
      (await p.locator('.profiles-dialog .settings-error').innerText()).includes('changed'),
    );
    assert.equal(await field(p, 'Company name').inputValue(), 'Stale replacement');
    assert.equal(await readFile(profiles(), 'utf8'), bytes);
    await control(p, 'Reload profiles').click();
    await until(async () => !(await select(p).isDisabled()));
    await select(p).selectOption({ label: 'External update' });
    assert.equal(await field(p, 'Company name').inputValue(), 'Example & Co');
    await check(p, 'Company name').uncheck();
    await check(p, 'Company address').uncheck();
    await preview(p);
    assert.match(await p.locator('.profile-review').innerText(), /already match/);
    await control(p, 'Apply import').click();
    await dialog(p).waitFor({ state: 'hidden' });
    await p.waitForFunction(
      () =>
        document.querySelector('.status-message')?.textContent ===
        'Profile values already match · no document changes',
    );
    await p.getByRole('tab', { name: 'Drawing', exact: true }).click();
    assert.equal(await button(p, 'Redo').isDisabled(), false);
    assert.doesNotMatch(await p.title(), /^●/);
    await action(p, 'Redo', 'Redo complete');
    await action(p, 'Undo', 'Undo complete');
    await action(p, 'Save project', 'Project saved');
    assert.deepEqual(p.errors, []);
    await p.close();
  });
}
