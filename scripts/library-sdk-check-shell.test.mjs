// Recording Docker boundary tests only: no SDK execution/acceptance is claimed.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {chmodSync,copyFileSync,cpSync,mkdtempSync,mkdirSync,readFileSync,rmSync,symlinkSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
import {capture} from './library-sdk-check.mjs';

const source=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const revision='a'.repeat(40),image='ghcr.io/uor-foundation/prismpm-sdk@sha256:'+'b'.repeat(64);
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
 const input=from.startsWith(prefix)?path.join(root,from.slice(prefix.length)):from==='/usr/local/bin/prismpm-devcontainer-init'?path.join(root,'sdk/devcontainer-init.sh'):null;
 if(!input)process.exit(65);fs.cpSync(input,args[3],{recursive:true,verbatimSymlinks:true});process.exit(0);
}
if(args[0]==='container'&&args[1]==='start'){
 const value={scope:'installed-native-library-only',build_id:'d'.repeat(64),checks:['read-only-check','std','no_std','exact-package-replay','two-root-reproduction','product-refusal','missing-root','wrong-result-root','parameterized-root','nominal-impostor','false-generated-acceptance','restored-acceptance'],unclaimed:['application','browser','holo','production-release','deployment']};
 process.stdout.write(process.env.RECORDED_OUTPUT===undefined?JSON.stringify(value):process.env.RECORDED_OUTPUT);process.exit(Number(process.env.RECORDED_START_STATUS||0));
}
if(args[0]==='container'&&args[1]==='inspect'){console.log(process.env.RECORDED_EXIT_STATUS||'0');process.exit(0);}
if(args[0]==='container'&&args[1]==='rm')process.exit(0);
process.exit(64);
`;

function fixture(t){
 const work=mkdtempSync(join(tmpdir(),'prismpm-library-shell-'));t.after(()=>rmSync(work,{recursive:true,force:true}));
 const root=join(work,'source');mkdirSync(root);
 for(const row of capture(source,revision).files){const destination=join(root,row.path);mkdirSync(dirname(destination),{recursive:true});if(row.kind==='directory')mkdirSync(destination,{recursive:true});else if(row.kind==='file')copyFileSync(join(source,row.path),destination);else symlinkSync(row.target,destination);}
 const bin=join(work,'bin');mkdirSync(bin);writeFileSync(join(bin,'docker'),dockerMock);chmodSync(join(bin,'docker'),0o755);
 writeFileSync(join(bin,'git'),'#!/usr/bin/env node\nconst args=process.argv.slice(2);if(args.includes("rev-parse"))console.log(process.env.RECORDED_REVISION);else if(!args.includes("status"))process.exit(64);\n');chmodSync(join(bin,'git'),0o755);
 return{work,root,env:{...process.env,PATH:bin+':'+process.env.PATH,RECORDED_SOURCE:root,RECORDED_CALLS:join(work,'calls.jsonl'),RECORDED_IMAGE:image,RECORDED_REVISION:revision}};
}
function execute(context){
 const result=spawnSync('bash',[join(context.root,'scripts/library-sdk-check.sh'),image,revision],{encoding:'utf8',env:context.env,timeout:30000,maxBuffer:1024*1024});
 assert.equal(result.error,undefined);assert.equal(result.signal,null);
 const calls=readFileSync(context.env.RECORDED_CALLS,'utf8').trim().split('\n').map(JSON.parse);return{result,calls};
}
function accepted(context){
 const {result,calls}=execute(context);assert.equal(result.status,0,result.stderr.slice(-2000));
 const created=calls.filter(row=>row[0]==='container'&&row[1]==='create');assert.equal(created.length,2);
 const runtime=created[1];for(const arg of ['--read-only','--network','none','--user','1000:1000','--cap-drop','ALL','--security-opt','no-new-privileges','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=8g','PRISMPM_EPHEMERAL_HOME=1','CARGO_NET_OFFLINE=true'])assert.ok(runtime.includes(arg),arg);
 assert.ok(!runtime.includes('--mount')&&!runtime.includes('--volume')&&!runtime.includes('--entrypoint'));
 assert.deepEqual(runtime.slice(-4),[image,'node','scripts/library-sdk-check.mjs','run']);
 const start=calls.findIndex(row=>row[0]==='container'&&row[1]==='start');assert.ok(start>=0,'runtime must actually start');
 const inspected=calls.findIndex(row=>row[0]==='container'&&row[1]==='inspect');assert.ok(inspected>start,'runtime exit must be inspected after execution');
 assert.equal(calls.filter(row=>row[0]==='container'&&row[1]==='start').length,1);
 assert.ok(calls.some(row=>row[0]==='container'&&row[1]==='cp'&&row[2].endsWith('/scripts/library-sdk-check.test.mjs')));
 assert.match(result.stdout,/native-library SDK closure passed/);
}

test('real shell invokes the confined Docker sequence and inspects actual terminal status',t=>{
 accepted(fixture(t));
 for(const field of ['RECORDED_START_STATUS','RECORDED_EXIT_STATUS']){const context=fixture(t);context.env[field]='7';const {result}=execute(context);assert.notEqual(result.status,0);assert.doesNotMatch(result.stdout,/closure passed/);}
 for(const output of ['', 'TAP version 13\n1..0 # SKIP\n','{}']){const context=fixture(t);context.env.RECORDED_OUTPUT=output;const {result}=execute(context);assert.notEqual(result.status,0);assert.doesNotMatch(result.stdout,/closure passed/);}
});

test('real shell orchestration tests reject removed execution and changed runtime entrypoint mutants',t=>{
 for(const [before,after] of [
  ['docker container start --attach "$container"','true'],
  ['node scripts/library-sdk-check.mjs run)','node scripts/browser-api-sdk-check.mjs run)'],
 ]){const context=fixture(t),path=join(context.root,'scripts/library-sdk-check.sh'),script=readFileSync(path,'utf8');assert.equal(script.split(before).length,2);writeFileSync(path,script.replace(before,after));assert.throws(()=>accepted(context));}
});
