// Setup-order tests use recording tool doubles; actual cold-cache compiler
// evidence is a separate local/hosted check, never inferred from these doubles.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {chmodSync,copyFileSync,existsSync,mkdirSync,mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {test} from 'node:test';
import {validateHelperBootstrapWorkflow} from './binary-sdk-check.mjs';

function fixture(t){
 const root=mkdtempSync(join(tmpdir(),'semantic-helper-bootstrap-'));t.after(()=>rmSync(root,{recursive:true,force:true}));
 mkdirSync(join(root,'scripts'));mkdirSync(join(root,'bin'));
 copyFileSync(new URL('./semantic-source-bootstrap.sh',import.meta.url),join(root,'scripts/semantic-source-bootstrap.sh'));
 writeFileSync(join(root,'Cargo.lock'),'recording lock\n');writeFileSync(join(root,'rust-toolchain.toml'),'[toolchain]\nchannel = \"1.97.1\"\n');
 const git=(...args)=>{const result=spawnSync('git',args,{cwd:root,encoding:'utf8'});assert.equal(result.status,0,result.stderr);return result.stdout.trim();};
 git('init','--quiet');git('add','.');git('-c','user.name=Setup test','-c','user.email=setup@example.invalid','commit','--quiet','-m','recording setup source');
 const evidence=join(root,'../'+root.split('/').at(-1)+'-evidence');
 t.after(()=>rmSync(evidence,{recursive:true,force:true}));
 writeFileSync(join(root,'bin/rustup'),String.raw`#!/usr/bin/env node
const fs=require('node:fs'),path=require('node:path'),a=process.argv.slice(2);
fs.appendFileSync(process.env.CALLS,JSON.stringify({args:a,offline:process.env.CARGO_NET_OFFLINE})+'\n');
if(a.join(' ')==='toolchain install 1.97.1 --profile minimal')process.exit(0);
if(a[0]==='which'&&a[1]==='--toolchain'&&a[2]==='1.97.1'&&['rustc','cargo'].includes(a[3])){console.log(__filename);process.exit(0);}
if(a[0]!=='run'||a[1]!=='1.97.1')process.exit(65);
if(a[3]==='--version'){console.log(a[2]+' recording version');process.exit(0);}
if(a.slice(2).join(' ')==='cargo fetch --locked')process.exit(Number(process.env.FETCH_STATUS||0));
if(a[2]==='cargo'&&a[3]==='build'){
 if(process.env.BUILD_STATUS)process.exit(Number(process.env.BUILD_STATUS));
 if(process.env.SWAP_SOURCE_ON_BUILD){const r=require('node:child_process').spawnSync('git',['-c','user.name=Setup test','-c','user.email=setup@example.invalid','commit','--quiet','--allow-empty','-m','clean source moved']);if(r.status!==0)process.exit(67);}
 const target=a[a.indexOf('--target-dir')+1];fs.mkdirSync(path.join(target,'debug'),{recursive:true});
 fs.writeFileSync(path.join(target,'debug/prismpm-semantic-source'),'#!/bin/sh\nprintf "[]"\n',{mode:0o755});process.exit(0);
}
process.exit(66);
`);chmodSync(join(root,'bin/rustup'),0o755);
 // Keep tool calls and build outputs outside the clean source snapshot.
 git('add','bin');git('-c','user.name=Setup test','-c','user.email=setup@example.invalid','commit','--quiet','-m','recording tool');
 const source=git('rev-parse','HEAD'),calls=join(evidence,'calls.jsonl'),target=join(evidence,'target');
 const run=(extra={},selected=source)=>spawnSync('bash',[join(root,'scripts/semantic-source-bootstrap.sh'),selected,target,evidence],{cwd:root,encoding:'utf8',env:{...process.env,PATH:join(root,'bin')+':'+process.env.PATH,CALLS:calls,...extra}});
 return{root,evidence,target,source,run,git,calls:()=>readFileSync(calls,'utf8').trim().split('\n').map(JSON.parse)};
}

test('bootstrap selects pinned tools, acquires locked inputs, then builds offline before readiness',t=>{
 const f=fixture(t),result=f.run();assert.equal(result.status,0,result.stderr+readFileSync(join(f.evidence,'bootstrap.log'),'utf8'));
 const calls=f.calls();assert.deepEqual(calls.map(row=>row.args),[
  ['toolchain','install','1.97.1','--profile','minimal'],['run','1.97.1','rustc','--version'],['run','1.97.1','cargo','--version'],
  ['which','--toolchain','1.97.1','rustc'],['which','--toolchain','1.97.1','cargo'],
  ['run','1.97.1','cargo','fetch','--locked'],['run','1.97.1','cargo','build','--locked','--offline','--jobs','2','--package','repo-conformance','--bin','prismpm-semantic-source','--target-dir',f.target],
 ]);assert.equal(calls[5].offline,'false');assert.equal(calls[6].offline,'true');
 assert.equal(readFileSync(join(f.evidence,'source-revision.txt'),'utf8').trim(),f.source);
 assert.equal(readFileSync(join(f.evidence,'bootstrap-exit-status.txt'),'utf8'),'0\n');
 assert.equal(readFileSync(join(f.evidence,'semantic-source-path.txt'),'utf8'),join(f.target,'debug/prismpm-semantic-source')+'\n');
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
 assert.notEqual(f.run({SWAP_SOURCE_ON_BUILD:'1'}).status,0,'a clean HEAD move during compilation cannot publish readiness');
 assert.notEqual(f.git('rev-parse','HEAD'),f.source);assert.equal(f.git('status','--porcelain'),'');
 assert.ok(existsSync(join(f.target,'debug/prismpm-semantic-source')),'the recording build finished before the source recheck');
 assert.ok(!existsSync(join(f.evidence,'semantic-source-path.txt')));
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
  const start=workflow.indexOf('      - name: Build the same-source semantic helper'),end=workflow.indexOf('      - name: Check ',start),next=workflow.indexOf('      - name: ',end+10);
  assert.throws(()=>validate(workflow.slice(0,start)+workflow.slice(end,next)+workflow.slice(start,end)+workflow.slice(next)),'owners before bootstrap rejected');
 }
});
