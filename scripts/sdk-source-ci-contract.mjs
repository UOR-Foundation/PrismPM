// Source-CI response/envelope joins only. These handles are NOT authenticated
// provider responses, whole-archive admission, image-execution or SDK evidence.
// The legacy dual-native sdk-construction-handoff authority is not extended.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {validateConfig} from './sdk-candidate.mjs';

const repository='UOR-Foundation/PrismPM',workflow='.github/workflows/ci-parallel.yml';
const records=new WeakMap();
const sha=b=>'sha256:'+createHash('sha256').update(b).digest('hex');
const keys=(v,names)=>{assert(v&&typeof v==='object'&&!Array.isArray(v));assert.deepEqual(Object.keys(v).sort(),names.slice().sort(),'closed source-CI fields required');};
const integer=(n,max=Number.MAX_SAFE_INTEGER)=>assert(Number.isSafeInteger(n)&&n>0&&n<=max);
const digest=v=>{assert.equal(typeof v,'string');assert.match(v,/^sha256:[0-9a-f]{64}$/);};
const oid=v=>{assert.equal(typeof v,'string');assert.match(v,/^[0-9a-f]{40}$/);};
const decode=(b,max)=>{assert(Buffer.isBuffer(b)&&b.length>0&&b.length<=max);return JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(b));};
function time(v){
 assert.equal(typeof v,'string');assert.match(v,/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{3})?Z$/);
 const n=Date.parse(v);assert(Number.isSafeInteger(n));assert.equal(new Date(n).toISOString(),v.includes('.')?v:v.replace('Z','.000Z'),'real UTC calendar time required');return n;
}
const checkout='Run actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683';
const node='Run actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38';
const buildx='Run docker/setup-buildx-action@8d2750c68a42422c14e847fe6c8ac0403b4cbd6f';
const upload='Run actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02';
const producerName='Construct source-CI SDK input';
const producerSteps=['Set up job',checkout,node,buildx,'Install the exact OCI transport',producerName,upload,
 'Post '+buildx,'Post '+node,'Post '+checkout,'Complete job'];
const consumerSteps=['Set up job',checkout,'Admit source-CI SDK input','Run the unchanged owning gate',
 'Retire source-CI SDK input','Post '+checkout,'Complete job'];
// This closed inventory is the original matrix, not a new feature selection.
const gates=[
 ['Lint & Dependency Policy',45,false],['Model & Audits',45,false],['Workspace Unit Tests',180,true],
 ['Conformance / contracts',120,true],['Conformance / core',120,true],['Conformance / holo-stdlib',180,false],
 ['Conformance / desktop-browser',210,false],['Conformance / system-oci',180,true],
 ['Conformance / verification-schemas',90,true],['Fixtures & Reviewed Golden Outputs',90,false],
 ['Lean 4 Proofs & Formal Verification',75,false],['Packaging, API & Upstream Conformance',120,false]
];
const allowedNames=new Set([producerName,...gates.map(x=>x[0])]);
const fileNames=['image-config.json','image-manifest.json','input-manifest.json','input-policy.json','standards.lock'];
const unclaimed=['provider-authentication','git-signature-authentication','whole-zip-and-oci-integrity',
 'image-import','installed-sdk','gate-execution','full-vv','release'];

function steps(job,names,start,end){
 assert(Array.isArray(job.steps)&&job.steps.length===names.length,'complete original source-CI steps required');
 let number=0,last=start;
 for(const [i,s]of job.steps.entries()){
  assert.equal(s.name,names[i]);integer(s.number);assert(s.number>number);number=s.number;
  assert.equal(s.status,'completed');assert.equal(s.conclusion,'success');
  const a=time(s.started_at),b=time(s.completed_at);assert(a>=last&&b>=a&&b<=end,'ordered successful source-CI step bounds required');last=b;
 }
}
function jobIdentity(job,expected){
 integer(job.id);assert.equal(job.run_id,expected.run_id);assert.equal(job.run_attempt,expected.run_attempt);
 assert.equal(job.head_sha,expected.event_revision);assert.deepEqual(job.labels,['ubuntu-24.04']);
 assert.equal(job.runner_group_name,'GitHub Actions');
}
function successfulJob(job,names,expected,earliest,now){
 jobIdentity(job,expected);assert.equal(job.status,'completed');assert.equal(job.conclusion,'success');
 const start=time(job.started_at),end=time(job.completed_at);assert(start>=earliest&&end>=start&&end<=now,'bounded actual source-CI job required');
 steps(job,names,start,end);return {start,end};
}
function provider(value,expected,now){
 keys(value,['run','jobs','artifacts']);const {run,jobs,artifacts}=value;
 assert.equal(run.id,expected.run_id);assert.equal(run.run_attempt,expected.run_attempt);
 for(const r of [run.repository,run.head_repository]){assert.equal(r.id,expected.repository_id);assert.equal(r.full_name,repository);}
 assert.equal(run.path,workflow);assert.equal(run.event,expected.event);assert.equal(run.head_sha,expected.event_revision);
 // A terminal successful producer is sufficient, never a fabricated successful
 // containing run. Other gate failures do not alter this metadata-only scope.
 // GitHub may report the containing run queued between a successful producer
 // and its downstream gates. That admission state does not erase the strict
 // terminal producer/upload/artifact joins and grants no execution authority.
 assert((['queued','in_progress'].includes(run.status)&&run.conclusion===null)||(run.status==='completed'&&['success','failure'].includes(run.conclusion)),
  'actual active or completed non-cancelled containing run required');
 const created=time(run.created_at),started=time(run.run_started_at),updated=time(run.updated_at);
 assert(created<=started&&started<=updated&&updated<=now,'ordered actual run timestamps required');
 assert(Array.isArray(jobs)&&jobs.length>0&&jobs.length<=13&&new Set(jobs.map(j=>j.id)).size===jobs.length);
 assert(new Set(jobs.map(j=>j.name)).size===jobs.length&&jobs.every(j=>allowedNames.has(j.name)),'closed unique source-CI job inventory required');
 for(const j of jobs){integer(j.id);assert.equal(j.run_id,expected.run_id);assert.equal(j.run_attempt,expected.run_attempt);assert.equal(j.head_sha,expected.event_revision);}
 const producers=jobs.filter(j=>j.id===expected.producer_job_id);assert.equal(producers.length,1);const producer=producers[0];assert.equal(producer.name,producerName);
 const interval=successfulJob(producer,producerSteps,expected,started,now);
 // Aggregate updated_at may remain at creation while this run is queued
 // between jobs. Actual selected job/step bounds are checked against now;
 // the aggregate run timestamp is not a selected-job completion watermark.
 assert(Array.isArray(artifacts)&&artifacts.length===1,'one exact source-CI input artifact required');const artifact=artifacts[0];
 assert.equal(artifact.id,expected.artifact_id);assert.equal(artifact.name,`source-ci-sdk-${expected.run_id}-${expected.run_attempt}-${expected.source_revision}`);
 assert.equal(artifact.expired,false);integer(artifact.size_in_bytes,64*1024**3+96*1024**2);digest(artifact.digest);
 assert.equal(artifact.workflow_run.id,expected.run_id);assert.equal(artifact.workflow_run.head_sha,expected.event_revision);
 assert.equal(artifact.workflow_run.repository_id,expected.repository_id);assert.equal(artifact.workflow_run.head_repository_id,expected.repository_id);
 const uploadStep=producer.steps[6],uploaded=time(artifact.created_at);
 assert(uploaded>=time(uploadStep.started_at)&&uploaded<=time(uploadStep.completed_at),'artifact must originate in this successful upload step');
 assert(time(artifact.expires_at)>now&&time(artifact.expires_at)>uploaded,'unexpired current attempt artifact required');
 return {producer,artifact,interval,created,started};
}
function selected(value){
 keys(value,['repository_id','run_id','run_attempt','producer_job_id','artifact_id','event','event_revision','base_revision',
  'source_revision','source_tree','source_parents','workflow_revision','workflow_digest','input_policy_digest','input_manifest_digest','standards_digest']);
 const descriptors=Object.getOwnPropertyDescriptors(value),expected={};
 for(const key of Object.keys(value)){assert(Object.hasOwn(descriptors[key],'value'),'data-only selected identity required');expected[key]=descriptors[key].value;}
 for(const key of ['repository_id','run_id','run_attempt','producer_job_id','artifact_id'])integer(expected[key]);
 for(const key of ['event_revision','source_revision','source_tree','workflow_revision'])oid(expected[key]);
 assert(['pull_request','push','workflow_dispatch'].includes(expected.event));
 assert(Array.isArray(expected.source_parents)&&expected.source_parents.length<=2);expected.source_parents=Object.freeze([...expected.source_parents]);expected.source_parents.forEach(oid);
 if(expected.event==='pull_request'){oid(expected.base_revision);assert.notEqual(expected.base_revision,expected.event_revision);
  assert.deepEqual(expected.source_parents,[expected.base_revision,expected.event_revision]);assert(!expected.source_parents.includes(expected.source_revision));}
 else{assert.equal(expected.base_revision,null);assert.equal(expected.source_revision,expected.event_revision);}
 for(const key of ['workflow_digest','input_policy_digest','input_manifest_digest','standards_digest'])digest(expected[key]);
 return Object.freeze(expected);
}
function metadata(envelope,files,expected){
 keys(envelope,['schema','scope','source','run_id','run_attempt','producer_job_id','platform','archive','image','members','unclaimed']);
 assert.equal(envelope.schema,'prismpm/source-ci-sdk-input/1');assert.equal(envelope.scope,'same-run-image-input-only-not-execution-evidence');
 keys(envelope.source,['revision','tree','parents','event_revision','base_revision','workflow_revision','workflow_digest']);
 assert.deepEqual(envelope.source,{revision:expected.source_revision,tree:expected.source_tree,
  parents:expected.source_parents,
  event_revision:expected.event_revision,base_revision:expected.base_revision,workflow_revision:expected.workflow_revision,workflow_digest:expected.workflow_digest});
 assert(Array.isArray(envelope.source.parents)&&envelope.source.parents.length<=2);envelope.source.parents.forEach(oid);
 assert.equal(envelope.run_id,expected.run_id);assert.equal(envelope.run_attempt,expected.run_attempt);assert.equal(envelope.producer_job_id,expected.producer_job_id);
 assert.equal(envelope.platform,'linux/amd64');assert.deepEqual(envelope.unclaimed,unclaimed);
 keys(envelope.archive,['path','byte_length','digest']);assert.equal(envelope.archive.path,'sdk.oci.tar');integer(envelope.archive.byte_length,64*1024**3);digest(envelope.archive.digest);
 assert(files instanceof Map&&files.size===fileNames.length);assert.deepEqual([...files.keys()].sort(),fileNames);
 let total=0;const held=new Map();for(const [path,b]of files){assert(Buffer.isBuffer(b)&&b.length>0&&b.length<=(path==='input-manifest.json'?32*1024**2:64*1024**2));total+=b.length;assert(total<=96*1024**2);held.set(path,Buffer.from(b));}
 assert.deepEqual(envelope.members,fileNames.map(path=>({path,byte_length:held.get(path).length,digest:sha(held.get(path))})),'complete original metadata member bindings required');
 for(const [name,key]of [['input-policy.json','input_policy_digest'],['input-manifest.json','input_manifest_digest'],['standards.lock','standards_digest']])
  assert.equal(sha(held.get(name)),expected[key],'independently selected complete source input differs');
 const policy=decode(held.get('input-policy.json'),65536),manifest=decode(held.get('input-manifest.json'),32*1024**2);
 assert.equal(policy.schema,'prismpm/sdk-vv-input-policy/1');assert.equal(policy.source_revision,expected.source_revision);
 assert.equal(manifest.schema,'prismpm/sdk-vv-input-closure/1');assert.equal(manifest.scope,'sdk-vv-inputs-only');assert.deepEqual(manifest.policy,policy);
 assert.equal(manifest.source.revision,expected.source_revision);assert.equal(manifest.source.tree,expected.source_tree);
 assert(Array.isArray(manifest.source.files)&&manifest.source.files.length>0&&manifest.source.files.length<=100000);
 assert.equal(new Set(manifest.source.files.map(f=>f.path)).size,manifest.source.files.length,'unique complete source inventory required');
 for(const [path,want]of [[workflow,expected.workflow_digest],['standards.lock',expected.standards_digest]]){
  const rows=manifest.source.files.filter(f=>f.path===path);assert.equal(rows.length,1);assert.equal(rows[0].mode,'100644');assert.equal('sha256:'+rows[0].sha256,want);
 }
 const mb=held.get('image-manifest.json'),cb=held.get('image-config.json'),m=decode(mb,4*1024**2),c=decode(cb,4*1024**2);
 keys(envelope.image,['manifest_digest','manifest_byte_length','config_digest','config_byte_length']);
 assert.deepEqual(envelope.image,{manifest_digest:sha(mb),manifest_byte_length:mb.length,config_digest:sha(cb),config_byte_length:cb.length});
 assert.equal(m.schemaVersion,2);assert.equal(m.mediaType,'application/vnd.oci.image.manifest.v1+json');
 assert.equal(m.config.mediaType,'application/vnd.oci.image.config.v1+json');assert.equal(m.config.digest,sha(cb));assert.equal(m.config.size,cb.length);
 assert.equal(m.config.urls,undefined);assert.equal(m.config.data,undefined);
 assert(Array.isArray(m.layers)&&m.layers.length>0&&m.layers.length<=256);
 for(const layer of m.layers){digest(layer.digest);integer(layer.size,64*1024**3);assert.equal(layer.urls,undefined);assert.equal(layer.data,undefined);
  assert(['application/vnd.oci.image.layer.v1.tar','application/vnd.oci.image.layer.v1.tar+gzip','application/vnd.oci.image.layer.v1.tar+zstd'].includes(layer.mediaType));}
 validateConfig(c,'amd64',expected.source_revision);
 return {archive:{...envelope.archive},members:held};
}

export function validateSourceCiContract(providerBytes,envelopeBytes,files,selection,observedAt){
 integer(observedAt);const expected=selected(selection),p=provider(decode(providerBytes,4*1024**2),expected,observedAt);
 const checked=metadata(decode(envelopeBytes,65536),files,expected);
 const handle=Object.freeze({scope:'source-ci-response-metadata-only-not-authentication-or-image-execution'});
 records.set(handle,{expected,archive:checked.archive,members:checked.members,envelope:Buffer.from(envelopeBytes),
  producer:p.interval,artifact:JSON.parse(JSON.stringify(p.artifact)),runStarted:p.started});return handle;
}
export function sourceCiArchiveDataDescriptor(handle){
 const record=records.get(handle);assert(record,'original source-CI metadata handle required');
 return Object.freeze({byte_length:record.archive.byte_length,digest:record.archive.digest,
  scope:'expected-original-byte-identity-only-not-whole-oci-or-provider-authority'});
}
// A closed DATA plan for the original whole-stream verifier. This is NOT a
// provider-authentication or execution handle. Never accept a caller map or a
// JSON copy in place of the buffers retained by the original metadata join.
export function sourceCiArchiveIntegrityPlan(handle){
 const row=records.get(handle);assert(row,'original source-CI metadata handle required');
 return {artifact:{byte_length:row.artifact.size_in_bytes,digest:row.artifact.digest},
  archive:{byte_length:row.archive.byte_length,digest:row.archive.digest},
  members:new Map([['source-ci.json',Buffer.from(row.envelope)],...[...row.members].map(([name,b])=>[name,Buffer.from(b)])])};
}
// Final accounting is based on the full authentic job/ordered-step intervals,
// not a caller elapsed/budget field or a newly reset consumer process clock.
// Authentic acquisition and live pre-execution clock enforcement are separate
// required owners; this pure check cannot authorize starting any child process.
export function sourceCiCompletedActiveBudget(handle,providerBytes,consumerJobId,observedAt){
 const record=records.get(handle);assert(record,'original source-CI metadata handle required');integer(observedAt);integer(consumerJobId);
 const raw=decode(providerBytes,4*1024**2),p=provider(raw,record.expected,observedAt);
 assert.deepEqual(p.artifact,record.artifact,'provider artifact changed across custody');assert.deepEqual(p.interval,record.producer,'provider producer bounds changed');
 const jobs=raw.jobs.filter(j=>j.id===consumerJobId);assert.equal(jobs.length,1);const job=jobs[0],gate=gates.find(g=>g[0]===job.name&&g[2]);assert(gate,'one of the original five SDK-owning gates required');
 const consumer=successfulJob(job,consumerSteps,record.expected,p.interval.end,observedAt);
 const producerActive=p.interval.end-p.interval.start,consumerActive=consumer.end-consumer.start,limit=gate[1]*60000;
 // Provider timestamps have second resolution. Add one second at each end
 // rather than granting unobserved subsecond work outside either owning job.
 const charged=producerActive+consumerActive+4000;
 assert(charged<=limit,'original gate active budget exhausted by construction/import/execution/retirement');
 return Object.freeze({scope:'provider-response-active-accounting-only-not-gate-acceptance',gate:gate[0],limit_ms:limit,
  producer_active_ms:producerActive,consumer_active_ms:consumerActive,clock_resolution_reserve_ms:4000,charged_ms:charged,
  producer_admission_queue_ms:p.interval.start-p.started,consumer_admission_queue_ms:consumer.start-p.interval.end,
  unclaimed:['provider-authentication','live-pre-execution-admission','gate-execution','full-vv','release']});
}
