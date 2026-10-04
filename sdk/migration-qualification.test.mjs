// Parser falsification only; real installed execution belongs to the native gate.
import assert from 'node:assert/strict';
import {test} from 'node:test';
import {historicalLock,migrationChecks,verifyMigration,expectedMigrationProcesses,inventoryMutants} from './migration-qualification.mjs';
import {lockFixture} from '../scripts/library-sdk-fixture.mjs';

function fixture() {
  const target=lockFixture(),historical=historicalLock();
  return {target,value:{schema:'prismpm/installed-lock-migration/1',platform:'linux/amd64',
    historical_sha256:historical.source.sha256,target_sha256:target.hash(target.bytes),checks:[...migrationChecks],
    processes:expectedMigrationProcesses(target.lock,'linux/amd64').map(row=>({...row,stdout_sha256:'1'.repeat(64),stderr_sha256:'2'.repeat(64)})),
    proposal:{schema:'prismpm/sdk-lock-migration/1',compatibility_review:'required',generated_output_diff:'required',security_review:'required',
      patch:[{op:'test',path:'',value:JSON.parse(historical.document)},{op:'replace',path:'',value:target.lock}]}}};
}
test('migration reader binds original historical bytes and complete independent target',()=>{
  const {target,value}=fixture();verifyMigration(value,target.bytes);
  for(const change of [v=>v.historical_sha256='0'.repeat(64),v=>v.target_sha256='0'.repeat(64),
    v=>v.platform='linux/other',v=>v.extra=true,v=>v.checks.pop(),v=>v.checks.reverse(),
    v=>v.proposal.patch.reverse(),v=>v.proposal.patch.shift(),v=>v.proposal.patch[0].value.inventory.pop(),
    v=>v.proposal.patch[1].value.platforms.reverse(),v=>v.proposal.security_review='complete',
    v=>v.proposal.patch[1].path='/platforms',v=>v.proposal.patch[1].value.platforms[0].inventory.pop(),
    v=>v.processes.pop(),v=>v.processes[0].exit_code=0,v=>v.processes[2].input_sha256='0'.repeat(64),
    v=>v.processes[1].argv.pop(),v=>v.processes[4].stdout_sha256='missing']) {
    const invalid=structuredClone(value);change(invalid);assert.throws(()=>verifyMigration(invalid,target.bytes));
  }
  assert.throws(()=>verifyMigration(value,Buffer.from(target.bytes+'\n')));
  for(const mutant of inventoryMutants(target.lock,'linux/amd64')) {
    assert.deepEqual(mutant.platforms.map(row=>[row.platform,row.manifest_digest]),target.lock.platforms.map(row=>[row.platform,row.manifest_digest]));
    for(const row of mutant.platforms) {
      assert.deepEqual(JSON.parse(row.inventory_document).artifacts,row.inventory);
      assert.equal(row.inventory_digest,'sha256:'+target.hash(row.inventory_document));
    }
    assert.notEqual(mutant.platforms[0].inventory_digest,target.lock.platforms[0].inventory_digest);
  }
});
