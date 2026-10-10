#!/usr/bin/env node
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {cpSync, existsSync, lstatSync, mkdtempSync, readFileSync, readdirSync, renameSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join, relative, resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath, pathToFileURL} from 'node:url';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const run = (command,args,cwd) => {
  const result=spawnSync(command,args,{cwd,encoding:'utf8',maxBuffer:64*1024*1024,stdio:['ignore','pipe','pipe']});
  if(result.status!==0) throw new Error(`${command} ${args.join(' ')} failed (${result.status}):\n${result.stdout}\n${result.stderr}`);
  return result.stdout.trim();
};

export function packageTreeManifest(root, crateName) {
  const rows=[];
  const walk=(dir)=>{
    for(const name of readdirSync(dir).sort()){
      if(name==='MANIFEST.sha256') continue;
      const path=join(dir,name), st=lstatSync(path);
      assert.ok(!st.isSymbolicLink(), `vendored package contains symlink ${path}`);
      if(st.isDirectory()) walk(path);
      else if(st.isFile()){
        const rel=relative(root,path).replaceAll('\\','/');
        assert.match(rel,/^[A-Za-z0-9_./-]+$/);
        rows.push([sha(readFileSync(path)),rel]);
      } else throw new Error(`unsupported package entry ${path}`);
    }
  };
  walk(root);
  rows.sort((a,b)=>Buffer.from(a[1]).compare(Buffer.from(b[1])));
  assert.ok(rows.some(([,p])=>p===crateName), 'crate archive is not in the vendor closure');
  return rows.map(([digest,path])=>`${digest}  ${path}\n`).join('');
}

export function updateDependencyRegister(source,{revision,version,manifestSha,crateSha}) {
  assert.equal(version,'0.3.0','unsupported LexLean package version');
  assert.match(revision,/^[0-9a-f]{40}$/);
  assert.match(manifestSha,/^[0-9a-f]{64}$/);
  assert.match(crateSha,/^[0-9a-f]{64}$/);
  assert.equal([...source.matchAll(/^\[\[dependency\]\]\nid = "lexlean"$/gm)].length,1,'expected exactly one LexLean dependency row');
  const start=source.indexOf('[[dependency]]\nid = "lexlean"');
  assert.ok(start>=0,'lexlean dependency row is absent');
  const next=source.indexOf('\n[[dependency]]',start+1);
  const end=next<0?source.length:next;
  const old=source.slice(start,end);
  assert.match(old,/^version = "0\.3\.0"$/m);
  assert.match(old,/^source = "vendored"$/m);
  const blocks=old.trim().split(/\n(?=\[\[dependency\.artifact\]\])/);
  assert.equal(blocks.length,3,'expected the closed LexLean dependency and two artifacts');
  const canonicalBlock=(block,patterns)=>{
    const lines=block.split('\n').map(line=>line.trim()).filter(line=>line&&!line.startsWith('#'));
    assert.equal(lines.length,patterns.length,'unexpected or duplicate LexLean identity field');
    patterns.forEach((pattern,index)=>assert.match(lines[index],pattern));
  };
  canonicalBlock(blocks[0],[/^\[\[dependency\]\]$/,/^id = "lexlean"$/,/^version = "0\.3\.0"$/,/^revision = "[0-9a-f]{40}"$/,/^source = "vendored"$/]);
  canonicalBlock(blocks[1],[/^\[\[dependency\.artifact\]\]$/,/^kind = "tree-manifest"$/,/^path = "vendor\/lexlean\/MANIFEST\.sha256"$/,/^sha256 = "[0-9a-f]{64}"$/,/^tree_root = "vendor\/lexlean"$/]);
  canonicalBlock(blocks[2],[/^\[\[dependency\.artifact\]\]$/,/^kind = "file"$/,/^path = "vendor\/lexlean\/lexlean-0\.3\.0\.crate"$/,/^sha256 = "[0-9a-f]{64}"$/]);
  let changed=old;
  const replaceRequired=(pattern,replacement,label)=>{
    assert.equal([...changed.matchAll(new RegExp(pattern.source,pattern.flags+'g'))].length,1,`expected exactly one valid ${label}`);
    changed=changed.replace(pattern,replacement);
  };
  replaceRequired(/^version = "0\.3\.0"$/m,`version = "${version}"`,'LexLean version');
  replaceRequired(/^revision = "[0-9a-f]{40}"$/m,`revision = "${revision}"`,'LexLean revision');
  replaceRequired(/(path = "vendor\/lexlean\/MANIFEST\.sha256"\nsha256 = ")[0-9a-f]{64}(")/,`$1${manifestSha}$2`,'LexLean manifest digest');
  replaceRequired(/(path = "vendor\/lexlean\/lexlean-0\.3\.0\.crate"\nsha256 = ")[0-9a-f]{64}(")/,`$1${crateSha}$2`,'LexLean crate digest');
  // A reproducibly packaged, already-selected dependency is a valid no-op.
  // Closed canonical identity blocks reject even mixed-validity duplicate keys.
  return source.slice(0,start)+changed+source.slice(end);
}

export function assertCleanCheckout(lexRoot, revision) {
  assert.match(revision,/^[0-9a-f]{40}$/,'LexLean revision must be a full commit');
  assert.equal(run('git',['rev-parse','HEAD'],lexRoot),revision,'LexLean checkout is not the selected revision');
  // Ignored build outputs are excluded by Git; every untracked source is refused.
  assert.equal(run('git',['status','--porcelain','--untracked-files=all'],lexRoot),'','LexLean source is dirty');
}

export function reconcile(repoRoot, lexRoot, revision) {
  repoRoot=resolve(repoRoot); lexRoot=resolve(lexRoot);
  assertCleanCheckout(lexRoot, revision);

  run('cargo',['xtask','check-package'],lexRoot);
  run('cargo',['xtask','release-artifacts'],lexRoot);

  const identity=JSON.parse(readFileSync(join(lexRoot,'release/release-identity.json'),'utf8'));
  assert.equal(identity.spec,'lexlean/release-identity/1');
  assert.equal(identity.package,'lexlean');
  assert.equal(identity.version,'0.3.0');
  assert.equal(identity.lean_toolchain,'leanprover/lean4:v4.32.1');
  const version=identity.version, crateName=`lexlean-${version}.crate`;
  const crateBytes=readFileSync(join(lexRoot,'release/lexlean.crate'));
  assert.equal(sha(crateBytes),identity.crate_sha256,'LexLean release identity crate digest drift');
  // A second owning package invocation must reproduce the captured archive.
  run('cargo',['xtask','release-artifacts'],lexRoot);
  assert.deepEqual(readFileSync(join(lexRoot,'release/lexlean.crate')),crateBytes,'LexLean package is not reproducible');

  const work=mkdtempSync(join(tmpdir(),'prismpm-lexlean-reconcile-'));
  const unpack=join(work,'unpack');
  try {
    run('mkdir',['-p',unpack],repoRoot);
    run('tar',['-xzf',join(lexRoot,'release/lexlean.crate'),'-C',unpack],repoRoot);
    const packageRoot=join(unpack,`lexlean-${version}`);
    assert.equal(run('git',['rev-parse',`${revision}^{tree}`],lexRoot).length,40);
    const vcs=JSON.parse(readFileSync(join(packageRoot,'.cargo_vcs_info.json'),'utf8'));
    assert.equal(vcs.git?.sha1,revision,'packaged .cargo_vcs_info.json does not bind selected revision');

    const stage=join(repoRoot,'vendor','.lexlean-reconcile-stage');
    assert.ok(!existsSync(stage),'prior reconciliation stage exists; refusing replacement');
    cpSync(packageRoot,stage,{recursive:true,errorOnExist:true});
    writeFileSync(join(stage,crateName),crateBytes);
    const manifest=packageTreeManifest(stage,crateName);
    writeFileSync(join(stage,'MANIFEST.sha256'),manifest);
    const manifestSha=sha(Buffer.from(manifest)), crateSha=sha(crateBytes);

    const vendor=join(repoRoot,'vendor','lexlean'), backup=join(repoRoot,'vendor','.lexlean-reconcile-backup');
    assert.ok(!existsSync(backup),'prior reconciliation backup exists; refusing replacement');
    const dependencyPath=join(repoRoot,'model/dependencies.toml');
    const dependencies=updateDependencyRegister(readFileSync(dependencyPath,'utf8'),{revision,version,manifestSha,crateSha});
    renameSync(vendor,backup);
    try { renameSync(stage,vendor); } catch(error){ renameSync(backup,vendor); throw error; }
    rmSync(backup,{recursive:true,force:true});

    writeFileSync(dependencyPath,dependencies);

    // Resolve only after the exact packaged dependency is in place. Existing locked
    // versions remain preferred; new transitive requirements are admitted once here.
    run('cargo',['metadata','--format-version','1'],repoRoot);
    run('cargo',['check','--workspace','--all-targets','--all-features','--locked'],repoRoot);
    // Generated documents and locks still belong to their normal writers.
    // Validate them only after those writers finish via --validate-generated.
    return {revision,source_tree:run('git',['rev-parse',`${revision}^{tree}`],lexRoot),version,crate_sha256:crateSha,manifest_sha256:manifestSha};
  } finally { rmSync(work,{recursive:true,force:true}); }
}

export function validateGenerated(repoRoot) {
  // This public xtask entry point includes audit_dependencies and all other audits.
  // A preparation check is never substituted for either complete VV pass.
  run('cargo',['run','--locked','--offline','-q','-p','xtask','--','validate'],resolve(repoRoot));
}

if(process.argv[1] && import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
  const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
  if(process.argv.length===3 && process.argv[2]==='--validate-generated') {
    validateGenerated(root);
    process.exit(0);
  }
  if(process.argv.length!==4){
    console.error('usage: node scripts/reconcile-lexlean.mjs <lexlean-checkout> <40-hex-revision> | --validate-generated');
    process.exit(2);
  }
  const result=reconcile(resolve(dirname(fileURLToPath(import.meta.url)),'..'),process.argv[2],process.argv[3]);
  process.stdout.write(JSON.stringify(result,null,2)+'\n');
}
