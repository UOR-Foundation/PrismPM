import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {verifyVectors,verifyMaxima,verifySizeMaxima,verifyModelMutation,verifyInventory,frozenInputs,prerequisite} from '../../tests/browser-session/checks.mjs';
import {mutations} from '../../tests/browser-session/mutations.mjs';
import {sha} from '../../tests/browser-session/compile.mjs';

const source = name => JSON.parse(/\\semanticdata\{(.*)\}/.exec(readFileSync(
  new URL('../../stdlib/src/Foundation/Browser/Application/V1/' + name + '.lex.tex', import.meta.url), 'utf8'))[1]);

test('DK-26 declares source-owned transitions and a closed canonical byte entry', () => {
  const model = source('Session'), wire = source('SessionWire');
  const declarations = new Map(model.declarations.map(row => [row.name, row]));
  for (const name of ['initializeSourceSession', 'readOnlySourceSession', 'beginSourceCommand',
    'settleSourceCommand', 'continueSourceCommand', 'unknownSourceCommand', 'closeSourceSession',
    'rebindSourceSession', 'sourceSessionContextValid']) assert.equal(declarations.get(name)?.kind, 'definition', name);
  assert.equal(wire.declarations.find(row => row.name === 'sourceSessionWireBytes')?.result.kind, 'bytes');
});

test('DK-26 actual source-owned session kernel, complete limits and guard mutations',{timeout:3500000},async t=>{
 const inputs=frozenInputs(),inventory=verifyInventory();
 const actual=await prerequisite(t,'complete source/kernel/axiom closure and every finite vector in native std/no_std and Wasm',()=>verifyVectors(undefined,inputs));
 const build=actual.build;
 const maxima=await prerequisite(t,'all combined aggregate boundaries and input one-over in actual generated execution',()=>verifyMaxima(build));
 const domains=await prerequisite(t,'maximum Guest/Object/Head result domains and exact one-over rejection',()=>verifyMaxima(build,true));
 const sizes=await prerequisite(t,'all full presentation shapes and maximum Commit writer/error/size parity',()=>verifySizeMaxima(build));
 await prerequisite(t,'closed kernel diagnostics, budgets and owning inventories',verifyInventory);
 const mutants=[];
 for(const mutation of mutations)mutants.push(await prerequisite(t,'actual source mutation '+mutation.id+' fails native std/no_std and Wasm behavioral assertions',()=>{
  assert.deepEqual(frozenInputs(),inputs,'immutable source/tool inputs before each genuine mutation');
  return verifyModelMutation(mutation.id,build.sources,inputs);
 }));
 assert.deepEqual(frozenInputs(),inputs,'immutable complete owning source/tool input closure');
 const evidence={capability:'DK-26',scope:'private-pure-kernel',source:build.verified.source_id,attestation:build.verified.attestation_id,
  ir:build.generation.ir_sha256,wasm:sha(build.wasmBytes),fixture:sha(build.fixtureBytes),size:sha(build.sizeBytes),inventory,
  observed:{wire:actual.wasm,fixture:actual.fixture,size:actual.size},maxima,domains,sizes,mutants,inputs,cacheRetirement:build.cacheRetirement};
 writeFileSync(join(build.work,'session-acceptance.json'),JSON.stringify(evidence,null,2)+'\n',{flag:'wx'});
 t.diagnostic(JSON.stringify(evidence));
});
