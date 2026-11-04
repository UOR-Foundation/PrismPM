// Real local Git graph fixtures only; no remote writes, fetching or SDK work.
import assert from 'node:assert/strict';
import childProcess from 'node:child_process';
import {cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {syncBuiltinESMExports} from 'node:module';
import {tmpdir} from 'node:os';
import {dirname, join} from 'node:path';
import {performance} from 'node:perf_hooks';
import {fileURLToPath} from 'node:url';
import test, {after, before} from 'node:test';
import {diagnosticInitialAncestor, verifyDiagnosticAncestor} from './native-resource-ancestry.mjs';

const reviewed = 'de25a2a4e2b0850b974eb9146875b83093e7f278';
const source = join(dirname(fileURLToPath(import.meta.url)), '..');
const execute = childProcess.execFileSync;
const environment = {PATH:'/usr/bin:/bin',LANG:'C',LC_ALL:'C',GIT_CONFIG_NOSYSTEM:'1',
  GIT_CONFIG_GLOBAL:'/dev/null',GIT_CONFIG_SYSTEM:'/dev/null',GIT_NO_LAZY_FETCH:'1',GIT_OPTIONAL_LOCKS:'0',
  GIT_TERMINAL_PROMPT:'0',GIT_ALLOW_PROTOCOL:'',GIT_AUTHOR_NAME:'Offline graph fixture',
  GIT_AUTHOR_EMAIL:'fixture@example.invalid',GIT_COMMITTER_NAME:'Offline graph fixture',
  GIT_COMMITTER_EMAIL:'fixture@example.invalid',GIT_AUTHOR_DATE:'2026-11-04T00:00:00Z',GIT_COMMITTER_DATE:'2026-11-04T00:00:00Z'};
const git = (root,args,input,replace=false,env=environment) => execute('/usr/bin/git',[
  ...(replace?[]:['--no-replace-objects']),'-C',root,'-c','core.hooksPath=/dev/null','-c','core.fsmonitor=false',...args],
  {env,encoding:'utf8',input,timeout:10000,maxBuffer:16*1024**2,stdio:['pipe','pipe','pipe']});
let scratch, template;
before(()=>{
  scratch=mkdtempSync(join(tmpdir(),'native-ancestry-fixtures-'));template=join(scratch,'template');mkdirSync(template);
  git(template,['init','--quiet','--template=']);
  const ids=git(source,['-c','core.commitGraph=false','rev-list',reviewed]).trimEnd().split('\n');
  const bytes=execute('/usr/bin/git',['--no-replace-objects','-C',source,'cat-file','--batch'],
    {env:environment,input:ids.join('\n')+'\n',timeout:10000,maxBuffer:16*1024**2});
  const raw=join(scratch,'raw');mkdirSync(raw);const paths=[];let offset=0;
  for(const oid of ids){
    const end=bytes.indexOf(10,offset);assert(end>=0);const [actual,type,size]=bytes.subarray(offset,end).toString('ascii').split(' ');
    assert.equal(actual,oid);assert.equal(type,'commit');assert(/^\d+$/.test(size));
    offset=end+1;const path=join(raw,oid);writeFileSync(path,bytes.subarray(offset,offset+Number(size)));paths.push(path);
    offset+=Number(size);assert.equal(bytes[offset++],10);
  }
  assert.equal(offset,bytes.length);
  // Only exact raw commit objects are needed for commit ancestry. Importing
  // their original bytes reproduces their source OIDs without source writes.
  const actual=git(template,['hash-object','-w','--no-filters','-t','commit','--stdin-paths'],paths.join('\n')+'\n').trimEnd().split('\n');
  assert.equal(actual.length,ids.length);assert(actual.every((id,index)=>id===ids[index]),'imported raw commits preserve all original OIDs');assert(ids.includes(diagnosticInitialAncestor));
});
after(()=>rmSync(scratch,{recursive:true,force:true}));
function fixture(t,imported=true){
  const root=mkdtempSync(join(scratch,'repo-'));git(root,['init','--quiet','--template=']);
  if(imported)cpSync(join(template,'.git/objects'),join(root,'.git/objects'),{recursive:true});
  if(imported)git(root,['update-ref','HEAD',reviewed]);
  return root;
}
function commit(root,parent){
  const tree=git(root,['mktree'],'').trim();
  return git(root,['commit-tree',tree,...(parent?['-p',parent]:[])], 'synthetic fixture commit\n').trim();
}
function head(root,revision){git(root,['update-ref','HEAD',revision]);}
function rejected(root,revision){assert.throws(()=>verifyDiagnosticAncestor(revision,root));}

test('the independently imported real c302 to reviewed de25 graph yields the exact strict proof record',t=>{
  assert.equal(diagnosticInitialAncestor,'c30276799786c1c9093d4740f41d6376180b8567');
  const root=fixture(t),expected={before:diagnosticInitialAncestor,after:reviewed,strict_ancestor:true};
  assert.deepEqual(verifyDiagnosticAncestor(reviewed,root),expected);
  assert.equal(JSON.stringify(verifyDiagnosticAncestor(reviewed,root)),JSON.stringify(expected));
});

test('the actual checked-out source uses the same independently pinned strict proof',()=>{
  const actual=git(source,['rev-parse','HEAD']).trim();
  assert.deepEqual(verifyDiagnosticAncestor(actual),{before:diagnosticInitialAncestor,after:actual,strict_ancestor:true});
});

test('equality, malformed revisions, different HEAD, missing objects and blob HEAD cannot produce a proof',t=>{
  const root=fixture(t);
  for(const revision of [diagnosticInitialAncestor,'main','A'.repeat(40),'a'.repeat(39),'a'.repeat(41),'0'.repeat(40)])rejected(root,revision);
  const older=git(root,['rev-parse',diagnosticInitialAncestor+'^']).trim();rejected(root,older);
  const missing='f'.repeat(40);writeFileSync(join(root,'.git/HEAD'),missing+'\n');rejected(root,missing);
  const blob=git(root,['hash-object','-w','--stdin'],'fixture blob, never a commit\n').trim();
  writeFileSync(join(root,'.git/HEAD'),blob+'\n');rejected(root,blob);
});

test('a missing pinned ancestor fails even with a valid checked-out commit',t=>{
  const root=fixture(t,false),revision=commit(root);head(root,revision);rejected(root,revision);
});

test('nonancestor and disconnected complete local histories are refused',t=>{
  const root=fixture(t),older=git(root,['rev-parse',diagnosticInitialAncestor+'^']).trim();
  head(root,older);rejected(root,older);
  const disconnected=commit(root);head(root,disconnected);rejected(root,disconnected);
});

test('replacement refs cannot remove real ancestry or invent ancestry for a disconnected commit',t=>{
  const root=fixture(t),disconnected=commit(root);
  git(root,['update-ref','refs/replace/'+reviewed,disconnected]);
  assert.throws(()=>git(root,['merge-base','--is-ancestor',diagnosticInitialAncestor,reviewed],undefined,true));
  assert.equal(verifyDiagnosticAncestor(reviewed,root).strict_ancestor,true);
  git(root,['update-ref','-d','refs/replace/'+reviewed]);
  const falseParent=commit(root,diagnosticInitialAncestor);
  git(root,['update-ref','refs/replace/'+disconnected,falseParent]);head(root,disconnected);
  assert.equal(git(root,['merge-base','--is-ancestor',diagnosticInitialAncestor,disconnected],undefined,true),'');
  rejected(root,disconnected);
});

test('legacy grafts that really invent an ancestor and shallow boundaries are refused',t=>{
  const root=fixture(t),disconnected=commit(root);head(root,disconnected);
  const graft=join(root,'.git/info/grafts');mkdirSync(dirname(graft),{recursive:true});writeFileSync(graft,disconnected+' '+diagnosticInitialAncestor+'\n');
  assert.equal(git(root,['merge-base','--is-ancestor',diagnosticInitialAncestor,disconnected],undefined,true),'');
  assert.throws(()=>verifyDiagnosticAncestor(disconnected,root),/legacy grafts/);
  rmSync(graft);head(root,reviewed);writeFileSync(join(root,'.git/shallow'),diagnosticInitialAncestor+'\n');
  assert.throws(()=>verifyDiagnosticAncestor(reviewed,root),/complete offline commit history/);
});

test('configured promisor transport cannot fetch a missing ancestor, even when its local protocol is allowed',t=>{
  const root=fixture(t),marker=join(root,'fetch-attempt'),helper=join(root,'fake-fetch.cjs');
  writeFileSync(helper,`require('node:fs').writeFileSync(${JSON.stringify(marker)},'local fixture fetch attempted');process.exit(73);`);
  rmSync(join(root,'.git/objects',diagnosticInitialAncestor.slice(0,2),diagnosticInitialAncestor.slice(2)));
  git(root,['config','extensions.partialClone','fixture']);git(root,['config','remote.fixture.promisor','true']);
  git(root,['config','remote.fixture.url','ext::'+process.execPath+' '+helper]);git(root,['config','protocol.ext.allow','always']);
  const permissive={...environment,GIT_ALLOW_PROTOCOL:'ext'};delete permissive.GIT_NO_LAZY_FETCH;
  assert.throws(()=>git(root,['cat-file','-t',diagnosticInitialAncestor],undefined,false,permissive));
  assert(existsSync(marker),'the planted transport really runs without the independent offline guards');rmSync(marker);
  rejected(root,reviewed);assert(!existsSync(marker),'the actual proof never invokes configured transport');
});

test('absolute direct Git calls discard inherited selectors, config, alternates and tracing',t=>{
  const root=fixture(t),trace=join(root,'inherited-trace'),calls=[];
  const planted={GIT_DIR:'/not-the-repository',GIT_WORK_TREE:'/other',GIT_COMMON_DIR:'/other',GIT_OBJECT_DIRECTORY:'/other',
    GIT_ALTERNATE_OBJECT_DIRECTORIES:'/other',GIT_INDEX_FILE:'/other/index',GIT_GRAFT_FILE:'/other/grafts',
    GIT_REPLACE_REF_BASE:'refs/fixture',GIT_EXEC_PATH:'/other',GIT_CONFIG_COUNT:'1',GIT_CONFIG_KEY_0:'core.commitGraph',
    GIT_CONFIG_VALUE_0:'true',GIT_CONFIG_PARAMETERS:"'protocol.allow=always'",GIT_TRACE:trace,GIT_TRACE2_EVENT:trace};
  const previous=Object.fromEntries(Object.keys(planted).map(key=>[key,process.env[key]]));
  const spy=t.mock.method(childProcess,'execFileSync',(command,args,options)=>{calls.push({command,args,options});return execute(command,args,options);});
  syncBuiltinESMExports();
  try{
    Object.assign(process.env,planted);assert.equal(verifyDiagnosticAncestor(reviewed,root).strict_ancestor,true);
  }finally{
    for(const [key,value] of Object.entries(previous)){if(value===undefined)delete process.env[key];else process.env[key]=value;}
    spy.mock.restore();syncBuiltinESMExports();
  }
  assert(calls.length>=7);assert(!existsSync(trace));
  for(const {command,args,options} of calls){
    assert.equal(command,'/usr/bin/git');assert(args.includes('--no-replace-objects'));assert(args.includes('--no-optional-locks'));assert(args.includes('--no-pager'));
    for(const config of ['core.commitGraph=false','protocol.allow=never','core.hooksPath=/dev/null','core.fsmonitor=false'])assert(args.includes(config));
    assert.equal(options.env.GIT_NO_LAZY_FETCH,'1');assert.equal(options.env.GIT_ALLOW_PROTOCOL,'');assert.equal(options.env.GIT_CONFIG_NOSYSTEM,'1');
    for(const key of Object.keys(planted))assert(!Object.hasOwn(options.env,key));
    assert.deepEqual(options.stdio,['ignore','pipe','pipe']);assert.equal(options.maxBuffer,65536);assert.equal(options.killSignal,'SIGKILL');
    assert(options.timeout>0&&options.timeout<=10000);
  }
});

test('the monotonic ten-second budget is shared across direct child calls rather than renewed',t=>{
  const root=fixture(t),calls=[];let clock=0;
  const spy=t.mock.method(childProcess,'execFileSync',(command,args,options)=>{calls.push(options.timeout);return execute(command,args,options);});
  const now=t.mock.method(performance,'now',()=>{clock+=2000;return clock;});syncBuiltinESMExports();
  try{assert.throws(()=>verifyDiagnosticAncestor(reviewed,root),/deadline exhausted/);}
  finally{now.mock.restore();spy.mock.restore();syncBuiltinESMExports();}
  assert(calls.length>=2&&calls.length<7);assert(calls.every((value,index)=>index===0||value<calls[index-1]));
});

test('caller budgets only narrow the limit and invalid values reject before any Git child',t=>{
  const root=fixture(t),calls=[];
  const spy=t.mock.method(childProcess,'execFileSync',(command,args,options)=>{calls.push(command);return execute(command,args,options);});syncBuiltinESMExports();
  try{
    for(const budget of [0,-1,10001,Number.MAX_SAFE_INTEGER,Infinity,NaN,1.5,'3',null])
      assert.throws(()=>verifyDiagnosticAncestor(reviewed,root,budget),/budget must narrow/);
  }finally{spy.mock.restore();syncBuiltinESMExports();}
  assert.equal(calls.length,0);
});

test('an expired three-millisecond caller budget refuses before starting any Git child',t=>{
  const root=fixture(t),calls=[];let tick=-4;
  const spy=t.mock.method(childProcess,'execFileSync',(command,args,options)=>{calls.push(command);return execute(command,args,options);});
  const now=t.mock.method(performance,'now',()=>{tick+=4;return tick;});syncBuiltinESMExports();
  try{assert.throws(()=>verifyDiagnosticAncestor(reviewed,root,3),/deadline exhausted/);}
  finally{now.mock.restore();spy.mock.restore();syncBuiltinESMExports();}
  assert.equal(calls.length,0);
});

test('an actual blocked local Git config read times out and cannot yield a successful proof',t=>{
  const root=fixture(t),config=join(root,'.git/config');rmSync(config);
  execute('/usr/bin/mkfifo',[config],{timeout:1000,stdio:['ignore','pipe','pipe']});
  const started=performance.now();
  assert.throws(()=>verifyDiagnosticAncestor(reviewed,root),error=>error.code==='ETIMEDOUT'&&error.signal==='SIGKILL');
  const elapsed=performance.now()-started;assert(elapsed>=9000&&elapsed<15000,'one bounded real direct-child timeout');
});
