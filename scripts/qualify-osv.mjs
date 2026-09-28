// Read-only CI proposal construction. Never uploads database payloads or publishes a release.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createReadStream} from 'node:fs';
import {lstat, mkdir, readFile, realpath, writeFile} from 'node:fs/promises';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {ecosystems, metadataUrl, validateManifest} from './refresh-osv.mjs';

const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const output=join(root,'.prism/cache/osv-candidate');
const proposalPath=join(root,'.prism/cache/osv-refresh.json');
const changedPaths=['model/authorities.toml','model/osv-databases.json','standards.lock'];
const sourcePaths=['.github/workflows/osv-input-qualification.yml','scripts/qualify-osv.mjs',
  'scripts/qualify-osv.test.mjs','scripts/refresh-osv.mjs','scripts/refresh-osv.test.mjs',
  'Cargo.lock','model/authorities.toml','model/osv-databases.json','standards.lock'];
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const now=()=>Math.floor(Date.now()/1000);

function run(executable,args,options={}) {
  return execFileSync(executable,args,{cwd:root,encoding:'utf8',timeout:60000,
    maxBuffer:4*1024*1024,stdio:['ignore','pipe','pipe'],...options});
}

async function json(path) {return JSON.parse(await readFile(path,'utf8'));}
async function put(name,value) {
  await writeFile(join(output,name),JSON.stringify(value,null,2)+'\n',{flag:'wx'});
}
async function sourceIdentity() {
  const commit=run('git',['rev-parse','HEAD']).trim();
  assert.match(commit,/^[0-9a-f]{40}$/u);
  run('git',['ls-files','--error-unmatch','--',...sourcePaths]);
  return {commit,files:Object.fromEntries(await Promise.all(sourcePaths.map(async path=>
    [path,hash(await readFile(join(root,path)))])))};
}

export function checkOfficialMetadata(row,metadata) {
  assert.equal(metadata.bucket,'osv-vulnerabilities');
  assert.equal(metadata.name,`${row.ecosystem}/all.zip`);
  assert.equal(metadata.generation,row.generation);
  assert.equal(metadata.size,String(row.size));
  assert.equal(metadata.md5Hash,row.md5);
  assert.equal(metadata.timeCreated,row.source_created);
  assert.equal(metadata.updated,row.source_updated);
}

export async function checkCachedObject(path,row) {
  const info=await lstat(path);
  assert.ok(info.isFile()&&!info.isSymbolicLink(),'regular acquired cache object');
  assert.equal(await realpath(path),resolve(path),'cache object has no symlink ancestors');
  assert.equal(info.size,row.size,'exact cached object size');
  const sha=createHash('sha256'),md5=createHash('md5');let bytes=0;
  for await(const chunk of createReadStream(path)) {
    bytes+=chunk.length;assert.ok(bytes<=row.size,'bounded cached object');
    sha.update(chunk);md5.update(chunk);
  }
  assert.equal(bytes,row.size);assert.equal(sha.digest('hex'),row.sha256);
  assert.equal(md5.digest('base64'),row.md5);
  return {ecosystem:row.ecosystem,generation:row.generation,bytes,sha256:row.sha256,md5:row.md5};
}

async function officialMetadata(row) {
  const response=await fetch(metadataUrl(row.ecosystem,row.generation),
    {redirect:'error',signal:AbortSignal.timeout(30000)});
  assert.equal(response.status,200,'exact official generation remains available');
  const chunks=[];let bytes=0;
  for await(const chunk of response.body) {
    bytes+=chunk.length;assert.ok(bytes<=65536,'bounded official metadata');chunks.push(chunk);
  }
  const raw=Buffer.concat(chunks),metadata=JSON.parse(raw.toString('utf8'));
  checkOfficialMetadata(row,metadata);return hash(raw);
}

export function checkFreshProposal(manifest,clock) {
  const expiry=validateManifest(manifest);
  assert.deepEqual(manifest.databases.map(row=>row.ecosystem),ecosystems);
  assert.ok(manifest.databases.every(row=>Date.parse(row.acquired_at)<=clock*1000+999),
    'acquisition cannot be in the future');
  assert.ok(clock<expiry,'proposal has expired');return expiry;
}

export function checkChangedPaths(text,expected) {
  const actual=text.trim()?text.trim().split('\n').sort():[];
  assert.deepEqual(actual,[...expected].sort(),'candidate changes only intended authority inputs');
}

async function acquireCandidate() {
  run('git',['diff','--exit-code','HEAD']);
  const source=await sourceIdentity();
  await mkdir(dirname(output),{recursive:true});
  await mkdir(output,{recursive:false});
  // The existing acquirer owns generation pinning, HTTP headers, MD5/SHA and disk bounds.
  run(process.execPath,['scripts/refresh-osv.mjs','acquire'],{timeout:55*60000,maxBuffer:1024*1024});
  const proposal=await readFile(proposalPath),manifest=JSON.parse(proposal);
  checkFreshProposal(manifest,now());
  const objects=[];
  for(const row of manifest.databases) {
    const metadataSha256=await officialMetadata(row);
    objects.push({...await checkCachedObject(join(root,'.prism/cache/authorities/sha256',row.sha256),row),
      metadataSha256});
  }
  const expiry=checkFreshProposal(manifest,now());
  assert.equal(hash(await readFile(proposalPath)),hash(proposal),'reviewed proposal remains unchanged');
  assert.deepEqual(await sourceIdentity(),source,'acquisition cannot change tracked source');
  run('git',['diff','--exit-code','HEAD']);
  await writeFile(join(output,'proposal.json'),proposal,{flag:'wx'});
  await put('review.json',{schema:'prismpm/osv-candidate-review/1',scope:'input-acquisition-only',
    reviewedAt:new Date().toISOString(),expiry:new Date(expiry*1000).toISOString(),
    source,proposalSha256:hash(proposal),objects});
  // This applies only inside the disposable candidate runner, not an authoritative checkout.
  run(process.execPath,['scripts/refresh-osv.mjs','apply']);
  checkChangedPaths(run('git',['diff','--name-only','HEAD']),changedPaths.slice(0,2));
}

async function finishCandidate() {
  const review=await json(join(output,'review.json'));
  assert.equal(review.schema,'prismpm/osv-candidate-review/1');
  assert.equal(review.scope,'input-acquisition-only');
  assert.equal(run('git',['rev-parse','HEAD']).trim(),review.source.commit);
  const proposal=await readFile(join(output,'proposal.json')),manifest=JSON.parse(proposal);
  assert.equal(hash(proposal),review.proposalSha256);
  assert.equal(hash(await readFile(proposalPath)),review.proposalSha256);
  assert.equal(hash(await readFile(join(root,'model/osv-databases.json'))),review.proposalSha256);
  checkFreshProposal(manifest,now());
  checkChangedPaths(run('git',['diff','--name-only','HEAD']),changedPaths.slice(0,2));
  for(const path of sourcePaths.filter(path=>!changedPaths.includes(path)))
    assert.equal(hash(await readFile(join(root,path))),review.source.files[path],`unchanged tool input ${path}`);
  const target=join(root,'.prism/cache/osv-candidate-build');
  const env={...process.env,CARGO_TARGET_DIR:target,CARGO_INCREMENTAL:'0',CARGO_PROFILE_DEV_DEBUG:'0',CARGO_BUILD_JOBS:'2'};
  const buildSource=await sourceIdentity();
  const buildArgs=['build','--locked','--offline','-p','prismpm','--bin','prismpm','-p','xtask','--bin','xtask'];
  const buildLog=run('cargo',buildArgs,{env,timeout:20*60000,maxBuffer:8*1024*1024});
  await writeFile(join(output,'build.log'),buildLog,{flag:'wx'});
  const cli=join(target,'debug/prismpm'),writer=join(target,'debug/xtask');
  const commands=[['prismpm',['--json','--project',root,'authority','resolve']],
    ['prismpm',['--json','--project',root,'authority','resolve','--locked']],
    ['xtask',['validate-model','--write']],['xtask',['validate-model']]];
  const results=[];
  for(const [tool,args] of commands)results.push({tool,args,stdout:run(tool==='prismpm'?cli:writer,args,{env,timeout:5*60000})});
  run(process.execPath,['--test','scripts/refresh-osv.test.mjs','scripts/qualify-osv.test.mjs']);
  checkChangedPaths(run('git',['diff','--name-only','HEAD']),changedPaths);
  for(const row of manifest.databases)await checkCachedObject(join(root,'.prism/cache/authorities/sha256',row.sha256),row);
  checkFreshProposal(manifest,now());
  const patch=run('git',['diff','--binary','HEAD','--',...changedPaths]);
  assert.ok(Buffer.byteLength(patch)>0&&Buffer.byteLength(patch)<=1024*1024,'bounded nonempty candidate patch');
  await writeFile(join(output,'candidate.patch'),patch,{flag:'wx'});
  const inputs=await sourceIdentity();
  await put('receipt.json',{schema:'prismpm/osv-candidate/1',scope:'review-only-authority-update',
    productionAccepted:false,sourceCommit:review.source.commit,buildSourceInputs:buildSource.files,candidateInputs:inputs.files,
    proposalSha256:review.proposalSha256,reviewSha256:hash(await readFile(join(output,'review.json'))),
    patchSha256:hash(patch),tooling:{node:process.version,cargo:run('cargo',['--version']).trim(),
      rustc:run('rustc',['--version']).trim(),prismpmSha256:hash(await readFile(cli)),xtaskSha256:hash(await readFile(writer)),buildArgs},
    results,completedAt:new Date().toISOString()});
}

if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length,3,'usage: node scripts/qualify-osv.mjs acquire|finish');
  if(process.argv[2]==='acquire')await acquireCandidate();
  else if(process.argv[2]==='finish')await finishCandidate();
  else throw Error('unknown closed qualification operation');
}
