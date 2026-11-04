// Setup-order tests use recording tool doubles; actual cold-cache compiler
// evidence is a separate local/hosted check, never inferred from these doubles.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {chmodSync,copyFileSync,existsSync,mkdirSync,mkdtempSync,readFileSync,rmSync,symlinkSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {test} from 'node:test';
import {validateHelperBootstrapWorkflow} from './binary-sdk-check.mjs';

function fixture(t){
 const root=mkdtempSync(join(tmpdir(),'semantic-helper-bootstrap-'));t.after(()=>rmSync(root,{recursive:true,force:true}));
 mkdirSync(join(root,'scripts'));mkdirSync(join(root,'bin'));
 for(const name of ['semantic-source-bootstrap.sh','source-oracles-bootstrap.sh'])copyFileSync(new URL('./'+name,import.meta.url),join(root,'scripts',name));
 mkdirSync(join(root,'sdk/oracles'),{recursive:true});
 for(const name of ['package.json','package-lock.json'])copyFileSync(new URL('../sdk/oracles/'+name,import.meta.url),join(root,'sdk/oracles',name));
 writeFileSync(join(root,'tools.lock'),'[sdk-tools]\nnode = "22.23.2"\n');
 writeFileSync(join(root,'Cargo.lock'),'recording lock\n');writeFileSync(join(root,'rust-toolchain.toml'),'[toolchain]\nchannel = \"1.97.1\"\n');
 const git=(...args)=>{const result=spawnSync('git',args,{cwd:root,encoding:'utf8'});assert.equal(result.status,0,result.stderr);return result.stdout.trim();};
 git('init','--quiet');git('add','.');git('-c','user.name=Setup test','-c','user.email=setup@example.invalid','commit','--quiet','-m','recording setup source');
 const evidence=join(root,'../'+root.split('/').at(-1)+'-evidence');
 t.after(()=>rmSync(evidence,{recursive:true,force:true}));
 writeFileSync(join(root,'bin/rustup'),String.raw`#!/usr/bin/env node
const fs=require('node:fs'),path=require('node:path'),a=process.argv.slice(2);
fs.appendFileSync(process.env.CALLS,JSON.stringify({args:a,offline:process.env.CARGO_NET_OFFLINE,nodeReadyDuringBuild:a[3]==='build'?fs.existsSync(path.join(process.env.EVIDENCE,'node-oracles-path.txt')):undefined})+'\n');
if(a.join(' ')==='toolchain install 1.97.1 --profile minimal --no-self-update')process.exit(0);
if(a[0]==='which'&&a[1]==='--toolchain'&&a[2]==='1.97.1'&&['rustc','cargo'].includes(a[3])){console.log(__filename);process.exit(0);}
if(a[0]!=='run'||a[1]!=='1.97.1')process.exit(65);
if(a[3]==='--version'){console.log(a[2]+' recording version');process.exit(0);}
if(a.slice(2).join(' ')==='cargo fetch --locked')process.exit(Number(process.env.FETCH_STATUS||0));
if(a[2]==='cargo'&&a[3]==='build'){
 if(process.env.BUILD_STATUS)process.exit(Number(process.env.BUILD_STATUS));
 if(process.env.BUILD_COPY_DRIFT)fs.appendFileSync(process.env.ORACLE_ROOT+'/package-lock.json','\nchanged during Rust build\n');
 if(process.env.SWAP_SOURCE_ON_BUILD){const r=require('node:child_process').spawnSync('git',['-c','user.name=Setup test','-c','user.email=setup@example.invalid','commit','--quiet','--allow-empty','-m','clean source moved']);if(r.status!==0)process.exit(67);}
 const target=a[a.indexOf('--target-dir')+1];fs.mkdirSync(path.join(target,'debug'),{recursive:true});
 fs.writeFileSync(path.join(target,'debug/prismpm-semantic-source'),'#!/bin/sh\nprintf "[]"\nexit '+Number(process.env.HELPER_PROBE_STATUS||0)+'\n',{mode:0o755});process.exit(0);
}
process.exit(66);
`);chmodSync(join(root,'bin/rustup'),0o755);
 writeFileSync(join(root,'bin/node'),`#!${process.execPath}
const fs=require('node:fs'),cp=require('node:child_process'),a=process.argv.slice(2);
fs.appendFileSync(process.env.NODE_CALLS,JSON.stringify({args:a,nodePath:process.env.NODE_PATH,nodeOptions:process.env.NODE_OPTIONS})+'\\n');
if(a.length===1&&a[0]==='--version'){console.log(process.env.RECORD_NODE_VERSION||'v22.23.2');process.exit(0);}
const r=cp.spawnSync(process.execPath,a,{stdio:'inherit',env:process.env});if(r.error)throw r.error;process.exit(r.status??1);
`);chmodSync(join(root,'bin/node'),0o755);
 writeFileSync(join(root,'bin/npm'),`#!${process.execPath}
const fs=require('node:fs'),path=require('node:path'),a=process.argv.slice(2),root=process.cwd();
fs.appendFileSync(process.env.NPM_CALLS,JSON.stringify({args:a,cwd:root,nodePath:process.env.NODE_PATH,nodeOptions:process.env.NODE_OPTIONS,configKeys:Object.keys(process.env).filter(k=>k.toLowerCase().startsWith('npm_config_'))})+'\\n');
if(a.includes('--version')){console.log('10.9.8 recording npm');process.exit(0);}
if(!a.includes('ci')||!['--ignore-scripts','--no-audit','--no-fund'].every(x=>a.includes(x)))process.exit(65);
if(process.env.NPM_STATUS)process.exit(Number(process.env.NPM_STATUS));
const manifest=JSON.parse(fs.readFileSync(path.join(root,'package.json'))),modules=path.join(root,'node_modules');fs.mkdirSync(modules);
function install(name,source){const directory=path.join(modules,name);fs.mkdirSync(directory);fs.writeFileSync(path.join(directory,'package.json'),JSON.stringify({name,version:manifest.dependencies[name],main:'index.cjs',type:'commonjs'}));fs.writeFileSync(path.join(directory,'index.cjs'),source);return directory;}
const yaml=install('js-yaml',"exports.load=s=>{if(s.includes('['))throw Error('recording syntax error');return {ready:true,count:2};};");
const ajv=install('ajv',"module.exports=class Ajv{compile(){return x=>x&&x.ready===true&&Number.isInteger(x.count)&&x.count>=1&&Object.keys(x).length===2;}};");
fs.mkdirSync(path.join(ajv,'dist'));fs.writeFileSync(path.join(ajv,'dist/2020.js'),"module.exports=class Ajv2020{compile(){return x=>Array.isArray(x)&&x.length===2&&typeof x[0]==='boolean'&&Number.isInteger(x[1])&&x[1]>=1;}};");
if(process.env.ORACLE_LOCATION){const outside=path.join(process.env.EVIDENCE,'ambient-module');fs.mkdirSync(outside,{recursive:true});if(process.env.ORACLE_LOCATION==='yaml'){fs.cpSync(yaml,outside,{recursive:true});fs.rmSync(yaml,{recursive:true});fs.symlinkSync(outside,yaml,'dir');}else{const foreign=path.join(outside,'2020.cjs');fs.copyFileSync(path.join(ajv,'dist/2020.js'),foreign);fs.rmSync(path.join(ajv,'dist/2020.js'));fs.symlinkSync(foreign,path.join(ajv,'dist/2020.js'));}}
if(process.env.SOURCE_DRIFT)fs.appendFileSync(path.join(process.env.SOURCE_ROOT,'sdk/oracles',process.env.SOURCE_DRIFT),'\\nchanged source\\n');
if(process.env.COPY_DRIFT)fs.appendFileSync(path.join(root,process.env.COPY_DRIFT),'\\nchanged copy\\n');
`);chmodSync(join(root,'bin/npm'),0o755);
 writeFileSync(join(root,'bin/git'),`#!${process.execPath}
const cp=require('node:child_process'),a=process.argv.slice(2);
if(a[0]==='status'&&process.env.GIT_STATUS_FAILURE)process.exit(Number(process.env.GIT_STATUS_FAILURE));
const r=cp.spawnSync('/usr/bin/git',a,{stdio:'inherit',env:process.env});if(r.error)throw r.error;process.exit(r.status??1);
`);chmodSync(join(root,'bin/git'),0o755);
 // Keep tool calls and build outputs outside the clean source snapshot.
 git('add','bin');git('-c','user.name=Setup test','-c','user.email=setup@example.invalid','commit','--quiet','-m','recording tool');
 const source=git('rev-parse','HEAD'),calls=join(evidence,'calls.jsonl'),target=join(evidence,'target'),nodeRoot=target+'-node-oracles',npmCalls=join(evidence,'npm-calls.jsonl'),nodeCalls=join(evidence,'node-calls.jsonl');
 const run=(extra={},selected=source)=>spawnSync('bash',[join(root,'scripts/semantic-source-bootstrap.sh'),selected,target,evidence],{cwd:root,encoding:'utf8',env:{...process.env,PATH:join(root,'bin')+':'+process.env.PATH,CALLS:calls,NPM_CALLS:npmCalls,NODE_CALLS:nodeCalls,SOURCE_ROOT:root,EVIDENCE:evidence,ORACLE_ROOT:nodeRoot,...extra}});
 const runNode=(extra={},selected=source)=>spawnSync('bash',[join(root,'scripts/source-oracles-bootstrap.sh'),selected,nodeRoot,evidence],{cwd:root,encoding:'utf8',env:{...process.env,PATH:join(root,'bin')+':'+process.env.PATH,CALLS:calls,NPM_CALLS:npmCalls,NODE_CALLS:nodeCalls,SOURCE_ROOT:root,EVIDENCE:evidence,ORACLE_ROOT:nodeRoot,...extra}});
 const rows=path=>existsSync(path)?readFileSync(path,'utf8').trim().split('\n').filter(Boolean).map(JSON.parse):[];
 return{root,evidence,target,nodeRoot,source,run,runNode,git,calls:()=>rows(calls),npmCalls:()=>rows(npmCalls),nodeCalls:()=>rows(nodeCalls)};
}

test('bootstrap selects pinned tools, acquires locked inputs, then builds offline before readiness',t=>{
 const f=fixture(t),result=f.run();assert.equal(result.status,0,result.stderr+readFileSync(join(f.evidence,'bootstrap.log'),'utf8'));
 const calls=f.calls();assert.deepEqual(calls.map(row=>row.args),[
  ['toolchain','install','1.97.1','--profile','minimal','--no-self-update'],['run','1.97.1','rustc','--version'],['run','1.97.1','cargo','--version'],
  ['which','--toolchain','1.97.1','rustc'],['which','--toolchain','1.97.1','cargo'],
  ['run','1.97.1','cargo','fetch','--locked'],['run','1.97.1','cargo','build','--locked','--offline','--jobs','2','--package','repo-conformance','--bin','prismpm-semantic-source','--target-dir',f.target],
 ]);assert.equal(calls[5].offline,'false');assert.equal(calls[6].offline,'true');assert.equal(calls[6].nodeReadyDuringBuild,false,'compound Node readiness waits for Rust');
 assert.equal(readFileSync(join(f.evidence,'source-revision.txt'),'utf8').trim(),f.source);
 assert.equal(readFileSync(join(f.evidence,'bootstrap-exit-status.txt'),'utf8'),'0\n');
 assert.equal(readFileSync(join(f.evidence,'semantic-source-path.txt'),'utf8'),join(f.target,'debug/prismpm-semantic-source')+'\n');
 assert.equal(readFileSync(join(f.evidence,'node-oracles-path.txt'),'utf8'),join(f.nodeRoot,'node_modules')+'\n');
 assert.notEqual(f.run({},'0'.repeat(40)).status,0,'wrong source never reaches tools');assert.equal(f.calls().length,7);
 writeFileSync(join(f.root,'Cargo.lock'),'changed');assert.notEqual(f.run().status,0,'dirty source never reaches tools');assert.equal(f.calls().length,7);
 writeFileSync(join(f.root,'Cargo.lock'),'recording lock\n');
 writeFileSync(join(f.root,'rust-toolchain.toml'),'[toolchain]\nchannel = "1.96.0"\n');
 f.git('add','.');f.git('-c','user.name=Setup test','-c','user.email=setup@example.invalid','commit','--quiet','-m','wrong committed pin');
 assert.notEqual(f.run({},f.git('rev-parse','HEAD')).status,0,'clean wrong source pin never reaches tools');assert.equal(f.calls().length,7);
 assert.ok(!existsSync(join(f.evidence,'semantic-source-path.txt')));
});
test('failed dependency acquisition retains diagnostics and cannot publish helper readiness',t=>{
 const f=fixture(t);assert.equal(f.run({FETCH_STATUS:'17'}).status,17);assert.equal(f.calls().length,6);
 assert.equal(readFileSync(join(f.evidence,'bootstrap-exit-status.txt'),'utf8'),'17\n');
 assert.ok(existsSync(join(f.evidence,'bootstrap.log')));assert.ok(!existsSync(join(f.evidence,'semantic-source-path.txt')));assert.ok(!existsSync(f.target));
});
test('failed offline build retains diagnostics and cannot publish helper readiness',t=>{
 const f=fixture(t);assert.equal(f.run({BUILD_STATUS:'18'}).status,18);assert.equal(f.calls().length,7);
 assert.equal(readFileSync(join(f.evidence,'bootstrap-exit-status.txt'),'utf8'),'18\n');
 assert.ok(!existsSync(join(f.evidence,'semantic-source-path.txt')));
 const moved=fixture(t);
 assert.notEqual(moved.run({SWAP_SOURCE_ON_BUILD:'1'}).status,0,'a clean HEAD move during compilation cannot publish readiness');
 assert.notEqual(moved.git('rev-parse','HEAD'),moved.source);assert.equal(moved.git('status','--porcelain'),'');
 assert.ok(existsSync(join(moved.target,'debug/prismpm-semantic-source')),'the recording build finished before the source recheck');
 assert.ok(!existsSync(join(moved.evidence,'semantic-source-path.txt')));assert.ok(!existsSync(join(moved.evidence,'node-oracles-path.txt')));
});
test('both workflows require the exact helper before offline owners and retain prebuild evidence',()=>{
 for(const generated of [false,true]){
  const workflow=readFileSync(new URL('../.github/workflows/'+(generated?'binary-generated-review.yml':'binary-sdk-qualification.yml'),import.meta.url),'utf8');
  const validate=value=>validateHelperBootstrapWorkflow(value,generated);validate(workflow);
  for(const [before,after] of [
   ['bash scripts/semantic-source-bootstrap.sh','echo skipped bootstrap'],['"$RUNNER_TEMP/binary-helper-target"','"$RUNNER_TEMP/other-target"'],
   ['          CARGO_NET_OFFLINE: "true"','          CARGO_NET_OFFLINE: "false"'],
   ['PRISMPM_SEMANTIC_SOURCE: ${{ runner.temp }}/binary-helper-target/debug/prismpm-semantic-source','PRISMPM_SEMANTIC_SOURCE: /tmp/unbound-helper'],
   ['      - name: Build the same-source semantic helper before offline boundary tests\n','      - name: Build the same-source semantic helper before offline boundary tests\n        if: false\n'],
   ['node scripts/binary-sdk-check.mjs tests >','node scripts/binary-sdk-check.mjs tests || true >'],
   ['            ${{ runner.temp }}/'+(generated?'binary-helper-evidence/':'binary-qualification/setup/'),''],
  ]){assert.ok(workflow.includes(before));assert.throws(()=>validate(workflow.replace(before,after)),undefined,before);}
  if(!generated){assert.throws(()=>validate(workflow.replace('      - name: Initialize source-bound prerequisite diagnostics','      - name: Omitted prerequisite diagnostics')));assert.throws(()=>validate(workflow.replace('      - name: Retain prerequisite and SDK phase status\n        if: always()','      - name: Retain prerequisite and SDK phase status\n        if: success()')));}
  assert.throws(()=>validate(workflow.replace('boundary-step-exit-status.txt','omitted-exit-status.txt')));
  const start=workflow.indexOf('      - name: Build the same-source semantic helper'),end=workflow.indexOf('      - name: Check ',start),next=workflow.indexOf('      - name: ',end+10);
  assert.throws(()=>validate(workflow.slice(0,start)+workflow.slice(end,next)+workflow.slice(start,end)+workflow.slice(next)),'owners before bootstrap rejected');
 }
});


const unready=f=>{for(const name of ['semantic-source-path.txt','node-oracles-path.txt','SHA256SUMS'])assert(!existsSync(join(f.evidence,name)),name+' must not claim compound readiness');};

test('npm failure retains acquisition diagnostics and cannot publish either compound prerequisite',t=>{
 const f=fixture(t),result=f.run({NPM_STATUS:'21'});assert.equal(result.status,21);
 assert.equal(f.calls().length,0);unready(f);
 assert.equal(readFileSync(join(f.evidence,'bootstrap-exit-status.txt'),'utf8'),'21\n');
 assert.equal(readFileSync(join(f.evidence,'node-prerequisite/node-oracles-bootstrap-exit-status.txt'),'utf8'),'21\n');
 assert(existsSync(join(f.evidence,'node-prerequisite/node-oracles-bootstrap.log')));
});

test('source and copied Node manifests cannot drift during locked dependency acquisition',t=>{
 for(const key of ['SOURCE_DRIFT','COPY_DRIFT'])for(const name of ['package.json','package-lock.json']){
  const f=fixture(t),result=f.run({[key]:name});assert.notEqual(result.status,0,key+':'+name);unready(f);assert.equal(f.calls().length,0);
 }
});

test('the compound prerequisite rechecks private manifests after the Rust build before publishing both receipts',t=>{
 const f=fixture(t),result=f.run({BUILD_COPY_DRIFT:'1'});assert.notEqual(result.status,0);assert.equal(f.calls().length,7);unready(f);
 assert(existsSync(join(f.evidence,'node-prerequisite/node-oracles-path.txt')),'child receipt scopes only the earlier Node component');
});

test('stale readiness and any preexisting unowned Node directory are refused without removing its contents',t=>{
 const f=fixture(t);mkdirSync(f.nodeRoot,{recursive:true});writeFileSync(join(f.nodeRoot,'unowned'),'retained');
 for(const name of ['semantic-source-path.txt','node-oracles-path.txt','SHA256SUMS'])writeFileSync(join(f.evidence,name),'stale');
 assert.notEqual(f.run().status,0);unready(f);assert.equal(f.calls().length,0);assert.equal(f.npmCalls().length,0);
 assert.equal(readFileSync(join(f.nodeRoot,'unowned'),'utf8'),'retained');
 const completed=fixture(t);assert.equal(completed.run().status,0);assert.notEqual(completed.run().status,0);unready(completed);
 assert(existsSync(join(completed.nodeRoot,'node_modules')),'a previous private tree is not silently adopted or deleted');
});

test('js-yaml and the actual Ajv2020 owner entry must resolve inside private node_modules',t=>{
 for(const location of ['yaml','2020']){
  const f=fixture(t);assert.notEqual(f.run({ORACLE_LOCATION:location}).status,0,location);unready(f);assert.equal(f.calls().length,0);
  assert(existsSync(join(f.evidence,'ambient-module')),'the deliberately external dependency remains outside the private closure');
 }
});

test('the Node-only prerequisite uses exact private configuration and probes without claiming Rust readiness',t=>{
 const f=fixture(t),original=readFileSync(join(f.root,'sdk/oracles/package-lock.json'));
 const result=f.runNode({NODE_PATH:'/ambient/never-used',NODE_OPTIONS:'--no-warnings',npm_config_dry_run:'true',NPM_CONFIG_PREFIX:'/ambient/never-used',NPM_CONFIG_CACHE:'/ambient/never-used'});
 assert.equal(result.status,0,result.stderr+readFileSync(join(f.evidence,'node-oracles-bootstrap.log'),'utf8'));
 assert.equal(f.calls().length,0);assert(!existsSync(join(f.evidence,'semantic-source-path.txt')));
 assert.equal(readFileSync(join(f.evidence,'node-oracles-path.txt'),'utf8'),join(f.nodeRoot,'node_modules')+'\n');
 assert(readFileSync(join(f.evidence,'npm-version.txt'),'utf8').startsWith('10.9.8'));
 const npm=f.npmCalls();assert.equal(npm.length,2);
 for(const row of npm){assert.equal(row.nodePath,undefined);assert.equal(row.nodeOptions,undefined);assert.deepEqual(row.configKeys,[]);
  for(const [flag,suffix] of [['--prefix',''],['--userconfig','/.npm-userconfig'],['--globalconfig','/.npm-globalconfig'],['--cache','/.npm-cache']])assert.equal(row.args[row.args.indexOf(flag)+1],f.nodeRoot+suffix);
  assert(row.args.includes('--global=false'));
 }
 assert.equal(npm[1].cwd,f.nodeRoot);for(const flag of ['ci','--ignore-scripts','--no-audit','--no-fund'])assert(npm[1].args.includes(flag));
 const probe=JSON.parse(readFileSync(join(f.evidence,'node-oracles-probe.json')));assert.equal(probe.yaml_probe,true);assert.equal(probe.ajv_probe,true);assert.equal(probe.ajv2020_probe,true);
 for(const value of Object.values(probe.entries))assert(value.entry.startsWith(join(f.nodeRoot,'node_modules')+'/'));
 assert.deepEqual(readFileSync(join(f.root,'sdk/oracles/package-lock.json')),original);assert.deepEqual(readFileSync(join(f.nodeRoot,'package-lock.json')),original);
});

test('wrong Node versions and cleanly committed source Node pins fail before dependency acquisition',t=>{
 const version=fixture(t);assert.notEqual(version.run({RECORD_NODE_VERSION:'v24.0.0'}).status,0);unready(version);assert.equal(version.npmCalls().length,0);assert.equal(version.calls().length,0);
 const source=fixture(t);writeFileSync(join(source.root,'tools.lock'),'[sdk-tools]\nnode = "24.0.0"\n');source.git('add','tools.lock');source.git('-c','user.name=Setup test','-c','user.email=setup@example.invalid','commit','--quiet','-m','wrong Node pin');
 assert.notEqual(source.run({},source.git('rev-parse','HEAD')).status,0);unready(source);assert.equal(source.npmCalls().length,0);assert.equal(source.calls().length,0);
});


test('helper output that looks successful cannot mask its failed runtime-audit exit or preserve stale root readiness',t=>{
 const f=fixture(t);mkdirSync(f.evidence);for(const name of ['semantic-source-path.txt','node-oracles-path.txt','SHA256SUMS','bootstrap-exit-status.txt'])writeFileSync(join(f.evidence,name),'0\n');
 const result=f.run({HELPER_PROBE_STATUS:'23'});assert.equal(result.status,23);unready(f);assert.equal(f.calls().length,7);
 assert.equal(readFileSync(join(f.evidence,'bootstrap-exit-status.txt'),'utf8'),'23\n');assert(existsSync(join(f.evidence,'bootstrap.log')));
});

test('empty output from a failed Git status check cannot publish source readiness',t=>{
 for(const nodeOnly of [false,true]){
  const f=fixture(t);mkdirSync(f.evidence);for(const name of ['semantic-source-path.txt','node-oracles-path.txt','SHA256SUMS'])writeFileSync(join(f.evidence,name),'stale\n');
  const result=nodeOnly?f.runNode({GIT_STATUS_FAILURE:'31'}):f.run({GIT_STATUS_FAILURE:'31'});assert.equal(result.status,31);
  assert(!existsSync(join(f.evidence,'node-oracles-path.txt')));assert.equal(f.calls().length,0);assert.equal(f.npmCalls().length,0);
  if(!nodeOnly)unready(f);
 }
});
