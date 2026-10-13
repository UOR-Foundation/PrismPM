// Recording Docker boundary tests only: no SDK execution/acceptance is claimed.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {chmodSync,copyFileSync,cpSync,mkdtempSync,mkdirSync,lstatSync,readFileSync,rmSync,symlinkSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
import {capture,canonical,hash,completedChecks,unclaimed} from './binary-sdk-check.mjs';

const source=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const revision='a'.repeat(40),image='ghcr.io/uor-foundation/prismpm-sdk@sha256:'+'b'.repeat(64);
const lineFeed=String.fromCharCode(10);
// Keep the repeated archive-only transport in Bash: no Node startup or prior
// call-log parsing is needed to record these fixed, validated arguments.
const dockerWrapper=`#!/usr/bin/env bash
set -euo pipefail
if [[ $1 == container && $2 == cp && $3 == *:/opt/prismpm/share/conformance-root/* ]]; then
 [[ $3 =~ ^c{64}:/opt/prismpm/share/conformance-root/[a-zA-Z0-9_./-]+$ ]] || exit 65
 test "$4" = - || { echo 'immutable SDK directory modes cannot be restored before populating the host copy' >&2; exit 68; }
 printf '["container","cp","%s","-"]\\012' "$3" >> "$RECORDED_CALLS"
 relative=\${3#*:/opt/prismpm/share/conformance-root/}
 input="$RECORDED_SOURCE/$relative"
 exec tar --format=pax --mode=a-w -c -C "$(dirname "$input")" "$(basename "$input")"
fi
exec node "$0-node" "$@"
`;
const dockerMock=`#!/usr/bin/env node
const fs=require('node:fs'),path=require('node:path'),args=process.argv.slice(2),root=process.env.RECORDED_SOURCE,log=process.env.RECORDED_CALLS;
const lineFeed=String.fromCharCode(10);
const earlier=fs.existsSync(log)?fs.readFileSync(log,'utf8').split(lineFeed).filter(Boolean).map(JSON.parse):[];
fs.appendFileSync(log,JSON.stringify(args)+lineFeed);
const statePath=process.env.RECORDED_STATE,state=fs.existsSync(statePath)?JSON.parse(fs.readFileSync(statePath)):{containers:[]};
const save=()=>fs.writeFileSync(statePath,JSON.stringify(state));
const imageId='sha256:'+'e'.repeat(64),value=flag=>args[args.indexOf(flag)+1];
const container=reference=>state.containers.find(row=>row.id===reference||row.name===reference);
if(args[0]==='pull')process.exit(0);
if(args[0]==='image'&&args[1]==='inspect'){
 if(args.includes('--format')){if(value('--format')!=='{{.Id}}')process.exit(64);console.log(imageId);process.exit(0);}
 console.log(JSON.stringify([{Id:imageId,Os:'linux',Architecture:process.arch==='x64'?'amd64':'arm64',RepoDigests:[process.env.RECORDED_IMAGE],Config:{Entrypoint:['/usr/local/bin/prismpm-devcontainer-init'],Volumes:null,Labels:{'org.opencontainers.image.revision':process.env.RECORDED_REVISION,'org.opencontainers.image.source':'https://github.com/UOR-Foundation/PrismPM','org.opencontainers.image.version':'0.3.0'}}}]));process.exit(0);
}
if(args[0]==='container'&&args[1]==='ls'){
 if(!args.includes('--all')||!args.includes('--no-trunc')||value('--format')!=='{{.ID}}')process.exit(64);
 const filter=value('--filter');
 if(!(filter.startsWith('name=^/')&&filter.endsWith('$'))&&!/^id=[a-f0-9]{64}$/.test(filter))process.exit(64);
 for(const row of state.containers)if(filter==='name=^/'+row.name+'$'||filter==='id='+row.id)console.log(row.id);
 process.exit(0);
}
if(args[0]==='container'&&args[1]==='create'){
 const name=value('--name'),label=value('--label'),source=name.endsWith('-source'),runtime=name.endsWith('-runtime');
 if(!args.includes('--name')||!args.includes('--label')||(!source&&!runtime)||!label.startsWith('io.prismpm.sdk-session=')||container(name))process.exit(64);
 const id=(source?'c':'d').repeat(64);if(container(id))process.exit(64);
 state.containers.push({id,name,owner:label.slice('io.prismpm.sdk-session='.length),image:process.env.RECORDED_IMAGE,mounts:source?'':'tmpfs::/tmp:true;',running:false,exit:0});save();console.log(id);process.exit(0);
}
if(args[0]==='container'&&args[1]==='cp'){
 if(!container(args[2].split(':')[0]))process.exit(1);
 const from=args[2].split(':').slice(1).join(':'),prefix='/opt/prismpm/share/conformance-root/';
 if(from.startsWith('/tmp/')){console.error('Docker archive API cannot read tmpfs');process.exit(66);}
 if(from==='/opt/prismpm/share/inventory.json'){fs.copyFileSync(process.env.RECORDED_INVENTORY,args[3]);process.exit(0);}
 const input=from.startsWith(prefix)?path.join(root,from.slice(prefix.length)):from==='/usr/local/bin/prismpm-devcontainer-init'?path.join(root,'sdk/devcontainer-init.sh'):null;
 if(!input)process.exit(65);
 if(from.startsWith(prefix))process.exit(68); // Source archives use the recording wrapper.
 fs.cpSync(input,args[3],{recursive:true,verbatimSymlinks:true});process.exit(0);
}
if(args[0]==='container'&&args[1]==='start'){
 const running=container(args.at(-1));if(!running||running.id!=='d'.repeat(64)||!args.includes('--attach'))process.exit(64);
 running.running=false;running.exit=Number(process.env.RECORDED_EXIT_STATUS||0);save();
 const fixture=require('node:child_process').spawnSync(process.execPath,[path.join(root,'scripts/binary-sdk-check-fixtures.mjs'),process.env.RECORDED_PROOF],{encoding:'utf8'});if(fixture.status!==0){process.stderr.write(fixture.stderr);process.exit(65);}const receipt=JSON.parse(fixture.stdout);
 const runtime=earlier.filter(row=>row[0]==='container'&&row[1]==='create').at(-1),last=runtime.slice(-4);
 const value={schema:'prismpm/installed-binary-check/1',scope:'installed-binary-package-only',status:'passed',sdk_image:last[0],source_revision:last[1],source_sha256:last[2],inventory_sha256:last[3],architecture:process.arch==='x64'?'amd64':'arm64',build_id:receipt.build_id,attestation_id:receipt.attestation_id,checks:JSON.parse(process.env.RECORDED_CHECKS),unclaimed:JSON.parse(process.env.RECORDED_UNCLAIMED)};
 const exportRoot=path.join(path.dirname(process.env.RECORDED_PROOF),'export');fs.mkdirSync(exportRoot);fs.writeFileSync(path.join(exportRoot,'binary-result.json'),process.env.RECORDED_OUTPUT===undefined?JSON.stringify(value):process.env.RECORDED_OUTPUT);fs.cpSync(process.env.RECORDED_PROOF,path.join(exportRoot,'prismpm-binary-evidence'),{recursive:true});
 if(process.env.RECORDED_ARCHIVE==='missing-proof'){require('node:child_process').spawnSync('tar',['-c','-C',exportRoot,'binary-result.json'],{stdio:['ignore','inherit','inherit']});}else if(process.env.RECORDED_ARCHIVE==='truncated'){process.stdout.write('broken archive');}else{const packed=require('node:child_process').spawnSync('tar',['-c','-C',exportRoot,'binary-result.json','prismpm-binary-evidence'],{stdio:['ignore','inherit','inherit']});if(packed.status!==0)process.exit(67);}
 process.exit(Number(process.env.RECORDED_START_STATUS||0));
}
if(args[0]==='container'&&args[1]==='inspect'){
 const reference=args[2]==='--format'?args.at(-1):args[2],row=container(reference);if(!row)process.exit(1);
 const format=value('--format');
 if(format==='{{.State.ExitCode}}')console.log(row.exit);
 else if(format==='{{.Id}}|{{.Name}}|{{.Image}}|{{.Config.Image}}|{{index .Config.Labels "io.prismpm.sdk-session"}}|{{range .Mounts}}{{.Type}}:{{.Name}}:{{.Destination}}:{{.RW}};{{end}}')console.log([row.id,'/'+row.name,imageId,row.image,row.owner,row.mounts].join('|'));
 else if(format==='{{.State.Running}}|{{.State.OOMKilled}}|{{json .ExecIDs}}|{{.State.ExitCode}}|{{json .State.Error}}')console.log([row.running,false,'null',row.exit,'""'].join('|'));
 else process.exit(64);
 process.exit(0);
}
if(args[0]==='container'&&args[1]==='stop'){
 const row=container(args.at(-1));if(!row||args.length!==5||args[2]!=='--time'||args[3]!=='5')process.exit(64);row.running=false;save();console.log(row.id);process.exit(0);
}
if(args[0]==='container'&&args[1]==='rm'){
 const row=container(args[2]);if(args.length!==3||!row||row.running)process.exit(64);state.containers=state.containers.filter(item=>item!==row);save();console.log(row.id);process.exit(0);
}
process.exit(64);
`;

function fixture(t){
 const work=mkdtempSync(join(tmpdir(),'prismpm-binary-shell-'));t.after(()=>{for(const relative of ['source/.cargo','source/.cargo/readonly-nested']){const path=join(work,relative);if(lstatSync(path,{throwIfNoEntry:false})?.isDirectory())chmodSync(path,0o700);}rmSync(work,{recursive:true,force:true});});
 const root=join(work,'source');mkdirSync(root);
 for(const row of capture(source,revision).files){const destination=join(root,row.path);mkdirSync(dirname(destination),{recursive:true});if(row.kind==='directory')mkdirSync(destination,{recursive:true});else if(row.kind==='file')copyFileSync(join(source,row.path),destination);else symlinkSync(row.target,destination);}
 const readonly=join(root,'.cargo/readonly-nested');mkdirSync(readonly);writeFileSync(join(readonly,'input'),'immutable source bytes');chmodSync(join(readonly,'input'),0o444);chmodSync(readonly,0o555);chmodSync(join(root,'.cargo'),0o555);
 const inventoryPath=join(work,'inventory.json');
 const commands=['cargo','devcontainer','docker','just','prismpm'].map(command=>({command,executable:'/usr/local/bin/'+command,sha256:'c'.repeat(64)}));
 const artifacts=['adapter','base-image','binary','crate','dependency-lock','oracle','schema','test-corpus','trust-root','workflow'].map(kind=>({id:kind,kind,version:'1',digest:'sha256:'+'d'.repeat(64)}));
 artifacts.push({id:'prismpm',kind:'binary',version:'0.3.0',digest:'sha256:'+'c'.repeat(64)},{id:'sdk-vv-source',kind:'test-corpus',version:revision,digest:'sha256:'+'e'.repeat(64)});artifacts.sort((a,b)=>a.id.localeCompare(b.id));
 writeFileSync(inventoryPath,canonical({schema:'prismpm/sdk-inventory/1',commands,artifacts}));
 const bin=join(work,'bin');mkdirSync(bin);writeFileSync(join(bin,'docker'),dockerWrapper);writeFileSync(join(bin,'docker-node'),dockerMock);chmodSync(join(bin,'docker'),0o755);
 const gitScript=['#!/usr/bin/env node','const args=process.argv.slice(2);if(args.includes("rev-parse"))console.log(process.env.RECORDED_REVISION);else if(!args.includes("status"))process.exit(64);',''].join(lineFeed);writeFileSync(join(bin,'git'),gitScript);chmodSync(join(bin,'git'),0o755);
 const gitProbe=spawnSync(join(bin,'git'),['rev-parse','HEAD'],{encoding:'utf8',env:{...process.env,RECORDED_REVISION:revision}});assert.equal(gitProbe.error,undefined);assert.equal(gitProbe.status,0);assert.equal(gitProbe.stdout,revision+lineFeed,'fixture git must be executable and emit a real newline');
 return{work,root,env:{...process.env,PATH:bin+':'+process.env.PATH,RECORDED_SOURCE:root,RECORDED_CALLS:join(work,'calls.jsonl'),RECORDED_STATE:join(work,'docker-state.json'),RECORDED_IMAGE:image,RECORDED_REVISION:revision,RECORDED_INVENTORY:inventoryPath,RECORDED_PROOF:join(work,'proof'),RECORDED_CHECKS:JSON.stringify(completedChecks),RECORDED_UNCLAIMED:JSON.stringify(unclaimed)}};
}
function execute(context){
 const result=spawnSync('bash',[join(context.root,'scripts/binary-sdk-check.sh'),image,revision],{encoding:'utf8',env:context.env,timeout:30000,maxBuffer:1024*1024});
 assert.equal(result.error,undefined);assert.equal(result.signal,null);
 const callsBytes=readFileSync(context.env.RECORDED_CALLS);
 assert.ok(callsBytes.length>0,'call log must contain a record');
 const callLines=callsBytes.toString('utf8').split(lineFeed);
 assert.equal(callLines.at(-1),'','call log delimiter must be an actual newline');
 const calls=callLines.slice(0,-1).filter(Boolean).map(JSON.parse);
 const roundTrip=['container','exec','line one'+lineFeed+'line two','quote"','slash\\'];
 const roundTripBytes=Buffer.from(JSON.stringify(roundTrip)+lineFeed);
 assert.deepEqual(roundTripBytes.toString('utf8').split(lineFeed).slice(0,-1).map(JSON.parse),[roundTrip],'JSONL preserves escaped multiline arguments');
 return{result,calls};
}
function accepted(context){
 assert.equal(lstatSync(join(context.root,'.cargo')).mode&0o777,0o555);assert.equal(lstatSync(join(context.root,'.cargo/readonly-nested')).mode&0o777,0o555);assert.equal(lstatSync(join(context.root,'.cargo/readonly-nested/input')).mode&0o777,0o444);
 const {result,calls}=execute(context);assert.equal(result.status,0,result.stderr.slice(-2000));
 const created=calls.filter(row=>row[0]==='container'&&row[1]==='create');assert.equal(created.length,2);
 const runtime=created[1];for(const arg of ['--read-only','--network','none','--user','1000:1000','--cap-drop','ALL','--security-opt','no-new-privileges','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=8g','PRISMPM_EPHEMERAL_HOME=1','CARGO_NET_OFFLINE=true'])assert.ok(runtime.includes(arg),arg);
 assert.ok(!runtime.includes('--mount')&&!runtime.includes('--volume')&&!runtime.includes('--entrypoint'));
 assert.deepEqual(runtime.slice(-9,-6),[image,'/bin/sh','-ec']);assert.match(runtime.at(-6),/node scripts\/binary-sdk-check\.mjs run "\$@" > \/tmp\/binary-result\.json/);assert.match(runtime.at(-6),/exec tar -c -C \/tmp binary-result\.json prismpm-binary-evidence/);assert.equal(runtime.at(-5),'binary-sdk');
 assert.deepEqual(runtime.slice(-4),[image,revision,hash(canonical(capture(context.root,revision))),hash(readFileSync(context.env.RECORDED_INVENTORY))]);
 const start=calls.findIndex(row=>row[0]==='container'&&row[1]==='start');assert.ok(start>=0,'runtime must actually start');
 const inspected=calls.findIndex(row=>row[0]==='container'&&row[1]==='inspect'&&row.includes('{{.State.ExitCode}}')&&row.at(-1)==='d'.repeat(64));assert.ok(inspected>start,'runtime exit must be inspected after execution');
 assert.equal(calls.filter(row=>row[0]==='container'&&row[1]==='start').length,1);
 assert.equal(calls[start].at(-1),'d'.repeat(64),'distinct runtime container');
 assert.deepEqual(calls.filter(row=>row[0]==='container'&&row[1]==='rm').map(row=>row.slice(2)),[['c'.repeat(64)],['d'.repeat(64)]],'both owned identities retire without force');
 assert.deepEqual(JSON.parse(readFileSync(context.env.RECORDED_STATE)).containers,[],'recorded live namespace is empty');
 assert.ok(calls.some(row=>row[0]==='container'&&row[1]==='cp'&&row[2].endsWith('/scripts/binary-sdk-check.test.mjs')));
 assert.ok(calls.filter(row=>row[0]==='container'&&row[1]==='cp'&&row[2].includes('/conformance-root/')).every(row=>row[3]==='-'),'source archives must be validated before materialization');
 assert.ok(!calls.some(row=>row[0]==='container'&&row[1]==='cp'&&row[2].includes(':/tmp/')));
 assert.match(result.stdout,/binary-package SDK closure passed/);
}

test('real shell invokes the confined Docker sequence and inspects actual terminal status',t=>{
 accepted(fixture(t));
 for(const field of ['RECORDED_START_STATUS','RECORDED_EXIT_STATUS']){const context=fixture(t);context.env[field]='7';const {result}=execute(context);assert.notEqual(result.status,0);assert.doesNotMatch(result.stdout,/closure passed/);}
 for(const archive of ['missing-proof','truncated']){const context=fixture(t);context.env.RECORDED_ARCHIVE=archive;assert.notEqual(execute(context).result.status,0);}
 for(const output of ['', 'TAP version 13\n1..0 # SKIP\n','{}']){const context=fixture(t);context.env.RECORDED_OUTPUT=output;const {result}=execute(context);assert.notEqual(result.status,0);assert.doesNotMatch(result.stdout,/closure passed/);}
});

test('real shell orchestration tests reject removed execution and changed runtime entrypoint mutants',t=>{
 for(const [before,after] of [
  ['docker container start --attach "$container"','true'],
  ['node scripts/binary-sdk-check.mjs run ', 'node scripts/browser-api-sdk-check.mjs run '],
 ]){const context=fixture(t),path=join(context.root,'scripts/binary-sdk-check.sh'),script=readFileSync(path,'utf8');assert.equal(script.split(before).length,2);writeFileSync(path,script.replace(before,after));assert.throws(()=>accepted(context));}
});
