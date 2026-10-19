"""HELD native hosted CA mount-lifecycle diagnostic; not installed SDK qualification."""
import argparse,base64,datetime,hashlib,json,os,re,secrets,selectors,signal,stat,subprocess,time
from pathlib import Path
SOURCE='4dcb7e325fff0e737ae4e02d2b36b3e86cd89176'
TREE='97b2e7bda856f90b5742cd01418f4503c0cd2dc4'
DIND='docker.io/library/docker@sha256:2ceb471176ad51e37145d43ce7cbf0fa5d644a2b185bd537f0ef695fb3a37497'
TRUST_SHA='663d7eb42625cb3087d6069b436a0e75408f17280182c98f9e09d76e03fa8999'
DOCKERFILE_SHA='0b889c92534dd05c49041a68b7405a4a4ed225c1171af2fc625f8c8c5b3d9094'
LABEL='io.lexlean-matmul.hosted-ca-mount-diagnostic'
CA='/run/prismpm-qualification-ca.crt'
NODE="import fs from 'node:fs';\nimport crypto from 'node:crypto';\nimport {spawnSync} from 'node:child_process';\nconst ca=process.env.PRISMPM_DIAGNOSTIC_CA_PATH;\nif(!['/run/prismpm-qualification-ca.crt','/etc/prismpm-qualification-ca.crt'].includes(ca))throw new Error('closed diagnostic CA path required');\nconst source='/probe/sdk-qualification-trust.mjs';\nconst hash=b=>crypto.createHash('sha256').update(b).digest('hex');\nconst err=e=>({name:e.name,code:e.code??null,message:String(e.message).slice(0,2048)});\nconst metadata=s=>({isFile:s.isFile(),isDirectory:s.isDirectory(),isSymbolicLink:s.isSymbolicLink(),...Object.fromEntries(['nlink','size','mode','uid','gid','dev','ino','mtimeNs','ctimeNs'].map(k=>[k,String(s[k])]))});\nconst pathname=p=>{const r={path:p};for(const [key,fn]of [['lstat',()=>metadata(fs.lstatSync(p,{bigint:true}))],['readlink',()=>fs.readlinkSync(p)],['realpath',()=>fs.realpathSync(p)]])try{r[key]=fn();}catch(e){r[key+'_error']=err(e);}return r;};\nconst r={schema:'matmul/CA-socket-mount-layout-observation/1',qualification:false,node:process.version,uid:process.getuid(),gid:process.getgid(),groups:process.getgroups(),layout:['/run','/var/run'].map(pathname),ca_path_before:pathname(ca),source_sha256:hash(fs.readFileSync(source)),NODE_EXTRA_CA_CERTS:process.env.NODE_EXTRA_CA_CERTS??null};\nlet fd;\ntry{\n fd=fs.openSync(ca,fs.constants.O_RDONLY|fs.constants.O_NONBLOCK|fs.constants.O_NOFOLLOW);\n const before=fs.fstatSync(fd,{bigint:true});r.fd_before=metadata(before);\n r.original_compound_predicates={regular:before.isFile(),single_link:before.nlink===1n,positive:before.size>0n,at_most65536:before.size<=65536n};\n r.original_compound_predicate=Object.values(r.original_compound_predicates).every(Boolean);\n r.readonly_mode=(before.mode&0o222n)===0n;\n const chunks=[];let n=0;\n try{while(n<=65536){const b=Buffer.alloc(Math.min(4096,65537-n));const count=fs.readSync(fd,b,0,b.length,null);if(count===0)break;chunks.push(b.subarray(0,count));n+=count;}const b=Buffer.concat(chunks);r.content={bytes:b.length,sha256:hash(b),complete:n<=65536};}catch(e){r.content_error=err(e);}\n r.fd_after=metadata(fs.fstatSync(fd,{bigint:true}));r.ca_lstat_after_read=metadata(fs.lstatSync(ca,{bigint:true}));\n}catch(e){r.open_or_stat_error=err(e);}finally{if(fd!==undefined)fs.closeSync(fd);}\nr.ca_path_after=pathname(ca);\n// Execute the exact original CLI, not a replacement predicate or product owner.\nconst argv=[source,ca];const v=spawnSync('/usr/local/bin/node',argv,{env:process.env,encoding:null,maxBuffer:131072,timeout:10000});\nconst stdout=v.stdout??Buffer.alloc(0),stderr=v.stderr??Buffer.alloc(0);\nr.original_validator={program:'/usr/local/bin/node',argv,status:v.status,signal:v.signal,error:v.error?err(v.error):null,stdout:{bytes:stdout.length,sha256:hash(stdout),base64:stdout.toString('base64')},stderr:{bytes:stderr.length,sha256:hash(stderr),base64:stderr.toString('base64')}};\nr.complete=v.error===undefined&&v.signal===null&&[0,1].includes(v.status);\nconsole.log(JSON.stringify(r));process.exitCode=r.complete?0:90;\n"
DIND_STAT=r'''for p in /run /var/run /run/prismpm-qualification-ca.crt /etc/prismpm-qualification-ca.crt /etc/docker/certs.d/172.29.88.1:5000/ca.crt; do
printf 'PATH=%s\n' "$p"
ls -ld "$p"; printf 'lstat_status=%s\n' "$?"
readlink "$p"; printf 'readlink_status=%s\n' "$?"
readlink -f "$p"; printf 'realpath_status=%s\n' "$?"
stat -c 'kind=%F mode=%f nlink=%h size=%s uid=%u gid=%g dev=%d ino=%i mtime=%Y ctime=%Z' "$p"; printf 'stat_status=%s\n' "$?"
done
for p in /run/prismpm-qualification-ca.crt /etc/prismpm-qualification-ca.crt /etc/docker/certs.d/172.29.88.1:5000/ca.crt; do
if test -f "$p" && test "$(stat -c %s "$p")" -le 65536; then sha256sum "$p"; printf 'body_hash_status=%s\n' "$?"; else printf 'body_unavailable=1\n'; fi
done
'''
sha=lambda b:hashlib.sha256(b).hexdigest()
def ref(p):
 b=Path(p).read_bytes();return dict(path=str(p),bytes=len(b),sha256=sha(b))
def write(p,value):
 with Path(p).open('x',encoding='utf-8')as f:json.dump(value,f,indent=2);f.write('\n')
def host_ca(p):
 r=dict(path=str(p))
 try:r['realpath']=str(p.resolve(strict=True))
 except OSError as e:r['realpath_error']=dict(type=type(e).__name__,errno=e.errno)
 try:
  s=p.lstat();r['stat']={k:getattr(s,k)for k in ['st_mode','st_nlink','st_uid','st_gid','st_dev','st_ino','st_size','st_mtime_ns','st_ctime_ns']};r['regular']=stat.S_ISREG(s.st_mode);r['directory']=stat.S_ISDIR(s.st_mode)
  if r['regular']and s.st_size<=65536:
   with p.open('rb')as f:b=f.read(65537)
   r.update(bytes=len(b),sha256=sha(b),complete=len(b)<=65536)
   if len(b)>65536:r['body_refusal']='bounded-read-exceeded65536'
  else:r.update(complete=False,body_refusal='not-regular-or-observed-size-over65536')
 except OSError as e:r['path_error']=dict(type=type(e).__name__,errno=e.errno)
 return r
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--execute',action='store_true');p.add_argument('--source',type=Path,required=True);p.add_argument('--architecture',choices=['amd64','arm64'],required=True);p.add_argument('--expected-script-sha256',required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--inner',action='store_true');p.add_argument('--tools-image');p.add_argument('--dind-image');p.add_argument('--whole-end-utc-ms',type=int);a=p.parse_args()
 assert a.execute and sha(Path(__file__).read_bytes())==a.expected_script_sha256
 assert os.environ.get('GITHUB_ACTIONS')=='true'and os.environ.get('RUNNER_ENVIRONMENT')=='github-hosted'and os.environ.get('GITHUB_REPOSITORY')=='UOR-Foundation/PrismPM'
 assert os.environ.get('RUNNER_OS')=='Linux'and os.environ.get('RUNNER_ARCH')==('X64'if a.architecture=='amd64'else'ARM64')and os.getuid()==os.getgid()==1001
 source=a.source.resolve();assert source==a.source and source.name=='root-a';runner_temp=Path(os.environ['RUNNER_TEMP']);assert runner_temp.is_absolute()and runner_temp.resolve()==runner_temp;trust=source/'scripts/sdk-qualification-trust.mjs';assert sha(trust.read_bytes())==TRUST_SHA and sha((source/'sdk/Dockerfile').read_bytes())==DOCKERFILE_SHA
 assert subprocess.check_output(['git','-C',str(source),'rev-parse','HEAD']).decode().strip()==SOURCE and subprocess.check_output(['git','-C',str(source),'rev-parse','HEAD^{tree}']).decode().strip()==TREE and subprocess.check_output(['git','-C',str(source),'status','--porcelain'])==b''
 assert a.output.is_absolute()and not os.path.lexists(a.output)and a.output.parent.resolve()==a.output.parent and source not in a.output.parents and not any(x in str(a.output)for x in [',','\n','\r'])
 assert runner_temp in a.output.parents
 assert Path(__file__).resolve().is_relative_to(source.parent)
 a.output.mkdir(mode=0o700);private=a.output/'private';private.mkdir(mode=0o700);ca=private/'ca.crt';key=private/'ca.key';config=a.output/'empty-docker-config';config.mkdir(mode=0o700);(config/'config.json').write_text('{"auths":{}}\n')
 session='matmul-hosted-ca-'+secrets.token_hex(12);tag='matmul-ca-diagnostic-tools:'+session;iidfile=a.output/'tools-image.id';begin=time.monotonic();end=begin+1800
 if a.inner:
  assert re.fullmatch('sha256:[0-9a-f]{64}',a.tools_image or '')and re.fullmatch('sha256:[0-9a-f]{64}',a.dind_image or '')and a.whole_end_utc_ms is not None
  remaining=a.whole_end_utc_ms/1000-time.time();assert 180<remaining<=1800;end=begin+remaining
 else:assert a.tools_image is a.dind_image is a.whole_end_utc_ms is None
 work_end=end-180;seq=0;captured_bytes=0;commands=[];resources=[];retired=[];variants=[];tools=None;primary=None;tag_removed=False;inner_result=None;outer_result=None;target=source/'target';target_initial=None
 env={'PATH':os.environ.get('PATH','/usr/local/bin:/usr/bin:/bin'),'LANG':'C','LC_ALL':'C'}
 def cap(label,argv,seconds=20,acceptable=(0,),clean=False):
  nonlocal seq,captured_bytes
  remain=(end if clean else work_end)-time.monotonic();assert remain>5;bound=min(seconds,remain-5);seq+=1;stem=f'{seq:03d}-{label}';op=a.output/(stem+'.stdout');ep=a.output/(stem+'.stderr');started=time.monotonic();timed=False;overflow=False;limit=8*1024**2
  with op.open('xb')as so,ep.open('xb')as se:
   child=subprocess.Popen(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,stdin=subprocess.DEVNULL,env=env,start_new_session=True)
   with selectors.DefaultSelector()as sel:
    sel.register(child.stdout,selectors.EVENT_READ,[so,0]);sel.register(child.stderr,selectors.EVENT_READ,[se,0])
    while sel.get_map():
     if time.monotonic()-started>bound:timed=True;break
     for item,_ in sel.select(.1):
      chunk=os.read(item.fileobj.fileno(),65536)
      if not chunk:sel.unregister(item.fileobj);continue
      room=min(limit+1-item.data[1],64*1024**2+1-captured_bytes);item.data[0].write(chunk[:max(0,room)]);item.data[1]+=len(chunk);captured_bytes+=min(len(chunk),max(0,room))
      if item.data[1]>limit or captured_bytes>64*1024**2:overflow=True;break
     if overflow:break
   if timed or overflow:os.killpg(child.pid,signal.SIGTERM)
   try:status=child.wait(timeout=3)
   except subprocess.TimeoutExpired:os.killpg(child.pid,signal.SIGKILL);status=child.wait(timeout=2)
  row=dict(argv=argv,status=status,timed_out=timed,overflow=overflow,elapsed_seconds=time.monotonic()-started,timeout_seconds=bound,stdout=ref(op),stderr=ref(ep));write(a.output/(stem+'.json'),row);commands.append(row)
  assert not timed and not overflow,'bounded command incomplete'
  assert acceptable is None or status in acceptable,label+' refused'
  return row,op.read_bytes(),ep.read_bytes()
 def docker(label,args,**kw):return cap(label,['docker','--host','unix:///var/run/docker.sock','--config',str(config),*args],**kw)
 def inspect(kind,id,clean=False):
  _,raw,_=docker('inspect-'+kind,[kind,'inspect',id],clean=clean);rows=json.loads(raw);assert len(rows)==1;return rows[0]
 def absent(kind,id,clean=False):
  row,raw,err=docker('absence-'+kind,[kind,'inspect',id],acceptable=(1,),clean=clean);assert json.loads(raw)==[]and re.search(b'No such|not found|does not exist',err,re.I);return row
 def owned(r,x):
  if r['kind']=='volume':assert x['Name']==r['name']and x['Driver']=='local'and x['Labels'].get(LABEL)==session and x.get('Options')in(None,{})
  else:assert x['Id']==r['id']and x['Name']=='/'+r['name']and x['Image']==r['image']and x['Config']['Labels'].get(LABEL)==session
  if r.get('initial'):
   before=r['initial']
   for k in (['Name','Driver','Labels','Options','Mountpoint']if r['kind']=='volume'else['Id','Name','Created','Image','Config','Path','Args']):assert x.get(k)==before.get(k)
   if r['kind']=='container':
    hc=json.loads(json.dumps(x['HostConfig']))
    if before['HostConfig'].get('OomKillDisable')is False and 'OomKillDisable'in hc and hc['OomKillDisable']is None:hc['OomKillDisable']=False
    assert hc==before['HostConfig']and sorted(x['Mounts'],key=lambda m:m['Destination'])==sorted(before['Mounts'],key=lambda m:m['Destination'])and x['State']['OOMKilled']is False
 def create(kind,role,args,image=None):
  name=session+'-'+role;absent(kind,name);r=dict(kind=kind,name=name,id=None,image=image,initial=None,retired=False);resources.append(r)
  _,raw,_=docker('create-'+role,args(name));id=raw.decode().strip();assert(id==name if kind=='volume'else re.fullmatch('[0-9a-f]{64}',id));r['id']=id;x=inspect(kind,id);owned(r,x);r['initial']=x;return r
 def retire(r):
  if r['retired']:return
  assert r['id']is not None,'unknownID: no name-only removal';x=inspect(r['kind'],r['id'],clean=True);owned(r,x)
  if r['kind']=='container'and x['State']['Running']:
   docker('normal-stop',['container','stop','--time','10',r['id']],seconds=20,clean=True);x=inspect('container',r['id'],clean=True);owned(r,x)
  if r['kind']=='container':assert not x['State']['Running']and x['State']['Pid']==0
  rm,raw,_=docker('normal-remove',[r['kind'],'rm',r['id']],seconds=120 if r['kind']=='volume'else 20,clean=True);assert raw.decode().strip()==r['id'];absence=[absent(r['kind'],id,clean=True)for id in dict.fromkeys([r['id'],r['name']])];r['retired']=True;retired.append(dict(kind=r['kind'],id=r['id'],name=r['name'],normal_remove=rm,absence=absence))
 def workspace_mounts():
  return ['--mount','type=bind,source='+str(source.parent)+',target='+str(source.parent),'--mount','type=bind,source='+str(source)+',target='+str(source)+',readonly','--mount','type=bind,source='+str(target)+',target='+str(target)]
 def workspace_graph():return [('bind',str(source.parent),str(source.parent),True),('bind',str(source),str(source),False),('bind',str(target),str(target),True)]
 def target_stat():
  t=target.lstat();assert stat.S_ISDIR(t.st_mode)and not target.is_symlink()and t.st_uid==t.st_gid==1001 and stat.S_IMODE(t.st_mode)==0o700
  return {k:getattr(t,k)for k in ['st_dev','st_ino','st_uid','st_gid','st_mode']}
 def source_clean():
  return subprocess.check_output(['git','-C',str(source),'rev-parse','HEAD']).decode().strip()==SOURCE and subprocess.check_output(['git','-C',str(source),'status','--porcelain'])==b''
 def tools_create(role,volume=None,socket_target=None,ca_target=CA):
  args=lambda name:['container','create','--pull=never','--name',name,'--label',LABEL+'='+session,'--init','--read-only','--network','none','--user','1001:1001','--group-add','0','--cap-drop','ALL','--security-opt','no-new-privileges','--cpus','2','--memory','1g','--memory-swap','1g','--pids-limit','256','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=1g,mode=1777',*workspace_mounts(),*(['--mount','type=volume,source='+volume['name']+',target='+socket_target]if volume else[]),'--mount','type=bind,source='+str(ca)+',target='+ca_target+',readonly','--mount','type=bind,source='+str(trust)+',target=/probe/sdk-qualification-trust.mjs,readonly','--env','NODE_EXTRA_CA_CERTS='+ca_target,'--env','PRISMPM_DIAGNOSTIC_CA_PATH='+ca_target,'--entrypoint','/usr/local/bin/node',tools,'--input-type=module','-e',NODE]
  owner=create('container',role,args,tools);initial=owner['initial'];hc=initial['HostConfig']
  assert initial['Config']['User']=='1001:1001'and hc['GroupAdd']==['0']and hc['ReadonlyRootfs']is True and hc['NetworkMode']=='none'and hc['Privileged']is False and hc['CapDrop']==['ALL']and'no-new-privileges'in hc['SecurityOpt']and hc['NanoCpus']==2000000000 and hc['Memory']==hc['MemorySwap']==1024**3 and hc['PidsLimit']==256
  expected=workspace_graph()+[('bind',str(ca),ca_target,False),('bind',str(trust),'/probe/sdk-qualification-trust.mjs',False)]
  actual=[(m['Type'],m.get('Name')if m['Type']=='volume'else m['Source'],m['Destination'],m['RW'])for m in initial['Mounts']]
  if volume:expected.append(('volume',volume['name'],socket_target,True))
  assert sorted(actual)==sorted(expected)
  return owner
 def observe_tools(owner):
  command,raw,_=docker('observe-tools',['container','start','--attach',owner['id']],seconds=30);observation=json.loads(raw);assert observation['schema']=='matmul/CA-socket-mount-layout-observation/1'and observation['uid']==observation['gid']==1001 and 0 in observation['groups']and observation['source_sha256']==TRUST_SHA and observation['node']=='v22.23.2'and observation['complete']is True
  terminal=inspect('container',owner['id']);owned(owner,terminal);assert terminal['State']['ExitCode']==0 and not terminal['State']['Running'];return dict(command=command,observation=observation,terminal=terminal)
 def dind_observe(daemon,label):return docker(label,['exec',daemon['id'],'/bin/sh','-c',DIND_STAT],acceptable=(0,))[0]
 def interrupted(signum,frame):raise InterruptedError('diagnostic interrupted')
 signal.signal(signal.SIGTERM,interrupted);signal.signal(signal.SIGINT,interrupted)
 try:
  docker('actual-daemon-from-current-namespace',['version','--format','{{json .Server}}'])
  if not a.inner:
   # This is only the original source-owned tools target, not SDK construction.
   docker('build-original-tools-stage',['buildx','build','--iidfile',str(iidfile),'--file',str(source/'sdk/Dockerfile'),'--target','registry_qualification_tools','--platform','linux/'+a.architecture,'--load','--tag',tag,str(source)],seconds=1200)
   tools=iidfile.read_text().strip();assert re.fullmatch('sha256:[0-9a-f]{64}',tools);_,raw,_=docker('tools-image',['image','inspect',tag]);factory=json.loads(raw)[0];assert factory['Id']==tools and factory['Architecture']==a.architecture and factory['Os']=='linux'and factory['Config'].get('Volumes')in(None,{})
   docker('pull-exact-dind',['pull','--platform','linux/'+a.architecture,DIND],seconds=180);_,raw,_=docker('exact-dind-image',['image','inspect',DIND]);dindmeta=json.loads(raw)[0];assert dindmeta['Architecture']==a.architecture and dindmeta['Os']=='linux'and dindmeta['Config'].get('Volumes')=={'/var/lib/docker':{}}and dindmeta['Config'].get('Entrypoint')==['dockerd-entrypoint.sh'];dind_id=dindmeta['Id'];assert re.fullmatch('sha256:[0-9a-f]{64}',dind_id)
   assert not os.path.lexists(target);target.mkdir(mode=0o700);assert subprocess.run(['git','-C',str(source),'check-ignore','--quiet','--','target/'],check=False).returncode==0
   target_initial=target_stat();socket_stat=Path('/var/run/docker.sock').stat();assert stat.S_ISSOCK(socket_stat.st_mode);socket_group=socket_stat.st_gid
   inherited=['GITHUB_ACTIONS','GITHUB_REPOSITORY','GITHUB_SHA','GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT','RUNNER_ENVIRONMENT','RUNNER_OS','RUNNER_ARCH','RUNNER_TEMP']
   outer_env=[v for k in inherited for v in ['--env',k+'='+os.environ[k]]]
   remaining=end-time.monotonic();assert remaining>330
   inner_end_ms=int((time.time()+remaining-120)*1000)
   self_path=str(Path(__file__).resolve());inner_path=a.output/'inner'
   outer=create('container','outer',lambda name:['container','create','--interactive','--init','--name',name,'--label',LABEL+'='+session,'--pull=never','--platform','linux/'+a.architecture,'--read-only','--user','1001:1001','--group-add',str(socket_group),'--cpus','2','--memory','1g','--memory-swap','1g','--pids-limit','256','--cap-drop','ALL','--security-opt','no-new-privileges','--tmpfs','/tmp:rw,exec,nosuid,nodev,size=1g,mode=1777',*workspace_mounts(),'--mount','type=bind,source='+str(runner_temp)+',target='+str(runner_temp),'--mount','type=bind,source=/var/run/docker.sock,target=/var/run/docker.sock','--workdir',str(source),*outer_env,'--entrypoint','/usr/bin/python3',tools,'-I','-B',self_path,'--execute','--inner','--source',str(source),'--architecture',a.architecture,'--expected-script-sha256',a.expected_script_sha256,'--tools-image',tools,'--dind-image',dind_id,'--whole-end-utc-ms',str(inner_end_ms),'--output',str(inner_path)],tools)
   oi=outer['initial'];oh=oi['HostConfig'];assert oi['Config']['User']=='1001:1001'and oh['GroupAdd']==[str(socket_group)]and oh['ReadonlyRootfs']is True and oh['Privileged']is False and oh['CapDrop']==['ALL']and'no-new-privileges'in oh['SecurityOpt']and oh['NanoCpus']==2000000000 and oh['Memory']==oh['MemorySwap']==1024**3 and oh['PidsLimit']==256
   expected=workspace_graph()+[('bind',str(runner_temp),str(runner_temp),True),('bind','/var/run/docker.sock','/var/run/docker.sock',True)]
   actual=[(m['Type'],m['Source'],m['Destination'],m['RW'])for m in oi['Mounts']];assert sorted(actual)==sorted(expected)
   attached,_,_=docker('original-style-outer-execution',['start','--attach','--interactive',outer['id']],seconds=max(1,end-time.monotonic()-105),acceptable=(0,90),clean=True)
   terminal=inspect('container',outer['id']);owned(outer,terminal);assert terminal['State']['Running']is False and terminal['State']['ExitCode']==attached['status']
   outer_result=dict(initial=oi,terminal=terminal,host_socket_stat={k:getattr(socket_stat,k)for k in ['st_mode','st_uid','st_gid','st_dev','st_ino']},execution=attached)
   inner_result=json.loads((inner_path/'result.json').read_bytes());assert inner_result['phase']=='inner'and inner_result['source_revision']==SOURCE and inner_result['source_tree']==TREE and inner_result['tools_image']==tools and inner_result['status']=='diagnostic-complete'and inner_result['qualification']is False
   assert attached['status']==0 and len(inner_result['normal_retirement'])==15 and not inner_result['cleanup_failures']
   retire(outer);assert target_stat()==target_initial and source_clean()
  else:
   tools=a.tools_image;dind_id=a.dind_image;target_initial=target_stat()
   _,raw,_=docker('original-built-tools-inner-identity',['image','inspect',tools]);assert json.loads(raw)[0]['Id']==tools and json.loads(raw)[0]['Architecture']==a.architecture
   _,raw,_=docker('original-locked-dind-inner-identity',['image','inspect',DIND]);assert json.loads(raw)[0]['Id']==dind_id and json.loads(raw)[0]['Architecture']==a.architecture
   for number,(socket_target,ca_target)in enumerate([('/var/run',CA),('/run',CA),('/var/run','/etc/prismpm-qualification-ca.crt')],1):
    # Fresh independent CA lifetime per case; the identical source-generated
    # certificate is shared by that case's DinD and second owner only.
    tls=private/('tls'+str(number));tls.mkdir(mode=0o700);ca=tls/'ca.crt';key=tls/'ca.key'
    cap('mint-ephemeral-CA'+str(number),['openssl','req','-x509','-newkey','rsa:2048','-nodes','-days','1','-sha256','-subj','/CN='+session+'-ca','-addext','basicConstraints=critical,CA:TRUE','-keyout',str(key),'-out',str(ca)],seconds=30);os.chmod(ca,0o444);os.chmod(key,0o400);original=host_ca(ca);assert original['regular']and original['stat']['st_nlink']==1 and 0<original['bytes']<=65536 and stat.S_IMODE(original['stat']['st_mode'])==0o444
    with(a.output/('public-fixture-ca'+str(number)+'.pem')).open('xb')as f:f.write(ca.read_bytes())
    # Baseline proves this exact newly-built tools runtime can read the fixture
    # before shared volume/DinD lifecycle, without presuming later outcomes.
    baseline=tools_create('baseline'+str(number),ca_target=ca_target);baseline_result=observe_tools(baseline);retire(baseline)
    data=create('volume','data'+str(number),lambda name:['volume','create','--label',LABEL+'='+session,name]);socket=create('volume','socket'+str(number),lambda name:['volume','create','--label',LABEL+'='+session,name]);host_before=host_ca(ca)
    daemon=create('container','daemon'+str(number),lambda name:['container','create','--pull=never','--name',name,'--label',LABEL+'='+session,'--privileged','--network','none','--env','DOCKER_TLS_CERTDIR=','--mount','type=volume,source='+data['name']+',target=/var/lib/docker','--mount','type=volume,source='+socket['name']+',target='+socket_target,*workspace_mounts(),'--mount','type=bind,source='+str(ca)+',target='+ca_target+',readonly','--mount','type=bind,source='+str(ca)+',target=/etc/docker/certs.d/172.29.88.1:5000/ca.crt,readonly',DIND,'dockerd','--group=root','--host=unix:///var/run/docker.sock','--data-root=/var/lib/docker'],dind_id)
    dc=daemon['initial'];hc=dc['HostConfig'];assert hc['Privileged']is True and hc['NetworkMode']=='none'
    expected=workspace_graph()+[('volume',data['name'],'/var/lib/docker',True),('volume',socket['name'],socket_target,True),('bind',str(ca),ca_target,False),('bind',str(ca),'/etc/docker/certs.d/172.29.88.1:5000/ca.crt',False)]
    actual=[(m['Type'],m.get('Name')if m['Type']=='volume'else m['Source'],m['Destination'],m['RW'])for m in dc['Mounts']];assert sorted(actual)==sorted(expected)
    host_after_create=host_ca(ca);docker('start-pinned-dind',['start',daemon['id']]);ready=[]
    for attempt in range(30):
     row,raw,_=docker('actual-dind-readiness',['exec',daemon['id'],'docker','--host','unix:///var/run/docker.sock','version','--format','{{.Server.Version}}'],acceptable=(0,1),seconds=10);ready.append(row)
     if row['status']==0:assert raw.decode().strip()=='28.4.0';break
     time.sleep(.25)
    else:raise AssertionError('pinned DinD28.4.0 not ready')
    stages=[dict(stage='after-dind-ready-before-owner',command=dind_observe(daemon,'dind-before-owner'),host_CA=host_ca(ca))]
    owner=tools_create('owner'+str(number),socket,socket_target,ca_target);stages.append(dict(stage='after-owner-create',command=dind_observe(daemon,'dind-after-owner-create'),host_CA=host_ca(ca)))
    owner_result=observe_tools(owner);stages.append(dict(stage='after-owner-start',command=dind_observe(daemon,'dind-after-owner-start'),host_CA=host_ca(ca)))
    retire(owner);stages.append(dict(stage='after-owner-retirement',command=dind_observe(daemon,'dind-after-owner-retirement'),host_CA=host_ca(ca)))
    variant=dict(target=socket_target,ca_target=ca_target,original_CA=original,host_before=host_before,host_after_dind_create=host_after_create,baseline=baseline_result,dind_initial=daemon['initial'],dind_readiness=ready,stages=stages,owner_initial=owner['initial'],owner=owner_result,CA_source_after=host_ca(ca),CA_source_unchanged=host_ca(ca)==original);write(a.output/('variant'+str(number)+'.json'),variant);variants.append(variant)
    retire(daemon);retire(socket);retire(data);assert sha(trust.read_bytes())==TRUST_SHA
   assert target_stat()==target_initial and source_clean()
 except BaseException as error:primary=dict(type=type(error).__name__,message=str(error)[:2048])
 finally:
  cleanup_failures=[]
  for r in reversed(resources):
   if not r['retired']:
    try:retire(r)
    except BaseException as error:cleanup_failures.append(dict(kind=r['kind'],id=r['id'],name=r['name'],type=type(error).__name__,message=str(error)[:2048]))
  if tools and not a.inner:
   try:
    _,raw,_=docker('final-owned-tools-tag',['image','inspect',tag],clean=True);assert json.loads(raw)[0]['Id']==tools;docker('normal-tools-tag-removal',['image','rm',tag],clean=True);absent('image',tag,clean=True);tag_removed=True
   except BaseException as error:cleanup_failures.append(dict(type=type(error).__name__,message='owned tools tag retirement refused'))
  try:
   assert set(f.name for f in private.iterdir()).issubset({'tls1','tls2','tls3'})
   for number in range(1,4):
    tls=private/('tls'+str(number))
    if tls.exists():
     assert tls.is_dir()and not tls.is_symlink()and set(f.name for f in tls.iterdir()).issubset({'ca.crt','ca.key'})
     for name in ['ca.key','ca.crt']:
      f=tls/name
      if os.path.lexists(f):assert f.is_file()and not f.is_symlink();f.unlink()
     tls.rmdir()
   assert not list(private.iterdir());private.rmdir()
  except BaseException as error:cleanup_failures.append(dict(type=type(error).__name__,message='owned privatekey/certificate retirement refused'))
  complete=primary is None and not cleanup_failures and((a.inner and len(variants)==3 and len(retired)==15)or(not a.inner and inner_result is not None and len(retired)==1 and tag_removed))
  result=dict(schema='matmul/native-hosted-CA-mount-lifecycle-diagnostic/3',phase='inner'if a.inner else'host',qualification=False,status='diagnostic-complete'if complete else'diagnostic-incomplete',source_revision=SOURCE,source_tree=TREE,helper_head=os.environ.get('GITHUB_SHA'),architecture=a.architecture,tools_factory='source-owned sdk/Dockerfile --target registry_qualification_tools',tools_image=tools,dind_reference=DIND,source_trust=ref(trust),script=ref(Path(__file__)),outer=outer_result,inner_result=ref(a.output/'inner/result.json')if not a.inner and(a.output/'inner/result.json').exists()else None,target_before=target_initial,target_after=target_stat()if target_initial is not None else None,variants=[dict(target=r['target'],ca_target=r['ca_target'],baseline_validator_status=r['baseline']['observation']['original_validator']['status'],owner_validator_status=r['owner']['observation']['original_validator']['status'])for r in variants]if a.inner else(inner_result['variants']if inner_result else[]),primary_failure=primary,cleanup_failures=cleanup_failures,normal_retirement=retired,owned_tools_tag_removed=tag_removed,private_key_contents_printed=False,private_fixture_retired=not private.exists(),CA_producer='original-style outer tools container'if a.inner else'no CA minted on host',elapsed_seconds=time.monotonic()-begin,commands=commands,scope='Original-style outer tools CA producer and sharedvolume lifecycle only; RUNNER_TEMP exactRW, sourceparentRW/sourceRO/targetRW and hostsocket are preserved. DinD/second-owner preserve sourceparent/source/target mounts. The original SDK outer/inner owning programs are not invoked. CA filesystem/sharedvolume lifecycle only. Builds original tools stage and pulls only lockedDinD; no SDKimage/archive, registrytransport/provider, originalSDKowner/browser/test/qualification. Actual outcomes are diagnostic, not acceptance; futurecorrectedSDK stillrequires freshsame-sourceC/T/Q.')
  write(a.output/'result.json',result)
 if not complete:raise SystemExit(90)
 print(json.dumps(dict(result=str(a.output/'result.json'),sha256=sha((a.output/'result.json').read_bytes()),qualification=False)))
if __name__=='__main__':main()
