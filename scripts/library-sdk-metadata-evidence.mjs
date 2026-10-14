// Retain original native gate inputs; joins never replace the owning executions.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {constants,closeSync,fstatSync,linkSync,lstatSync,mkdirSync,mkdtempSync,openSync,readdirSync,realpathSync,rmSync,writeFileSync} from 'node:fs';
import {basename,dirname,join,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {regularBytes,verifyImage,verifySource,validateAcquisitionBytes,validateCapturedLock,verifyResult} from './library-sdk-check.mjs';
import {validateQualification} from '../sdk/exporter-qualification.mjs';
import {verifyMigration} from '../sdk/migration-qualification.mjs';
import {verifyMetadataEvidence,joinNativeMetadata} from '../sdk/metadata-evidence.mjs';

const canonical=value=>JSON.stringify(value,(_,item)=>item&&typeof item==='object'&&!Array.isArray(item)
  ?Object.fromEntries(Object.keys(item).sort().map(key=>[key,item[key]])):item);
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const names=['acquisition.json','compiler.json','custody.json','image.json','inventory.json','result.json','source.json','standards.lock'].sort();
const bound=name=>name==='acquisition.json'?288*1024*1024:64*1024*1024;
const regular=regularBytes;
const contextKeys=['sdk_image','source_revision','architecture'];
function contextCheck(context) {
  assert.deepEqual(Object.keys(context).sort(),contextKeys.slice().sort());
  assert.match(context.source_revision,/^[a-f0-9]{40}$/);
  assert(['amd64','arm64'].includes(context.architecture));
  assert.match(context.sdk_image,/^[a-z0-9][a-z0-9./:_-]*@sha256:[a-f0-9]{64}$/);
}
export async function captureLane(directory,root,context) {
  contextCheck(context);directory=resolve(directory);root=resolve(root);
  assert.equal(realpathSync(directory),directory);assert.equal(realpathSync(root),root);
  const files=new Map(names.map(name=>[name,regular(join(directory,name),bound(name))]));
  assert([...files.values()].reduce((total,bytes)=>total+bytes.length,0)<=512*1024*1024);
  // Refuse malformed original bytes before expensive source-closure hashing.
  // A valid record still passes every independent image/source/lock check.
  const acquisition=validateAcquisitionBytes(files.get('acquisition.json'));
  const json=name=>JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(files.get(name)));
  verifyImage(json('image.json'),context.sdk_image,context.architecture,context.source_revision);
  const source=json('source.json');assert.equal(source.revision,context.source_revision);
  assert(files.get('standards.lock').equals(regular(join(root,'standards.lock'),16*1024*1024)));
  const lockBytes=Buffer.from(canonical(acquisition.lock));
  const binding=validateCapturedLock(lockBytes,context.sdk_image,context.architecture,
    files.get('standards.lock'),files.get('inventory.json'));
  await verifyMetadataEvidence(acquisition.metadata,lockBytes);
  assert.equal(acquisition.migration.schema,'prismpm/installed-lock-migration/2');
  verifyMigration(acquisition.migration,lockBytes);
  assert.equal(acquisition.migration.platform,binding.platform);
  const authority={archive_sha256:hash(regular(join(root,'vendor/lean4-prod/lean.tar'),64*1024*1024)),
    toolchain:regular(join(root,'lean-toolchain'),1024).toString().trim()};
  validateQualification(json('compiler.json'),binding,authority,files.get('inventory.json'));verifyResult(json('result.json'),binding,authority);
  assert.deepEqual(json('custody.json'),{scope:'filesystem-custody-only',checks:6,status:'passed'});
  // Cheap malformed evidence refuses before full source hashing; every possible
  // success still joins the entire fresh source after asynchronous replay.
  verifySource(root,source);
  const row={platform:binding.platform,sdk_image:context.sdk_image,source_revision:context.source_revision,
    inventory_document:files.get('inventory.json').toString(),standards_base64:files.get('standards.lock').toString('base64')};
  return {files,row,lockBytes,metadata:acquisition.metadata,
    manifest:{schema:'prismpm/sdk-native-metadata-evidence/1',scope:'original-native-gate-inputs-only',...context,
      files:[...files].map(([path,bytes])=>({path,byte_length:bytes.length,sha256:hash(bytes)})),
      unclaimed:['full-VV','SDK-release','consumer-adoption','product-readiness']}};
}
export async function writeLane(directory,output,root,context) {
  contextCheck(context);
  root=resolve(root);output=resolve(output);
  assert.equal(output,join(root,'target/library-sdk-evidence/linux-'+context.architecture));
  assert(!lstatSync(output,{throwIfNoEntry:false}),'fresh non-overwriting native evidence directory required');
  for(const path of [join(root,'target'),dirname(output)]) {
    const entry=lstatSync(path,{throwIfNoEntry:false});
    if(entry){assert(entry.isDirectory()&&!entry.isSymbolicLink());assert.equal(realpathSync(path),path);}
  }
  const captured=await captureLane(directory,root,context);
  for(const path of [join(root,'target'),dirname(output)]) {
    if(!lstatSync(path,{throwIfNoEntry:false}))mkdirSync(path,{mode:0o755});
    assert(lstatSync(path).isDirectory()&&!lstatSync(path).isSymbolicLink());assert.equal(realpathSync(path),path);
  }
  mkdirSync(output,{mode:0o755});
  for(const [name,bytes] of captured.files)writeFileSync(join(output,name),bytes,{flag:'wx',mode:0o444});
  writeFileSync(join(output,'evidence.json'),canonical(captured.manifest)+'\n',{flag:'wx',mode:0o444});
  return captured.manifest;
}
export async function readLane(directory,root,context) {
  directory=resolve(directory);assert.equal(realpathSync(directory),directory);
  assert.deepEqual(readdirSync(directory).sort(),[...names,'evidence.json'].sort());
  const captured=await captureLane(directory,root,context);
  assert(regular(join(directory,'evidence.json'),64*1024).equals(Buffer.from(canonical(captured.manifest)+'\n')));
  return captured;
}
export async function joinLanes(directories,root,image,revision) {
  assert(Array.isArray(directories)&&directories.length===2);
  const lanes=[];
  for(const [index,architecture] of ['amd64','arm64'].entries())lanes.push(await readLane(directories[index],root,
    {sdk_image:image,source_revision:revision,architecture}));
  assert(lanes[0].lockBytes.equals(lanes[1].lockBytes),'native lanes captured different immutable SDK lock bytes');
  // Both independent captures replay; raw object bytes, not timestamps, agree.
  assert.deepEqual(lanes[0].metadata.objects.map(({kind,reference,digest,byte_length,base64})=>({kind,reference,digest,byte_length,base64})),
    lanes[1].metadata.objects.map(({kind,reference,digest,byte_length,base64})=>({kind,reference,digest,byte_length,base64})));
  return {join:await joinNativeMetadata(lanes[0].metadata,lanes[0].lockBytes,lanes.map(lane=>lane.row),revision),
    lane_evidence:lanes.map(lane=>({platform:lane.row.platform,evidence_sha256:hash(canonical(lane.manifest)+'\n'),
      metadata_evidence_sha256:hash(canonical(lane.metadata))}))};
}

export async function packLane(directory,output,root,context) {
  contextCheck(context);
  output=resolve(output);assert.equal(realpathSync(dirname(output)),dirname(output));
  assert(!lstatSync(output,{throwIfNoEntry:false}),'fresh non-overwriting native evidence archive required');
  const parent=dirname(output),fields=['dev','ino','mode','size','uid','gid','nlink','mtimeNs','ctimeNs'];
  const original=lstatSync(parent,{bigint:true});assert(original.isDirectory()&&!original.isSymbolicLink());
  const fd=openSync(parent,constants.O_RDONLY|constants.O_DIRECTORY|constants.O_NOFOLLOW|constants.O_NONBLOCK);
  // A parent-process-qualified descriptor remains usable by the tar child
  // without inheriting private descriptors across exec.
  const held='/proc/'+process.pid+'/fd/'+fd;
  const checkParent=expected=>{
    assert.equal(realpathSync(held),parent,'archive parent namespace changed');
    assert.equal(realpathSync(parent),parent,'archive parent became aliased');
    for(const current of [fstatSync(fd,{bigint:true}),lstatSync(parent,{bigint:true})])
      for(const field of fields)assert.equal(current[field],expected[field],'archive parent identity changed');
  };
  let stage,stageIdentity,primary;const secondary=[];
  try {
    checkParent(original);
    const captured=await readLane(directory,root,context);
    checkParent(original);
    assert(!lstatSync(output,{throwIfNoEntry:false}),'fresh non-overwriting native evidence archive required');
    stage=mkdtempSync(join(held,'.native-metadata-'));stageIdentity=lstatSync(stage,{bigint:true});
    const stagedParent=fstatSync(fd,{bigint:true});
    const evidence=join(stage,'evidence');mkdirSync(evidence,{mode:0o700});
    for(const [name,bytes] of captured.files)writeFileSync(join(evidence,name),bytes,{flag:'wx',mode:0o444});
    writeFileSync(join(evidence,'evidence.json'),canonical(captured.manifest)+'\n',{flag:'wx',mode:0o444});
    const archive=join(stage,'evidence.tar');
    execFileSync('/usr/bin/tar',['--format=ustar','--sort=name','--mtime=@0','--owner=0','--group=0','--numeric-owner',
      '-cf',archive,'-C',stage,'evidence'],{timeout:120000,maxBuffer:1024*1024});
    checkParent(stagedParent);
    linkSync(archive,join(held,basename(output)));
  } catch(error){primary=error;}
  if(stage)try {
    const named=lstatSync(stage,{bigint:true});
    for(const field of ['dev','ino','mode','uid','gid'])assert.equal(named[field],stageIdentity[field],'archive staging namespace changed');
    rmSync(stage,{recursive:true});
  } catch(error){secondary.push(error);}
  try{closeSync(fd);}catch(error){secondary.push(error);}
  if(primary&&secondary.length)throw new AggregateError([primary,...secondary],'archive packing and retirement failed');
  if(primary)throw primary;if(secondary.length)throw new AggregateError(secondary,'archive retirement failed');
}

if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href) {
  const [operation,...args]=process.argv.slice(2);
  if(operation==='retain'||operation==='pack') {
    assert.equal(args.length,6);const [directory,output,root,image,revision,architecture]=args;
    await (operation==='retain'?writeLane:packLane)(directory,output,root,{sdk_image:image,source_revision:revision,architecture});
  } else {
    assert.equal(operation,'join');assert.equal(args.length,5);
    const [amd64,arm64,root,image,revision]=args;
    process.stdout.write(canonical(await joinLanes([amd64,arm64],root,image,revision))+'\n');
  }
}
