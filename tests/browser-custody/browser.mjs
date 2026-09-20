import assert from 'node:assert/strict';
import {readFileSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {withBrowser} from '../../sdk/browser/browser-test-server.mjs';
import {draft, repository, run, sha} from './compile.mjs';

const production = name => join(name === 'browser-fixture.mjs' ? draft : join(repository, 'sdk/browser'), name);
const routes = ['credential-custody.mjs', 'browser-fixture.mjs'];
const expectedCases = ['closed-bootstrap-and-synchronous-policy-capture', 'real-nonextractable-multiple-shared-slot-keys',
  'generated-sign-admission-contexts-and-capture', 'strict-reopen-never-generates-or-replaces', 'atomic-first-creation-race',
  'missing-custody-and-immutable-policy', ...['missing-key', 'unknown-key', 'missing-header', 'unknown-header',
    'application', 'slot', 'public-key', 'principal', 'unknown-field', 'missing-secret', 'header-bytes'].map(x => 'corrupt-' + x),
  'wrong-actual-key-possession', 'extractable-key-rejected', 'schema-index-rejected', 'schema-extra-store-rejected',
  'schema-keypath-rejected', 'atomic-upgrade-abort', 'atomic-upgrade-quota',
  'unsupported-strict-durability-rejected', 'strict-barrier-abort-and-explicit-reopen',
  'lost-upgrade-acknowledgment', 'lost-barrier-acknowledgment', 'closed-and-late-sign-custody',
  'schema-versionchange-invalidates-handle', ...['policy', 'initialize', 'open', 'sign'].map(x => 'generated-' + x + '-binding-observation'),
  'maximum64slots128names1mib-real-sign-and-reopen'];
const closure = () => Object.fromEntries([...routes, 'identity.mjs', 'effects-module.mjs', 'effects-wire.mjs']
  .map(name => [name, sha(readFileSync(production(name)))]));

async function journey(build, source) {
  return withBrowser(async ({browser, baseURL}) => {
    const page = await browser.newPage(), errors = [];
    page.on('pageerror', error => errors.push(error.message));
    for (const name of routes) await page.route('**/' + name, route => route.fulfill({status: 200,
      contentType: 'text/javascript', body: name === 'credential-custody.mjs' && source !== undefined ? source : readFileSync(production(name), 'utf8')}));
    await page.goto(baseURL);
    let timer;
    try {
      const result = await Promise.race([page.evaluate(async input => (await import('./browser-fixture.mjs')).runFixture(input),
        {wire: [...build.wasmBytes]}), new Promise((_, reject) => { timer = setTimeout(() => reject(Error('bounded custody browser journey timed out')), 180000); })]);
      assert.deepEqual(errors, [], 'no detached browser exceptions');
      assert.deepEqual(result.cases, expectedCases, 'all actual custody journeys executed');
      return result;
    } finally { clearTimeout(timer); }
  });
}

export async function verifyJourneys(t, build) {
  const before = closure(), result = await journey(build);
  assert.ok(result.calls.length > result.cases.length);
  assert.ok(result.maximum > 0 && result.maximum <= 2048 * 65536);
  const path = join(build.work, 'browser-custody-transcript.tsv'), altered = join(build.work, 'browser-custody-transcript-mutant.tsv');
  const rows = corrupt => result.calls.map((call, index) => {
    assert.match(call.request, /^(?:[0-9a-f]{2})+$/); assert.match(call.response, /^(?:[0-9a-f]{2})+$/);
    return 'Browser' + index + '\t' + call.request + '\t' + (corrupt && index === 0 ? 'ff' : call.response) + '\n';
  }).join('');
  writeFileSync(path, rows(false), {flag: 'wx'}); writeFileSync(altered, rows(true), {flag: 'wx'});
  for (const standard of [true, false]) {
    const executable = build.compileNative(standard);
    assert.match(run(executable, [path], build.runner), new RegExp('PASS ' + result.calls.length + ' complete custody vectors twice'));
    assert.throws(() => run(executable, [altered], build.runner), /native output mismatch/);
  }
  assert.deepEqual(closure(), before);
  const evidence = {cases: result.cases, calls: result.calls.length, maximum: result.maximum,
    transcript: sha(readFileSync(path)), sources: before, artifact: sha(build.wasmBytes)};
  writeFileSync(join(build.work, 'browser-custody-evidence.json'), JSON.stringify(evidence, null, 2) + '\n', {flag: 'wx'});
  t.diagnostic(JSON.stringify(evidence));
  return evidence;
}

export async function verifyHostMutants(t, build) {
  const before = closure(), source = readFileSync(production('credential-custody.mjs'), 'utf8');
  const mutants = [
    ['digest', '!same(new Uint8Array(await crypto.subtle.digest(\'SHA-256\', wire)), expected)', 'false', /expected custody artifact-mismatch/],
    ['public key alias', 'publicKey: row[3].slice(), principal: row[4]', 'publicKey: row[3], principal: row[4]', /public copies cannot mutate custody/],
    ['capture', 'export async function signCredential(handle, resource, value) {',
      'export async function signCredential(handle, resource, value) { await Promise.resolve();', /exact captured bytes are signed/],
    ['possession', 'await validateIdentity({privateKey: row.privateKey, publicKey: row.publicKey, principal: row.principal})',
      '({privateKey: row.privateKey, publicKey: row.publicKey, principal: row.principal})', /expected custody custody-corrupt/],
    ['strict durability', "if (transaction.durability !== 'strict') throw fail('storage-unavailable');", '', /expected custody storage-unavailable/],
    ['late close', "const signature = await signBytes(identity, authorized[2], payload);\n  if (state.closed) throw fail('custody-closed');",
      'const signature = await signBytes(identity, authorized[2], payload);', /expected custody custody-closed/],
    ['policy output', "if (!equal(admitted, policy)) throw fail('invalid-generated-output');", '', /expected custody invalid-generated-output/],
    ['authorization output', "if (!requested || !binding || !equal(authorized, [...requested, binding[3], binding[4]])) throw fail('invalid-generated-output');",
      "if (!requested || !binding) throw fail('invalid-generated-output');", /expected custody invalid-generated-output/],
  ];
  assert.equal(mutants.length, 8, 'all declared host mutation families');
  for (const [label, needle, replacement, message] of mutants) {
    assert.equal(source.split(needle).length, 2, 'one exact owning mutation ' + label);
    await assert.rejects(journey(build, source.replace(needle, replacement)), message, 'actual browser kills ' + label);
    t.diagnostic('actual browser rejected host mutation ' + label);
  }
  assert.deepEqual(closure(), before);
  return mutants.map(([label]) => label);
}
