import assert from 'node:assert/strict';
import {readFileSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {withBrowser} from '../../sdk/browser/browser-test-server.mjs';
import {draft, repository, run, sha} from './compile.mjs';

const routes = ['effects.mjs', 'effects-wire.mjs', 'effects-module.mjs', 'browser-fixture.mjs'];
const production = name => join(name === 'browser-fixture.mjs' ? draft : join(repository, 'sdk/browser'), name);
const closure = () => Object.fromEntries([
  ...routes.map(name => [name, production(name)]),
  ['identity.mjs', join(repository, 'sdk/browser/identity.mjs')],
  ['store.mjs', join(repository, 'sdk/browser/store.mjs')],
  ['browser-test-server.mjs', join(repository, 'sdk/browser/browser-test-server.mjs')],
  ['browser.mjs', join(draft, 'browser.mjs')],
  ['runner.rs', join(draft, 'runner.rs')],
].map(([name, path]) => [name, sha(readFileSync(path))]));
const expectedCases = ['closed-api-bootstrap-capture', 'bootstrap-session-observation-rejection',
  'aggregate-artifact-budget-early-rejection', 'all-cryptographic-effects',
  'all-storage-effects-conflict-reopen', 'artifact-resource-memory-bootstrap-rejection',
  'request-bindings-closed-completion-detached-capture', 'bounded-queue-once-only-promotion',
  'admission-slot-fault-custody', 'admission-active-fault-custody', 'admission-counter-fault-custody',
  'admission-closed-fault-custody', 'admission-uncertain-fault-custody',
  'known-adapter-failure-consumes-once', 'unknown-actual-durable-custody', 'close-actual-durable-custody',
  'promotion-completion-fault-custody', 'trap-completion-fault-custody',
  'completion-counter-completion-fault-custody', 'completion-closed-completion-fault-custody',
  'completion-uncertain-completion-fault-custody', 'completion-cleared-unknown-completion-fault-custody',
  'close-counter-fault-custody', 'close-open-fault-custody', 'close-uncertain-fault-custody',
  'close-active-fault-custody', 'close-waiter-fault-custody',
  'generated-guest-workspace-budget-compatibility', 'generated-guest-journal-budget-compatibility',
  'generated-guest-two-mib-output-bound'];

async function journey(build, replacements = {}) {
  return withBrowser(async ({browser, baseURL}) => {
    const page = await browser.newPage(), errors = [];
    page.on('pageerror', error => errors.push(error.message));
    for (const name of routes) {
      await page.route('**/' + name, route => route.fulfill({status: 200,
        contentType: 'text/javascript', body: replacements[name] ?? readFileSync(production(name), 'utf8')}));
    }
    await page.route(baseURL, route => route.fulfill({status: 200, headers: {
      'content-type': 'text/html', 'cross-origin-opener-policy': 'same-origin',
      'cross-origin-embedder-policy': 'require-corp',
    }, body: '<!doctype html><title>Private effect bridge acceptance</title>'}));
    await page.goto(baseURL);
    let timer;
    try {
      const result = await Promise.race([
        page.evaluate(async input => (await import('./browser-fixture.mjs')).runFixture(input),
          {wire: [...build.wasmBytes], guest: [...build.guestBytes]}),
        new Promise((_, reject) => { timer = setTimeout(() => reject(Error('bounded effect browser journey timed out')), 180000); }),
      ]);
      assert.deepEqual(errors, [], 'no detached browser errors');
      assert.deepEqual(result.cases, expectedCases, 'all effect journeys executed');
      return result;
    } finally { clearTimeout(timer); }
  });
}

export async function verifyJourneys(t, build) {
  const before = closure(), result = await journey(build);
  assert.ok(result.calls.some(call => call.kind === 'Wire') && result.calls.some(call => call.kind === 'Guest'));
  for (const call of result.calls) {
    assert.ok(['Wire', 'Guest'].includes(call.kind));
    assert.match(call.request, /^(?:[0-9a-f]{2})*$/);
    assert.match(call.response, /^(?:[0-9a-f]{2})+$/);
    if (call.kind === 'Guest') assert.equal(call.response, '7b' + call.request, 'actual generated fixture guest');
  }
  assert.ok(result.maximum.Wire > 0 && result.maximum.Wire <= 16384 * 65536);
  assert.ok(result.maximum.Guest > 0 && result.maximum.Guest <= 256 * 65536);
  const path = join(build.work, 'browser-effects-transcript.tsv');
  const rows = (mutatedKind = null) => {
    let changed = false;
    return result.calls.map((call, index) => {
      const mutate = !changed && call.kind === mutatedKind;
      if (mutate) changed = true;
      return 'Browser' + call.kind + index + '\t' + call.request + '\t' + (mutate ? 'ff' : call.response) + '\n';
    }).join('');
  };
  writeFileSync(path, rows(), {flag: 'wx'});
  const altered = Object.fromEntries(['Wire', 'Guest'].map(kind => {
    const destination = join(build.work, 'browser-effects-' + kind.toLowerCase() + '-mutant.tsv');
    writeFileSync(destination, rows(kind), {flag: 'wx'}); return [kind, destination];
  }));
  for (const standard of [true, false]) {
    const executable = build.compileNative(standard);
    assert.match(run(executable, [path], build.runner),
      new RegExp('PASS ' + result.calls.length + ' complete effects vectors twice'));
    for (const [kind, destination] of Object.entries(altered)) {
      assert.throws(() => run(executable, [destination], build.runner), /native output mismatch|assertion/,
        'changed actual browser ' + kind + ' response must fail ' + (standard ? 'std' : 'no_std') + ' replay');
    }
  }
  assert.deepEqual(closure(), before, 'browser/adapter source closure remained frozen');
  const evidence = {cases: result.cases, calls: result.calls.length, maximum: result.maximum,
    transcript: sha(readFileSync(path)), sources: before,
    artifacts: {wire: sha(build.wasmBytes), guest: sha(build.guestBytes)}};
  writeFileSync(join(build.work, 'browser-effects-evidence.json'), JSON.stringify(evidence, null, 2) + '\n', {flag: 'wx'});
  t.diagnostic(JSON.stringify(evidence));
}

export async function verifyHostMutants(t, build) {
  const before = closure(), source = readFileSync(production('effects.mjs'), 'utf8');
  const mutants = [
    ['delayed intent capture', 'submit(value) {', 'async submit(value) { await Promise.resolve();',
      /synchronous detached intent capture|expected private effect|invalid-input/],
    ['wire digest bypass', '!same(await artifactDigest(wire), expected)', 'false',
      /expected private effect artifact-mismatch/],
    ['aggregate artifact admission bypass', 'inspectEffectArtifactBudget(wireInput, guests.map(value => value.bytes));',
      '/* planted missing aggregate admission */', /expected private effect invalid-input|aggregate rejection precedes/],
    ['bootstrap session substitution',
      "if (!equal(state, [manifest, session, 0, false, false, [0], [0]])) throw fail('invalid-generated-output');",
      "if (!equal(state[0], manifest)) throw fail('invalid-generated-output');",
      /expected private effect invalid-generated-output/],
    ['admission state committed before validation',
      `this.#step([1, 1, this.#state, request], next => {
        const active = this.#pending.length === 0 ? [1, request] : this.#state[5];
        const waiter = this.#pending.length === 0 ? [0] : [1, request];
        if (this.#pending.length > 1 || next[2] !== request[3] + 1 || next[3] !== false || next[4] !== false
          || !equal(next[5], active) || !equal(next[6], waiter)) throw fail('invalid-generated-output');
      });`,
      `this.#step([1, 1, this.#state, request], () => {});
      const expected = [1, request];
      if (!equal(this.#pending.length === 0 ? this.#state[5] : this.#state[6], expected)) throw fail('invalid-generated-output');`,
      /admission termination uses last good complete modeled state/],
    ['completion metadata substitution',
      `if (next[2] !== this.#state[2] || next[3] !== this.#state[3]
          || next[4] !== (result[0] === 9)) throw fail('invalid-generated-output');`,
      '/* planted missing completion metadata validation */',
      /corrupted completion keeps active outcome unknown|fault termination uses last good complete modeled custody/],
    ['close custody substitution',
      `if (next[2] !== this.#state[2] || next[3] !== true || next[4] !== this.#state[4]
        || !equal(next[5], this.#state[5]) || !equal(next[6], this.#state[6])) throw fail('invalid-generated-output');`,
      '/* planted missing close custody validation */',
      /invalid close observation is rejected before replacing modeled state/],
    ['lost acknowledgment misreported as rejection', 'return [9];', 'return [8, [9]];',
      /unknown barrier retains modeled session|waiter is not presented as completed/],
    ['late completion after close', 'if (this.#closed || this.#pending[0] !== item) return;',
      '/* planted missing private late-completion guard */', /late completion cannot reenter (?:admission-)?closed generated session/],
    ['unchecked pending promotion',
      "if (!equal(next[5], expected) || !equal(next[6], [0])) throw fail('invalid-generated-output');",
      '/* planted missing pending-promotion validation */', /corrupted completion keeps active outcome unknown/],
  ];
  for (const [name, needle, replacement, failure] of mutants) {
    assert.equal(source.split(needle).length, 2, 'one exact mutation target: ' + name);
    const changed = source.replace(needle, replacement);
    await assert.rejects(journey(build, {'effects.mjs': changed}), failure, name);
    t.diagnostic('killed effect host guard mutation: ' + name);
  }
  assert.deepEqual(closure(), before, 'mutations never alter authoritative host sources');
}
