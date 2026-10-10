// Recording transport tests only. No real SDK formatting is claimed here.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {chmodSync,copyFileSync,mkdtempSync,mkdirSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
import {canonical} from './binary-sdk-check.mjs';
import {inventory} from './binary-sdk-check-fixtures.mjs';
const owner=dirname(fileURLToPath(import.meta.url));
const image='ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21';
const original='pub fn probe(){}\n';
const dockerMock=`#!/usr/bin/env node
const fs=require('node:fs'),path=require('node:path'),args=process.argv.slice(2),log=process.env.FORMAT_CALLS,statePath=process.env.FORMAT_STATE;
const state=fs.existsSync(statePath)?JSON.parse(fs.readFileSync(statePath)):{};
const save=()=>fs.writeFileSync(statePath,JSON.stringify(state));
fs.appendFileSync(log,JSON.stringify(args)+'\\n');
const id='c'.repeat(64);
if(args[0]==='pull')process.exit(0);
if(args[0]==='image'&&args[1]==='inspect'){console.log(JSON.stringify([{Os:'linux',Architecture:'amd64',RepoDigests:[process.env.FORMAT_IMAGE],Config:{Entrypoint:['/usr/local/bin/prismpm-devcontainer-init'],Volumes:null,Labels:{'org.opencontainers.image.revision':process.env.FORMAT_REVISION,'org.opencontainers.image.source':'https://github.com/UOR-Foundation/PrismPM','org.opencontainers.image.version':'0.3.0'}}}]));process.exit(0);}
if(args[0]==='container'&&args[1]==='create'){state.created=args;save();console.log(id);process.exit(0);}
if(args[0]==='container'&&args[1]==='start'){console.log(id);process.exit(0);}
if(args[0]==='container'&&args[1]==='inspect'){console.log(process.env.FORMAT_READY==='never'?'false':'true');process.exit(0);}
if(args[0]==='container'&&args[1]==='logs')process.exit(0);
if(args[0]==='container'&&args[1]==='rm')process.exit(0);
if(args[0]==='exec'&&args.includes('/tmp/prismpm-format-ready')){
 state.polls=(state.polls||0)+1;state.ready=state.polls>=3&&process.env.FORMAT_READY!=='never';save();process.exit(state.ready?0:1);
}
if(args[0]==='exec'){
 if(!state.ready){console.error('SDK initializer has not completed');process.exit(71);}
 if(args.includes('mkdir'))process.exit(0);
 if(args.includes('tar')&&args.includes('-x')){fs.mkdirSync(process.env.FORMAT_IMPORTED);const imported=require('node:child_process').spawnSync('tar',['-x','-C',process.env.FORMAT_IMPORTED],{input:fs.readFileSync(0)});if(imported.status!==0)process.exit(74);state.source=true;save();process.exit(0);}
 if(args.includes('tar')&&args.includes('-c')){if(!state.formatted||process.env.FORMAT_EXPORT_FAIL==='yes')process.exit(75);const exported=require('node:child_process').spawnSync('tar',['-c','-C',process.env.FORMAT_IMPORTED,'.'],{stdio:['ignore','inherit','inherit']});process.exit(exported.status);}
 if(args.includes('/usr/local/bin/prismpm-devcontainer-init')){if(!state.source)process.exit(72);if(process.env.FORMAT_CHANGE==='yes')fs.writeFileSync(path.join(process.env.FORMAT_IMPORTED,'crates/probe/src/lib.rs'),'pub fn probe() {}\\n');if(process.env.FORMAT_GENERATED_CHANGE==='yes'||args.at(-1).includes('cargo fmt --all'))fs.writeFileSync(path.join(process.env.FORMAT_IMPORTED,'stdlib/generated/package/src/lib.rs'),'pub fn probe() {}\\n');state.formatted=true;save();console.log('recording formatter only');process.exit(0);}
}
if(args[0]==='container'&&args[1]==='cp'){
 if(args[2].endsWith(':/opt/prismpm/share/inventory.json')){fs.copyFileSync(process.env.FORMAT_INVENTORY,args[3]);process.exit(0);}
 if(args[2].includes(':/tmp/')){console.error('Docker archive API cannot read tmpfs');process.exit(76);}
}
console.error('unexpected Docker operation '+JSON.stringify(args));process.exit(64);
`;
const git=(root,args)=>{const output=spawnSync('git',['-C',root,...args],{encoding:'utf8'});assert.equal(output.status,0,output.stderr);return output.stdout.trim();};
function fixture(t){
 const work=mkdtempSync(join(tmpdir(),'prismpm-format-shell-'));t.after(()=>rmSync(work,{recursive:true,force:true}));
 const root=join(work,'source');mkdirSync(join(root,'scripts'),{recursive:true});mkdirSync(join(root,'sdk'));mkdirSync(join(root,'crates/probe/src'),{recursive:true});mkdirSync(join(root,'stdlib/generated/package/src'),{recursive:true});
 for(const name of ['binary-sdk-format.sh','binary-sdk-check.mjs','library-sdk-check.mjs','browser-api-sdk-check.mjs'])copyFileSync(join(owner,name),join(root,'scripts',name));
 copyFileSync(join(owner,'../sdk/platform-lock.mjs'),join(root,'sdk/platform-lock.mjs'));writeFileSync(join(root,'crates/probe/src/lib.rs'),original);writeFileSync(join(root,'stdlib/generated/package/src/lib.rs'),original);
 git(root,['init','--quiet']);git(root,['add','--all']);git(root,['-c','user.name=PrismPM formatter test','-c','user.email=formatter@localhost','-c','commit.gpgsign=false','commit','--quiet','-m','Format transport fixture']);
 const revision=git(root,['rev-parse','HEAD']),bin=join(work,'bin');mkdirSync(bin);writeFileSync(join(bin,'docker'),dockerMock);chmodSync(join(bin,'docker'),0o755);
 const inventoryPath=join(work,'inventory.json');writeFileSync(inventoryPath,canonical(inventory()));
 return{work,root,revision,evidence:join(work,'evidence'),env:{...process.env,PATH:bin+':'+process.env.PATH,FORMAT_CALLS:join(work,'calls.jsonl'),FORMAT_STATE:join(work,'state.json'),FORMAT_IMPORTED:join(work,'imported'),FORMAT_INVENTORY:inventoryPath,FORMAT_IMAGE:image,FORMAT_REVISION:revision}};
}
function execute(context){
 const result=spawnSync('bash',[join(context.root,'scripts/binary-sdk-format.sh'),context.revision,context.evidence],{env:context.env,encoding:'utf8',timeout:30000,maxBuffer:1024*1024});assert.equal(result.error,undefined);assert.equal(result.signal,null);
 const calls=readFileSync(context.env.FORMAT_CALLS,'utf8').trim().split('\n').map(JSON.parse);return{result,calls};
}
test('formatter waits for the actual initialized session and preserves strict source-format failure',t=>{
 for(const changed of [false,true]){
  const context=fixture(t);if(changed)context.env.FORMAT_CHANGE='yes';const {result,calls}=execute(context);assert.equal(result.status,changed?1:0,result.stderr);
  const evidence=JSON.parse(readFileSync(join(context.evidence,'format-result.json')));assert.equal(evidence.status,changed?'review-required':'passed');assert.equal(evidence.source_revision,context.revision);
  assert.equal(readFileSync(join(context.root,'crates/probe/src/lib.rs'),'utf8'),original);assert.equal(git(context.root,['status','--porcelain']),'');assert.equal(readFileSync(join(context.env.FORMAT_IMPORTED,'stdlib/generated/package/src/lib.rs'),'utf8'),original);assert.deepEqual(evidence.changed_paths,changed?['crates/probe/src/lib.rs']:[]);
  const ready=calls.flatMap((row,index)=>row.includes('/tmp/prismpm-format-ready')?[index]:[]);assert.equal(ready.length,3);
  const initialized=calls.findIndex(row=>row[0]==='exec'&&row.includes('/usr/local/bin/prismpm-devcontainer-init'));assert.ok(initialized>ready.at(-1),'no concurrent initializer');
  assert.ok(calls.some(row=>row[0]==='exec'&&row.includes('tar')&&row.includes('-c')),'export live tmpfs');assert.ok(!calls.some(row=>row[0]==='container'&&row[1]==='cp'&&row[2].includes(':/tmp/')));
  const create=calls.find(row=>row[0]==='container'&&row[1]==='create');for(const value of ['1000:1000','--read-only','none','ALL','no-new-privileges','/tmp:rw,exec,nosuid,nodev,size=4g','PRISMPM_EPHEMERAL_HOME=1'])assert.ok(create.includes(value));assert.ok(!create.includes('--entrypoint')&&!create.includes('--mount'));
 }
 const generated=fixture(t);generated.env.FORMAT_GENERATED_CHANGE='yes';const generatedResult=execute(generated).result;assert.notEqual(generatedResult.status,0,'excluded generated source changes must fail');assert.match(generatedResult.stderr,/formatter changed non-owned Rust source/);
 const missing=fixture(t);missing.env.FORMAT_EXPORT_FAIL='yes';assert.notEqual(execute(missing).result.status,0,'failed live export must fail formatting gate');
 const failed=fixture(t);failed.env.FORMAT_READY='never';const {result,calls}=execute(failed);assert.notEqual(result.status,0);assert.ok(!calls.some(row=>row[0]==='exec'&&row.includes('/usr/local/bin/prismpm-devcontainer-init')));
 const mutant=fixture(t),path=join(mutant.root,'scripts/binary-sdk-format.sh'),source=readFileSync(path,'utf8');const start=source.indexOf('ready=0\n'),end=source.indexOf('docker exec "$container" mkdir /tmp/source',start);assert.ok(start>0&&end>start);writeFileSync(path,source.slice(0,start)+source.slice(end));git(mutant.root,['add','--all']);git(mutant.root,['-c','user.name=PrismPM formatter test','-c','user.email=formatter@localhost','-c','commit.gpgsign=false','commit','--quiet','-m','Remove readiness guard']);mutant.revision=git(mutant.root,['rev-parse','HEAD']);mutant.env.FORMAT_REVISION=mutant.revision;assert.notEqual(execute(mutant).result.status,0,'removed readiness guard must be killed');
});
