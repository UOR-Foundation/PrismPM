// Recording transports only. No container, mount, allocation or SDK executes.
import assert from 'node:assert/strict';
import {execFileSync, spawn, spawnSync} from 'node:child_process';
import {cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {once} from 'node:events';
import {setTimeout as delay} from 'node:timers/promises';
import test from 'node:test';
import {diagnosticPolicy} from './native-resource-diagnostic.mjs';

const sha = 'a'.repeat(40);
const actualPython = execFileSync('sh', ['-c', 'command -v python3'], {encoding: 'utf8'}).trim();
const runner = new URL('./native-resource-diagnostic.sh', import.meta.url);
function fixture(t, options = {}) {
  const root = mkdtempSync(join(tmpdir(), 'native-resource-unit-'));
  t.after(() => rmSync(root, {recursive: true, force: true}));
  for (const path of ['source/scripts', 'bin', 'daemon', 'temporary', 'expected', 'credential-config']) mkdirSync(join(root, path), {recursive: true});
  cpSync(runner, join(root, 'source/scripts/native-resource-diagnostic.sh'));
  for (const name of ['native-resource-diagnostic.mjs', 'native-resource-monitor.py']) writeFileSync(join(root, 'source/scripts', name), 'recording transport only\n');
  for (const arch of ['amd64', 'arm64']) writeFileSync(join(root, 'expected', `production-${arch}.json`), JSON.stringify({source_revision: sha}));
  const state = join(root, 'state.json'), record = join(root, 'commands.jsonl');
  writeFileSync(state, JSON.stringify({mounted: false, attached: false, containers: {}, counter: 0}));
  const program = `#!${process.execPath}
const fs=require('node:fs'),cp=require('node:child_process'),path=require('node:path');
const root=${JSON.stringify(root)},stateFile=${JSON.stringify(state)},record=${JSON.stringify(record)},options=${JSON.stringify(options)};
const command=path.basename(process.argv[1]),args=process.argv.slice(2),sha=${JSON.stringify(sha)};
function append(value){fs.appendFileSync(record,JSON.stringify(value)+'\\n');}
function read(){return JSON.parse(fs.readFileSync(stateFile));}function save(value){fs.writeFileSync(stateFile,JSON.stringify(value));}
function emit(value){process.stdout.write(typeof value==='string'?value:JSON.stringify(value));}
append({command,args,...(command==='docker'?{dockerConfig:process.env.DOCKER_CONFIG}:{})});
if(command==='sudo'){const a=args[0]==='-n'?args.slice(1):args;const tool=path.basename(a[0]);if(tool==='rm'){if(!a.at(-1).startsWith(path.join(root,'temporary')+'/'))throw Error('unowned fake root removal');const r=cp.spawnSync('/bin/rm',a.slice(1),{stdio:'inherit'});process.exit(r.status??1);}if(!['mkfs.ext4','losetup','mount','umount','chown','python3'].includes(tool))throw Error('unapproved fake sudo target');const r=cp.spawnSync(path.join(root,'bin',tool),a.slice(1),{stdio:'inherit',env:process.env});process.exit(r.status??1);}
if(command==='python3'){
 while(['-I','-B'].includes(args[0]))args.shift();
 if(args[0]?.endsWith('/native-resource-monitor.py')){
  const mode=args[1],plan=JSON.parse(fs.readFileSync(args[2]));
  if(mode==='baseline'){fs.writeFileSync(args[3],JSON.stringify({schema:'prismpm/native-resource-baseline/1',complete:true,source_revision:plan.source_revision,run_id:plan.run_id,run_attempt:plan.run_attempt,phase:plan.phase}));process.exit(0);}
  if(mode==='retire'){const complete=!options.populatedCgroup||!plan.owner.name.includes('-production-production0');fs.writeFileSync(args[4],JSON.stringify({complete,cid:plan.owner.cid,stopped:true,pid:0,owner_oom_killed:!!(options.terminalOOM&&plan.owner.name.includes('-production-production0')),resource_failure:!!(options.terminalOOM&&plan.owner.name.includes('-production-production0')),resource_observation_complete:!options.absentCgroup,original_cgroup:{state:options.absentCgroup?'absent':'empty'}}));process.exit(complete?0:1);}
  if(mode==='snapshot'){const identity=plan.owner?{cid:plan.owner.cid,pid:101,pid_start_ticks:42,cgroup_path:'/sys/fs/cgroup/fake',cgroup_dev:1,cgroup_ino:2,config_sha256:'c'.repeat(64),bind_identities:[],namespace_sha256:(options.namespaceDrift&&args[3].endsWith('/production-after-bootstrap.json')?'d':'n').repeat(64)}:null;fs.writeFileSync(args[3],JSON.stringify({complete:true,fast:{owner:identity,path_identities:[]},final_fast:{owner:identity,path_identities:[]}}));process.exit(0);}
  if(mode!=='watch')throw Error('unexpected monitor command');
  fs.writeFileSync(args[3],JSON.stringify({complete:true})+'\\n');
  if(options.staleMonitor&&plan.owner.name.includes('-production-production0'))fs.utimesSync(args[3],new Date(0),new Date(0));
  if(options.monitorDies&&plan.owner.name.includes('-production-production0'))process.exit(31);
  const timer=setInterval(()=>{if(fs.existsSync(plan.finish_file)){JSON.parse(fs.readFileSync(plan.finish_file));clearInterval(timer);process.exit(0);}},10);
 }else{const r=cp.spawnSync(${JSON.stringify(actualPython)},args,{stdio:'inherit',env:process.env});process.exit(r.status??1);}
}else if(command==='stat'){
 const target=args.at(-1),s=fs.lstatSync(target),format=args.find(x=>x.startsWith('--format=')).slice(9);
 if(format==='%d:%i')emit(s.dev+':'+s.ino+'\\n');
 else if(format==='%d:%i:%s:%b')emit(s.dev+':'+s.ino+':42949672960:'+((options.holes&&read().formatted)?1:83886080)+'\\n');else throw Error('unexpected stat');
}else if(command==='git'){
 if(args.includes('rev-parse'))emit(sha+'\\n');
 else if(args.includes('status')){if(options.dirty)emit(' M tracked\\n');}
 else if(args.includes('clone')){const dest=args.at(-1);fs.mkdirSync(path.join(dest,'scripts'),{recursive:true});fs.writeFileSync(path.join(dest,'scripts/sdk-image-inputs.mjs'),'fake SDK transport only\\n');}
 else if(!args.includes('checkout'))throw Error('unexpected git');
}else if(command==='uname')emit('x86_64\\n');
else if(command==='df')emit('Avail\\n'+(args.at(-1).endsWith('/store')?38654705664:(options.lowSpace?1:107374182400))+'\\n');
else if(command==='fallocate'){
 if(args.at(-1).endsWith('.exclusive-exhaustion-probe')){process.stderr.write('fallocate: No space left on device\\n');process.exit(options.quotaUnexpectedSuccess?0:1);}
 fs.writeFileSync(args.at(-1),'fake bounded store allocation');
}else if(command==='mkfs.ext4'){const s=read();s.formatted=true;save(s);if(options.replacedBacking){fs.renameSync(args.at(-1),args.at(-1)+'.original');fs.writeFileSync(args.at(-1),'foreign backing inode');}}
else if(command==='losetup'){
 const s=read();if(args.includes('--find')){s.attached=true;s.backing=args.at(-1);save(s);emit('/dev/loop999\\n');}
 else if(args.includes('--detach')){s.attached=false;save(s);}
 else emit({loopdevices:s.attached?[{name:'/dev/loop999','back-file':s.backing,offset:0,sizelimit:0}]:[]});
}else if(command==='mount'){const s=read();s.mounted=true;s.store=args.at(-1);save(s);}
else if(command==='umount'){if(options.busyUnmount){process.stderr.write('target is busy');process.exit(32);}const s=read();s.mounted=false;save(s);}
else if(command==='findmnt'){const s=read();if(!s.mounted)process.exit(1);emit({filesystems:[{target:s.store,source:'/dev/loop999',fstype:'ext4',options:'rw,nodev,nosuid,nodiscard'}]});}
else if(command==='chown'||command==='sync'){}
else if(command==='node'){
 const [script,action,...a]=args;
 if(action==='admit'){const plan=JSON.parse(process.env.NATIVE_DIAGNOSTIC_PLAN);if(plan.source_revision!==a[0]||plan.work_end_ms!==Number(a[1])||plan.whole_end_ms!==Number(a[2]))throw Error('fixture dispatch mismatch');fs.writeFileSync(a[3],JSON.stringify(plan),{flag:'wx'});}
 else if(action==='command'){
  // Explicit fake owner transport. Production uses sdk-command-owner's private
  // cancellation FDs, strict Python admission, subreaper and exhaustion proof.
  const r=JSON.parse(fs.readFileSync(a[0])),cap=r.log_path.endsWith('/cleanup.log')?16777216:268435456;
  const capture=fs.openSync(r.capture_path,'wx'),child=cp.spawn(r.argv[0],r.argv.slice(1),{stdio:['ignore','pipe','pipe'],env:process.env});
  let captured=0,stderr='',reason=null,total=fs.existsSync(r.log_path)?fs.statSync(r.log_path).size:0,done=false;
  function cancel(why){reason=reason||why;if(child.exitCode===null&&child.signalCode===null)child.kill('SIGTERM');}
  function output(bytes,isError){
   if(total+bytes.length>cap){cancel('command-log-limit');return;}
   fs.appendFileSync(r.log_path,bytes);total+=bytes.length;
   if(isError)stderr=(stderr+bytes.toString()).slice(-4096);
   else{const left=1048576-captured;if(left>0){const part=bytes.subarray(0,left);fs.writeSync(capture,part);captured+=part.length;}if(bytes.length>left&&!r.stream_output)cancel('command-capture-limit');}
  }
  child.stdout.on('data',b=>output(b,false));child.stderr.on('data',b=>output(b,true));
  const end=Math.min(r.deadline_ms,r.maximum_ms?Date.now()+r.maximum_ms:r.deadline_ms);
  const timer=setInterval(()=>{
   if(Date.now()>=end)cancel('deadline');
   if(fs.existsSync(r.cancel_file)){
    const c=JSON.parse(fs.readFileSync(r.cancel_file));
    if(c.schema!=='prismpm/native-resource-cancel/1'||c.command_id!==r.command_id||c.source_revision!==r.source_revision||c.run_id!==r.run_id||c.run_attempt!==r.run_attempt)cancel('invalid-cancel');else cancel(c.reason);
   }
  },10);
  child.on('error',()=>cancel('fake-transport-error'));
  child.on('close',(status,signal)=>{
   if(done)return;done=true;clearInterval(timer);fs.closeSync(capture);
   const unproven=(options.unprovenRetirement&&r.argv[0]==='node'&&r.argv[2]==='build-set')||(options.unprovenWatch&&r.command_id==='watch-production');
   const value={status,signal,reason,stderr,command_id:r.command_id,source_revision:r.source_revision,run_id:r.run_id,run_attempt:r.run_attempt,retirement:{descendants_absent:!unproven,uncertainty:unproven?'fixture-unproven':null}};
   fs.writeFileSync(r.result_path,JSON.stringify(value),{flag:'wx'});process.exit(reason||unproven?125:status??125);
  });
 }else if(action==='production-plan')emit({builder:a[2],platform:'linux/'+a[1],builds:[{target:'runtime',args:['--no-cache','--output','type=oci,tar=false,dest='+a[3]+'/runtime.oci']},{target:'cli-archive',args:['--output','type=local,dest='+a[3]+'/cli']}]});
 else if(action==='metadata'||action==='archive'){assertInsideStore(a[0]);if(!fs.existsSync(a[0]))throw Error('missing raw production export');fs.writeFileSync(a[3],JSON.stringify({source_revision:sha,arch:a[1]}));}
 else if(action==='verify'){
  for(const p of [a[3],a[4]]){assertInsideStore(p);if(!fs.existsSync(p))throw Error('raw pair disposed before final verifier');}
  for(const arch of ['amd64','arm64'])if(!fs.existsSync(path.join(a[0],'production-'+arch+'.json')))throw Error('missing genuine sibling record');
  fs.writeFileSync(a[5],JSON.stringify({diagnostic:true,verified:true}));
 }else if(action==='build-set'){
  if(options.buildFailure)process.exit(43);
  if(options.hangBuild){fs.writeFileSync(path.join(root,'sdk-started'),JSON.stringify({pid:process.pid}));Atomics.wait(new Int32Array(new SharedArrayBuffer(4)),0,0,600000);process.exit(89);}
  const plan=JSON.parse(a[2]);for(const row of plan.builds)exportDirectory(row.args[row.args.indexOf('--output')+1]);
 }else if(action==='build'){
  if(options.buildFailure)process.exit(43);exportDirectory(a[a.indexOf('--output')+1]);
 }else throw Error('unexpected node helper action '+action);
}else if(command==='docker'){
 if(args[0]!=='--host'||args[1]!=='unix:///var/run/docker.sock')throw Error('unbounded docker endpoint');
 const a=args.slice(2),s=read();
 if(a[0]==='buildx'&&a[1]==='version')emit('github.com/docker/buildx v0.28.0 unit\\n');
 else if(a[0]==='pull'){}
 else if(a[0]==='image'&&a[1]==='inspect')emit([{Id:'sha256:'+'1'.repeat(64),RepoDigests:[a[2]],Architecture:'amd64',Os:'linux',Config:{}}]);
 else if(a[0]==='info')emit(path.join(root,'daemon')+'\\n');
 else if(a[0]==='container'&&a[1]==='ls'){
  const filter=a[a.indexOf('--filter')+1];for(const c of Object.values(s.containers))if(filter==='id='+c.Id||filter==='name=^/'+c.Name.slice(1)+'$')emit(c.Id+'\\n');
 }else if(a[0]==='volume'){}
 else if(a[0]==='create'){
  const get=o=>a[a.indexOf(o)+1],name=get('--name'),cid=String(++s.counter).padStart(64,'0'),directory=path.dirname(get('--cidfile'));
  const labels={'org.prismpm.native-resource-owner':get('--label').split('=')[1]};
  const c={Id:cid,Name:'/'+name,Image:a.at(-1),Config:{Labels:labels,Image:a.at(-1)},HostConfig:{NetworkMode:'bridge',IpcMode:'private',PidMode:'',CgroupParent:'',Privileged:true,ReadonlyRootfs:true,Init:true,NanoCpus:2000000000,Memory:Number(get('--memory')),MemorySwap:Number(get('--memory-swap')),PidsLimit:4096,LogConfig:{Type:'local',Config:{'max-size':'16m','max-file':'2'}},Tmpfs:{'/tmp':'rw,noexec,nosuid,nodev,size=268435456'}},Mounts:[{Type:'bind',Source:directory+'/state',Destination:'/var/lib/buildkit',RW:true},{Type:'bind',Source:directory+'/run',Destination:'/run',RW:true}]};
  s.containers[cid]=c;save(s);fs.writeFileSync(get('--cidfile'),cid+'\\n');emit(cid+'\\n');
 }else if(a[0]==='inspect'){
  const c=s.containers[a.at(-1)];if(!c)process.exit(1);
  if(options.replacedOwner&&c.Name.includes('-production-production0'))c.Config.Labels['org.prismpm.native-resource-owner']='foreign-owner';emit(a.includes('--format')?{Id:c.Id,Name:c.Name,Image:c.Image,Config:{Image:c.Config.Image,Labels:c.Config.Labels},HostConfig:c.HostConfig,Mounts:c.Mounts}:[c]);
 }else if(a[0]==='start'||a[0]==='stop'){}
 else if(a[0]==='rm'){delete s.containers[a[1]];save(s);}
 else if(a[0]==='exec')emit('buildkitd github.com/moby/buildkit v0.26.2 unit\\n');
 else if(a[0]==='buildx'&&['create','inspect','rm'].includes(a[1])){}
 else if(a[0]==='buildx'&&a[1]==='build'){
  const value=a[a.indexOf('--output')+1];exportDirectory(value);const dest=value.split('dest=')[1].split(',')[0];fs.writeFileSync(path.join(dest,'proof'),'bounded-native-resource-preflight\\n');
 }else throw Error('unexpected Docker operation '+JSON.stringify(a));
}else throw Error('unexpected fake transport '+command);
function assertInsideStore(p){if(!p.includes('/store/'))throw Error('mutable workload escaped bounded store');}
function exportDirectory(value){const dest=value.split('dest=')[1].split(',')[0];assertInsideStore(dest);fs.mkdirSync(dest,{recursive:true});fs.writeFileSync(path.join(dest,'index.json'),'{}');}
`;
  for (const name of ['sudo','python3','stat','git','uname','df','fallocate','mkfs.ext4','losetup','mount','umount','findmnt','chown','sync','node','docker']) writeFileSync(join(root, 'bin', name), program, {mode: 0o755});
  const output=join(root,'evidence');
  const invocation = (phase='production', window=19800000) => {
    const now=Date.now();
    const env={...process.env,PATH:join(root,'bin')+':'+process.env.PATH,DOCKER_CONFIG:join(root,'credential-config'),RUNNER_TEMP:join(root,'temporary'),GITHUB_ACTIONS:'true',GITHUB_REPOSITORY:'UOR-Foundation/PrismPM',GITHUB_EVENT_NAME:'workflow_dispatch',GITHUB_REF:'refs/heads/fix/521-release-build-sets',GITHUB_SHA:sha,GITHUB_WORKFLOW_SHA:sha,GITHUB_RUN_ID:'123',GITHUB_RUN_ATTEMPT:'2',DIAGNOSTIC_SOURCE:sha,DISPATCH_VERSION:'0.4.0',REHEARSAL:'true',PUBLISH_CRATES:'false'};
    env.NATIVE_DIAGNOSTIC_PLAN=JSON.stringify(diagnosticPolicy(env,now+window-19800000));
    return {args:[join(root,'source/scripts/native-resource-diagnostic.sh'),phase,'amd64',sha,String(now+window),String(now+window+1800000),output,...(phase==='pair'?[join(root,'expected')]:[])],env};
  };
  const run = (phase='production',window) => {const value=invocation(phase,window);return spawnSync('bash',value.args,{env:value.env,encoding:'utf8',timeout:60000,maxBuffer:1024*1024});};
  const start = () => {const value=invocation();return spawn('bash',value.args,{env:value.env,stdio:['ignore','pipe','pipe']});};
  return {root,output,run,start,commands:()=>existsSync(record)?readFileSync(record,'utf8').trim().split('\n').map(JSON.parse):[],state:()=>JSON.parse(readFileSync(state))};
}
const docker = f => f.commands().filter(row=>row.command==='docker').map(row=>row.args.slice(2));
const successful = (f,result) => {
  assert.equal(result.error,undefined);assert.equal(result.signal,null);assert.equal(result.status,0,result.stderr);
  const receipt=JSON.parse(readFileSync(join(f.output,'result.json')));assert.equal(receipt.complete,true);assert.equal(receipt.cleanup_complete,true);
  assert.equal(f.state().mounted,false);assert.equal(f.state().attached,false);assert.deepEqual(f.state().containers,{});
};

test('production lifecycle records exact caps, one cold two-target buildset, readbacks, and complete owned retirement',t=>{
  const f=fixture(t);successful(f,f.run());const calls=f.commands(),d=docker(f);
  const allocation=calls.find(row=>row.command==='fallocate'&&!row.args.at(-1).includes('probe'));
  assert.deepEqual(allocation.args.slice(0,3),['--length','42949672960','--']);
  assert(calls.some(row=>row.command==='mkfs.ext4'&&row.args.includes('nodiscard,lazy_itable_init=0,lazy_journal_init=0')));
  assert(calls.some(row=>row.command==='mount'&&row.args.includes('nodiscard,nodev,nosuid')));
  const containers=d.filter(a=>a[0]==='create');assert.equal(containers.length,2);
  for(const args of containers){for(const flag of ['--pull=never','--privileged','--init','--read-only'])assert(args.includes(flag));assert.equal(args[args.indexOf('--cpus')+1],'2');assert.equal(args[args.indexOf('--pids-limit')+1],'4096');}
  const planPaths=readdirSync(f.output).filter(name=>name.endsWith('-plan.json')&&name!=='dispatch-plan.json').map(name=>join(f.output,name));
  execFileSync(actualPython,['-I','-B','-c',`import importlib.util,json,sys;spec=importlib.util.spec_from_file_location('resource_monitor',sys.argv[1]);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);[m.validate_plan(json.load(open(path))) for path in sys.argv[2:]]`,new URL('./native-resource-monitor.py',import.meta.url).pathname,...planPaths]);
  const builds=calls.filter(row=>row.command==='node'&&row.args[1]==='build-set');assert.equal(builds.length,1);
  const plan=JSON.parse(builds[0].args.at(-1));assert.deepEqual(plan.builds.map(row=>row.target),['runtime','cli-archive']);assert(plan.builds[0].args.includes('--no-cache'));
  assert.equal(calls.filter(row=>row.command==='git'&&row.args.includes('clone')).length,1);
  assert(calls.filter(row=>row.command==='git'&&row.args.includes('clone')).every(row=>row.args.includes('--no-hardlinks')));
  assert(f.commands().filter(row=>row.command==='docker').every(row=>row.dockerConfig===join(f.root,'credential-config')));
  assert.equal(d.filter(a=>a[0]==='rm').length,2);assert.equal(d.filter(a=>a[0]==='buildx'&&a[1]==='rm'&&a.includes('--keep-state')).length,2);
  assert(calls.findIndex(r=>r.command==='node'&&r.args[1]==='metadata')<calls.findLastIndex(r=>r.command==='docker'&&r.args[2]==='rm'));
});

test('pair uses distinct cold roots/builders and retains both raw directories through actual verifier invocation',t=>{
  const f=fixture(t);successful(f,f.run('pair'));const calls=f.commands();
  const clones=calls.filter(row=>row.command==='git'&&row.args.includes('clone'));assert.equal(clones.length,2);assert.notEqual(clones[0].args.at(-1),clones[1].args.at(-1));
  const builds=calls.filter(row=>row.command==='node'&&row.args[1]==='build');assert.equal(builds.length,2);
  const builders=builds.map(row=>row.args[row.args.indexOf('--builder')+1]);assert.equal(new Set(builders).size,2);
  for(const row of builds){assert(row.args.includes('--no-cache'));assert(row.args[row.args.indexOf('--output')+1].includes('type=oci,tar=false,dest='));assert(!row.args.some(a=>a.startsWith('--cache')));}
  assert.equal(calls.filter(row=>row.command==='node'&&row.args[1]==='verify').length,1);
  assert.equal(docker(f).filter(a=>a[0]==='rm').length,3);
});

for(const [name,options] of [['backing reserve',{lowSpace:true}],['formatted backing holes',{holes:true}],['unexpected exhaustion success',{quotaUnexpectedSuccess:true}],['SDK transport failure',{buildFailure:true}],['monitor crash',{monitorDies:true}],['stale monitor samples',{staleMonitor:true}],['replaced backing inode',{replacedBacking:true}],['dirty source',{dirty:true}],['kernel namespace drift across bootstrap',{namespaceDrift:true}]])
 test(name+' fails closed and never leaves a claimed complete result',t=>{
  const f=fixture(t,options),result=f.run();assert.equal(result.error,undefined);assert.notEqual(result.status,0);
  const receipt=JSON.parse(readFileSync(join(f.output,'result.json')));assert.equal(receipt.complete,false);
  if(options.lowSpace)assert(!f.commands().some(row=>row.command==='fallocate'));
  if(options.holes||options.quotaUnexpectedSuccess)assert(!f.commands().some(row=>row.command==='node'&&row.args[1]==='build-set'));
  if(options.buildFailure)assert.equal(receipt.cleanup_complete,true);
  if(options.replacedBacking){const path=f.commands().find(row=>row.command==='mkfs.ext4').args.at(-1);assert(existsSync(path));assert(existsSync(path+'.original'));assert.equal(readFileSync(path,'utf8'),'foreign backing inode');assert.equal(receipt.cleanup_complete,false);}
 });

test('changed owner label refuses container termination and filesystem disposal rather than cleaning an unowned container',t=>{
  const f=fixture(t,{replacedOwner:true}),result=f.run();assert.equal(result.error,undefined);assert.notEqual(result.status,0);
  const receipt=JSON.parse(readFileSync(join(f.output,'result.json')));assert.equal(receipt.complete,false);assert.equal(receipt.cleanup_complete,false);
  const foreign=Object.values(f.state().containers).find(c=>c.Name.includes('-production-production0'));assert(foreign);
  const removals=docker(f).filter(a=>['rm','stop'].includes(a[0]));assert(!removals.some(a=>a.includes(foreign.Id)));
  assert.equal(f.state().mounted,true);assert.equal(f.state().attached,true);
});

test('busy normal unmount is an incomplete cleanup and never detaches or deletes its backing store',t=>{
  const f=fixture(t,{busyUnmount:true}),result=f.run();assert.equal(result.error,undefined);assert.notEqual(result.status,0);
  const receipt=JSON.parse(readFileSync(join(f.output,'result.json')));assert.equal(receipt.cleanup_complete,false);
  assert(f.state().attached);assert(f.state().mounted);assert(!f.commands().some(row=>row.command==='losetup'&&row.args.includes('--detach')));
});


test('one inherited work deadline stops work before SDK construction and still allows bounded owned cleanup',t=>{
 const f=fixture(t),result=f.run('production',2000);assert.equal(result.error,undefined);assert.notEqual(result.status,0);
 const receipt=JSON.parse(readFileSync(join(f.output,'result.json')));assert.equal(receipt.complete,false);
 assert(!f.commands().some(row=>row.command==='node'&&['build','build-set'].includes(row.args[1])));
 assert(!docker(f).some(args=>args.includes('prune')));
});

test('SIGTERM stops only the supervised fake build and exact proven CID, then retires its owned filesystem',async t=>{
 const f=fixture(t,{hangBuild:true}),child=f.start();let stderr='';child.stderr.on('data',value=>{stderr+=value;});child.stdout.resume();
 const marker=join(f.root,'sdk-started');
 for(let i=0;i<600&&!existsSync(marker)&&child.exitCode===null;i++)await delay(50);
 if(!existsSync(marker)){child.kill('SIGTERM');await once(child,'exit');assert.fail('fake SDK transport did not start: '+stderr);}
 const fakePid=JSON.parse(readFileSync(marker)).pid;
 const done=once(child,'exit');child.kill('SIGTERM');const [status,signal]=await done;
 assert.equal(signal,null);assert.equal(status,1,stderr);
 const receipt=JSON.parse(readFileSync(join(f.output,'result.json')));assert.equal(receipt.complete,false);assert.equal(receipt.cleanup_complete,true);
 assert.deepEqual(f.state().containers,{});assert.equal(f.state().mounted,false);assert.equal(f.state().attached,false);
 assert(!f.commands().some(row=>row.command==='node'&&row.args[1]==='metadata'));
 assert.throws(()=>process.kill(fakePid,0),error=>error.code==='ESRCH','the explicit fake controller retired its fake child');
 const cancellations=readdirSync(join(f.output,'.control')).filter(name=>name.endsWith('.cancel.json'));assert(cancellations.length>0);
 assert(cancellations.map(name=>JSON.parse(readFileSync(join(f.output,'.control',name)))).some(value=>value.reason==='interrupted'));
});


test('unproven command retirement is incomplete and prevents disposal despite a successful fake leader',t=>{
 const f=fixture(t,{unprovenRetirement:true}),result=f.run();assert.equal(result.error,undefined);assert.notEqual(result.status,0);
 const receipt=JSON.parse(readFileSync(join(f.output,'result.json')));assert.equal(receipt.complete,false);assert.equal(receipt.cleanup_complete,false);
 assert.equal(f.state().mounted,true);assert.equal(f.state().attached,true);
 assert(!f.commands().some(row=>row.command==='node'&&row.args[1]==='metadata'));
});


test('unproven watcher descendant retirement preserves the store and cannot claim complete disposal',t=>{
 const f=fixture(t,{unprovenWatch:true}),result=f.run();assert.equal(result.error,undefined);assert.notEqual(result.status,0);
 const receipt=JSON.parse(readFileSync(join(f.output,'result.json')));assert.equal(receipt.complete,false);assert.equal(receipt.cleanup_complete,false);
 assert.equal(f.state().mounted,true);assert.equal(f.state().attached,true);
 assert(Object.values(f.state().containers).some(c=>c.Name.includes('-production-production0')));
 const requests=readdirSync(join(f.output,'.control')).filter(name=>name.endsWith('.request.json')).map(name=>JSON.parse(readFileSync(join(f.output,'.control',name))));
 assert.equal(new Set(requests.map(value=>value.command_id)).size,requests.length);
 assert(requests.filter(value=>value.command_id.startsWith('watch-')).every(value=>value.log_path===join(f.output,'cleanup.log')));
});

test('driver delegates ownership and never introduces numeric signal or cache/prune fallbacks',()=>{
 const text=readFileSync(runner,'utf8');assert(!/killpg|subprocess|process\.poll\(|kill\s+-(?:TERM|KILL|INT|HUP|[1-9])/.test(text));
 assert(text.includes('node "$helper" command "$request"'));assert(!text.includes('DOCKER_CONFIG='));
});


test('nonempty original BuildKit cgroup prevents container removal and state/filesystem disposal after stop',t=>{
 const f=fixture(t,{populatedCgroup:true}),result=f.run();assert.equal(result.error,undefined);assert.notEqual(result.status,0);
 const receipt=JSON.parse(readFileSync(join(f.output,'result.json')));assert.equal(receipt.complete,false);assert.equal(receipt.cleanup_complete,false);
 const production=Object.values(f.state().containers).find(c=>c.Name.includes('-production-production0'));assert(production);
 assert(!docker(f).some(args=>args[0]==='rm'&&args[1]===production.Id));
 assert.equal(f.state().mounted,true);assert.equal(f.state().attached,true);
 assert(readdirSync(f.output).some(name=>name.includes('-retirement-')));
});


test('last-tick terminal OOM fails the diagnostic after still completing proven owner and filesystem cleanup',t=>{
 const f=fixture(t,{terminalOOM:true}),result=f.run();assert.equal(result.error,undefined);assert.notEqual(result.status,0);
 const receipt=JSON.parse(readFileSync(join(f.output,'result.json')));assert.equal(receipt.complete,false);assert.equal(receipt.resource_failure,true);assert.equal(receipt.cleanup_complete,true);
 assert.deepEqual(f.state().containers,{});assert.equal(f.state().mounted,false);assert.equal(f.state().attached,false);
});


test('authoritatively absent stopped cgroup permits cleanup with explicitly unavailable optional terminal counters',t=>{
 const f=fixture(t,{absentCgroup:true});successful(f,f.run());
 const reports=readdirSync(f.output).filter(name=>name.includes('-retirement-')).map(name=>JSON.parse(readFileSync(join(f.output,name))));
 assert(reports.length>=2);assert(reports.every(value=>value.original_cgroup.state==='absent'&&value.resource_observation_complete===false));
 assert(reports.every(value=>value.owner_oom_killed===false&&value.resource_failure===false));
});
