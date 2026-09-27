import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {recoveryCorpus} from './corpus.mjs';

test('recovery owns new source roots without replacing Session late-completion admission', () => {
  const source = readFileSync(new URL('../../stdlib/src/Foundation/Browser/Application/V1/SessionJournalRecovery.lex.tex', import.meta.url), 'utf8');
  const declarations = JSON.parse(/\\semanticdata\{(.*)\}/.exec(source)[1]).declarations;
  for (const name of ['sourceRecoveryContextBytes', 'sourceRecoveryWireBytes', 'recoverQuiescentSourceSession'])
    assert.ok(declarations.some(row => row.name === name), 'actual source root ' + name);
  assert.ok(!declarations.some(row => row.name === 'settleSourceCommand'), 'no replacement late completion reducer');
});

test('independent recovery vectors preserve unresolved refusal and exact context coverage', () => {
  const corpus = recoveryCorpus();
  assert.equal(corpus.unchangedLateCompletion.length, 2);
  for (const name of ['Prepared', 'Unknown', 'ClosedPending', 'ClosedUnknown']) {
    assert.ok(corpus.recovery.some(row => row.id === name + 'CannotRecoverWithoutTerminal'));
    assert.ok(corpus.context.some(row => row.id === name + 'Context'));
  }
  for (const name of ['Principal', 'Scope', 'EpochRollback', 'SameExecution', 'InvalidPredecessor'])
    assert.ok(corpus.recovery.some(row => row.id === name));
});
