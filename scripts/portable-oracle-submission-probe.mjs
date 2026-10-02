// Supplemental oracle-boundary probes. Run each case in its own bounded
// devcontainer; the real upstream executor and generated artifact are retained.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const [oracle, artifactDirectory, browser, name, evidenceDirectory, ...extra] = process.argv.slice(2);
assert.equal(extra.length, 0);
assert.ok(oracle && artifactDirectory && browser && name && evidenceDirectory,
  'usage: probe ORACLE ARTIFACT_DIRECTORY BROWSER CASE NEW_EVIDENCE_DIRECTORY');
const sourcePath = fileURLToPath(new URL('../crates/prismpm/src/embedded/hologram-oracle.browser.mjs', import.meta.url));
const source = readFileSync(sourcePath, 'utf8');
const marker = "  await journey('modeled-vectors', async () => {";
assert.equal(source.split(marker).length, 2, 'probe insertion point must be unique');
const injections = {
  positive: '',
  'delayed-completion': `await page.evaluate(() => {
    const form = document.querySelector('#application-form');
    const original = form.removeAttribute.bind(form);
    window.probeCompletions = 0;
    form.removeAttribute = name => {
      if (name !== 'aria-busy') return original(name);
      setTimeout(() => {original(name); window.probeCompletions++;}, 250);
    };
  });`,
  'stuck-busy': `await page.evaluate(() => {
    const form = document.querySelector('#application-form');
    const original = form.removeAttribute.bind(form);
    form.removeAttribute = name => {if (name !== 'aria-busy') original(name);};
  });`,
  'missing-control': "await page.locator('#submit').evaluate(button => button.remove());",
  'body-unavailable': "await page.route('**/_hologram/intent', route => route.continue({method: 'GET'}));",
  'wrong-method': `await page.evaluate(() => {
    const original = window.fetch;
    window.fetch = (url, options) => original(url, {...options, method: 'GET', body: undefined});
  });`,
  'wrong-payload': `await page.evaluate(() => {
    const original = window.fetch;
    window.fetch = (url, options) => original(url, {...options, body:
      JSON.stringify({version: 1, name: 'application.invoke', payload: 'oracle-corrupted-payload'})});
  });`,
  duplicate: `await page.evaluate(() => document.querySelector('#application-form').addEventListener('submit', () => {
    void fetch('/_hologram/intent', {method: 'POST', headers: {'content-type': 'application/json'},
      body: JSON.stringify({version: 1, name: 'application.invoke', payload: document.querySelector('#request').value})});
  }));`,
  navigation: `await page.evaluate(() => document.querySelector('#application-form').addEventListener('submit', () => {
    location.assign('/index.html');
  }));`,
  'trigger-failure': `await page.evaluate(() => document.querySelector('#request').addEventListener('input', () => {
    document.querySelector('#submit')?.remove();
  }, {once: true}));`,
};
assert.ok(Object.hasOwn(injections, name), 'unknown probe case');
const artifactRoot = resolve(artifactDirectory);
const modelPath = join(artifactRoot, 'model.prism.json');
const modelBytes = readFileSync(modelPath);
const model = JSON.parse(modelBytes);
assert.equal(model.application.profile, 'prismpm/text-application/1');
assert.equal(model.application.name, 'Text Request', 'this probe uses the complete Text Request fixture');
const archive = join(artifactRoot, 'Text Request.holo');
const wasm = join(artifactRoot, 'core-wasm/prism_text_request_core_wasm.wasm');
const evidence = resolve(evidenceDirectory);
mkdirSync(evidence);
const driver = join(evidence, 'driver.mjs');
let driverBytes = name === 'positive' ? source : source.replace(marker, `${marker}\n    ${injections[name]}\n`);
if (name === 'delayed-completion') {
  const submit = '      await submit(vector);';
  assert.equal(source.split(submit).length, 2);
  driverBytes = driverBytes.replace(submit, `
      const completedBefore = await page.evaluate(() => window.probeCompletions);
      await submit(vector);
      assert.equal(await page.evaluate(() => window.probeCompletions), completedBefore + 1,
        'submission returned before delayed completion');`);
  const keyboard = '    await submit(recovery, page, true);';
  assert.equal(source.split(keyboard).length, 2);
  driverBytes = driverBytes.replace(keyboard, `
    const keyboardBefore = await page.evaluate(() => window.probeCompletions);
    await submit(recovery, page, true);
    assert.equal(await page.evaluate(() => window.probeCompletions), keyboardBefore + 1,
      'keyboard submission returned before delayed completion');`);
}
writeFileSync(driver, driverBytes, {flag: 'wx'});
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const subjects = {source: sourcePath, driver, oracle, model: modelPath, archive, wasm,
  node: process.execPath, browser};
const identities = () => Object.fromEntries(Object.entries(subjects).map(([key, path]) =>
  [`${key}_sha256`, sha(readFileSync(path))]));
const before = identities();
const result = spawnSync(resolve(oracle), [archive, modelPath, wasm, driver, process.execPath, resolve(browser)], {
  timeout: 120_000, maxBuffer: 1_048_576, encoding: 'utf8',
});
writeFileSync(join(evidence, 'stdout.txt'), result.stdout ?? '', {flag: 'wx'});
writeFileSync(join(evidence, 'stderr.txt'), result.stderr ?? '', {flag: 'wx'});
const diagnostics = (result.stderr ?? '').split('\n').flatMap(line => {
  try { const value = JSON.parse(line); return value.schema === 'prismpm/browser-submission-diagnostic/1' ? [value] : []; }
  catch { return []; }
});
let accepted = false;
let failure;
try {
  assert.equal(result.error, undefined, 'spawn, timeout or output-limit failure is not oracle evidence');
  assert.equal(result.signal, null, 'signal termination is not oracle evidence');
  assert.deepEqual(identities(), before, 'probe inputs changed during execution');
  if (name === 'positive' || name === 'delayed-completion') {
    assert.equal(result.status, 0);
    const report = JSON.parse(result.stdout.trim());
    assert.equal(report.schema, 'prismpm/hologram-oracle/2');
    assert.equal(report.portable_browser.status, 'passed');
    assert.equal(report.portable_browser.retries, 0);
    assert.equal(report.portable_browser.skipped, 0);
    assert.deepEqual(report.portable_browser.cases.map(row => row.name), [
      'attachment-assets', 'modeled-vectors', 'input-validation-recovery',
      'transport-failure-recovery', 'pre-init-privacy', 'delayed-init',
      'intent-boundaries', 'text-response-bounds', 'text-safe-rendering', 'detached-session']);
    assert.deepEqual(report.portable_browser.vector_indexes, [0, 2, 3]);
    assert.equal(report.portable_browser.browser_version, '151.0.7922.34');
    assert.equal(report.portable_browser.playwright, '1.62.1');
    assert.ok(report.portable_browser.cases.every(row => row.status === 'passed' && row.attempts === 1));
    assert.equal(diagnostics.length, 0);
  } else {
    assert.notEqual(result.status, 0);
    assert.equal(diagnostics.length, 1, 'require the actual submission diagnostic, not an unrelated crash');
    const diagnostic = diagnostics[0];
    assert.equal(diagnostic.keyboard, false);
    if (name === 'missing-control') {
      assert.equal(diagnostic.phase, 'initial-readiness');
      assert.equal(diagnostic.invocationCount, 0);
    } else if (name === 'trigger-failure') {
      assert.equal(diagnostic.phase, 'submission');
      assert.equal(diagnostic.invocationCount, 0);
    } else if (name === 'duplicate') {
      assert.equal(diagnostic.invocationCount, 2);
      assert.match(result.stderr, /submission must issue exactly one invocation/);
    } else if (name === 'navigation') {
      assert.equal(diagnostic.navigated, true);
    } else if (name === 'stuck-busy') {
      assert.equal(diagnostic.phase, 'completed-readiness');
      assert.equal(diagnostic.invocationCount, 1);
      assert.equal(diagnostic.navigated, false);
      assert.match(result.stderr, /waitForFunction.*Timeout/);
    } else {
      assert.equal(diagnostic.invocationCount, 1);
      assert.equal(diagnostic.navigated, false);
      const responses = diagnostic.events.filter(event => event.event === 'response');
      if (name === 'body-unavailable') {
        assert.equal(diagnostic.phase, 'response-body');
        assert.equal(responses.length, 1);
        assert.equal(responses[0].status, 405);
        assert.match(result.stderr, /Network\.getResponseBody.*No data found for resource/);
      } else {
        assert.equal(diagnostic.phase, 'submission');
        assert.equal(responses.length, 0, 'mismatched request must not satisfy response correlation');
        const requests = diagnostic.events.filter(event => event.event === 'request');
        assert.equal(requests.length, 1);
        assert.equal(requests[0].method, name === 'wrong-method' ? 'GET' : 'POST');
        assert.match(result.stderr, /waitForResponse.*Timeout/);
      }
    }
  }
  accepted = true;
} catch (error) { failure = String(error); }
const receipt = {schema: 'prismpm/portable-oracle-probe/1', case: name,
  ...before, node_version: process.version,
  exit_code: result.status, signal: result.signal, diagnostics,
  probe_passed: accepted, product_acceptance: 'not-established', ...(failure ? {failure} : {})};
writeFileSync(join(evidence, 'result.json'), `${JSON.stringify(receipt)}\n`, {flag: 'wx'});
console.log(JSON.stringify(receipt));
if (!accepted) process.exitCode = 1;
