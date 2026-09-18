import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {verifyView,verifyModelMutation,refuseBypasses} from '../../tests/browser-view/checks.mjs';
test('DK-15 exact registered modeled interaction, presentation and rejections',()=>{
  refuseBypasses();
  const source=readFileSync(new URL('../../stdlib/src/Foundation/View/Workspace/V1/Interaction.lex.tex',import.meta.url),'utf8');
  const ast=JSON.parse(/\\semanticdata\{(.*)\}/.exec(source)[1]);
  for(const name of['workspaceInteractionBytes','workspacePresentationBytes']){
    const definitions=ast.declarations.filter(d=>d.name===name);assert.equal(definitions.length,1);assert.equal(definitions[0].kind,'definition');assert.ok(definitions[0].body);
  }
  assert.match(source,/\\importmodule\{Foundation.View.V1.Interaction\}/);
  const register=readFileSync(new URL('../../model/browser-view-diagnostics.toml',import.meta.url),'utf8');
  const expected=ast.declarations.find(d=>d.name==='WorkspaceInteractionError').constructors.map((d,index)=>({byte:index+1,name:d.name}));
  assert.deepEqual([...register.matchAll(/^byte = ([0-9]+)\nname = "([A-Za-z]+)"$/gm)].map(m=>({byte:Number(m[1]),name:m[2]})),expected);
});
test('DK-15 complete fresh modeled View semantics and actual compiler mutants',{timeout:3500000},async t=>{
  await verifyView(t);
  for(const kind of['session','rows'])await t.test('actual generated '+kind+' mutation is rejected',()=>verifyModelMutation(kind));
});
