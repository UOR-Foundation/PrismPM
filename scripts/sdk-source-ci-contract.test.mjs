// Pure synthetic metadata/refusal controls, never authentication/SDK acceptance.
import assert from 'node:assert/strict';
import test from 'node:test';
import {readFileSync} from 'node:fs';
import {validateSourceCiContract,sourceCiArchiveDataDescriptor,sourceCiCompletedActiveBudget} from './sdk-source-ci-contract.mjs';
import {validateConstructionAuthority} from './sdk-construction-handoff.mjs';
import {bytes,clone,fixture,sha,sdkGates} from './sdk-source-ci-contract-fixture.mjs';
const admit=f=>validateSourceCiContract(bytes(f.provider),bytes(f.envelope),f.files,f.expected,f.now);
test('terminal producer/upload inside the actual in-progress containing run grants metadata only',()=>{
 const f=fixture(),h=admit(f);assert(Object.isFrozen(h));assert.deepEqual(Object.keys(h),['scope']);assert.match(h.scope,/metadata-only-not-authentication-or-image-execution/);
 const d=sourceCiArchiveDataDescriptor(h);assert.equal(d.digest,sha(f.archive));assert.equal(d.byte_length,f.archive.length);assert.equal(d.status,undefined);
 assert.match(d.scope,/not-whole-oci-or-provider-authority/);assert(Object.isFrozen(d));
 assert.equal(f.provider.run.status,'in_progress');assert.equal(f.provider.run.conclusion,null);
});
test('copies and caller success flags cannot impersonate the original metadata handle',()=>{
 const f=fixture(),h=admit(f);for(const forged of [{},clone(h),{verified:true},f.envelope,f.provider]){
  assert.throws(()=>sourceCiArchiveDataDescriptor(forged));assert.throws(()=>sourceCiCompletedActiveBudget(forged,bytes(f.consumer()),790,f.now));
 }
 const before=sourceCiArchiveDataDescriptor(h);f.envelope.archive.digest='sha256:'+'0'.repeat(64);f.expected.run_attempt=55;f.provider.jobs[0].completed_at='2026-10-14T21:00:00Z';
 assert.deepEqual(sourceCiArchiveDataDescriptor(h),before,'mutable caller records cannot alter retained private bindings');
});
test('repository event run attempt workflow producer and upload substitutions are refused',()=>{
 for(const mutate of [f=>f.provider.run.repository.full_name='other/repo',f=>f.provider.run.repository.id=124,
  f=>f.provider.run.head_repository.id=124,f=>f.provider.run.head_repository.full_name='fork/PrismPM',
  f=>f.provider.run.id++,f=>f.provider.run.run_attempt++,f=>f.provider.run.event='push',f=>f.provider.run.head_sha='0'.repeat(40),
  f=>f.provider.run.path='.github/workflows/sdk-candidate.yml',f=>{f.provider.run.status='queued';f.provider.run.conclusion='success';},
  f=>f.provider.run.conclusion='success',f=>{f.provider.run.status='completed';f.provider.run.conclusion='cancelled';},
  f=>f.provider.jobs[0].id++,f=>f.provider.jobs[0].run_attempt++,f=>f.provider.jobs[0].head_sha=f.expected.source_revision,
  f=>f.provider.jobs[0].status='in_progress',f=>f.provider.jobs[0].conclusion='failure',f=>f.provider.jobs[0].name='Unpublished native SDK construction (amd64)',
  f=>f.provider.jobs[0].labels=['ubuntu-24.04-arm'],f=>f.provider.jobs[0].runner_group_name='private',
  f=>f.provider.jobs.push(clone(f.provider.jobs[0])),f=>f.provider.jobs.push({...f.provider.jobs[0],id:4,name:'foreign gate'}),
  f=>f.provider.jobs=[],f=>f.provider.artifacts=[],f=>f.provider.artifacts.push(clone(f.provider.artifacts[0]))]){
  const f=fixture();mutate(f);assert.throws(()=>admit(f));
 }
});
test('exact current artifact ID provenance digest size expiry and upload window are required',()=>{
 for(const mutate of [a=>a.id++,a=>a.name='latest',a=>a.expired=true,a=>a.digest='sha256:bad',a=>a.size_in_bytes=0,
  a=>a.size_in_bytes=65*1024**3,a=>a.workflow_run.id++,a=>a.workflow_run.head_sha='0'.repeat(40),
  a=>a.workflow_run.repository_id++,a=>a.workflow_run.head_repository_id++,a=>a.expires_at='2026-10-14T21:00:00Z',
  a=>a.expires_at='2026-02-31T00:00:00Z',a=>a.created_at='2026-10-14T01:00:00Z',a=>a.created_at='2026-10-14T02:00:00Z']){
  const f=fixture();mutate(f.provider.artifacts[0]);assert.throws(()=>admit(f));
 }
});
test('complete successful producer steps cannot be skipped reordered renamed duplicated or renumbered',()=>{
 for(const mutate of [s=>s.pop(),s=>s.splice(6,1),s=>s[6].conclusion='skipped',s=>s[6].status='in_progress',
  s=>s[6].name='Upload a success receipt only',s=>s[6].number=s[5].number,
  s=>[s[5],s[6]]=[s[6],s[5]],s=>s[0].number=0,s=>s[0].started_at='2026-10-14T01:10:00Z',
  s=>s[4].completed_at='2026-10-14T02:10:00Z']){
  const f=fixture();mutate(f.provider.jobs[0].steps);assert.throws(()=>admit(f));
 }
});
test('UTC calendar bounds future observations and producer dependency ordering are checked',()=>{
 for(const mutate of [f=>f.provider.run.created_at='2026-10-14T01:00:00Z',
  f=>f.provider.run.run_started_at='2026-02-31T00:00:00Z',f=>f.provider.run.updated_at='2026-10-14T23:00:00Z',
  f=>f.provider.run.updated_at='2026-10-14T00:57:00Z',f=>f.provider.jobs[0].started_at='2026-10-14T00:00:00Z',
  f=>f.provider.jobs[0].completed_at='2026-10-14T00:00:00Z',f=>f.now=NaN]){
  const f=fixture();mutate(f);assert.throws(()=>admit(f));
 }
});
test('closed source selection and PR synthetic parents do not collapse event workflow and checkout identity',()=>{
 const f=fixture();assert.notEqual(f.expected.workflow_revision,f.expected.source_revision);assert.notEqual(f.expected.source_revision,f.expected.event_revision);admit(f);
 for(const mutate of [f=>f.expected.untrusted_budget_ms=9000000,f=>f.expected.repository_id=0,f=>f.expected.event='schedule',
  f=>f.expected.source_parents.reverse(),f=>f.expected.base_revision=f.expected.event_revision,
  f=>f.envelope.source.parents.reverse(),f=>f.envelope.source.tree='e'.repeat(40),f=>f.envelope.source.workflow_revision=f.expected.source_revision,
  f=>f.envelope.source.workflow_digest='sha256:'+'0'.repeat(64),f=>f.envelope.source.event_revision=f.expected.source_revision,
  f=>Object.defineProperty(f.expected,'run_id',{enumerable:true,get(){throw Error('caller getter');}})]){
  const bad=fixture();mutate(bad);assert.throws(()=>admit(bad));
 }
 for(const event of ['push','workflow_dispatch']){
  const direct=fixture();direct.expected.event=event;direct.expected.base_revision=null;direct.expected.source_revision=direct.expected.event_revision;
  // A changed source requires a genuinely different independent input closure;
  // merely changing the event/profile is not a synthetic-checkout escape.
  assert.throws(()=>admit(direct));
 }
});
test('closed envelope original-byte metadata and exact source closure digests are not replaceable',()=>{
 for(const mutate of [f=>f.envelope.status='passed',f=>f.envelope.schema='other',f=>f.envelope.scope='accepted',f=>f.envelope.unclaimed=[],
  f=>f.envelope.run_id++,f=>f.envelope.run_attempt++,f=>f.envelope.producer_job_id++,f=>f.envelope.platform='linux/arm64',
  f=>f.envelope.archive.path='../sdk.oci.tar',f=>f.envelope.archive.byte_length=0,f=>f.envelope.archive.digest='sha256:bad',
  f=>f.envelope.members.pop(),f=>f.envelope.members.reverse(),f=>f.envelope.members[0].digest='sha256:'+'0'.repeat(64),
  f=>f.files.set('extra',Buffer.from('unaccepted')),f=>f.files.delete('standards.lock'),
  f=>{f.files.set('input-policy.json',bytes({...JSON.parse(f.files.get('input-policy.json')),advisory_tree:'f'.repeat(40)}));f.seal();},
  f=>{f.files.set('input-manifest.json',bytes({...JSON.parse(f.files.get('input-manifest.json')),source:{files:[]}}));f.seal();},
  f=>{f.files.set('standards.lock',bytes({schema:'prismpm/standards-lock/1',authorities:['foreign'],oracles:[]}));f.seal();}]){
  const f=fixture();mutate(f);assert.throws(()=>admit(f));
 }
});
test('coherently resealed OCI metadata still checks original manifest config native and source constraints',()=>{
 for(const [name,mutate]of [
  ['image-config.json',c=>c.architecture='arm64'],['image-config.json',c=>c.os='windows'],
  ['image-config.json',c=>c.config.Labels['org.opencontainers.image.revision']='e'.repeat(40)],
  ['image-config.json',c=>c.config.Labels['org.opencontainers.image.source']='https://github.com/other/repo'],
  ['image-config.json',c=>c.config.Labels['org.opencontainers.image.version']='0.2.0'],
  ['image-config.json',c=>c.config.Labels['org.opencontainers.image.created']='2026-10-14T00:00:00Z'],
  ['image-manifest.json',m=>m.mediaType='application/vnd.docker.distribution.manifest.v2+json'],
  ['image-manifest.json',m=>m.config.size++],['image-manifest.json',m=>m.config.urls=['https://github.com/other/repo']],
  ['image-manifest.json',m=>m.layers=[]],['image-manifest.json',m=>m.layers[0].data='YQ=='],
  ['image-manifest.json',m=>m.layers[0].urls=['https://github.com/other/repo']],['image-manifest.json',m=>m.layers[0].size=0],
  ['image-manifest.json',m=>m.layers[0].mediaType='application/octet-stream']]){
  const f=fixture(),v=JSON.parse(f.files.get(name));mutate(v);f.files.set(name,bytes(v));
  if(name==='image-config.json'){const m=JSON.parse(f.files.get('image-manifest.json'));m.config.digest=sha(f.files.get(name));m.config.size=f.files.get(name).length;f.files.set('image-manifest.json',bytes(m));}
  f.seal();assert.throws(()=>admit(f));
 }
});
test('fatal UTF-8 size limits and malformed provider/envelope JSON fail without handles',()=>{
 const f=fixture();for(const b of [Buffer.alloc(0),Buffer.from('{'),Buffer.from([0xff]),Buffer.alloc(4*1024**2+1,32)]){
  assert.throws(()=>validateSourceCiContract(b,bytes(f.envelope),f.files,f.expected,f.now));
 }
 assert.throws(()=>validateSourceCiContract(bytes(f.provider),Buffer.alloc(65537,32),f.files,f.expected,f.now));
});
test('all five fixed budgets charge complete producer plus complete consumer and retirement, never admission queue',()=>{
 for(const [name,minutes]of sdkGates){
  const f=fixture(),h=admit(f),p=f.consumer(name),budget=sourceCiCompletedActiveBudget(h,bytes(p),790,f.now);
  assert.equal(budget.limit_ms,minutes*60000);assert.equal(budget.producer_active_ms,3600000);assert.equal(budget.consumer_active_ms,120000);
  assert.equal(budget.charged_ms,3724000);assert.equal(budget.producer_admission_queue_ms,120000);assert.equal(budget.consumer_admission_queue_ms,18*3600000);
  assert(budget.consumer_admission_queue_ms>budget.limit_ms,'provider queue is admission wait, not original job active time');
  assert.match(budget.scope,/accounting-only-not-gate-acceptance/);
  const exact=f.consumer(name,minutes*60000-3600000-4000,Date.parse('2026-10-14T02:00:00Z'));
  assert.equal(sourceCiCompletedActiveBudget(h,bytes(exact),790,f.now).charged_ms,minutes*60000);
  const over=f.consumer(name,minutes*60000-3600000-3999,Date.parse('2026-10-14T02:00:00Z'));
  assert.throws(()=>sourceCiCompletedActiveBudget(h,bytes(over),790,f.now),/budget exhausted/);
 }
});
test('active accounting requires unchanged provider bindings complete successful original gate and actual retirement',()=>{
 for(const mutate of [p=>p.jobs[1].conclusion='failure',p=>p.jobs[1].status='in_progress',p=>p.jobs[1].name='Conformance / holo-stdlib',
  p=>p.jobs[1].steps.splice(4,1),p=>p.jobs[1].steps[4].conclusion='skipped',p=>p.jobs[1].steps[3].name='Run only selected cases',
  p=>p.jobs[1].run_attempt++,p=>p.jobs[1].started_at='2026-10-14T01:30:00Z',p=>p.jobs[1].completed_at='2026-10-14T23:00:00Z',
  p=>p.artifacts[0].digest='sha256:'+'0'.repeat(64),p=>p.jobs[0].completed_at='2026-10-14T02:00:01Z',
  p=>p.run.updated_at='2026-10-14T23:00:00Z']){
  const f=fixture(),h=admit(f),p=f.consumer();mutate(p);assert.throws(()=>sourceCiCompletedActiveBudget(h,bytes(p),790,f.now));
 }
 const f=fixture(),h=admit(f),p=f.consumer();p.elapsed_ms=1;assert.throws(()=>sourceCiCompletedActiveBudget(h,bytes(p),790,f.now),'caller budget cannot replace closed provider bounds');
});
test('original twelve-gate matrix names deadlines commands max3 and dual-native authority remain untouched',()=>{
 const workflow=readFileSync(new URL('../.github/workflows/ci-parallel.yml',import.meta.url),'utf8');
 const rows=[...workflow.matchAll(/          - name: ([^\n]+)\n            timeout: (\d+)\n            commands: (\|\n(?:              [^\n]*\n)+|[^\n]+\n)/g)];
 assert.equal(rows.length,12);assert(workflow.includes('max-parallel: 3'));assert(workflow.includes('fail-fast: false'));
 for(const [name,minutes]of sdkGates){const r=rows.find(r=>r[1]===name);assert(r);assert.equal(Number(r[2]),minutes);}
 assert.equal(rows.filter(r=>!sdkGates.some(([n])=>n===r[1])).length,7);
 const legacy=readFileSync(new URL('./sdk-construction-handoff.mjs',import.meta.url),'utf8');
 assert(legacy.includes("'workflow_dispatch'"));assert(legacy.includes("'completed'"));assert(legacy.includes("'success'"));
 assert(!legacy.includes('sdk-source-ci-contract'));
 const f=fixture();assert.throws(()=>validateConstructionAuthority(bytes(f.provider),{
  revision:f.expected.source_revision,run_id:f.expected.run_id,run_attempt:f.expected.run_attempt,artifact_ids:{amd64:12,arm64:13}
 },f.now,Buffer.from(workflow)),'the same-run metadata profile must not enter legacy dual-native authority');
});
test('queued containing runs retain strict successful producer upload artifact and current-attempt joins',()=>{
 const f=fixture();f.provider.run.status='queued';const handle=admit(f);
 assert.match(handle.scope,/metadata-only-not-authentication-or-image-execution/);
 const final=f.consumer();final.run.status='queued';
 assert.equal(sourceCiCompletedActiveBudget(handle,bytes(final),790,f.now).charged_ms,3724000);
 for(const mutate of [f=>f.provider.run.conclusion='failure',f=>f.provider.run.run_attempt++,
  f=>f.provider.jobs[0].conclusion='failure',f=>f.provider.jobs[0].status='queued',
  f=>f.provider.jobs[0].steps[6].conclusion='skipped',f=>f.provider.artifacts[0].expired=true,
  f=>f.provider.artifacts[0].workflow_run.id++]){
  const bad=fixture();bad.provider.run.status='queued';mutate(bad);assert.throws(()=>admit(bad));
 }
});
test('stale aggregate run updated_at is not a watermark for authentic selected job completion',()=>{
 // Actual run 38617521389 remained queued/null with updated_at=created_at
 // 2026-10-14T18:43:08Z while selected jobs had completed by 20:35 UTC.
 // Retained raw provider response: pr129-456-unit-core-terminal-custody-v2-20261014/run-before.stdout.
 // These are synthetic refusal inputs; their JSON itself is not authentication.
 const f=fixture();f.provider.run.status='queued';
 f.provider.run.run_started_at=f.provider.run.created_at;f.provider.run.updated_at=f.provider.run.created_at;
 const handle=admit(f),final=f.consumer();final.run.updated_at=f.provider.run.created_at;
 assert.equal(sourceCiCompletedActiveBudget(handle,bytes(final),790,f.now).charged_ms,3724000);
 for(const mutate of [p=>p.run.updated_at='2026-10-14T23:00:00Z',
  p=>p.run.updated_at='2026-10-14T00:56:00Z',p=>p.run.run_attempt++,
  p=>p.jobs[1].run_attempt++,p=>p.jobs[1].completed_at='2026-10-14T23:00:00Z']){
  const invalid=clone(final);mutate(invalid);
  assert.throws(()=>sourceCiCompletedActiveBudget(handle,bytes(invalid),790,f.now));
 }
});
