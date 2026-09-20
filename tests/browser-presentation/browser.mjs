import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {withBrowser} from '../../sdk/browser/browser-test-server.mjs';
import {maximumCases} from './maximum-fixtures.mjs';

const here = dirname(fileURLToPath(import.meta.url)), root = resolve(here, '../..');
const names = ['presentation-dom.mjs', 'presentation-wire.mjs', 'browser-fixture.mjs', 'maximum-fixtures.mjs'];
const path = name => join(['browser-fixture.mjs', 'maximum-fixtures.mjs'].includes(name) ? here : join(root, 'sdk/browser'), name);
const sha = value => createHash('sha256').update(value).digest('hex');
const closure = () => Object.fromEntries([...names.map(name => [name, path(name)]),
  ['identity.mjs', join(root, 'sdk/browser/identity.mjs')],
  ['browser-test-server.mjs', join(root, 'sdk/browser/browser-test-server.mjs')],
  ['browser.mjs', fileURLToPath(import.meta.url)],
].map(([name, path]) => [name, sha(readFileSync(path))]));
export const expectedCases = [
  'semantic-safe-text-catalogue-focus-live',
  'full-validation-stale-and-secret-rejection-before-mutation',
  'keyed-edit-focus-reset-and-modeled-lifecycle',
  'source-owned-draft-reset-context-and-stale-response',
  'exact-default-bytes-and-native-edited-values',
  'required-utf8-request-bounds-and-choice-bindings',
  'serialized-reentrancy-stale-results-and-private-diagnostics',
  'terminal-close-listeners-late-result-and-root-ownership',
  'native-byte-brands-detached-shared-and-closed-arity',
  'closed-options-and-immutable-catalogue-capture',
  'combined-structural-maxima-and-overruns',
  'actual-native-keyboard-default-and-button-submission',
];

export async function journey(build, replacements = {}) {
  return withBrowser(async ({browser, baseURL}) => {
    const page = await browser.newPage(), errors = [];
    page.on('pageerror', error => errors.push(error.message));
    for (const name of names) await page.route('**/' + name, route => route.fulfill({status: 200,
      contentType: 'text/javascript', body: replacements[name] ?? readFileSync(path(name), 'utf8')}));
    await page.route(baseURL, route => route.fulfill({status: 200, headers: {
      'content-type': 'text/html', 'cross-origin-opener-policy': 'same-origin',
      'cross-origin-embedder-policy': 'require-corp',
    }, body: '<!doctype html><title>Private typed presentation acceptance</title>'}));
    await page.goto(baseURL);
    let timer;
    try {
      return await Promise.race([(async () => {
        const input = build ? {wire: [...build.wasmBytes], fixture: [...build.fixtureBytes], labels: [...build.labelsBytes]} : {adapterOnly: true};
        const initial = await page.evaluate(async input =>
          (await import('./browser-fixture.mjs')).runFixture(input), input);
        assert.equal(initial.modelChecked, Boolean(build), 'evidence level is explicit');
        const view = page.locator('[data-keyboard-presentation]');
        await view.getByLabel('Input', {exact: true}).fill('from real keyboard');
        await view.getByLabel('Input', {exact: true}).press('Enter');
        await page.waitForFunction(() => globalThis.__presentationJourney.count() === 1);
        await view.getByRole('button', {name: 'First', exact: true}).focus();
        await view.getByRole('button', {name: 'First', exact: true}).press('Enter');
        await page.waitForFunction(() => globalThis.__presentationJourney.count() === 2);
        const result = await page.evaluate(() => globalThis.__presentationJourney.finish());
        assert.deepEqual(result.cases, expectedCases, 'every named DOM journey actually ran');
        assert.deepEqual(errors, [], 'no detached browser exceptions');
        assert.equal(result.keyboard[0][3][0][1], 'from real keyboard');
        return result;
      })(), new Promise((_, reject) => { timer = setTimeout(() => reject(Error('presentation browser journey deadline')), 90000); })]);
    } finally { clearTimeout(timer); }
  });
}

export async function verifyJourneys(t, build) {
  assert.ok(build?.wasmBytes && build?.fixtureBytes && build?.labelsBytes,
    'generated codec, modeled fixture and source catalogue are mandatory for owning acceptance');
  const before = closure(), result = await journey(build);
  assert.equal(result.modelChecked, true); assert.ok(result.calls.length > 0);
  assert.ok(result.maxMemory > 0 && result.maxMemory <= 16384 * 65536);
  assert.deepEqual(closure(), before, 'actual DOM and codec source closure remained frozen');
  t?.diagnostic(JSON.stringify({cases: result.cases, calls: result.calls.length,
    maximum: result.maxMemory, sources: before}));
  return result;
}

export async function verifyMaximum(t, build) {
  assert.ok(build?.wasmBytes, 'generated codec is mandatory for maximum acceptance');
  const before = closure();
  const results = await withBrowser(async ({browser, baseURL}) => {
    const observed = [];
    for (const id of maximumCases) {
      const page = await browser.newPage(), errors = [];
      page.on('pageerror', error => errors.push(error.message));
      for (const name of names) await page.route('**/' + name, route => route.fulfill({status: 200,
        contentType: 'text/javascript', body: readFileSync(path(name), 'utf8')}));
      await page.goto(baseURL);
      let timer;
      try {
        const result = await Promise.race([
          page.evaluate(async input => (await import('./browser-fixture.mjs')).runMaximumFixture(input), {wire: [...build.wasmBytes], id}),
          new Promise((_, reject) => { timer = setTimeout(() => reject(Error('presentation maximum browser deadline')), 180000); }),
        ]);
        assert.deepEqual(errors, []); assert.equal(result.id, id); observed.push(result);
      } finally { clearTimeout(timer); await page.close(); }
    }
    return observed;
  });
  assert.deepEqual(results.map(row => row.id), maximumCases);
  for (const result of results) {
    assert.equal(result.frame_length, 67108864);
    assert.equal(result.request_sha256, result.response_sha256);
    assert.ok(result.maximum_memory > 0 && result.maximum_memory <= 16384 * 65536);
    assert.equal(result.modelChecked, true);
    t?.diagnostic(JSON.stringify(result));
  }
  assert.deepEqual(closure(), before);
  return results;
}

export async function verifyMutants(t, build) {
  const before = closure(), original = readFileSync(path('presentation-dom.mjs'), 'utf8');
  const mutants = [
    ['unsafe DOM sink', 'node.textContent = text', 'node.innerHTML = text', /text-only closed DOM/],
    ['stale equal revision', "if (frame && next[1] <= frame[1]) fail('stale');", '', /expected presentation refusal stale/],
    ['lost edited control value', 'const preserve = retained && record.defaultValue === defaultValue && record.draftEpoch === content[6];', 'const preserve = false;', /keyed edits survive/],
    ['missing modeled draft epoch', ' && record.draftEpoch === content[6]', '', /modeled draft epoch clears/],
    ['changed kind steals implicit focus', 'surviving?.tag === focused?.tag ? surviving : undefined', 'surviving', /changed control kind cannot inherit/],
    ['reset restores stale selection', ' && chosen.preservedDraft', '', /same-kind reset retains focus/],
    ['normalized generated defaults', 'value = record.defaultValue;', 'value = control.value;', /untouched defaults retain/],
    ['forgotten listener cleanup', "root.removeEventListener('submit', onSubmit);", '', /terminal close removes/],
    ['old owner removes new DOM', 'if (roots.get(root) === ownership)', 'if (true)', /closed view and late result/],
  ];
  for (const [name, from, to, diagnostic] of mutants) {
    assert.equal(original.split(from).length, 2, 'one exact actual adapter mutation: ' + name);
    const changed = original.replace(from, to);
    await assert.rejects(journey(build, {'presentation-dom.mjs': changed}), diagnostic, name);
    t?.diagnostic('killed ' + name);
  }
  assert.deepEqual(closure(), before, 'mutants never modify authoritative source');
}
