import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
import {baseline,canonical,decode,factor,hash,profiles} from './workspace-corpus-zero-blocks.mjs';
import {verifyZeroBlocks,zeroBlockNames} from '../sdk/browser/corpus-zero-blocks.mjs';
import {sourceRoots} from './browser-api-sdk-check.mjs';
const root=fileURLToPath(new URL('../',import.meta.url));

function independentExpansion(data,key){
  const definitions=new Map(data.declarations.map(d=>[d.name,d])),cache=new Map();
  assert.equal(definitions.size,data.declarations.length,'unique independent fixture identity');
  function get(name,stack=[]){
    assert(!stack.includes(name),'nonrecursive independent fixture data');
    assert(stack.length<profiles[key].referenceDepth,'bounded independent reference depth');
    if(cache.has(name))return cache.get(name);
    const row=definitions.get(name);assert(row,'required independent data');
    assert.equal(row.kind,'definition');assert.deepEqual(row.parameters,[]);assert.deepEqual(row.result,{kind:'bytes'});
    const value=bytes(row.body,[...stack,name]);cache.set(name,value);return value;
  }
  function bytes(x,stack,depth=0){
    assert(depth<=32,'bounded independent concatenation');
    if(x.kind==='bytes'){assert.match(x.hex,/^(?:[0-9a-f]{2})*$/);assert(x.hex.length<=512);return Buffer.from(x.hex,'hex');}
    if(x.kind==='call'){assert.deepEqual(x.arguments,[]);assert.deepEqual(Object.keys(x.function),['name']);return get(x.function.name,stack);}
    assert.equal(x.kind,'primitive');assert.equal(x.operation,'append');assert.deepEqual(x.result,{kind:'bytes'});assert.equal(x.arguments.length,2);
    const out=Buffer.concat(x.arguments.map(a=>bytes(a,stack,depth+1)));assert(out.length<=profiles[key].maximum);return out;
  }
  return get;
}
for(const key of ['workspace','query']){
  const spec=profiles[key],source=readFileSync(join(root,'stdlib/src/Foundation/Browser/V1',spec.module+'.lex.tex'),'utf8');
  const original=decode(source).data;
  function check(data){const text=source.replace(decode(source).json,canonical(data));verifyZeroBlocks(data,key,text,independentExpansion(data,key));}
  test(key+': complete original metadata, every old value and fixed vector digest',()=>{
    check(original);const expand=independentExpansion(original,key),old=baseline().modules[key].old;
    assert.equal(old.length,spec.oldCount);assert.equal(old.filter(r=>r.value).length,spec.dataValues);
    const requests=original.declarations.filter(d=>d.name.startsWith('request'));assert.equal(requests.length,spec.requests);
    assert.equal(hash(requests.map(d=>{const id=d.name.slice(7);return id+'\t'+expand(d.name).toString('hex')+'\t'+expand('response'+id).toString('hex')+'\n';}).join('')),spec.vectorSha);
  });
  test(key+': mechanical rewrite is exact and deterministic from hash-bound original',()=>{
    const expand=independentExpansion(original,key),old=original.declarations.slice(spec.helpers).map((d,i)=>{
      const frozen=baseline().modules[key].old[i];return frozen.literal?{...d,body:{kind:'bytes',hex:expand(d.name).toString('hex')}}:d;
    });
    const restored=source.replace(decode(source).json,canonical({...original,declarations:old}));
    assert.equal(hash(restored),spec.sourceSha,'reconstructed exact original source bytes');
    const first=factor(restored,key),second=factor(restored,key);
    assert.equal(first.source,source);assert.equal(second.source,source);assert.equal(canonical(first.baseline),canonical(baseline().modules[key]));
    assert.throws(()=>factor(restored.replace('Boolean','Booleann'),key),/exact original corpus source authority/);
    assert.throws(()=>factor(source,key),/exact original corpus source authority/);
  });
  const mutants=[
    ['missing zero helper',d=>d.declarations.shift(),/complete original and zero-block closure/],
    ['duplicate zero helper',d=>d.declarations.push(structuredClone(d.declarations[0])),/unique independent fixture identity/],
    ['extra helper',d=>d.declarations.push({...structuredClone(d.declarations[0]),name:'fixtureZeroRun257'}),/complete original and zero-block closure/],
    ['changed zero seed',d=>{d.declarations[0].body.hex='01';},/exact zero-block inventory and bodies/],
    ['zero helper cycle',d=>{d.declarations[0].body={kind:'call',function:{name:d.declarations[0].name},arguments:[]};},/exact zero-block inventory and bodies/],
    ['omitted append half',d=>{d.declarations[1].body.arguments.pop();},/exact zero-block inventory and bodies/],
    ['shorter run',d=>{d.declarations[1].body=structuredClone(d.declarations[0].body);},/exact zero-block inventory and bodies/],
    ['altered old type',d=>{d.declarations[spec.helpers].result={kind:'list',element:{kind:'uint8'}};},/exact original name\/type\/parameters\/axiom metadata/],
    ['altered axiom policy',d=>{d.declarations[spec.helpers].axioms=['propext'];},/exact original name\/type\/parameters\/axiom metadata/],
    ['old value changed',d=>{d.declarations[spec.helpers].body={kind:'bytes',hex:'ff'};},/exact original data (?:length|value)/],
    ['swapped original concatenation',d=>{const row=d.declarations.slice(spec.helpers).find(x=>x.body.kind==='primitive'&&baseline().modules[key].old.find(a=>a.name===x.name)?.literal);row.body.arguments.reverse();},/exact original data value/],
    ['257 byte literal',d=>{d.declarations[spec.helpers].body={kind:'bytes',hex:'00'.repeat(257)};},/bounded literal C initializer/],
    ['external data call',d=>{d.declarations[spec.helpers].body={kind:'call',function:{module:'Foundation.Browser.V1.Workspace',name:'reduceWorkspaceBytes'},arguments:[]};},/deep-equal|deeply equal/],
    ['dynamic call arguments',d=>{d.declarations[spec.helpers].body={kind:'call',function:{name:zeroBlockNames(key)[0]},arguments:[{kind:'nat',value:'1'}]};},/deep-equal|deeply equal/],
    ['data self cycle',d=>{const row=d.declarations[spec.helpers];row.body={kind:'call',function:{name:row.name},arguments:[]};},/exact zero-block constant reference/],
    ['unknown field',d=>{d.declarations[spec.helpers].body={kind:'bytes',hex:'00',ignored:true};},/deep-equal|deeply equal/],
    ['conditional simulation',d=>{d.declarations[spec.helpers].body={kind:'if',condition:{kind:'bool',value:true},then_value:{kind:'bytes',hex:'00'},else_value:{kind:'bytes',hex:'01'}};},/deep-equal|deeply equal/],
    ['over-depth data',d=>{let body={kind:'bytes',hex:'00'};for(let n=0;n<34;n++)body={kind:'primitive',operation:'append',result:{kind:'bytes'},arguments:[body,{kind:'bytes',hex:''}]};d.declarations[spec.helpers].body=body;},/bounded fixture concatenation depth/],
    ['altered probe',d=>{const row=d.declarations.find(x=>x.name.startsWith('probe'));row.body.arguments[0].function.name='wrongEntry';},/exact unchanged original nonliteral body/],
    ['dropped original',d=>d.declarations.pop(),/complete original and zero-block closure/],
    ['old identity reordered',d=>{[d.declarations[spec.helpers],d.declarations[spec.helpers+1]]=[d.declarations[spec.helpers+1],d.declarations[spec.helpers]];},/exact original ordered declaration names/],
    ['unused zero helper with unchanged values',d=>{
      const helper=d.declarations.find(x=>x.name==='fixtureZeroRun'+(key==='workspace'?63:200));
      function inline(x){if(Array.isArray(x))return x.map(inline);if(!x||typeof x!=='object')return x;
        if(x.kind==='call'&&x.function.name===helper.name)return structuredClone(helper.body);
        return Object.fromEntries(Object.entries(x).map(([k,v])=>[k,inline(v)]));}
      for(const row of d.declarations.slice(spec.helpers))row.body=inline(row.body);
    },/complete used zero-block closure/],
  ];
  for(const [label,mutate,message] of mutants)test(key+': refuses '+label,()=>{
    const changed=structuredClone(original);mutate(changed);assert.throws(()=>check(changed),message);
  });
  test(key+': independent expansion refuses real cycle even without frozen-body comparison',()=>{
    const changed=structuredClone(original),name=changed.declarations[0].name;
    changed.declarations[0].body={kind:'call',function:{name},arguments:[]};
    assert.throws(()=>independentExpansion(changed,key)(name),/nonrecursive independent fixture data/);
  });
}
test('new oracle and authority are in existing installed source tree; authoring code is not an oracle import',()=>{
  assert(sourceRoots.includes('sdk/browser'));
  const verifier=readFileSync(join(root,'sdk/browser/corpus-zero-blocks.mjs'),'utf8');
  assert(!verifier.includes("from '../../scripts/")&&!verifier.includes("from './workspace-corpus-zero-blocks"));
  const docker=readFileSync(join(root,'sdk/Dockerfile'),'utf8');
  assert(docker.includes('COPY --from=source_inputs /prepared/source/ .'));
  for(const path of ['sdk/browser/workspace-model-test.mjs','sdk/browser/query-model-test.mjs','tests/browser-view/query-corpus.mjs']){
    const consumer=readFileSync(join(root,path),'utf8');assert(consumer.includes('verifyZeroBlocks'));assert(!consumer.includes("from '../../scripts/workspace-corpus-zero-blocks"));
  }
});
