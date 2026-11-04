// Conformance witness over real retained native-library compiler/package output.
// Never construct positive compiler evidence or modify its original directory.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {copyFileSync,existsSync,mkdirSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {checkAccepted} from './library-sdk-check.mjs';
import {runtimeAudit} from './semantic-source.mjs';

const [project,encodedReceipt,archive,toolchain]=process.argv.slice(2);
const receipt=JSON.parse(encodedReceipt),compiler={toolchain};
const canonical=value=>JSON.stringify(value,(_,item)=>item&&typeof item==='object'&&!Array.isArray(item)
 ?Object.fromEntries(Object.keys(item).sort().map(key=>[key,item[key]])):item);
const hash=value=>createHash('sha256').update(value).digest('hex');
// LexLean's source-defined attestation recipe, checked against the original
// genuine attestation before resealing any owned negative copy.
const lexleanAttestationId=value=>{
 const {attestation_id,...body}=value,data=Buffer.from(canonical(body)),label=Buffer.from('attestation-body');
 const labelLength=Buffer.alloc(4),bodyLength=Buffer.alloc(8);labelLength.writeUInt32BE(label.length);bodyLength.writeBigUInt64BE(BigInt(data.length));
 return createHash('sha256').update('lexlean-attestation-v1\0').update(labelLength).update(label).update(bodyLength).update(data).digest('hex');
};
const original=join(project,receipt.verified_root),bytes=readFileSync(join(original,'manifest.json'));
const baseline=JSON.parse(bytes);
const auditBytes=readFileSync(join(original,'lexlean-attestation.json'));
const audit=JSON.parse(auditBytes);
assert.equal(lexleanAttestationId(audit),audit.attestation_id,'original audit must bind the actual LexLean framed recipe');
const acceptanceBytes=readFileSync(join(original,'library-acceptance.json')),acceptance=JSON.parse(acceptanceBytes);
const sourceNames=new Set(JSON.parse(readFileSync(join(project,'.prism/build',receipt.build_id,'lexlean/snapshot.json'))).modules
 .flatMap(module=>module.declarations.map(row=>module.lean_module+'.'+row.lean_name)));
const runtimeRow=value=>value.declarations.find(row=>!sourceNames.has(row.name));
assert(runtimeRow(audit),'the genuine language-1.2 fixture must audit its imported runtime');
const auditedNames=new Set(audit.declarations.map(row=>row.name));
const unimportedRuntime=runtimeAudit(['import LexLeanRuntime.Reasoning\n']).filter(row=>!auditedNames.has(row.name));
assert(unimportedRuntime.length>0,'source-derived extra module must be absent from the genuine imported runtime');
checkAccepted(project,receipt,archive,compiler);
const mutations=[
 value=>{for(const phase of value.exporter_owner.phases)phase.processes[2].argv.push('--unowned');value.processes[7].argv.push('--unowned');},
 value=>{for(const phase of value.exporter_owner.phases)phase.lexlean_manifest_sha256='f'.repeat(64);},
 value=>{for(const phase of value.exporter_owner.phases)phase.artifacts[0].sha256='f'.repeat(64);value.artifacts[0].sha256='f'.repeat(64);},
 value=>{for(const phase of value.exporter_owner.phases)phase.processes[2].exporter.source_archive_sha256='f'.repeat(64);value.processes[7].exporter.source_archive_sha256='f'.repeat(64);},
 value=>{value.exporter_owner.phases[0].processes[2].exporter.acquisition={schema:'prismpm/exporter-acquisition/1',mode:'sdk-seed'};},
 value=>{value.exporter_owner.phases[0].processes[2].exporter.executable.sha256='f'.repeat(64);},
 value=>{delete value.exporter_owner;value.schema='prismpm/library-verification-manifest/1';},
];
for(const mutate of mutations){
 const changed=structuredClone(baseline);mutate(changed);
 const body=Buffer.from(canonical(changed)),id=hash(body),directory=join(project,'.prism/verified',id);
 assert(!existsSync(directory),'mutation output must be newly owned');mkdirSync(directory);
 try{
  for(const name of ['lexlean-attestation.json','library-acceptance.json'])copyFileSync(join(original,name),join(directory,name));
  writeFileSync(join(directory,'manifest.json'),body,{flag:'wx'});
  assert.throws(()=>checkAccepted(project,{...receipt,attestation_id:id,verified_root:'.prism/verified/'+id},archive,compiler));
 }finally{rmSync(directory,{recursive:true});}
}
const auditMutations=[
 value=>{value.declarations.splice(value.declarations.indexOf(runtimeRow(value)),1);},
 value=>{value.declarations.push(structuredClone(runtimeRow(value)));},
 value=>{runtimeRow(value).name='LexLeanRuntime.undeclaredAuditProbe';},
 value=>{const row=runtimeRow(value);row.policy={kind:'none',axioms:[]};row.observed=[];},
 value=>{const row=value.declarations.find(row=>!sourceNames.has(row.name)&&row.observed.length>0);assert(row);row.policy.axioms=[];row.observed=[];},
 value=>{runtimeRow(value).observed=['sorryAx'];},
 value=>{value.declarations=value.declarations.filter(row=>sourceNames.has(row.name));},
 value=>{value.declarations.push(...structuredClone(unimportedRuntime));},
 value=>{const row=runtimeRow(value);row.policy.axioms=['sorryAx'];row.observed=['sorryAx'];},
];
for(const mutate of auditMutations){
 const changedAudit=structuredClone(audit);mutate(changedAudit);
 changedAudit.attestation_id=lexleanAttestationId(changedAudit);
 const changedAuditBytes=Buffer.from(canonical(changedAudit)),changed=structuredClone(baseline);
 changed.lexlean_attestation_sha256=hash(changedAuditBytes);
 const changedAcceptance=structuredClone(acceptance);changedAcceptance.lexlean_attestation_id=changedAudit.attestation_id;
 const changedAcceptanceBytes=Buffer.from(canonical(changedAcceptance));changed.acceptance_sha256=hash(changedAcceptanceBytes);
 const body=Buffer.from(canonical(changed)),id=hash(body),directory=join(project,'.prism/verified',id);
 assert(!existsSync(directory),'audit mutation output must be newly owned');mkdirSync(directory);
 try{
  writeFileSync(join(directory,'library-acceptance.json'),changedAcceptanceBytes,{flag:'wx'});
  writeFileSync(join(directory,'lexlean-attestation.json'),changedAuditBytes,{flag:'wx'});
  writeFileSync(join(directory,'manifest.json'),body,{flag:'wx'});
  assert.throws(()=>checkAccepted(project,{...receipt,attestation_id:id,verified_root:'.prism/verified/'+id},archive,compiler),
   /complete published declaration audit|exact source-owned runtime axiom audit/);
 }finally{rmSync(directory,{recursive:true});}
}
assert(readFileSync(join(original,'manifest.json')).equals(bytes));
assert(readFileSync(join(original,'lexlean-attestation.json')).equals(auditBytes));
assert(readFileSync(join(original,'library-acceptance.json')).equals(acceptanceBytes));
checkAccepted(project,receipt,archive,compiler);
console.log(JSON.stringify({schema:'prismpm/native-library-reader-regression/1',real_baseline:'passed',mutations_rejected:mutations.length+auditMutations.length,restored_baseline:'passed'}));
