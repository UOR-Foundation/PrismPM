// Source-custody checks only; the complete generated owner remains mandatory.
import assert from 'node:assert/strict';
import test from 'node:test';
import {provenanceProfile,presentationOutputRows} from '../browser-presentation/provenance.mjs';
import {modules,frozenInputs,assertFrozenInputs} from './compile.mjs';
import {sourceRoots,suites} from '../../scripts/browser-api-sdk-check.mjs';

test('session provenance admits only the registered complete family, never a caller subset',()=>{
  for(const value of [null,[],{},'','session/../presentation','caller-family',
    {toString(){throw Error('coercion');}}])
    assert.throws(()=>provenanceProfile(value),/registered complete provenance family/);
  const profile=provenanceProfile('session');
  assert.deepEqual(profile.outputRows.filter(row=>row.kind==='lean').map(row=>
    row.path.slice('modules/PrismPM/'.length,-'.lean'.length).replaceAll('/','.')),modules);
  assert.equal(profile.outputRows.length,50);
  assert.deepEqual(provenanceProfile('presentation').outputRows,presentationOutputRows);
  assert.equal(presentationOutputRows.length,30);
  assert.throws(()=>{profile.outputRows.pop();},TypeError);
  assert.throws(()=>{profile.outputRows[0].path='omitted';},TypeError);
});

test('session frozen input authority cannot be forged by copying coherent hash labels',()=>{
  const inputs=frozenInputs();
  assertFrozenInputs(inputs);
  assert.throws(()=>assertFrozenInputs({...inputs}),/actual frozen session input closure/);
  for(const path of Object.keys(inputs))
    assert.ok(sourceRoots.some(root=>path===root||path.startsWith(root+'/')),
      'installed SDK covers complete session owner input '+path);
});

test('source and installed session registration retain the entire original owner and added custody checks',()=>{
  const owner=suites.find(row=>row.id==='DK-26');
  assert.deepEqual(owner.files,['sdk/browser/session-model-test.mjs',
    'tests/browser-session/wire.test.mjs','tests/browser-session/provenance.test.mjs']);
  assert.equal(owner.minimum,37);assert.equal(owner.deadline,3600000);
});
