// Additive source/parser controls only; actual compiler/installed owners remain mandatory.
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync,symlinkSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {test} from 'node:test';
import {capture,verifySource,sourceRoots,sourceAliases,checkAccepted as checkWithAuthority,verifyResult,qualificationChecks,completedChecks,unclaimed} from './binary-sdk-check.mjs';
import {parserFixture,sourceAuthorityFixture} from './binary-sdk-check-fixtures.mjs';
const revision='a'.repeat(40),image='ghcr.io/uor-foundation/prismpm-sdk@sha256:'+'b'.repeat(64);
const temporary=t=>{const root=mkdtempSync(join(tmpdir(),'prismpm-binary-language12-'));t.after(()=>rmSync(root,{recursive:true,force:true}));return root;};
const put=(root,path,bytes)=>{mkdirSync(dirname(join(root,path)),{recursive:true});writeFileSync(join(root,path),bytes);};
const checkAccepted=(root,receipt)=>checkWithAuthority(root,receipt,sourceAuthorityFixture);
function source(root){
 for(const path of sourceRoots)if(!Object.hasOwn(sourceAliases,path))put(root,path+'/input','source');
 for(const [path,target]of Object.entries(sourceAliases)){const full=join(root,path);mkdirSync(dirname(full),{recursive:true});if(path.includes('/embedded/')){const actual=resolve(dirname(full),target);mkdirSync(dirname(actual),{recursive:true});writeFileSync(actual,'bound alias target');}symlinkSync(target,full);}
}

test('Binary capture independently refuses an unregistered helper package build script',t=>{
 const root=temporary(t);source(root);const expected=capture(root,revision),path='crates/conformance/build.rs';
 put(root,path,'fn main() {}');assert.throws(()=>capture(root,revision),/unregistered conformance build script/);
 assert.throws(()=>verifySource(root,expected),/unregistered conformance build script/);
 rmSync(join(root,path));symlinkSync('missing',join(root,path));assert.throws(()=>capture(root,revision),/unregistered conformance build script/);
});

test('language-1.2 qualification result cannot replace the original language-1.1 owner or omit runtime controls',()=>{
 const args=[image,revision,'amd64','1'.repeat(64),'2'.repeat(64)];
 const value={schema:'prismpm/installed-binary-language-1.2-check/1',scope:'installed-binary-package-only',status:'passed',sdk_image:image,source_revision:revision,source_sha256:args[3],inventory_sha256:args[4],architecture:'amd64',build_id:'3'.repeat(64),attestation_id:'4'.repeat(64),checks:qualificationChecks('1.2'),unclaimed:[...unclaimed]};
 verifyResult(value,...args,'1.2');assert.throws(()=>verifyResult(value,...args));
 for(const change of [v=>v.checks.pop(),v=>v.checks=[...completedChecks],v=>v.schema='prismpm/installed-binary-check/1']){
  const changed=structuredClone(value);change(changed);assert.throws(()=>verifyResult(changed,...args,'1.2'));
 }
});

test('language-1.2 parser binds complete source-derived runtime audit with original project policies',t=>{
 const read=(root,receipt)=>checkWithAuthority(root,receipt,sourceAuthorityFixture,'1.2');
 const valid=temporary(t),receipt=parserFixture(valid,undefined,undefined,sourceAuthorityFixture,'1.2');read(valid,receipt);
 assert.throws(()=>checkAccepted(valid,receipt),'a1.2 witness cannot silently qualify a1.1 owner');
 for(const change of [
  ({lex})=>lex.declarations.pop(),
  ({lex})=>lex.declarations.push(lex.declarations.at(-1)),
  ({lex})=>lex.declarations.at(-1).name='LexLeanRuntime.unknownBinaryProbe',
  ({lex})=>{const row=lex.declarations.at(-1);row.observed=['sorryAx'];row.policy.axioms=['sorryAx'];},
  ({lex})=>{lex.declarations[0].observed=['sorryAx'];},
  ({lex})=>{lex.spec='lexlean/attestation/1';},
 ]){const root=temporary(t);assert.throws(()=>read(root,parserFixture(root,change,undefined,sourceAuthorityFixture,'1.2')));}
});
