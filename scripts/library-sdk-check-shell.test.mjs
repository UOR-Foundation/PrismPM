// Recording Docker boundary tests only: no SDK execution/acceptance is claimed.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {chmodSync,copyFileSync,cpSync,mkdtempSync,mkdirSync,readFileSync,rmSync,symlinkSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
import {capture} from './library-sdk-check.mjs';
import {lockFixture,resultFixture} from './library-sdk-fixture.mjs';

const source=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const revision='a'.repeat(40),sdk=lockFixture(),image=sdk.image;
const dockerMock=`#!/usr/bin/env node
const fs=require('node:fs'),path=require('node:path'),args=process.argv.slice(2),root=process.env.RECORDED_SOURCE,log=process.env.RECORDED_CALLS;
const earlier=fs.existsSync(log)?fs.readFileSync(log,'utf8').trim().split('\\n').filter(Boolean).map(JSON.parse):[];
fs.appendFileSync(log,JSON.stringify(args)+'\\n');
const id='c'.repeat(64);
if(args[0]==='pull')process.exit(0);
if(args[0]==='image'&&args[1]==='inspect'){console.log(JSON.stringify([{Os:'linux',Architecture:process.arch==='x64'?'amd64':'arm64',RepoDigests:[process.env.RECORDED_IMAGE],Config:{Entrypoint:['/usr/local/bin/prismpm-devcontainer-init'],Volumes:null,Labels:{'org.opencontainers.image.revision':process.env.RECORDED_REVISION,'org.opencontainers.image.source':'https://github.com/UOR-Foundation/PrismPM','org.opencontainers.image.version':'0.3.0'}}}]));process.exit(0);}
if(args[0]==='container'&&args[1]==='create'){console.log(id);process.exit(0);}
if(args[0]==='container'&&args[1]==='cp'){
 const from=args[2].split(':').slice(1).join(':'),prefix='/opt/prismpm/share/conformance-root/';
 if(from==='/opt/prismpm/share/inventory.json'){fs.copyFileSync(process.env.RECORDED_INVENTORY,args[3]);process.exit(0);}
 const input=from.startsWith(prefix)?path.join(root,from.slice(prefix.length)):from==='/usr/local/bin/prismpm-devcontainer-init'?path.join(root,'sdk/devcontainer-init.sh'):null;
 if(!input)process.exit(65);fs.cpSync(input,args[3],{recursive:true,verbatimSymlinks:true});process.exit(0);
}
if(args[0]==='container'&&args[1]==='start'){
 const created=earlier.filter(row=>row[0]==='container'&&row[1]==='create').at(-1);
 if(created.includes('acquire-lock')){process.stdout.write(process.env.RECORDED_LOCK_OUTPUT===undefined?fs.readFileSync(process.env.RECORDED_LOCK):process.env.RECORDED_LOCK_OUTPUT);process.exit(Number(process.env.RECORDED_START_STATUS||0));}
 if(!fs.readFileSync(0).equals(fs.readFileSync(process.env.RECORDED_LOCK)))process.exit(66);
 process.stdout.write(process.env.RECORDED_OUTPUT===undefined?fs.readFileSync(process.env.RECORDED_RESULT):process.env.RECORDED_OUTPUT);process.exit(Number(process.env.RECORDED_RUNTIME_STATUS||0));
}
if(args[0]==='container'&&args[1]==='inspect'){console.log(process.env.RECORDED_EXIT_STATUS||'0');process.exit(0);}
if(args[0]==='container'&&args[1]==='rm')process.exit(0);
process.exit(64);
`;

function fixture(t){
 const work=mkdtempSync(join(tmpdir(),'prismpm-library-shell-'));t.after(()=>rmSync(work,{recursive:true,force:true}));
 const root=join(work,'source');mkdirSync(root);
 for(const row of capture(source,revision).files){const destination=join(root,row.path);mkdirSync(dirname(destination),{recursive:true});if(row.kind==='directory')mkdirSync(destination,{recursive:true});else if(row.kind==='file')copyFileSync(join(source,row.path),destination);else symlinkSync(row.target,destination);}
 writeFileSync(join(root,'standards.lock'),sdk.standards);
 const architecture=process.arch==='x64'?'amd64':'arm64';
 writeFileSync(join(work,'lock.json'),sdk.bytes);
 writeFileSync(join(work,'inventory.json'),sdk.lock.platforms.find(row=>row.platform==='linux/'+architecture).inventory_document);
 writeFileSync(join(work,'result.json'),JSON.stringify(resultFixture(sdk.binding(architecture),{
  archive_sha256:createHash('sha256').update(readFileSync(join(root,'vendor/lean4-prod/lean.tar'))).digest('hex'),toolchain:readFileSync(join(root,'lean-toolchain'),'utf8').trim()})));
 const bin=join(work,'bin');mkdirSync(bin);writeFileSync(join(bin,'docker'),dockerMock);chmodSync(join(bin,'docker'),0o755);
 writeFileSync(join(bin,'git'),'#!/usr/bin/env node\nconst args=process.argv.slice(2);if(args.includes("rev-parse"))console.log(process.env.RECORDED_REVISION);else if(!args.includes("status"))process.exit(64);\n');chmodSync(join(bin,'git'),0o755);
 return{work,root,env:{...process.env,DOCKER_CONFIG:join(work,'absent-credentials'),PATH:bin+':'+process.env.PATH,RECORDED_SOURCE:root,RECORDED_CALLS:join(work,'calls.jsonl'),RECORDED_IMAGE:image,RECORDED_REVISION:revision,RECORDED_LOCK:join(work,'lock.json'),RECORDED_INVENTORY:join(work,'inventory.json'),RECORDED_RESULT:join(work,'result.json')}};
}
function execute(context){
 const result=spawnSync('bash',[join(context.root,'scripts/library-sdk-check.sh'),image,revision],{encoding:'utf8',env:context.env,timeout:30000,maxBuffer:1024*1024});
 assert.equal(result.error,undefined);assert.equal(result.signal,null);
 const calls=readFileSync(context.env.RECORDED_CALLS,'utf8').trim().split('\n').map(JSON.parse);return{result,calls};
}
function accepted(context){
 const {result,calls}=execute(context);assert.equal(result.status,0,result.stderr.slice(-2000));
 const created=calls.filter(row=>row[0]==='container'&&row[1]==='create');assert.equal(created.length,3);
 const online=created[1];for(const arg of ['--read-only','--network','bridge','--cap-drop','ALL','--security-opt','no-new-privileges','--memory','512m','--pids-limit','128','DOCKER_CONFIG=/run/prismpm-registry-auth'])assert.ok(online.includes(arg),arg);
 assert.deepEqual(online.slice(-4),[image,'scripts/library-sdk-check.mjs','acquire-lock',image]);
 assert.equal(online[online.indexOf('--entrypoint')+1],'node');
 assert.equal(online[online.indexOf('--user')+1],process.getuid()+':'+process.getgid());
 const mount=online.indexOf('--mount');
 if(mount>=0)assert.equal(online[mount+1],'type=bind,source='+context.env.DOCKER_CONFIG+',target=/run/prismpm-registry-auth,readonly');
 assert.ok(!online.includes('PRISMPM_EPHEMERAL_HOME=1'));
 const runtime=created[2];for(const arg of ['--interactive','--read-only','--network','none','--user','1000:1000','--cap-drop','ALL','--security-opt','no-new-privileges','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=8g','PRISMPM_EPHEMERAL_HOME=1','CARGO_NET_OFFLINE=true'])assert.ok(runtime.includes(arg),arg);
 assert.ok(!runtime.includes('--mount')&&!runtime.includes('--volume')&&!runtime.includes('--entrypoint'));
 assert.deepEqual(runtime.slice(-5),[image,'node','scripts/library-sdk-check.mjs','run',image]);
 const start=calls.findIndex(row=>row[0]==='container'&&row[1]==='start');assert.ok(start>=0,'runtime must actually start');
 const inspected=calls.findIndex(row=>row[0]==='container'&&row[1]==='inspect');assert.ok(inspected>start,'runtime exit must be inspected after execution');
 const starts=calls.flatMap((row,index)=>row[0]==='container'&&row[1]==='start'?[index]:[]);assert.equal(starts.length,2);
 for(const index of starts)assert.equal(calls[index+1][1],'inspect');
 assert.ok(calls[starts[1]].includes('--interactive'));
 assert.ok(calls.some(row=>row[0]==='container'&&row[1]==='cp'&&row[2].endsWith('/scripts/library-sdk-check.test.mjs')));
 assert.match(result.stdout,/native-library SDK closure passed/);
}

test('real shell invokes the confined Docker sequence and inspects actual terminal status',t=>{
 accepted(fixture(t));
 for(const field of ['RECORDED_START_STATUS','RECORDED_RUNTIME_STATUS','RECORDED_EXIT_STATUS']){const context=fixture(t);context.env[field]='7';const {result}=execute(context);assert.notEqual(result.status,0);assert.doesNotMatch(result.stdout,/closure passed/);}
 for(const output of ['', '{}',sdk.bytes.toString()+'\n']){const context=fixture(t);context.env.RECORDED_LOCK_OUTPUT=output;const {result}=execute(context);assert.notEqual(result.status,0);assert.doesNotMatch(result.stdout,/closure passed/);}
 const authenticated=fixture(t);mkdirSync(authenticated.env.DOCKER_CONFIG);accepted(authenticated);
 for(const output of ['', 'TAP version 13\n1..0 # SKIP\n','{}']){const context=fixture(t);context.env.RECORDED_OUTPUT=output;const {result}=execute(context);assert.notEqual(result.status,0);assert.doesNotMatch(result.stdout,/closure passed/);}
});

test('real shell orchestration tests reject removed execution and changed runtime entrypoint mutants',t=>{
 for(const [before,after] of [
  ['docker container start --attach "$container"','true'],
  ['docker container start --attach --interactive "$container"','true'],
  ['--entrypoint node --user','--user'],
  ['node scripts/library-sdk-check.mjs run "$image")','node scripts/browser-api-sdk-check.mjs run "$image")'],
 ]){const context=fixture(t),path=join(context.root,'scripts/library-sdk-check.sh'),script=readFileSync(path,'utf8');assert.equal(script.split(before).length,2);writeFileSync(path,script.replace(before,after));assert.throws(()=>accepted(context));}
});
