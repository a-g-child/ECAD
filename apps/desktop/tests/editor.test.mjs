import { registerTemplateInlineText } from './template-inline-text.mjs';
import { registerTemplateClickPlacement } from './template-click-placement.mjs';
import { registerTemplateContextualTools } from './template-contextual-tools.mjs';
import { registerPropertyCompletionTests } from './template-property-completion.mjs';
import { registerTemplateSelectionTests } from './template-selection.mjs';
import { registerTemplateArtworkPointerTests } from './template-artwork-pointer.mjs';
import { registerTemplateArtworkTests } from './template-artwork.mjs';
/** Local integration tests: real Rust controller + Vite + Chromium, no mocked electrical model. */
import { test, before, after } from 'node:test';
import assert from 'node:assert/strict';
import { spawn, execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdtemp, readFile, writeFile, mkdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { chromium } from 'playwright';
import { registerControlsTests } from './controls.mjs';
import { registerTransformTests } from './transforms.mjs';
import { registerAnnotationTests } from './annotations.mjs';
import { registerMultilineTests } from './multiline.mjs';
import { registerPathTests } from './paths.mjs';
import { registerRibbonTests } from './ribbons.mjs';
import { registerArrangementTests } from './arrangement.mjs';
import { registerDefaultsTests } from './defaults.mjs';
import { registerMetadataTests } from './metadata.mjs';
import { registerProfileTests } from './profiles.mjs';
import { registerSheetTests } from './sheets.mjs';
import { registerSheetEditingTests } from './sheet-editing.mjs';
import { registerProjectPageTests } from './project-pages.mjs';
import { registerExplorerTests } from './explorer.mjs';
import { registerTemplateEditorTests } from './template-editor.mjs';
import { registerTemplateTableTests } from './template-tables.mjs';
import { registerTemplateTableInteractionTests } from './template-table-interactions.mjs';
import { registerTemplateMarginTests } from './template-margins.mjs';
const root = resolve(import.meta.dirname, '../../..');
const processes = [];
let browser;
let directory;
let project;
let backendLog;
const hostBinary = resolve(
  root,
  `target/debug/examples/browser_host${process.platform === 'win32' ? '.exe' : ''}`,
);
const route = 'route:00000000000000000000000000000001';
const label = 'label:00000000000000000000000000000001';
const primary = process.platform === 'darwin' ? 'Meta' : 'Control';
function start(command, args, cwd, env = {}) {
  const child = spawn(command, args, {
    cwd,
    env: { ...process.env, ...env },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  let output = '';
  child.stdout.on('data', (d) => (output += d));
  child.stderr.on('data', (d) => (output += d));
  processes.push(child);
  const log = () => output;
  log.stop = () =>
    new Promise((done) => {
      if (child.exitCode !== null || child.signalCode !== null) return done();
      child.once('exit', done);
      child.kill('SIGTERM');
    });
  return log;
}
async function until(fn, timeout = 20000) {
  const end = Date.now() + timeout;
  while (Date.now() < end) {
    try {
      if (await fn()) return;
    } catch {}
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error('Timed out waiting for local UI server');
}
before(async () => {
  directory = await mkdtemp(resolve(tmpdir(), 'ecad-ui-'));
  project = resolve(directory, 'demo.ecad.json');
  await promisify(execFile)(
    'cargo',
    ['build', '--offline', '--locked', '-p', 'ecad-bridge', '--example', 'browser_host'],
    { cwd: root },
  );
  backendLog = start(hostBinary, [], root, { ECAD_UI_PROJECT: project });
  await until(() => backendLog().includes('Browser harness listening'));
  const initial = JSON.parse(await readFile(project, 'utf8'));
  const templates = JSON.parse(
    await readFile(
      resolve(root, 'crates/ecad-adapters/tests/fixtures/authoring-saved-templates-v1.json'),
      'utf8',
    ),
  );
  initial.template_revisions.push(...templates.revisions);
  await replaceProject(initial);

  const frontendLog = start(
    'npm',
    ['exec', '--', 'vite', '--host', '127.0.0.1', '--port', '1421', '--strictPort'],
    resolve(root, 'apps/desktop'),
  );
  await until(async () => {
    const response = await fetch('http://127.0.0.1:1421');
    return response.ok;
  });
  if (frontendLog().includes('error')) throw new Error(frontendLog());
  browser = await chromium.launch({ headless: true });
});
after(async () => {
  if (browser) await browser.close();
  for (const child of processes) child.kill('SIGTERM');
  if (directory) await rm(directory, { recursive: true, force: true });
});
// Object-editing suites explicitly choose Electrical; explorer suites exercise the default Pages mode.
async function page({ electrical = true } = {}) {
  const page = await browser.newPage({ viewport: { width: 1360, height: 880 } });
  const errors = [];
  page.on('pageerror', (e) => errors.push(e.message));
  page.errors = errors;
  page.setDefaultTimeout(10000);
  await page.goto('http://127.0.0.1:1421');
  await page.waitForFunction(() =>
    document.querySelector('.status-message')?.textContent?.startsWith('Ready'),
  );
  if (electrical) {
    await page.getByRole('button', { name: 'Electrical explorer', exact: true }).click();
    await page.waitForFunction(
      () => document.querySelector('.tree')?.dataset.mode === 'electrical',
    );
  }
  await until(
    async () =>
      !(await page.getByRole('button', { name: 'Save project', exact: true }).isDisabled()),
  );
  await page.locator('.canvas').focus();
  return page;
}
const tree = (p, target) => p.locator(`.tree-item[data-object="${target}"]`);
const input = (p, key) => p.locator(`[data-property="${key}"] :is(input,textarea)`);
const button = (p, name) => p.getByRole('button', { name, exact: true });
async function changed(p, fn) {
  await fn();
  await p.waitForFunction(
    () =>
      document.querySelector('.status-message')?.textContent ===
      'Property updated · one undo operation',
  );
}
async function pressAction(p, name, expected) {
  await button(p, name).click();
  if (expected)
    await p.waitForFunction(
      (message) => document.querySelector('.status-message')?.textContent === message,
      expected,
    );
  // The same status can occur on consecutive Undo operations. Wait for the
  // ordinary command controls to unlock before inspecting its resulting state.
  await until(async () => !(await button(p, 'Save project').isDisabled()));
}
async function bounds(p) {
  return p.locator('.canvas').boundingBox();
}

test('real property edit, validation geometry stability, guarded save and exact undo/redo', async () => {
  const p = await page();
  const point = await p.locator(`svg [data-object="${route}"] .hit-path`).evaluate((node) => {
    const point = node.getPointAtLength(node.getTotalLength() / 2);
    const screen = point.matrixTransform(node.getScreenCTM());
    return { x: screen.x, y: screen.y };
  });
  await p.mouse.click(point.x, point.y);
  await input(p, 'stroke_width').waitFor();
  assert.equal(await input(p, 'stroke_width').inputValue(), '500');
  const before = await bounds(p);
  await input(p, 'stroke_width').fill('0');
  await input(p, 'stroke_width').press('Enter');
  await p.waitForFunction(() =>
    document.querySelector('.status-message')?.textContent?.includes('positive'),
  );
  assert.deepEqual(await bounds(p), before);
  assert.equal(await input(p, 'stroke_width').inputValue(), '0');
  assert.equal(await input(p, 'stroke_width').evaluate((e) => document.activeElement === e), true);
  await changed(p, async () => {
    await input(p, 'stroke_width').fill('700');
    await input(p, 'stroke_width').press('Enter');
  });
  assert.equal(
    await p.locator(`svg [data-object="${route}"] .ink`).getAttribute('stroke-width'),
    '0.7',
  );
  await pressAction(p, 'Save project', 'Project saved');
  let saved = JSON.parse(await readFile(project, 'utf8'));
  assert.equal(saved.drawing.routes[0].appearance.overrides.stroke_width_um, 700);
  await pressAction(p, 'Undo', 'Undo complete');
  assert.equal(await input(p, 'stroke_width').inputValue(), '500');
  assert.match(await p.title(), /^●/);
  await pressAction(p, 'Redo', 'Redo complete');
  assert.equal(await input(p, 'stroke_width').inputValue(), '700');
  assert.doesNotMatch(await p.title(), /^●/);
  await pressAction(p, 'Undo', 'Undo complete');
  await pressAction(p, 'Save project', 'Project saved');
  assert.deepEqual(p.errors, []);
  await p.close();
});

test('mixed selection uses applicable subsets and keyboard undo without container shifts', async () => {
  const p = await page();
  await tree(p, route).click();
  await tree(p, label).click({ modifiers: ['Shift'] });
  await until(
    async () => (await input(p, 'drawing_colour').getAttribute('placeholder')) === 'Mixed',
  );
  assert.equal(await input(p, 'drawing_colour').getAttribute('placeholder'), 'Mixed');
  assert.equal(
    await p.locator('[data-property="stroke_width"] .property-count').innerText(),
    '1/2',
  );
  const before = await bounds(p);
  await changed(p, async () => {
    await input(p, 'drawing_colour').fill('#b03040');
    await input(p, 'drawing_colour').press('Enter');
  });
  assert.equal(
    await p.locator(`svg [data-object="${route}"] .ink`).getAttribute('stroke'),
    '#b03040',
  );
  assert.equal(
    await p.locator(`svg [data-object="${label}"] .ink`).getAttribute('fill'),
    '#b03040',
  );
  assert.deepEqual(await bounds(p), before);
  await p.keyboard.press(`${primary}+z`);
  await until(
    async () => (await input(p, 'drawing_colour').getAttribute('placeholder')) === 'Mixed',
  );
  assert.equal(
    await p.locator(`svg [data-object="${route}"] .ink`).getAttribute('stroke'),
    '#2b6c5e',
  );
  const viewbox = await p.locator('svg.drawing').getAttribute('viewBox');
  await p.locator('.canvas').hover();
  await p.mouse.wheel(0, 300);
  await until(async () => (await p.locator('svg.drawing').getAttribute('viewBox')) !== viewbox);
  assert.notEqual(await p.locator('svg.drawing').getAttribute('viewBox'), viewbox);
  const zoomed = await p.locator('svg.drawing').getAttribute('viewBox');
  await p.locator('.canvas').focus();
  await p.keyboard.press('f');
  await until(async () => (await p.locator('svg.drawing').getAttribute('viewBox')) !== zoomed);
  const fitted = (await p.locator('svg.drawing').getAttribute('viewBox')).split(' ').map(Number);
  const expected = viewbox.split(' ').map(Number);
  const tolerance = expected[2] / (await bounds(p)).width;
  assert.ok(
    fitted.every((v, i) => Math.abs(v - expected[i]) <= tolerance),
    JSON.stringify({ fitted, expected }),
  );
  await mkdir(resolve(root, 'target/ui'), { recursive: true });
  await p.screenshot({ path: resolve(root, 'target/ui/editor.png') });
  assert.deepEqual(p.errors, []);
  await p.close();
});

test('long editable values and notifications preserve layout, typing is protected, right click retains selection', async () => {
  const p = await page();
  await tree(p, label).click();
  await input(p, 'drawing_text').waitFor();
  const before = await bounds(p);
  await input(p, 'drawing_text').fill('w m r e');
  await input(p, 'drawing_text').press('f');
  assert.equal(
    await p.locator(`svg [data-object="${label}"] text`).textContent(),
    'CONVEYOR / SIGNAL HARNESS',
  );
  await changed(p, async () => {
    await input(p, 'drawing_text').fill(
      'Very long <img src=x onerror=window.__injected=1> '.repeat(250),
    );
    await input(p, 'drawing_text').press(`${primary}+Enter`);
  });
  assert.deepEqual(await bounds(p), before);
  assert.equal(await p.locator('img').count(), 0);
  await pressAction(p, 'Undo', 'Undo complete');
  await tree(p, label).click({ button: 'right' });
  await input(p, 'drawing_text').waitFor();
  assert.equal(await p.locator('.inspector-pane .pane-tag').innerText(), '1 SELECTED');
  await p.evaluate(() => {
    document.querySelector('.status-message').textContent = 'A very long diagnostic '.repeat(300);
  });
  assert.deepEqual(await bounds(p), before);
  await p.setViewportSize({ width: 980, height: 700 });
  assert.equal(await p.evaluate(() => document.documentElement.scrollWidth > innerWidth), false);
  assert.deepEqual(p.errors, []);
  await p.close();
});

test('report action uses actual electrical identities and external save conflicts preserve file and edits', async () => {
  const p = await page();
  await button(p, 'Connection reports').click();
  await p.locator('.report-pane:not([hidden])').waitFor();
  assert.equal(await p.locator('.report-pane tr').count(), 3);
  assert.match(await p.locator('.report-body').innerText(), /-Controller/);
  const reportBounds = await bounds(p);
  await tree(p, route).click();
  await input(p, 'stroke_width').waitFor();
  await changed(p, async () => {
    await input(p, 'stroke_width').fill('650');
    await input(p, 'stroke_width').press('Enter');
  });
  assert.deepEqual(await bounds(p), reportBounds);
  assert.equal(await p.locator('.report-body tr').count(), 0);
  await button(p, 'Refresh connection table').click();
  await until(async () => (await p.locator('.report-body tr').count()) === 3);
  await button(p, 'Close').click();
  const original = await readFile(project, 'utf8');
  await writeFile(project, 'external edit');
  await button(p, 'Save project').click();
  await p.waitForFunction(() =>
    document.querySelector('.status-message')?.classList.contains('error'),
  );
  assert.equal(await readFile(project, 'utf8'), 'external edit');
  assert.equal(await input(p, 'stroke_width').inputValue(), '650');
  await writeFile(project, original);
  await pressAction(p, 'Save project', 'Project saved');
  await pressAction(p, 'Undo', 'Undo complete');
  await pressAction(p, 'Save project', 'Project saved');
  assert.deepEqual(p.errors, []);
  await p.close();
});

test('move group previews connected routes, cancels cleanly and commits one exact undoable edit', async () => {
  const p = await page();
  const a = 'anchor:00000000000000000000000000000001';
  const b = 'anchor:00000000000000000000000000000002';
  const points = () =>
    p.locator(`svg [data-object="${route}"] polyline.ink`).getAttribute('points');
  const original = await points();
  const before = await bounds(p);
  await tree(p, a).click();
  await tree(p, b).click({ modifiers: ['Shift'] });
  await p.locator('.canvas').focus();
  await p.keyboard.press('m');
  const x = p.getByRole('textbox', { name: 'Move X (µm)', exact: true });
  const y = p.getByRole('textbox', { name: 'Move Y (µm)', exact: true });
  await x.waitFor();
  assert.deepEqual(await bounds(p), before);
  await x.fill('10000');
  await y.fill('5000');
  await until(async () => (await points()) === '10,5 30,5 30,20 60,20 60,5');
  assert.doesNotMatch(await p.title(), /^●/);
  await y.press('Escape');
  await p.waitForFunction(
    () => document.querySelector('.status-message')?.textContent === 'Move cancelled',
  );
  await until(async () => !(await x.isVisible()));
  assert.equal(await points(), original);
  assert.deepEqual(await bounds(p), before);
  assert.doesNotMatch(await p.title(), /^●/);
  await p.locator('.canvas').focus();
  await p.keyboard.press('m');
  await x.waitFor();
  await x.fill('10000');
  await y.fill('5000');
  await until(async () => (await points()) === '10,5 30,5 30,20 60,20 60,5');
  await x.fill('invalid');
  await until(async () => (await points()) === original);
  assert.equal(await button(p, 'Apply').isDisabled(), true);
  await x.fill('10000');
  await until(async () => !(await button(p, 'Apply').isDisabled()));
  await button(p, 'Apply').click();
  await p.waitForFunction(
    () =>
      document.querySelector('.status-message')?.textContent ===
      'Selection moved · one undo operation',
  );
  assert.equal(await points(), '10,5 30,5 30,20 60,20 60,5');
  assert.match(await p.title(), /^●/);
  await pressAction(p, 'Save project', 'Project saved');
  const saved = JSON.parse(await readFile(project, 'utf8'));
  assert.deepEqual(saved.drawing.routes[0].points_um, [
    [10000, 5000],
    [30000, 5000],
    [30000, 20000],
    [60000, 20000],
    [60000, 5000],
  ]);
  await pressAction(p, 'Undo', 'Undo complete');
  assert.equal(await points(), original);
  await pressAction(p, 'Redo', 'Redo complete');
  assert.equal(await points(), '10,5 30,5 30,20 60,20 60,5');
  await pressAction(p, 'Undo', 'Undo complete');
  await pressAction(p, 'Save project', 'Project saved');
  assert.deepEqual(p.errors, []);
  await p.close();
});

test('pointer move uses visible terminal snapping, cycling and temporary suppression', async () => {
  const p = await page();
  await tree(p, label).click();
  await p.locator('.canvas').focus();
  await p.keyboard.press('m');
  await p.getByRole('textbox', { name: 'Move X (µm)', exact: true }).waitFor();
  const screen = await p.locator('svg.drawing').evaluate((svg) => {
    const point = new DOMPoint(100, 0).matrixTransform(svg.getScreenCTM());
    return { x: point.x, y: point.y };
  });
  await p.mouse.move(screen.x, screen.y);
  await until(async () => (await p.locator('.snap-marker').count()) === 1);
  await p.waitForFunction(() =>
    document.querySelector('.status-message')?.textContent?.startsWith('Terminal snap'),
  );
  await p.keyboard.press('Tab');
  await p.waitForFunction(() =>
    document.querySelector('.status-message')?.textContent?.startsWith('Endpoint snap'),
  );
  await p.keyboard.down('Alt');
  await until(async () => (await p.locator('.snap-marker').count()) === 0);
  await p.keyboard.up('Alt');
  await until(async () => (await p.locator('.snap-marker').count()) === 1);
  await p.screenshot({ path: resolve(root, 'target/ui/move-preview.png') });
  await until(async () => !(await button(p, 'Apply').isDisabled()));
  await p.mouse.click(screen.x, screen.y);
  await p.waitForFunction(
    () =>
      document.querySelector('.status-message')?.textContent ===
      'Selection moved · one undo operation',
  );
  assert.equal(await p.locator(`svg [data-object="${label}"] text.ink`).getAttribute('x'), '100');
  assert.equal(await p.locator(`svg [data-object="${label}"] text.ink`).getAttribute('y'), '0');
  await pressAction(p, 'Undo', 'Undo complete');
  assert.doesNotMatch(await p.title(), /^●/);
  assert.deepEqual(p.errors, []);
  await p.close();
});

registerControlsTests({ page, tree, button, until, pressAction, root, route, label });

registerTransformTests({
  page,
  tree,
  button,
  until,
  pressAction,
  bounds,
  root,
  route,
  project: () => project,
});

registerAnnotationTests({
  page,
  button,
  until,
  pressAction,
  bounds,
  input,
  root,
  project: () => project,
});

registerMultilineTests({
  page,
  tree,
  button,
  input,
  until,
  pressAction,
  changed,
  bounds,
  root,
  project: () => project,
});

registerPathTests({
  page,
  button,
  until,
  pressAction,
  bounds,
  input,
  root,
  project: () => project,
});

registerRibbonTests({ page, tree, button, input, until, bounds, root, project: () => project });

registerArrangementTests({
  page,
  tree,
  button,
  until,
  pressAction,
  bounds,
  root,
  project: () => project,
});

registerDefaultsTests({ page, button, until, bounds, root, project: () => project, pressAction });

registerMetadataTests({ page, button, until, bounds, root, project: () => project, pressAction });

registerProfileTests({ page, button, until, bounds, root, project: () => project, pressAction });

registerSheetTests({
  page,
  button,
  until,
  bounds,
  root,
  project: () => project,
  pressAction,
  replaceProject,
});

registerSheetEditingTests({ page, button, until, bounds, project: () => project, pressAction });

// Fixtures are written outside the app, then opened by a fresh real Rust host. No test mutation API.
async function replaceProject(data) {
  await backendLog.stop();
  await writeFile(project, JSON.stringify(data, null, 2) + '\n');
  backendLog = start(hostBinary, [], root, { ECAD_UI_PROJECT: project });
  await until(() => backendLog().includes('Browser harness listening'));
}
registerExplorerTests({
  page,
  button,
  until,
  bounds,
  root,
  project: () => project,
  pressAction,
  replaceProject,
});

registerProjectPageTests({
  page,
  button,
  until,
  bounds,
  root,
  project: () => project,
  pressAction,
  replaceProject,
});

registerTemplateEditorTests({ page, button, until, project: () => project, root, pressAction });

registerTemplateTableTests({ page, button, until, project: () => project, root });

registerTemplateTableInteractionTests({ page, button, until, project: () => project, root });

registerTemplateMarginTests({ page, button, until, project: () => project, root });

registerTemplateArtworkTests({ page, button, until });

registerTemplateArtworkPointerTests({ page, button, until });

registerTemplateSelectionTests({ page, button, until });

registerPropertyCompletionTests({ page, button, until });

registerTemplateContextualTools({ page, button, until });

registerTemplateClickPlacement({ page, button, until });

registerTemplateInlineText({ page, button, until });
