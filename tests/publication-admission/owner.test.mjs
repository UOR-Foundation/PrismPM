import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {verifyWire,verifyModelMutation,inventory,prerequisite} from './checks.mjs';

test('OC-09 owns distinct source publication stages and preserves public deployment refusal', () => {
  const source = readFileSync(new URL('../../stdlib/src/Production/PublicationAdmission/V1.lex.tex', import.meta.url), 'utf8');
  assert.match(source, /publicationTransition/);
  const lifecycle = readFileSync(new URL('../../crates/prismpm/src/lifecycle.rs', import.meta.url), 'utf8');
  assert.match(lifecycle, /GitHub Pages publication uses its protected Pages workflow/);
});

test('OC-09 complete generated conditional publication admission', {timeout:3500000}, async t=>{
 const build=await verifyWire(t),mutants=[];
 await prerequisite(t,'closed protocol diagnostics and complete independent wire shape',inventory);
 for(const kind of['binding','coverage','timeline','trailing','preimage','partition'])mutants.push(await prerequisite(t,'actual LexLean '+kind+' defect fails native and Wasm',()=>verifyModelMutation(kind)));
 writeFileSync(join(build.work,'publication-acceptance.json'),JSON.stringify({capability:'OC-09',scope:'conditional-source-admission-only',source:build.verified.source_id,attestation:build.verified.attestation_id,maximum:build.maximum,mutants},null,2)+'\n',{flag:'wx'});
});
