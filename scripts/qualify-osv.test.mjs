import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdtemp,readFile,rm,symlink,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {checkCachedObject,checkChangedPaths,checkFreshProposal,checkOfficialMetadata} from './qualify-osv.mjs';

const manifest=async()=>JSON.parse(await readFile(new URL('../model/osv-databases.json',import.meta.url),'utf8'));
test('independent official metadata binds every acquired identity field',async()=>{
  for(const row of (await manifest()).databases) {
    const metadata={bucket:'osv-vulnerabilities',name:`${row.ecosystem}/all.zip`,generation:row.generation,
      size:String(row.size),md5Hash:row.md5,timeCreated:row.source_created,updated:row.source_updated};
    checkOfficialMetadata(row,metadata);
    for(const field of Object.keys(metadata))assert.throws(()=>checkOfficialMetadata(row,{...metadata,[field]:'changed'}));
  }
});
test('independent cache review detects same-size corruption, MD5/SHA substitution and symlinks',async()=>{
  const directory=await mkdtemp(join(tmpdir(),'osv-candidate-cache-'));
  try {
    const path=join(directory,'blob'),bytes=Buffer.from('abc');await writeFile(path,bytes);
    const row={ecosystem:'crates.io',generation:'1789723713081168',size:bytes.length,
      sha256:createHash('sha256').update(bytes).digest('hex'),md5:createHash('md5').update(bytes).digest('base64')};
    assert.equal((await checkCachedObject(path,row)).bytes,3);
    for(const patch of [{size:2},{size:4},{sha256:'0'.repeat(64)},{md5:'A'.repeat(22)+'=='}])
      await assert.rejects(checkCachedObject(path,{...row,...patch}));
    await writeFile(path,'abd');await assert.rejects(checkCachedObject(path,row));await writeFile(path,bytes);
    await symlink(path,join(directory,'link'));await assert.rejects(checkCachedObject(join(directory,'link'),row));
    await symlink(directory,join(directory,'parent'));await assert.rejects(checkCachedObject(join(directory,'parent/blob'),row));
    await assert.rejects(checkCachedObject(directory,row));
  }finally{await rm(directory,{recursive:true,force:true});}
});
test('candidate freshness retains all five inputs and exact oldest-source expiry',async()=>{
  const value=await manifest(),clock=Math.ceil(Math.max(...value.databases.map(row=>Date.parse(row.acquired_at)))/1000);
  const expiry=checkFreshProposal(value,clock);
  checkFreshProposal(value,expiry-1);assert.throws(()=>checkFreshProposal(value,expiry));
  assert.throws(()=>checkFreshProposal(value,clock-86400));
  for(const mutate of [v=>v.databases.pop(),v=>v.databases.reverse(),v=>v.freshness_policy_seconds++]) {
    const changed=structuredClone(value);mutate(changed);assert.throws(()=>checkFreshProposal(changed,clock));
  }
});
test('candidate diff cannot omit an intended input or add unrelated changes',()=>{
  const expected=['model/authorities.toml','model/osv-databases.json','standards.lock'];
  checkChangedPaths(expected.join('\n')+'\n',expected);
  for(const paths of [expected.slice(0,2),[...expected,'SPEC.md'],[...expected,expected[0]],[]])
    assert.throws(()=>checkChangedPaths(paths.join('\n'),expected));
});
test('workflow uploads only small review artifacts without write or signing permissions',async()=>{
  const workflow=await readFile(new URL('../.github/workflows/osv-input-qualification.yml',import.meta.url),'utf8');
  assert.match(workflow,/contents: read/u);assert.match(workflow,/persist-credentials: false/u);
  assert.match(workflow,/push: never/u);assert.match(workflow,/timeout-minutes: 120/u);
  assert.match(workflow,/branches: \['chore\/osv-inputs-\*'\]/u);assert.doesNotMatch(workflow,/pull_request:/u);
  assert.doesNotMatch(workflow,/secrets\.|write-all|contents: write|pull_request_target|id-token:|packages:/u);
  for(const action of workflow.matchAll(/uses: ([^\s]+)/gu))assert.match(action[1],/@[0-9a-f]{40}$/u);
  const paths=workflow.split('          path: |\n')[1].split('          include-hidden-files:')[0].trim().split('\n').map(value=>value.trim());
  assert.deepEqual(paths,['proposal.json','review.json','receipt.json','candidate.patch'].map(name=>`.prism/cache/osv-candidate/${name}`));
});
