// Synthetic response/envelope data for pure refusal owners only. No authentic
// provider response, complete source pack, usable SDK or execution is provided.
import {createHash} from 'node:crypto';
export const sha=b=>'sha256:'+createHash('sha256').update(b).digest('hex');
export const bytes=v=>Buffer.from(JSON.stringify(v));
export const clone=v=>JSON.parse(JSON.stringify(v));
export const producerName='Construct source-CI SDK input';
export const sdkGates=[['Workspace Unit Tests',180],['Conformance / contracts',120],['Conformance / core',120],
 ['Conformance / system-oci',180],['Conformance / verification-schemas',90]];
const checkout='Run actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683';
const node='Run actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38';
const buildx='Run docker/setup-buildx-action@8d2750c68a42422c14e847fe6c8ac0403b4cbd6f';
const upload='Run actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02';
const producerSteps=['Set up job',checkout,node,buildx,'Install the exact OCI transport',producerName,upload,
 'Post '+buildx,'Post '+node,'Post '+checkout,'Complete job'];
const consumerSteps=['Set up job',checkout,'Admit source-CI SDK input','Run the unchanged owning gate',
 'Retire source-CI SDK input','Post '+checkout,'Complete job'];
const iso=n=>new Date(n).toISOString();
function job(expected,id,name,names,start,duration){
 const end=start+duration,slot=Math.floor(duration/names.length);
 return {id,run_id:expected.run_id,run_attempt:expected.run_attempt,head_sha:expected.event_revision,name,status:'completed',conclusion:'success',
  labels:['ubuntu-24.04'],runner_group_name:'GitHub Actions',started_at:iso(start),completed_at:iso(end),
  steps:names.map((name,i)=>({name,number:i<7?i+1:i+6,status:'completed',conclusion:'success',started_at:iso(start+i*slot),completed_at:iso(start+(i+1)*slot)}))};
}
export function fixture(){
 const now=Date.parse('2026-10-14T22:00:00Z'),source='a'.repeat(40),head='b'.repeat(40),base='c'.repeat(40),tree='d'.repeat(40);
 const workflow=Buffer.from('synthetic source-CI workflow data; not executable');
 const standards=bytes({schema:'prismpm/standards-lock/1',authorities:[],oracles:[]});
 const policy=bytes({schema:'prismpm/sdk-vv-input-policy/1',source_revision:source,historical_revision:'1'.repeat(40),
  historical_tag:'2'.repeat(40),bootstrap_sha256:'3'.repeat(64),advisory_revision:'4'.repeat(40),advisory_tree:'5'.repeat(40)});
 const manifest=bytes({schema:'prismpm/sdk-vv-input-closure/1',scope:'sdk-vv-inputs-only',policy:JSON.parse(policy),
  source:{revision:source,tree,files:[{path:'.github/workflows/ci-parallel.yml',mode:'100644',sha256:sha(workflow).slice(7)},
   {path:'standards.lock',mode:'100644',sha256:sha(standards).slice(7)}]},advisory:{revision:'4'.repeat(40)},artifacts:[]});
 const config=bytes({os:'linux',architecture:'amd64',config:{Labels:{'org.opencontainers.image.revision':source,
  'org.opencontainers.image.source':'https://github.com/UOR-Foundation/PrismPM',
  'org.opencontainers.image.version':'0.3.0','org.opencontainers.image.created':'1970-01-01T00:00:00Z'}}});
 const image=bytes({schemaVersion:2,mediaType:'application/vnd.oci.image.manifest.v1+json',
  config:{mediaType:'application/vnd.oci.image.config.v1+json',digest:sha(config),size:config.length},
  layers:[{mediaType:'application/vnd.oci.image.layer.v1.tar+gzip',digest:sha(Buffer.from('synthetic layer descriptor only')),size:1}]});
 const files=new Map([['image-config.json',config],['image-manifest.json',image],['input-manifest.json',manifest],['input-policy.json',policy],['standards.lock',standards]]);
 const expected={repository_id:123,run_id:456,run_attempt:2,producer_job_id:789,artifact_id:12,event:'pull_request',event_revision:head,base_revision:base,
  source_revision:source,source_tree:tree,source_parents:[base,head],workflow_revision:base,workflow_digest:sha(workflow),
  input_policy_digest:sha(policy),input_manifest_digest:sha(manifest),standards_digest:sha(standards)};
 const archive=Buffer.from('actual small regular file bytes for DATA-custody tests, NOT OCI');
 const envelope={schema:'prismpm/source-ci-sdk-input/1',scope:'same-run-image-input-only-not-execution-evidence',
  source:{revision:source,tree,parents:[base,head],event_revision:head,base_revision:base,workflow_revision:base,workflow_digest:sha(workflow)},
  run_id:456,run_attempt:2,producer_job_id:789,platform:'linux/amd64',archive:{path:'sdk.oci.tar',byte_length:archive.length,digest:sha(archive)},
  image:{manifest_digest:sha(image),manifest_byte_length:image.length,config_digest:sha(config),config_byte_length:config.length},
  members:[],unclaimed:['provider-authentication','git-signature-authentication','whole-zip-and-oci-integrity','image-import','installed-sdk','gate-execution','full-vv','release']};
 const producerStart=Date.parse('2026-10-14T01:00:00Z'),producer=job(expected,789,producerName,producerSteps,producerStart,3600000);
 const provider={run:{id:456,run_attempt:2,repository:{id:123,full_name:'UOR-Foundation/PrismPM'},head_repository:{id:123,full_name:'UOR-Foundation/PrismPM'},
  head_sha:head,event:'pull_request',path:'.github/workflows/ci-parallel.yml',status:'in_progress',conclusion:null,
  created_at:'2026-10-14T00:57:00.000Z',run_started_at:'2026-10-14T00:58:00.000Z',updated_at:'2026-10-14T02:00:00.000Z'},jobs:[producer],
  artifacts:[{id:12,name:`source-ci-sdk-456-2-${source}`,expired:false,size_in_bytes:20000,digest:sha(Buffer.from('synthetic provider ZIP descriptor')),
   created_at:producer.steps[6].started_at,expires_at:'2026-10-15T00:00:00.000Z',
   workflow_run:{id:456,head_sha:head,repository_id:123,head_repository_id:123}}]};
 const f={now,expected,envelope,files,provider,archive,workflow};
 f.seal=()=>{
  f.envelope.members=[...f.files].sort(([a],[b])=>a.localeCompare(b)).map(([path,b])=>({path,byte_length:b.length,digest:sha(b)}));
  const mb=f.files.get('image-manifest.json'),cb=f.files.get('image-config.json');
  f.envelope.image={manifest_digest:sha(mb),manifest_byte_length:mb.length,config_digest:sha(cb),config_byte_length:cb.length};
 };
 f.consumer=(name=sdkGates[4][0],duration=120000,start=Date.parse('2026-10-14T20:00:00Z'))=>{
  const result=clone(f.provider);result.jobs.push(job(f.expected,790,name,consumerSteps,start,duration));
  result.run.updated_at=iso(start+duration);return result;
 };
 f.seal();return f;
}
