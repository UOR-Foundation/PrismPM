import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {openAccountGenesis} from '../../sdk/browser/account-genesis.mjs';
import {corpus, encode} from './corpus.mjs';
import {canonical, mutations, mutateAccountGenesisSource} from './mutations.mjs';
import {journeyNames, verifyObservations} from './browser.mjs';
import {verifySemanticCounterexample} from './host-mutations.mjs';
import {verifyNativeInventory} from './checks.mjs';

test('private account factory captures closed data options and refuses shape-only artifacts', async () => {
  for (const value of [null, {}, {wire: new Uint8Array()}, {wire: [], wireDigest: []}])
    await assert.rejects(openAccountGenesis(value), {code: 'invalid-input'});
  let read = false;
  await assert.rejects(openAccountGenesis({get wire() {read = true; return new Uint8Array();},
    wireDigest: new Uint8Array(32)}), {code: 'invalid-input'});
  assert.equal(read, false);
});

test('canonical source and every canonical mutant bind one changed module and a real counterexample', () => {
  const names = ['Foundation.Browser.Application.V1.AccountGenesis',
    'Foundation.Browser.Application.V1.AccountGenesisWire'];
  const original = new Map(names.map(name => [name,
    readFileSync(new URL('../../stdlib/src/' + name.replaceAll('.', '/') + '.lex.tex', import.meta.url))]));
  const checkCanonical = source => {
    const text = source.toString('utf8'), json = /\\semanticdata\{(.*)\}/.exec(text)[1];
    const data = JSON.parse(json);
    assert.equal(json, JSON.stringify(canonical(data)), 'actual canonical semantic JSON');
    const available = new Set([...text.matchAll(/\\importmodule\{([^}]+)\}/g)].map(row => row[1]));
    available.add(/\\begin\{lexlean\}\{([^}]+)\}/.exec(text)[1]);
    const visit = value => {
      if (!value || typeof value !== 'object') return;
      if (typeof value.module === 'string') assert.ok(available.has(value.module),
        'qualified semantic reference requires its direct import: ' + value.module);
      for (const child of Object.values(value))
        if (Array.isArray(child)) child.forEach(visit); else visit(child);
    };
    visit(data);
  };
  for (const source of original.values()) checkCanonical(source);
  const ids = new Set(corpus().map(row => row.id));
  for (const mutation of mutations) {
    const source = new Map(original);
    mutateAccountGenesisSource(source, mutation.id);
    for (const bytes of source.values()) checkCanonical(bytes);
    assert.equal([...source].filter(([name, bytes]) => !bytes.equals(original.get(name))).length, 1);
    assert.ok(ids.has(mutation.probe));
  }
});

test('observation contract requires exact operation tags and native inventory', () => {
  const operations = [[2, 1], [2, 1], [2], [2, 1], [2, 1], [], [], [2, 1], [], [], [], [2]];
  const calls = [], journeys = journeyNames.map((id, index) => {
    const start = calls.length;
    for (const operation of operations[index]) calls.push({request: encode([1, operation]).toString('hex')});
    return {id, start, end: calls.length};
  });
  verifyObservations({calls, journeys});
  const changed = structuredClone({calls, journeys}); changed.calls[0].request = encode([1, 0]).toString('hex');
  assert.throws(() => verifyObservations(changed), /exact source operation tags/);
  const summary = 'PASS Probe\nPASS 1 account-genesis vectors twice\n';
  verifyNativeInventory(summary, [{id: 'Probe'}]);
  assert.throws(() => verifyNativeInventory(summary.replace('1 account-genesis', '1 journal account-genesis'),
    [{id: 'Probe'}]), /exact native case inventory/);
  const runner = readFileSync(new URL('./runner.rs', import.meta.url), 'utf8');
  assert.ok(runner.includes('println!("PASS {count} account-genesis vectors twice");'));
  assert.ok(runner.includes('println!("PASS binary account-genesis twice");'));
});

test('counterfeit semantic error labels and fields never establish assertion provenance', () => {
  const record = {journey: 'invalid-point', check: 'curve-import', expected: 'invalid-key', actual: 'accepted'};
  for (const error of [record, Object.freeze({...record}),
    Object.assign(Error('actual account-genesis semantic counterexample'), record),
    Error('planted ordinary SDK failure: curve-import')])
    assert.throws(() => verifySemanticCounterexample(error, 'curve-import'), /privately branded/);
});
