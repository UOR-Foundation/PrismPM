import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import test from 'node:test';
import {packageTreeManifest,updateDependencyRegister} from './reconcile-lexlean.mjs';

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
