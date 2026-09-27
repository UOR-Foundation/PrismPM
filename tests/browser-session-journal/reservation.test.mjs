// Construction/oracle checks, not generated-source acceptance.
import assert from 'node:assert/strict';
import test from 'node:test';
import {readFileSync} from 'node:fs';
import {abstractPosition, graphCapacity, exploreReservedGraph, UINT32_MAX} from './reservation-oracle.mjs';
import {stateAt, resolutionTrace, reservationCorpus} from './reservation-corpus.mjs';
import {reservationModule, reservationMutations, mutateReservationSource} from './reservation-mutations.mjs';
import {verifyReservationNativeInventory} from './reservation-checks.mjs';
import {reservationMaximumCorpus} from './reservation-maxima.mjs';
import {decodeEffectWire as decode} from '../../sdk/browser/effects-wire.mjs';

test('native evidence requires every exact vector and no extra or repeated success rows', () => {
  const rows = [{id: 'first-vector'}, {id: 'second-vector'}];
  const output = 'PASS first-vector\nPASS second-vector\nPASS 2 journal reservation vectors twice\n';
  verifyReservationNativeInventory(output, rows);
  for (const changed of ['', output.replace('PASS first-vector\n', ''),
    output + 'PASS unexpected\n', output.replace('PASS second-vector', 'PASS first-vector'),
    'PASS 2 journal reservation vectors twice\n'])
    assert.throws(() => verifyReservationNativeInventory(changed, rows));
});

test('independent graph retains every finite outcome and optional checkpoint path', () => {
  let states = 0, edges = 0;
  for (let maximum = 2; maximum <= 5; maximum++) for (let retained = 0; retained <= maximum; retained++) {
    for (let remaining = 0; remaining < 5; remaining++) for (const phase of [1, 2, 3]) {
      const value = {phase, remaining, maximum, retained}, needed = graphCapacity(value);
      const visited = exploreReservedGraph(value, needed + 2);
      states += visited.states; edges += visited.edges;
    }
    for (const phase of [0, 3]) {
      const value = {phase, remaining: null, maximum, retained};
      const visited = exploreReservedGraph(value, graphCapacity(value) + 2);
      states += visited.states; edges += visited.edges;
    }
  }
  assert.ok(states > 10000 && edges > states);
});

test('exact full commands preserve the 512-step maximum and finite terminal close', () => {
  for (const [steps, records, maximum] of [[1, 7, 2], [512, 1029, 2], [512, 516, 1024]]) {
    const {operations, state} = resolutionTrace(steps, UINT32_MAX - records, maximum);
    assert.equal(operations.length, records); assert.equal(state[2], UINT32_MAX);
    assert.equal(state[5][2], 3); assert.deepEqual(state[5][4], [0]);
    assert.equal(operations.filter(x => x.kind === 4).length, steps - 1);
    assert.ok(operations.every(x => x.next[4] <= maximum));
    assert.deepEqual(operations.filter(x => x.kind !== 4 && x.kind !== 10).map(x => x.kind), [2, 5, 6, 9, 6]);
  }
});

test('reservation corpus includes every operation guard and complete boundary traces', () => {
  const rows = reservationCorpus(), names = new Set(rows.map(row => row.id));
  assert.equal(names.size, rows.length);
  for (let kind = 1; kind <= 10; kind++) assert.ok(names.has('ReservationKind' + kind + 'Exhausted'));
  for (const name of ['ReservationRedundantCheckpoint', 'ReservationUnsafeEmptyReplay',
    'ReservationTerminalEmptyReplay', 'ReservationOneBeginOneShort', 'ReservationFullBeginOneShort', 'ReservationWideBeginOneShort'])
    assert.ok(names.has(name), name);
  assert.equal(rows.filter(x => x.id.startsWith('ReservationFullTrace')).length, 1029);
  assert.ok(rows.length > 5000);
});

test('source composition uses exact pinned call arities and a bounded recursive fold', () => {
  const base = new URL('../../stdlib/src/', import.meta.url), modules = new Map();
  const root = 'Foundation.Browser.Application.V1.SessionJournalReservation';
  function capture(name) {
    if (modules.has(name)) return;
    const text = readFileSync(new URL(name.replaceAll('.', '/') + '.lex.tex', base), 'utf8');
    const declarations = JSON.parse(/\\semanticdata\{(.*)\}/.exec(text)[1]).declarations;
    modules.set(name, declarations);
    for (const match of text.matchAll(/\\importmodule\{([^}]+)\}/g)) capture(match[1]);
  }
  capture(root + 'Wire');
  function structureParameters(declaration) {
    assert.ok(Array.isArray(declaration.type_parameters), 'required pinned structure type_parameters');
    assert.ok(Array.isArray(declaration.parameters), 'required pinned structure parameters');
  }
  const reservation = modules.get(root).find(row => row.kind === 'structure');
  structureParameters(reservation);
  for (const field of ['type_parameters', 'parameters']) {
    const changed = {...reservation}; delete changed[field];
    assert.throws(() => structureParameters(changed), /required pinned structure/);
  }
  for (const name of [root, root + 'Wire']) {
    function visit(value) {
      if (!value || typeof value !== 'object') return;
      if (value.kind === 'call') {
        const module = value.function.module ?? name;
        const target = modules.get(module)?.find(x => x.name === value.function.name);
        assert.equal(target?.kind, 'definition', module + '.' + value.function.name);
        assert.equal(value.arguments.length, target.parameters.length, 'exact source call arity');
      }
      Object.values(value).forEach(child => Array.isArray(child) ? child.forEach(visit) : visit(child));
    }
    modules.get(name).forEach(visit);
  }
  assert.equal(modules.get(root).find(x => x.name === 'reserveSessionJournalOrdinary').recursive_argument, 'count');
  assert.equal(modules.get(root).find(x => x.name === 'reserveSessionJournalContinuationsTree').recursive_argument, 'depth');
  assert.equal(modules.get(root).find(x => x.name === 'reserveSessionJournalContinuations').result.kind, 'result');
  const ready = stateAt(); assert.equal(ready[0][8], 2);
});

test('actual source mutations target all operation guards and independent refusal probes', () => {
  const base = new URL('../../stdlib/src/', import.meta.url);
  const source = readFileSync(new URL(reservationModule.replaceAll('.', '/') + '.lex.tex', base));
  const rows = new Set(reservationCorpus().map(row => row.id));
  assert.equal(reservationMutations.length, 21);
  for (const mutation of reservationMutations) {
    const inputs = new Map([[reservationModule, source]]);
    assert.equal(mutateReservationSource(inputs, mutation.id).probe, mutation.probe);
    assert.ok(rows.has(mutation.probe)); assert.notDeepEqual(inputs.get(reservationModule), source);
  }
});

test('balanced-fold predecessor mutant retains its actual distinguishing corpus state', () => {
  const mutation = reservationMutations.find(row => row.id === 'reservation-split-state');
  const row = reservationCorpus().find(row => row.id === mutation.probe);
  assert.equal(row.id, 'ReservationCost45');
  assert.deepEqual(abstractPosition(decode(row.request)[2]),
    {phase: 1, remaining: 3, maximum: 2, retained: 0});
  assert.deepEqual(decode(row.response), [1, 0, 11]);
});

test('full payload maxima retain unsafe recovery refusal and a safe final Close', () => {
  const rows = reservationMaximumCorpus();
  assert.equal(rows.length, 7); assert.equal(new Set(rows.map(row => row.id)).size, 7);
  const unsafe = rows.find(row => row.id === 'MetadataMaximumRecoveryPlan');
  assert.deepEqual(decode(unsafe.response), [1, 1, 14]);
  const safe = rows.find(row => row.id === 'ReservationMaximumRecoveryWithFinalCloseReserve');
  const request = decode(safe.request), after = decode(safe.response)[2];
  assert.equal(request[2][2], UINT32_MAX - 2); assert.equal(after[2], UINT32_MAX - 1);
  assert.equal(request[3][6][1], 67108864);
  assert.equal(request[3][10][1][3][1], 67108864);
  const terminal = decode(rows.find(row => row.id === 'ReservationMaximumRecoveryFinalCloseAtExhaustion').response)[2];
  assert.equal(terminal[2], UINT32_MAX); assert.equal(terminal[5][2], 3);
});
