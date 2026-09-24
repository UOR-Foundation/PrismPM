// Acceptance-only browser driver. Expected application behavior comes from
// modeled byte vectors; all successful calls use the real upstream session.
import assert from 'node:assert/strict';
import {createInterface} from 'node:readline';
import {createRequire} from 'node:module';

const require = createRequire('/opt/prismpm/oracles/package.json');
assert.equal(require('playwright/package.json').version, '1.62.1');
const {chromium} = require('playwright');
const input = createInterface({input: process.stdin, crlfDelay: Infinity})[Symbol.asyncIterator]();
const first = await input.next();
assert.equal(first.done, false);
const {origin, application: app, browser_executable: browserExecutable} = JSON.parse(first.value);
assert.equal(browserExecutable, process.arch === 'x64'
  ? '/ms-playwright/chromium_headless_shell-1234/chrome-headless-shell-linux64/chrome-headless-shell'
  : '/ms-playwright/chromium_headless_shell-1234/chrome-linux/headless_shell');
assert.ok(Number.isInteger(app.request_maximum) && app.request_maximum > 0 && app.request_maximum <= 65_536,
  'request bound exceeds pinned portable View transport');
assert.ok(Number.isInteger(app.response_maximum) && app.response_maximum > 0 && app.response_maximum <= 1_048_576,
  'response bound exceeds pinned portable View transport');
const url = new URL(origin);
assert.equal(url.hostname, '127.0.0.1');
assert.equal(url.protocol, 'http:');
assert.equal(url.origin, origin);
const text = app.profile === 'prismpm/text-application/1';
const profile = text ? 'utf8-text' : 'legacy-numeric';
const decode = new TextDecoder('utf-8', {fatal: true, ignoreBOM: true});
const view = app.view;
const cases = [];
const vectorIndexes = [];
const browser = await chromium.launch({headless: true, executablePath: browserExecutable});
assert.equal(browser.browserType().name(), 'chromium');
assert.equal(browser.version(), '151.0.7922.34');
const context = await browser.newContext();
const page = await context.newPage();
const unexpected = [];
const allowed = new Set(['/', '/index.html', '/app.css', '/app.js', '/favicon.ico', '/_hologram/intent']);
context.on('request', request => {
  const target = new URL(request.url());
  if (target.origin !== origin || !allowed.has(target.pathname) || target.search) unexpected.push(request.url());
});
page.setDefaultTimeout(10_000);
const output = page.locator('#result');
async function shows(expected, target = page) {
  await target.waitForFunction(value => document.querySelector('#result')?.textContent === value, expected);
}
async function fill(vector, target = page) {
  const request = decode.decode(Uint8Array.from(vector.request));
  if (text) await target.locator('#request').fill(request);
  else {
    const [version, operation, left, right] = request.split('\t');
    assert.equal(version, '1');
    const option = view.operations.find(item => item.request_name === operation);
    assert.ok(option, 'modeled operation must have a generated View option');
    await target.locator('#left').fill(left);
    await target.locator('#right').fill(right);
    await target.locator('#operation').selectOption(String(option.discriminant));
  }
}
function displayed(vector) {
  const value = decode.decode(Uint8Array.from(vector.response));
  if (text) return value;
  const [version, kind, body, extra] = value.split('\t');
  assert.equal(version, '1');
  assert.equal(extra, undefined);
  if (kind === 'ok') return body;
  assert.equal(kind, 'error');
  assert.ok(['division-by-zero', 'overflow'].includes(body));
  return body === 'division-by-zero' ? view.division_error : view.overflow_error;
}
function applicable(vector) {
  let request;
  try { request = decode.decode(Uint8Array.from(vector.request)); } catch { return false; }
  if (text) return vector.request.length <= app.request_maximum;
  const fields = request.split('\t');
  const canonical = value => {
    if (!/^-?(0|[1-9][0-9]*)$/.test(value)) return false;
    const number = BigInt(value);
    return number >= -(1n << 63n) && number < (1n << 63n) && number.toString() === value;
  };
  return fields.length === 4 && fields[0] === '1'
    && view.operations.some(item => item.request_name === fields[1])
    && canonical(fields[2]) && canonical(fields[3]);
}
const valid = app.acceptance_vectors.map((vector, index) => ({vector, index})).filter(({vector}) => applicable(vector));
assert.ok(valid.length > 0, 'no modeled request can exercise the actual View');
const recovery = valid[0].vector;
async function submit(vector, target = page, keyboard = false) {
  await fill(vector, target);
  let bodyBuffer = null;
  const response = target.waitForResponse(async reply => {
    if (reply.url() === `${origin}/_hologram/intent`) {
      try { bodyBuffer = await reply.body(); } catch (_) {}
      return true;
    }
    return false;
  });
  if (keyboard) {
    if (text) await target.locator('#request').press('Control+Enter');
    else await target.locator('#right').press('Enter');
  } else await target.locator('#submit').click();
  const reply = await response;
  assert.equal(reply.status(), 200);
  assert.deepEqual(reply.request().postDataJSON(), {version: 1, name: 'application.invoke',
    payload: decode.decode(Uint8Array.from(vector.request))});
  const replyBody = bodyBuffer ?? await reply.body();
  const envelope = JSON.parse(replyBody.toString('utf-8'));
  assert.deepEqual(envelope, {version: 1, outputs: [decode.decode(Uint8Array.from(vector.response))]});
  await shows(displayed(vector), target);
}
async function journey(name, work) {
  await work();
  cases.push({name, status: 'passed', attempts: 1});
}
try {
  await journey('attachment-assets', async () => {
    await page.goto(`${origin}/`);
    assert.equal(await page.title(), view.title);
    assert.equal(await page.locator('h1').textContent(), view.heading);
    assert.equal(await output.getAttribute('aria-live'), 'polite');
    assert.equal(await page.locator('#submit').textContent(), view.submit_label);
    assert.equal((await page.locator('script').all()).length, 1);
    assert.equal(await page.locator('script').getAttribute('src'), 'app.js');
  });
  await journey('modeled-vectors', async () => {
    for (const {vector, index} of valid) {
      await submit(vector);
      vectorIndexes.push(index);
    }
    await submit(recovery, page, true);
  });
  await journey('input-validation-recovery', async () => {
    const requests = [];
    const listener = request => requests.push(request);
    page.on('request', listener);
    if (text) {
      for (const value of ['x'.repeat(app.request_maximum + 1), '\ud800']) {
        await page.locator('#request').evaluate((element, value) => {element.value = value;}, value);
        await page.locator('#submit').click();
        await shows(view.input_error);
        assert.equal(await page.locator('#request').getAttribute('aria-invalid'), 'true');
        assert.equal(await page.locator('#request').evaluate(element => document.activeElement === element), true);
      }
    } else {
      for (const value of ['', '+1', '9223372036854775808', '-0', '<script>']) {
        await page.locator('#left').fill(value);
        await page.locator('#submit').click();
        await shows(view.input_error);
      }
    }
    page.off('request', listener);
    assert.equal(requests.length, 0, 'invalid browser input must not reach the guest');
    await submit(recovery);
  });
  await journey('transport-failure-recovery', async () => {
    await page.route('**/_hologram/intent', route => route.abort());
    await fill(recovery);
    await page.locator('#submit').click();
    await shows(text ? view.response_error : view.input_error);
    await page.unroute('**/_hologram/intent');
    await submit(recovery);
  });
  await journey('pre-init-privacy', async () => {
    for (const mode of ['disabled', 'blocked']) {
      const privateContext = await browser.newContext({javaScriptEnabled: mode !== 'disabled'});
      if (mode === 'blocked') await privateContext.route('**/app.js', route => route.abort());
      const privatePage = await privateContext.newPage();
      await privatePage.goto(`${origin}/`);
      await privatePage.waitForLoadState('networkidle');
      const requests = [];
      privatePage.on('request', request => requests.push(request.url()));
      const field = text ? '#request' : '#left';
      await privatePage.locator(field).fill('private-oracle-draft-9137');
      assert.equal(await privatePage.locator('#submit').isDisabled(), true);
      await shows(text ? view.response_error : view.input_error, privatePage);
      await privatePage.locator('#application-form').evaluate(form => {
        HTMLFormElement.prototype.submit.call(form);
        form.requestSubmit();
      });
      await privatePage.waitForTimeout(100);
      assert.equal(privatePage.url(), `${origin}/`);
      assert.deepEqual(requests, [], 'draft must not navigate or enter any request');
      await privateContext.close();
    }
  });
  await journey('delayed-init', async () => {
    const delayed = await browser.newContext();
    let release;
    const pending = new Promise(resolve => {release = resolve;});
    await delayed.route('**/app.js', async route => {await pending; await route.continue();});
    const delayedPage = await delayed.newPage();
    await delayedPage.goto(`${origin}/`, {waitUntil: 'commit'});
    await fill(recovery, delayedPage);
    assert.equal(await delayedPage.locator('#submit').isDisabled(), true);
    await shows(text ? view.response_error : view.input_error, delayedPage);
    const before = [];
    delayedPage.on('request', request => {if (request.method() !== 'GET') before.push(request.url());});
    // The separate privacy cases force native submission. Here retain the
    // document and its still-loading module to witness input-before-ready.
    await delayedPage.waitForTimeout(100);
    assert.deepEqual(before, []);
    assert.equal(delayedPage.url(), `${origin}/`);
    release();
    await delayedPage.waitForLoadState('networkidle');
    const retained = await delayedPage.evaluate(selector => document.querySelector(selector).value, text ? '#request' : '#left');
    const expectedInput = decode.decode(Uint8Array.from(recovery.request));
    assert.equal(retained, text ? expectedInput : expectedInput.split('\t')[2]);
    const response = delayedPage.waitForResponse(reply => reply.url() === `${origin}/_hologram/intent`);
    await delayedPage.locator('#submit').click();
    const reply = await response;
    assert.equal(reply.status(), 200);
    assert.deepEqual(reply.request().postDataJSON(), {version: 1, name: 'application.invoke', payload: expectedInput});
    await shows(displayed(recovery), delayedPage);
    await delayed.close();
  });
  await journey('intent-boundaries', async () => {
    const body = {version: 1, name: 'application.invoke', payload: decode.decode(Uint8Array.from(recovery.request))};
    const headers = {'content-type': 'application/json', origin};
    const request = (value, options = {}) => fetch(`${origin}/_hologram/intent`, {
      method: 'POST', headers, body: JSON.stringify(value), ...options,
    });
    for (const value of [{...body, extra: true}, {...body, version: 2}, {...body, name: 'host.exec'},
      {...body, payload: 'x'.repeat(65_537)}]) assert.equal((await request(value)).status, 400);
    assert.equal((await request(body, {headers: {...headers, origin: 'https://untrusted.invalid'}})).status, 403);
    assert.equal((await request(body, {headers: {'content-type': 'application/json'}})).status, 403);
    assert.equal((await request(body, {headers: {...headers, 'content-type': 'text/plain'}})).status, 415);
    assert.equal((await request(body, {body: 'x'.repeat(65_536 * 6 + 257)})).status, 413);
    assert.equal((await fetch(`${origin}/_hologram/intent`)).status, 405);
    assert.equal((await fetch(`${origin}/?draft=private`)).status, 400);
    assert.equal((await fetch(`${origin}/%2e%2e/secrets`)).status, 404);
    await submit(recovery);
  });
  if (text) {
    await journey('text-response-bounds', async () => {
      for (const body of [Buffer.from([255]), Buffer.from(JSON.stringify({version: 1, outputs: ['x'.repeat(app.response_maximum + 1)]})),
        Buffer.from(JSON.stringify({version: 1, outputs: []})), Buffer.from('x'.repeat(app.response_maximum * 6 + 257))]) {
        await page.route('**/_hologram/intent', route => route.fulfill({status: 200, contentType: 'application/json', body}));
        await fill(recovery);
        await page.locator('#submit').click();
        await shows(view.response_error);
        await page.unroute('**/_hologram/intent');
        await submit(recovery);
      }
    });
    await journey('text-safe-rendering', async () => {
      // Renderer-boundary fault injection, not an application response oracle:
      // a non-echo guest need not return markup merely because its input did.
      const markup = '<img src=x onerror=alert(1)>🌱';
      const value = Buffer.byteLength(markup) <= app.response_maximum ? markup : '<b>'.slice(0, app.response_maximum);
      await page.route('**/_hologram/intent', route => route.fulfill({status: 200,
        contentType: 'application/json', body: JSON.stringify({version: 1, outputs: [value]})}));
      await fill(recovery);
      await page.locator('#submit').click();
      await shows(value);
      assert.equal(await output.locator('*').count(), 0);
      await page.unroute('**/_hologram/intent');
      await submit(recovery);
    });
  }
  process.stdout.write(`${JSON.stringify({stage: 'live'})}\n`);
  const stopped = await input.next();
  assert.equal(stopped.done, false);
  assert.deepEqual(JSON.parse(stopped.value), {stage: 'stopped'});
  await journey('detached-session', async () => {
    await fill(recovery);
    const pending = page.waitForResponse(response => response.url() === `${origin}/_hologram/intent`);
    await page.locator('#submit').click();
    assert.equal((await pending).status(), 410);
    await shows(text ? view.response_error : view.input_error);
    assert.equal((await fetch(`${origin}/app.js`)).status, 410);
  });
  assert.deepEqual(unexpected, []);
  process.stdout.write(`${JSON.stringify({schema: 'prismpm/portable-browser-oracle/1', profile,
    engine: browser.browserType().name(), browser_version: browser.version(), playwright: '1.62.1', cases, vector_indexes: vectorIndexes,
    skipped: 0, retries: 0, status: 'passed'})}\n`);
} finally {
  await browser.close();
  process.stdin.destroy();
}
