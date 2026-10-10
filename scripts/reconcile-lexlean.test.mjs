import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import test from 'node:test';
import {assertCleanCheckout,packageTreeManifest,reconcile,updateDependencyRegister} from './reconcile-lexlean.mjs';

test('tree manifest closes package bytes including the crate archive',t=>{
 const root=mkdtempSync(join(tmpdir(),'reconcile-manifest-')); t.after(()=>rmSync(root,{recursive:true,force:true}));
 mkdirSync(join(root,'src')); writeFileSync(join(root,'src/lib.rs'),'x\n'); writeFileSync(join(root,'lexlean-0.3.0.crate'),'crate');
 const manifest=packageTreeManifest(root,'lexlean-0.3.0.crate');
 assert.match(manifest,/  lexlean-0\.3\.0\.crate\n/); assert.match(manifest,/  src\/lib\.rs\n/);
 const lines=manifest.trimEnd().split('\n'); assert.deepEqual(lines,[...lines].sort((a,b)=>Buffer.from(a.slice(66)).compare(Buffer.from(b.slice(66)))));
});

test('dependency update changes only the lexlean identity fields',()=>{
 const source='spec = "prismpm/dependencies/1"\n\n[[dependency]]\nid = "lexlean"\nversion = "0.3.0"\nrevision = "'+'1'.repeat(40)+'"\nsource = "vendored"\n\n[[dependency.artifact]]\nkind = "tree-manifest"\npath = "vendor/lexlean/MANIFEST.sha256"\nsha256 = "'+'2'.repeat(64)+'"\ntree_root = "vendor/lexlean"\n\n[[dependency.artifact]]\nkind = "file"\npath = "vendor/lexlean/lexlean-0.3.0.crate"\nsha256 = "'+'3'.repeat(64)+'"\n\n[[dependency]]\nid = "next"\nrevision = "'+'4'.repeat(40)+'"\n';
 const out=updateDependencyRegister(source,{revision:'a'.repeat(40),version:'0.3.0',manifestSha:'b'.repeat(64),crateSha:'c'.repeat(64)});
 assert.match(out,/revision = "a{40}"/); assert.match(out,/sha256 = "b{64}"/); assert.match(out,/sha256 = "c{64}"/); assert.match(out,/id = "next"/);
});

// These subprocess fixtures qualify orchestration only. They never provide
// evidence that Cargo, LexLean, PrismPM or an SDK acceptance gate ran.
import {createHash} from 'node:crypto';
import {chmodSync,readFileSync,symlinkSync} from 'node:fs';
import {dirname,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';

const repository=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const helperSource=readFileSync(join(repository,'scripts/reconcile-lexlean.mjs'),'utf8');
const validationArgs=['run','--locked','--offline','-q','-p','xtask','--','validate'];

function validationFixture(t, source=helperSource, exitStatus=0) {
 const root=mkdtempSync(join(tmpdir(),'reconcile invocation '));
 t.after(()=>rmSync(root,{recursive:true,force:true}));
 mkdirSync(join(root,'scripts')); mkdirSync(join(root,'bin'));
 const helper=join(root,'scripts/reconcile-lexlean.mjs');
 writeFileSync(helper,source);
 const cargo=join(root,'bin/cargo');
 writeFileSync(cargo,`#!${process.execPath}\nconst fs=require('node:fs');\nconst args=process.argv.slice(2);\nfs.writeFileSync(process.env.INVOCATION_LOG,JSON.stringify({args,cwd:process.cwd()}));\nif(JSON.stringify(args)!==JSON.stringify(${JSON.stringify(validationArgs)})){process.stderr.write('unsupported xtask command\\n');process.exit(2);}\nprocess.exit(${exitStatus});\n`);
 chmodSync(cargo,0o755);
 const log=join(root,'invocation.json');
 const result=spawnSync(process.execPath,[helper,'--validate-generated'],{
  encoding:'utf8',cwd:tmpdir(),env:{...process.env,PATH:`${join(root,'bin')}:${process.env.PATH}`,INVOCATION_LOG:log},
 });
 return {root,result,invocation:JSON.parse(readFileSync(log,'utf8'))};
}

test('manifest contains actual file SHA-256, not a literal expression',t=>{
 const root=mkdtempSync(join(tmpdir(),'reconcile-digest-'));t.after(()=>rmSync(root,{recursive:true,force:true}));
 writeFileSync(join(root,'lexlean-0.3.0.crate'),'archive');
 assert.equal(packageTreeManifest(root,'lexlean-0.3.0.crate'),`${createHash('sha256').update('archive').digest('hex')}  lexlean-0.3.0.crate\n`);
});

test('manifest rejects an absent archive or a symlink',t=>{
 const root=mkdtempSync(join(tmpdir(),'reconcile-manifest-negative-'));t.after(()=>rmSync(root,{recursive:true,force:true}));
 writeFileSync(join(root,'file'),'bytes');
 assert.throws(()=>packageTreeManifest(root,'lexlean-0.3.0.crate'),/archive is not in/);
 symlinkSync('file',join(root,'lexlean-0.3.0.crate'));
 assert.throws(()=>packageTreeManifest(root,'lexlean-0.3.0.crate'),/symlink/);
});

test('validation command exists in the real dispatcher and includes dependency auditing',()=>{
 const dispatcher=readFileSync(join(repository,'xtask/src/main.rs'),'utf8');
 assert.match(dispatcher,/"validate"\s*=>\s*validate_all\(&root, false\)/);
 assert.match(dispatcher,/fn validate_all\([\s\S]*?audit_all\(root\)/);
 assert.match(dispatcher,/fn audit_all\([\s\S]*?audit::audit_dependencies\(root\)\?/);
});

test('validation CLI actually invokes supported offline xtask in its repository',t=>{
 const {root,result,invocation}=validationFixture(t);
 assert.equal(result.status,0,result.stderr);
 assert.deepEqual(invocation,{args:validationArgs,cwd:root});
});

test('validation CLI propagates a real child-process failure',t=>{
 const {result,invocation}=validationFixture(t,helperSource,23);
 assert.notEqual(result.status,0);
 assert.match(result.stderr,/failed \(23\)/);
 assert.deepEqual(invocation.args,validationArgs);
});

test('unsupported-command mutation is rejected by the subprocess boundary',t=>{
 const old="'xtask','--','validate'";
 assert.ok(helperSource.includes(old));
 const mutation=helperSource.replace(old,"'xtask','--','audit-dependencies'");
 const {result}=validationFixture(t,mutation);
 assert.notEqual(result.status,0);
 assert.match(result.stderr,/unsupported xtask command/);
});

test('reconciliation workflow pins selected source and has no repository writer',()=>{
 const workflow=readFileSync(join(repository,'.github/workflows/m0-lexlean-reconcile.yml'),'utf8');
 const selected='3e79a5a0e7059dc3a425aa17010e0a6721b24c44';
 assert.match(workflow,/workflow_dispatch:/);
 assert.match(workflow,/branches: \[work\/compression-m0-reconcile\]/);
 assert.doesNotMatch(workflow,/pull_request_target:/);
 assert.equal((workflow.match(new RegExp(selected,'g'))??[]).length,3);
 assert.match(workflow,/permissions:\n  contents: read/);
 assert.equal((workflow.match(/persist-credentials: false/g)??[]).length,2);
 assert.doesNotMatch(workflow,/contents: write|packages: write|id-token: write|git (?:commit|push)|gh (?:pr|release)|cargo publish/);
 assert.match(workflow,/git diff --binary HEAD/);
 assert.match(workflow,/actions\/upload-artifact@[0-9a-f]{40}/);
 assert.equal((workflow.match(/if: always\(\)/g)??[]).length,2);
 assert.match(workflow,/name: m0-lexlean-reconciliation-proposal/);
 assert.match(workflow,/name: m0-lexlean-reconciliation-diagnostics/);
 assert.match(workflow,/partial-dependency\.json/);
 assert.match(workflow,/Diagnostic-only partial preparation/);
 assert.ok(workflow.indexOf('--validate-generated')>workflow.indexOf('just package-release-crates-check'));
});


test('owning package boundary rejects untracked and tracked source, allowing ignored outputs',t=>{
 const root=mkdtempSync(join(tmpdir(),'reconcile-clean-source-'));t.after(()=>rmSync(root,{recursive:true,force:true}));
 const git=(...args)=>{
  const result=spawnSync('git',args,{cwd:root,encoding:'utf8'});
  assert.equal(result.status,0,result.stderr);return result.stdout.trim();
 };
 git('init','--quiet');
 writeFileSync(join(root,'.gitignore'),'build-output/\n');
 writeFileSync(join(root,'source.lex.tex'),'source\n');
 git('add','.gitignore','source.lex.tex');
 git('-c','user.name=Reconciliation test fixture','-c','user.email=fixture@example.invalid','commit','--quiet','-m','fixture source');
 const revision=git('rev-parse','HEAD');
 mkdirSync(join(root,'build-output'));writeFileSync(join(root,'build-output/result'),'ignored build output');
 assert.doesNotThrow(()=>assertCleanCheckout(root,revision));
 writeFileSync(join(root,'untracked.lex.tex'),'untracked source\n');
 // Run the actual preparation entry point: it must reject before invoking Cargo.
 assert.throws(()=>reconcile(root,root,revision),/LexLean source is dirty/);
 rmSync(join(root,'untracked.lex.tex'));
 writeFileSync(join(root,'source.lex.tex'),'changed source\n');
 assert.throws(()=>reconcile(root,root,revision),/LexLean source is dirty/);
});
