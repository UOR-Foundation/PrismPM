// Synthetic transport adversaries only; none of these fixtures is acceptance.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {chmodSync,copyFileSync,mkdirSync,mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join} from 'node:path';
import test from 'node:test';
import {sourceSdkVerificationArguments,validateSourceSdkMetadata,verifySourceSdkHandoff} from './source-sdk-handoff.mjs';

const sha=bytes=>'sha256:'+createHash('sha256').update(bytes).digest('hex');
const encode=value=>Buffer.from(JSON.stringify(value));
const revision='a'.repeat(40),arch=process.arch==='x64'?'amd64':process.arch;
const repository='ghcr.io/uor-foundation/prismpm-sdk',ref='refs/heads/release/v0.4',event='workflow_dispatch';
const trust='standards/trust/sigstore-trusted-root-cosign-3.1.3.json';
function graph(change=value=>value,indexChange=value=>value){
 const platforms={};
 for(const platform of ['amd64','arm64']){
  const config=encode(change({architecture:platform,os:'linux',config:{Labels:{
   'org.opencontainers.image.revision':revision,'org.opencontainers.image.source':'https://github.com/UOR-Foundation/PrismPM',
   'org.opencontainers.image.version':'0.4.0','org.opencontainers.image.created':'1970-01-01T00:00:00Z'}}},platform));
  const manifest=encode({schemaVersion:2,mediaType:'application/vnd.oci.image.manifest.v1+json',
   config:{mediaType:'application/vnd.oci.image.config.v1+json',digest:sha(config),size:config.length},layers:[]});
  const child={mediaType:'application/vnd.oci.image.manifest.v1+json',digest:sha(manifest),size:manifest.length,platform:{os:'linux',architecture:platform}};
  platforms[platform]={manifest,config,child,childImage:repository+'@'+child.digest};
 }
 const index=encode(indexChange({schemaVersion:2,mediaType:'application/vnd.oci.image.index.v1+json',
  manifests:Object.values(platforms).map(value=>value.child)}));
 return {index,platforms,...platforms[arch],image:repository+'@'+sha(index)};
}
function fixture(t,g=graph(),{store='classic',fail,inspectChange=value=>value}={}){
 const source=mkdtempSync(join(tmpdir(),'prismpm-source-handoff-'));
 t.after(()=>rmSync(source,{recursive:true}));mkdirSync(join(source,dirname(trust)),{recursive:true});
 copyFileSync(new URL('../'+trust,import.meta.url),join(source,trust));
 const calls=[],options={image:g.image,revision,arch,ref,event,source};
 const launch=async(command,args,settings)=>{
  calls.push({command,args,settings});
  assert.equal(settings.cwd,source);assert.equal(settings.environment,process.env);
  assert(settings.timeout>0&&settings.limit>0);
  const result=bytes=>({status:0,signal:null,stderr:Buffer.alloc(0),stdout:Buffer.isBuffer(bytes)?bytes:Buffer.from(bytes)});
  if(fail?.(command,args))return {status:23,signal:null,stderr:Buffer.from('unit transport failure'),stdout:Buffer.alloc(0)};
  if(command==='git')return result(args[0]==='rev-parse'?revision+'\n':'');
  if(command==='cosign')return result('[{"unit_fixture":true}]\n');
  if(command==='oras'){
   const platform=Object.values(g.platforms).find(value=>value.childImage===args[2]);assert(platform);
   assert.deepEqual(args,['manifest','fetch-config',platform.childImage]);return result(platform.config);
  }
  assert.equal(command,'docker');
  if(args[0]==='buildx'){
   assert.deepEqual(args.slice(0,4),['buildx','imagetools','inspect','--raw']);
   if(args[4]===g.image)return result(g.index);
   const platform=Object.values(g.platforms).find(value=>value.childImage===args[4]);assert(platform);return result(platform.manifest);
  }
  if(args[0]==='pull'){assert.deepEqual(args,['pull','--platform','linux/'+arch,g.image]);return result('pulled fixture\n');}
  assert.deepEqual(args.slice(0,2),['image','inspect']);assert.equal(args.at(-1),g.image);
  const descriptor=args.includes('--platform')?g.child:{digest:sha(g.index),mediaType:'application/vnd.oci.image.index.v1+json',size:g.index.length};
  const value={Id:store==='classic'?sha(g.config):descriptor.digest,Os:'linux',Architecture:arch,RepoDigests:[g.image],
   Config:{Labels:{'org.opencontainers.image.revision':revision},Volumes:null},...(store==='classic'?{}:{Descriptor:descriptor})};
  return result(encode([inspectChange(value)]));
 };
 return {source,g,calls,options,launch};
}

test('exact release SDK index signer identity binds source, release ref and actual trigger',()=>{
 const g=graph(),args=sourceSdkVerificationArguments(g.image,revision,ref,event,'/trusted-root');
 for(const [flag,value]of [['--certificate-identity','https://github.com/UOR-Foundation/PrismPM/.github/workflows/release.yml@'+ref],
  ['--certificate-github-workflow-sha',revision],['--certificate-github-workflow-ref',ref],['--certificate-github-workflow-trigger',event],
  ['--certificate-oidc-issuer','https://token.actions.githubusercontent.com']])assert.equal(args[args.indexOf(flag)+1],value);
 assert.equal(args.at(-1),g.image);
 for(const values of [[g.image.replace('prismpm-sdk@','prismpm-sdk-candidate@'),revision,ref,event],
  [g.image,revision,'refs/heads/main',event],[g.image,revision,'refs/tags/v0.4.0',event],
  [g.image,revision,ref,'pull_request'],[g.image,'bad',ref,event],[repository+':mutable',revision,ref,event]])
  assert.throws(()=>sourceSdkVerificationArguments(...values,'/trusted-root'));
 assert.doesNotThrow(()=>sourceSdkVerificationArguments(g.image,revision,ref,'push','/trusted-root'));
});

test('metadata verifies actual config bytes and same-source labels through the signed index chain',()=>{
 const g=graph();assert.equal(validateSourceSdkMetadata(g.index,g.manifest,g.config,g.image,arch,revision).config_digest,sha(g.config));
 for(const [index,manifest,config]of [[Buffer.concat([g.index,Buffer.from(' ')]),g.manifest,g.config],
  [g.index,Buffer.concat([g.manifest,Buffer.from(' ')]),g.config],[g.index,g.manifest,Buffer.from('x'.repeat(g.config.length))],
  [g.index,g.manifest,Buffer.concat([g.config,Buffer.from(' ')])]])
  assert.throws(()=>validateSourceSdkMetadata(index,manifest,config,g.image,arch,revision));
 const stale=graph(value=>{value.config.Labels['org.opencontainers.image.revision']='b'.repeat(40);return value;});
 assert.throws(()=>validateSourceSdkMetadata(stale.index,stale.manifest,stale.config,stale.image,arch,revision));
});

for(const store of ['classic','containerd'])test('authenticated handoff uses original daemon and '+store+' identity rules',async t=>{
 const f=fixture(t,graph(),{store}),receipt=await verifySourceSdkHandoff(f.options,f.launch);
 assert.equal(receipt.image_reference,f.g.image);assert.equal(receipt.loaded.store,store);
 assert.equal(receipt.scope,'authenticated-transport-only-not-source-or-sdk-acceptance');
 assert.equal(Buffer.from(receipt.config_base64,'base64').equals(f.g.config),true);
 assert.deepEqual(Object.keys(receipt.platforms),['amd64','arm64']);
 for(const platform of ['amd64','arm64']){
  const retained=receipt.platforms[platform];
  assert.equal(retained.architecture,platform);
  assert.deepEqual(Buffer.from(retained.index_base64,'base64'),f.g.index);
  assert.deepEqual(Buffer.from(retained.manifest_base64,'base64'),f.g.platforms[platform].manifest);
  assert.deepEqual(Buffer.from(retained.config_base64,'base64'),f.g.platforms[platform].config);
  assert.equal(retained.chain.config_digest,sha(f.g.platforms[platform].config));
 }
 assert.deepEqual(receipt.chain,receipt.platforms[arch].chain);
 assert(f.calls.findIndex(row=>row.command==='cosign')<f.calls.findIndex(row=>row.command==='docker'));
 const pull=f.calls.findIndex(row=>row.command==='docker'&&row.args[0]==='pull');
 assert.equal(f.calls.filter(row=>row.command==='oras').length,2);
 assert(f.calls.filter(row=>row.command==='oras').every(row=>f.calls.indexOf(row)<pull));
 assert.equal(f.calls.filter(row=>row.command==='docker'&&row.args[0]==='pull').length,1);
 assert.equal(f.calls.filter(row=>row.command==='docker'&&row.args[0]==='image').length,2);
 assert(!f.calls.some(row=>row.args.includes('create')||row.args.includes('build')));
});

test('failed index signature prevents image acquisition and daemon mutation',async t=>{
 const f=fixture(t,graph(),{fail:command=>command==='cosign'});
 await assert.rejects(verifySourceSdkHandoff(f.options,f.launch),/cosign failed/);
 assert(!f.calls.some(row=>row.command==='docker'||row.command==='oras'));
});
test('changed pinned trust root prevents every external command',async t=>{
 const f=fixture(t);writeFileSync(join(f.source,trust),'untrusted');
 await assert.rejects(verifySourceSdkHandoff(f.options,f.launch),/trust root differs/);assert.deepEqual(f.calls,[]);
});
for(const [label,answer]of [['wrong HEAD','b'.repeat(40)+'\n'],['dirty source',' M tracked-source\n']])
 test(label+' prevents authenticated-image acquisition',async t=>{
  const f=fixture(t),launch=async(command,args,settings)=>{
   if(command==='git'&&args[0]===(label==='wrong HEAD'?'rev-parse':'status'))
    return {status:0,signal:null,stderr:Buffer.alloc(0),stdout:Buffer.from(answer)};
   return f.launch(command,args,settings);
  };
  await assert.rejects(verifySourceSdkHandoff(f.options,launch));
  assert(!f.calls.some(row=>['cosign','docker','oras'].includes(row.command)));
 });
test('source drift during image acquisition cannot produce a completed receipt',async t=>{
 const f=fixture(t);let heads=0;
 const launch=async(command,args,settings)=>{
  if(command==='git'&&args[0]==='rev-parse'&&++heads===2)
   return {status:0,signal:null,stderr:Buffer.alloc(0),stdout:Buffer.from('b'.repeat(40)+'\n')};
  return f.launch(command,args,settings);
 };
 await assert.rejects(verifySourceSdkHandoff(f.options,launch),/source revision differs/);
 assert(f.calls.some(row=>row.command==='docker'&&row.args[0]==='pull'));
});
test('self-consistent wrong-source config is rejected before pulling',async t=>{
 const g=graph(value=>{value.config.Labels['org.opencontainers.image.revision']='b'.repeat(40);return value;});
 const f=fixture(t,g);await assert.rejects(verifySourceSdkHandoff(f.options,f.launch));
 assert(!f.calls.some(row=>row.command==='docker'&&row.args[0]==='pull'));
});
for(const platform of ['amd64','arm64'])test(platform+' config must bind the same source even when the other platform is valid',async t=>{
 const g=graph((value,selected)=>{
  if(selected===platform)value.config.Labels['org.opencontainers.image.revision']='b'.repeat(40);
  return value;
 });
 const f=fixture(t,g);await assert.rejects(verifySourceSdkHandoff(f.options,f.launch));
 assert(!f.calls.some(row=>row.command==='docker'&&row.args[0]==='pull'));
});
for(const member of ['manifest','config'])test('corrupted nonnative '+member+' blocks the native pull',async t=>{
 const g=graph(),foreign=g.platforms[arch==='amd64'?'arm64':'amd64'];
 foreign[member]=Buffer.concat([foreign[member],Buffer.from(' ')]);
 const f=fixture(t,g);await assert.rejects(verifySourceSdkHandoff(f.options,f.launch));
 assert(!f.calls.some(row=>row.command==='docker'&&row.args[0]==='pull'));
});
for(const [label,change]of [['missing platform',value=>({...value,manifests:value.manifests.slice(0,1)})],
 ['duplicate platform',value=>({...value,manifests:[...value.manifests,value.manifests[0]]})],
 ['unexpected platform',value=>({...value,manifests:[...value.manifests,{...value.manifests[0],platform:{os:'linux',architecture:'riscv64'}}]})]])
 test('signed SDK index with '+label+' is rejected before child acquisition',async t=>{
  const f=fixture(t,graph(value=>value,change));await assert.rejects(verifySourceSdkHandoff(f.options,f.launch));
  assert(!f.calls.some(row=>row.command==='oras'||row.command==='docker'&&row.args[0]==='pull'));
 });
test('nonnative metadata transport failure blocks the native pull',async t=>{
 const g=graph(),foreign=g.platforms[arch==='amd64'?'arm64':'amd64'];
 const f=fixture(t,g,{fail:(command,args)=>command==='oras'&&args[2]===foreign.childImage});
 await assert.rejects(verifySourceSdkHandoff(f.options,f.launch),/oras failed/);
 assert(!f.calls.some(row=>row.command==='docker'&&row.args[0]==='pull'));
});
for(const change of [value=>({...value,RepoDigests:[repository+'@sha256:'+'f'.repeat(64)]}),
 value=>({...value,Id:'sha256:'+'f'.repeat(64)}),value=>({...value,Architecture:'wrong'})])
 test('actual loaded image outside admitted chain is rejected',async t=>{
  const f=fixture(t,graph(),{inspectChange:change});await assert.rejects(verifySourceSdkHandoff(f.options,f.launch));
 });

for(const status of [0,23])test('wrapper preserves argv, stdin and supplied image only after verification status '+status,t=>{
 const source=mkdtempSync(join(tmpdir(),'prismpm-source-wrapper-'));t.after(()=>rmSync(source,{recursive:true}));
 mkdirSync(join(source,'scripts'));mkdirSync(join(source,'bin'));
 copyFileSync(new URL('./source-sdk-handoff.sh',import.meta.url),join(source,'scripts/source-sdk-handoff.sh'));
 const node=join(source,'bin/node');writeFileSync(node,'#!/usr/bin/env bash\nprintf "%s\\n" "$@" > "$FAKE_ARGUMENTS"\nexit "$FAKE_STATUS"\n');chmodSync(node,0o755);
 const g=graph(),capture=join(source,'args'),result=spawnSync('bash',[join(source,'scripts/source-sdk-handoff.sh'),
  g.image,revision,arch,ref,event,'bash','-c','printf "%s\\n" "$PRISMPM_TEST_SDK_IMAGE" "$1"; cat','original-owner','argument with spaces'],
  {encoding:'utf8',input:'original stdin\n',env:{...process.env,PATH:join(source,'bin')+':'+process.env.PATH,FAKE_ARGUMENTS:capture,FAKE_STATUS:String(status)}});
 assert.equal(result.status,status,result.stderr);
 assert.deepEqual(readFileSync(capture,'utf8').trim().split('\n'),[join(source,'scripts/source-sdk-handoff.mjs'),g.image,revision,arch,ref,event]);
 assert.equal(result.stdout,status?'':g.image+'\nargument with spaces\noriginal stdin\n');
});
