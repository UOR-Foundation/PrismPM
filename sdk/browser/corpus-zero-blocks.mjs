// Independent old-value oracle: never imports the authoring transform or its evaluator.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {lstatSync,readFileSync,realpathSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
const digest=value=>createHash('sha256').update(value).digest('hex');
const stable=value=>JSON.stringify(value,(_,x)=>x&&typeof x==='object'&&!Array.isArray(x)
  ?Object.fromEntries(Object.keys(x).sort().map(k=>[k,x[k]])):x);
const path=fileURLToPath(new URL('./corpus-zero-blocks.baseline.json',import.meta.url));
const stat=lstatSync(path);assert(stat.isFile()&&stat.size<=2*1024**2&&realpathSync(path)===path);
const raw=readFileSync(path);assert.equal(raw.length,stat.size);
assert.equal(digest(raw),'605a0fb21574e98e59d7a381d008552ca4f05fa859dd342ae1abe0c1c9cc8de1','frozen original data authority');
const authority=JSON.parse(raw);assert.equal(stable(authority)+'\n',raw.toString());
assert.deepEqual(Object.keys(authority).sort(),['modules','source_revision','spec']);
assert.equal(authority.spec,'prismpm/workspace-corpus-zero-blocks/1');
assert.equal(authority.source_revision,'5adbcd9460556d476fd141c4364cecf582e2bca0');
assert.deepEqual(Object.keys(authority.modules),['query','workspace']);
const profiles=Object.freeze({workspace:{old:847,new:22,data:802,maximum:1104665},query:{old:1573,new:29,data:1510,maximum:1166280}});
function selected(key){assert(Object.hasOwn(profiles,key),'closed data oracle profile');return authority.modules[key];}
export function zeroBlockNames(key){return selected(key).zero_helpers.map(d=>d.name);}
function dataGrammar(x,names,depth=0){
  assert(x&&typeof x==='object'&&!Array.isArray(x),'closed zero-block data');
  assert(depth<=32,'bounded fixture concatenation depth');
  if(x.kind==='bytes'){
    assert.deepEqual(Object.keys(x).sort(),['hex','kind']);assert.equal(typeof x.hex,'string');
    assert.match(x.hex,/^(?:[0-9a-f]{2})*$/);assert(x.hex.length<=512,'bounded literal C initializer');
  }else if(x.kind==='call'){
    assert.deepEqual(Object.keys(x).sort(),['arguments','function','kind']);
    assert.deepEqual(x.arguments,[]);assert.deepEqual(Object.keys(x.function),['name']);
    assert(names.has(x.function.name),'exact zero-block constant reference');
  }else{
    assert.deepEqual(Object.keys(x).sort(),['arguments','kind','operation','result']);
    assert.equal(x.kind,'primitive');assert.equal(x.operation,'append');assert.deepEqual(x.result,{kind:'bytes'});
    assert.equal(x.arguments.length,2);x.arguments.forEach(v=>dataGrammar(v,names,depth+1));
  }
}
export function verifyZeroBlocks(data,key,source,expand){
  const spec=profiles[key],a=selected(key);
  assert.equal(a.old.length,spec.old);assert.equal(a.zero_helpers.length,spec.new);
  assert.equal(data.declarations.length,spec.old+spec.new,'complete original and zero-block closure');
  const names=new Set(data.declarations.map(d=>d.name));assert.equal(names.size,data.declarations.length,'unique closed data identity');
  const additions=data.declarations.slice(0,spec.new);
  assert.equal(stable(additions),stable(a.zero_helpers),'exact zero-block inventory and bodies');
  const match=/\\semanticdata\{([^\n]+)\}\n/.exec(source);assert(match);
  assert.equal(digest(source.replace(match[1],'{}')),a.wrapper_sha256,'exact original corpus wrapper');
  const zeroNames=new Set(additions.map(d=>d.name));let values=0;
  for(let i=0;i<spec.old;i++){
    const d=data.declarations[i+spec.new],old=a.old[i];assert.equal(d.name,old.name,'exact original ordered declaration names');
    const metadata=Object.fromEntries(Object.entries(d).filter(([k])=>k!=='body'));
    assert.equal(digest(stable(metadata)),old.metadata_sha256,'exact original name/type/parameters/axiom metadata');
    if(!old.literal)assert.equal(digest(stable(d.body)),old.body_sha256,'exact unchanged original nonliteral body');
    else dataGrammar(d.body,zeroNames);
    if(old.value){
      const actual=expand(d.name);assert(Buffer.isBuffer(actual),'independently expanded exact data bytes');
      assert.equal(actual.length,old.value.bytes,'exact original data length');
      assert.equal(digest(actual),old.value.sha256,'exact original data value');
      assert(actual.length<=spec.maximum);
      if(old.literal)assert(actual.length<=256,'exact original bounded literal helper value');
      values++;
    }
  }
  assert.equal(values,spec.data,'complete original data value inventory');
  // Each newly added helper is closed data, bounded and independently expanded.
  const helperDepths=new Map();
  function helperDepth(x){
    if(x.kind==='bytes')return 0;
    if(x.kind==='call'){assert(helperDepths.has(x.function.name),'topological nonrecursive zero-block definition');return helperDepths.get(x.function.name);}
    return Math.max(...x.arguments.map(helperDepth));
  }
  for(const d of additions){
    dataGrammar(d.body,zeroNames);const bytes=expand(d.name),length=Number(d.name.slice('fixtureZeroRun'.length));
    assert(Number.isInteger(length)&&length>=1&&length<=256,'bounded zero-block size');
    assert.equal(bytes.length,length);assert(bytes.every(value=>value===0),'literal zero-block value');
    const depth=helperDepth(d.body)+1;assert(depth<=9,'bounded acyclic zero-block reference depth');helperDepths.set(d.name,depth);
  }
  const defs=new Map(data.declarations.map(d=>[d.name,d]));
  const pending=a.old.filter(row=>row.value).map(row=>row.name),visited=new Set();
  function references(x){
    if(!x||typeof x!=='object')return;
    if(x.kind==='call'&&!x.function.module)pending.push(x.function.name);
    for(const child of Object.values(x))if(Array.isArray(child))child.forEach(references);else references(child);
  }
  for(let i=0;i<pending.length;i++){
    const name=pending[i];if(visited.has(name))continue;visited.add(name);
    assert(defs.has(name),'closed zero-block data references');references(defs.get(name).body);
  }
  for(const d of additions)assert(visited.has(d.name),'complete used zero-block closure');
}
