import assert from 'node:assert/strict';
import {spawnSync,execFileSync} from 'node:child_process';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync,renameSync,symlinkSync,openSync,closeSync,ftruncateSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join} from 'node:path';
import {test} from 'node:test';
import {capture,verifySource,verifyImage,verifyTap,runSuites,sourceRoots,suites} from './browser-api-sdk-check.mjs';

const revision='a'.repeat(40),image='ghcr.io/uor-foundation/prismpm-sdk@sha256:'+'b'.repeat(64);
const temporary=t=>{const root=mkdtempSync(join(tmpdir(),'prismpm-sdk-binding-'));t.after(()=>rmSync(root,{recursive:true,force:true}));return root;};
const put=(root,path,bytes)=>{mkdirSync(dirname(join(root,path)),{recursive:true});writeFileSync(join(root,path),bytes);};
function source(root){
 for(const path of sourceRoots){
  if(path!=='.cargo'&&path.includes('.')&&!path.endsWith('/rust')||path==='lean-toolchain'||path==='sdk/Dockerfile')put(root,path,path+'\n');
  else put(root,path+'/source.txt',path+'\n');
 }
}
function inspected(){return[{Os:'linux',Architecture:'amd64',RepoDigests:[image],Config:{Entrypoint:['/usr/local/bin/prismpm-devcontainer-init'],Volumes:null,Labels:{
 'org.opencontainers.image.revision':revision,
 'org.opencontainers.image.source':'https://github.com/UOR-Foundation/PrismPM',
 'org.opencontainers.image.version':'0.3.0',
}}}];}

test('current SDK source closure binds helper, compiler, suite and every selected byte',t=>{
 const root=temporary(t);source(root);const expected=capture(root,revision);verifySource(root,expected);
 for(const path of['scripts/browser-api-sdk-check.mjs','sdk/browser/source.txt','vendor/lexlean/source.txt','tests/browser-api/source.txt']){
  const bytes=readFileSync(join(root,path));put(root,path,Buffer.concat([bytes,Buffer.from('x')]));assert.throws(()=>verifySource(root,expected));put(root,path,bytes);
 }
 for(const mutate of[
  value=>value.files.pop(),value=>value.files.push(value.files[0]),value=>value.extra=true,
  value=>value.revision=[revision],value=>value.files[0].sha256=[value.files[0].sha256],
  value=>value.files[0].extra=true,value=>value.files.reverse(),value=>value.files[0].size++,
 ]){const changed=structuredClone(expected);mutate(changed);assert.throws(()=>verifySource(root,changed));}
 put(root,'tests/browser-api/nested/target/ordinary','keep');assert.throws(()=>verifySource(root,expected));
});

test('source binding rejects missing, aliased, nonregular and oversized inputs',t=>{
 const root=temporary(t);source(root);const expected=capture(root,revision);
 const path=join(root,'sdk/browser/source.txt'),saved=readFileSync(path);rmSync(path);assert.throws(()=>verifySource(root,expected));
 symlinkSync('../stdlib-sources.tar',path);assert.throws(()=>capture(root,revision));rmSync(path);put(root,'sdk/browser/source.txt',saved);
 renameSync(join(root,'stdlib'),join(root,'stdlib-real'));symlinkSync('stdlib-real',join(root,'stdlib'));assert.throws(()=>capture(root,revision),/parent alias/);rmSync(join(root,'stdlib'));renameSync(join(root,'stdlib-real'),join(root,'stdlib'));
 rmSync(path);execFileSync('mkfifo',[path]);assert.throws(()=>capture(root,revision),/bounded regular/);rmSync(path);
 const fd=openSync(path,'wx');try{ftruncateSync(fd,64*1024*1024+1);}finally{closeSync(fd);}assert.throws(()=>capture(root,revision),/bounded regular/);
});

test('SDK identity is immutable, exact-source and native-platform bound',()=>{
 verifyImage(inspected(),image,'amd64',revision);
 for(const mutate of[
  value=>value[0].Os='windows',value=>value[0].Architecture='arm64',value=>value[0].RepoDigests=[],value=>value[0].RepoDigests=image,
  value=>value[0].Config.Entrypoint=['/bin/true'],value=>value[0].Config.Volumes={'/opt/prismpm':{}},value=>value[0].Config.Volumes='',
  value=>value[0].Config.Labels['org.opencontainers.image.revision']='c'.repeat(40),
  value=>value[0].Config.Labels['org.opencontainers.image.source']='https://example.invalid',
  value=>value[0].Config.Labels['org.opencontainers.image.version']='0.2.0',
  value=>value.push(value[0]),
 ]){const changed=inspected();mutate(changed);assert.throws(()=>verifyImage(changed,image,'amd64',revision));}
 assert.throws(()=>verifyImage(inspected(),'ghcr.io/uor-foundation/prismpm-sdk:latest','amd64',revision));
 assert.throws(()=>verifyImage(inspected(),image,'amd64',[revision]));
});

const testSource=(count,skip=false)=>"import {test} from 'node:test';\n"+Array.from({length:count},(_,index)=>`test('case ${index}',${skip&&index===0?'{skip:true},':''}()=>{});\n`).join('');
function testFixtures(root){for(const suite of suites)for(const file of suite.files)put(root,'sdk/browser/'+file,testSource(suite.minimum));}

test('release acceptance actually invokes every closed owning suite and rejects omission or skip',t=>{
 const root=temporary(t);testFixtures(root);const calls=[];
 const launch=(program,args,options)=>{calls.push(args);return spawnSync(program,args,options);};
 assert.equal(runSuites(root,launch,()=>{}).length,8);
 assert.deepEqual(calls.map(args=>args.slice(4)),suites.map(row=>row.files.map(file=>'sdk/browser/'+file)));
 const path='sdk/browser/identity.test.mjs',second='sdk/browser/identity.browser.test.mjs';
 put(root,path,testSource(1));put(root,second,testSource(1));assert.throws(()=>runSuites(root,spawnSync,()=>{}),/incomplete test suite/);
 put(root,path,testSource(10,true));put(root,second,testSource(10));assert.throws(()=>runSuites(root,spawnSync,()=>{}),/incomplete pass set|skipped/);
 put(root,path,testSource(10));
 const duplicate=(...args)=>{const result=spawnSync(...args);result.stdout+='# tests 20\n';return result;};
 assert.throws(()=>runSuites(root,duplicate,()=>{}),/duplicate tests/);
});

test('real TAP parsing rejects missing, duplicate, zero and unsuccessful summaries',t=>{
 const root=temporary(t);put(root,'complete.mjs',testSource(3));
 const env={...process.env};delete env.NODE_TEST_CONTEXT;
 const output=spawnSync(process.execPath,['--test','--test-reporter=tap',join(root,'complete.mjs')],{encoding:'utf8',env});assert.equal(output.status,0);verifyTap(output.stdout,3);
 for(const changed of[
  output.stdout.replace(/^# tests .*\n/m,''),output.stdout+'# tests 3\n',
  output.stdout.replace('# tests 3','# tests 0'),output.stdout.replace('# fail 0','# fail 1'),
  output.stdout.replace('# cancelled 0','# cancelled 1'),output.stdout.replace('# skipped 0','# skipped 1'),
  output.stdout.replace('# todo 0','# todo 1'),output.stdout.replace('1..3','1..4'),
  output.stdout.replace('ok 2 -','ok 1 -'),output.stdout.replace('ok 1 -','not ok 1 -'),
  output.stdout.replace('# tests 3','# tests 03'),output.stdout.replace('TAP version 13',''),
 ])assert.throws(()=>verifyTap(changed,3));
});

test('owning release test kills a removed complete-TAP acceptance guard',t=>{
 const root=temporary(t),source=readFileSync(new URL('./browser-api-sdk-check.mjs',import.meta.url),'utf8');
 const before='const tests=verifyTap(output.stdout,suite.minimum)',after='const tests=suite.minimum';
 assert.equal(source.split(before).length,2);put(root,'browser-api-sdk-check.mjs',source.replace(before,after));
 put(root,'browser-api-sdk-check.test.mjs',readFileSync(new URL('./browser-api-sdk-check.test.mjs',import.meta.url)));
 const env={...process.env};delete env.NODE_TEST_CONTEXT;
 const result=spawnSync(process.execPath,['--test','--test-reporter=tap','--test-name-pattern=release acceptance actually',join(root,'browser-api-sdk-check.test.mjs')],{encoding:'utf8',env,timeout:15000,maxBuffer:1024*1024});
 assert.equal(result.error,undefined);assert.equal(result.status,1);assert.match(result.stdout,/Missing expected exception/);
});
