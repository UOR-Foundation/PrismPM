// Recording daemon-boundary controls; these are not actual SDK execution.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {chmodSync,copyFileSync,mkdtempSync,mkdirSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {test} from 'node:test';

const image='ghcr.io/uor-foundation/prismpm-sdk@sha256:'+'b'.repeat(64);
const daemon=`#!/usr/bin/env node
const fs=require('node:fs'),args=process.argv.slice(2),file=process.env.SESSION_STATE;
const state=fs.existsSync(file)?JSON.parse(fs.readFileSync(file)):{volumes:{},containers:{}};
fs.appendFileSync(process.env.SESSION_CALLS,JSON.stringify(args)+'\\n');
const save=()=>fs.writeFileSync(file,JSON.stringify(state)),mode=process.env.SESSION_MODE;
const label=()=>args[args.indexOf('--label')+1].split('=').slice(1).join('='),name=()=>args.at(-1);
const finish=(value,status=0)=>{if(value!==undefined)console.log(value);save();process.exit(status);};
if(args[0]==='image')finish('sha256:'+'d'.repeat(64));
if(args[1]==='ls'){
 const filter=args[args.indexOf('--filter')+1];
 if(mode==='listing-error')finish(undefined,11);
 if(args[0]==='volume'){
  const selected=filter.slice(6,-1);
  finish(mode==='existing-volume'?selected:Object.hasOwn(state.volumes,selected)?selected:'');
 }
 const selected=filter.startsWith('id=')?Object.values(state.containers).find(row=>row.id===filter.slice(3)):state.containers[filter.slice(7,-1)];
 finish(mode==='existing-container'?'f'.repeat(64):selected?.id??'');
}
if(args[0]==='volume'){
 if(args[1]==='create'){state.volumes[name()]={owner:mode==='wrong-volume-label'?'foreign':label(),created:'2026-10-12T00:00:00Z'};finish(name(),mode==='ambiguous-volume'?7:0);}
 if(args[1]==='inspect'){const row=state.volumes[args[2]];if(!row)finish(undefined,1);finish([args[2],'local',row.owner,row.created,'local','null'].join('|'));}
 if(args[1]==='rm'){if(mode==='volume-rm-fails'||Object.keys(state.containers).length)finish(undefined,9);delete state.volumes[name()];finish(name());}
}
if(args[0]==='container'){
 if(args[1]==='create'){
  const selected=args[args.indexOf('--name')+1],volume=args.includes('--volume')?args[args.indexOf('--volume')+1].split(':')[0]:'';
  state.containers[selected]={name:selected,id:'c'.repeat(64),image:mode==='wrong-image-request'?'foreign@sha256:'+'e'.repeat(64):process.env.SESSION_IMAGE,imageId:'sha256:'+(mode==='wrong-image-id'?'e':'d').repeat(64),owner:mode==='wrong-container-label'?'foreign':label(),volume,running:false};
  finish(mode==='ambiguous-output'?'not-an-id':'c'.repeat(64),mode==='ambiguous-container'?7:0);
 }
 const row=Object.values(state.containers).find(row=>row.name===args[2]||row.id===args[2]||row.id===args.at(-1));
 if(!row)finish(undefined,1);
 if(args[1]==='inspect'){
  const format=args[args.indexOf('--format')+1];
  if(format.includes('.State.Running'))finish([String(row.running),mode==='oom'?'true':'false',mode==='active-execs'?'["'+ 'f'.repeat(64)+'"]':'null','0','""'].join('|'));
  finish([row.id,'/'+row.name,row.imageId,row.image,row.owner,(row.volume?'volume:'+row.volume+':/config:true;':'')+'tmpfs::/tmp:true;'].join('|'));
 }
 if(args[1]==='start'){row.running=true;finish(row.id);}
 if(args[1]==='stop'){row.running=false;finish(row.id);}
 if(args[1]==='rm'){if(mode==='container-rm-fails')finish(undefined,9);delete state.containers[row.name];finish(row.id);}
}
if(args[0]==='test-control'){
 if(mode==='replacement-container')Object.values(state.containers)[0].owner='foreign';
 if(mode==='replacement-volume')Object.values(state.volumes)[0].created='2026-10-13T00:00:00Z';
 finish('');
}
finish(undefined,64);
`;

function execute(t,mode,{primary=0,mutate}={}){
 const work=mkdtempSync(join(tmpdir(),'sdk-owned-recording-'));t.after(()=>rmSync(work,{recursive:true,force:true}));
 const bin=join(work,'bin');mkdirSync(bin);
 const helper=join(work,'session.sh');copyFileSync(new URL('./sdk-owned-session.sh',import.meta.url),helper);
 if(mutate)writeFileSync(helper,mutate(readFileSync(helper,'utf8')));
 writeFileSync(join(bin,'docker'),daemon);chmodSync(join(bin,'docker'),0o755);
 const script=`set -euo pipefail
source "$SESSION_HELPER"
sdk_owned_begin
cleanup() { local primary=$? failed=0; trap - EXIT; if ! sdk_owned_cleanup; then failed=1; fi; if test "$primary" -ne 0; then exit "$primary"; fi; exit "$failed"; }
trap cleanup EXIT
sdk_owned_volume config
volume=$SDK_SESSION_VOLUME
sdk_owned_container registry "$SESSION_IMAGE" "$volume" --read-only --tmpfs /tmp:rw,nosuid,nodev --volume "$volume:/config" "$SESSION_IMAGE" sleep infinity
sdk_owned_docker container start "$SDK_SESSION_CONTAINER" >/dev/null
sdk_owned_docker test-control >/dev/null
exit "$SESSION_PRIMARY"
`;
 const context={...process.env,PATH:bin+':'+process.env.PATH,SESSION_STATE:join(work,'state.json'),SESSION_CALLS:join(work,'calls.jsonl'),SESSION_HELPER:helper,SESSION_MODE:mode,SESSION_IMAGE:image,SESSION_PRIMARY:String(primary)};
 const result=spawnSync('bash',['-c',script],{env:context,encoding:'utf8',timeout:10000,maxBuffer:1024*1024});
 assert.equal(result.error,undefined);assert.equal(result.signal,null);
 const calls=readFileSync(context.SESSION_CALLS,'utf8').trim().split('\n').filter(Boolean).map(JSON.parse);
 const state=JSON.parse(readFileSync(context.SESSION_STATE));return{result,calls,state};
}

test('recording owned session authenticates identities, stops execution, and confirms both absences',t=>{
 const {result,calls,state}=execute(t,'success');assert.equal(result.status,0,result.stderr);
 assert.deepEqual(state,{volumes:{},containers:{}});
 assert.ok(calls.some(row=>row[1]==='stop'));assert.ok(calls.filter(row=>row[1]==='rm').every(row=>!row.includes('--force')));
 const receipts=result.stderr.trim().split('\n').map(JSON.parse);assert.equal(receipts.length,2);assert.ok(receipts.every(row=>row.absent));
});
for(const mode of ['existing-volume','existing-container','wrong-volume-label','wrong-container-label','wrong-image-request','wrong-image-id','replacement-container','replacement-volume','container-rm-fails','volume-rm-fails','listing-error','active-execs','oom']){
 test('recording owned session refuses '+mode+' without granting qualification',t=>{
  const {result,calls,state}=execute(t,mode);assert.notEqual(result.status,0);
  if(mode.startsWith('existing-')||mode.startsWith('wrong-')||mode==='replacement-container'||mode==='active-execs')assert.ok(!calls.some(row=>row[0]==='container'&&row[1]==='rm'));
  if(mode==='replacement-volume'||mode==='wrong-volume-label'||mode==='existing-volume')assert.ok(!calls.some(row=>row[0]==='volume'&&row[1]==='rm'));
  if(mode==='replacement-container')assert.equal(Object.values(state.containers)[0].owner,'foreign');
 });
}
for(const mode of ['ambiguous-volume','ambiguous-container','ambiguous-output']){
 test('recording owned session reconciles only its authenticated side effect after '+mode,t=>{
  const {result,state}=execute(t,mode);assert.notEqual(result.status,0);assert.deepEqual(state,{volumes:{},containers:{}});
 });
}
test('recording cleanup failure preserves the exact primary verification failure',t=>{
 assert.equal(execute(t,'container-rm-fails',{primary:23}).result.status,23);
});
test('recording owner-label controls kill the real session validation mutant',t=>{
 const mutate=value=>{assert.ok(value.includes('test "$owner" = "$SDK_SESSION_OWNER"'));return value.replaceAll('test "$owner" = "$SDK_SESSION_OWNER"','true');};
 assert.equal(execute(t,'wrong-container-label',{mutate}).result.status,0,'mutant is demonstrably admitted; unchanged refusal assertion kills it');
 assert.notEqual(execute(t,'wrong-container-label').result.status,0);
});
