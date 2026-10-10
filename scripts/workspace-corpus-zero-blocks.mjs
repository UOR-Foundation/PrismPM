// Source-owned fixture compaction; not compiler, runtime or performance acceptance.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {lstatSync,readFileSync,realpathSync,writeFileSync} from 'node:fs';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const baselinePath=join(root,'sdk/browser/corpus-zero-blocks.baseline.json');
const BASELINE_SHA256='605a0fb21574e98e59d7a381d008552ca4f05fa859dd342ae1abe0c1c9cc8de1';
export const canonical=value=>JSON.stringify(value,(_,x)=>x&&typeof x==='object'&&!Array.isArray(x)
  ?Object.fromEntries(Object.keys(x).sort().map(k=>[k,x[k]])):x);
export const hash=value=>createHash('sha256').update(value).digest('hex');
export const profiles=Object.freeze({
  workspace:Object.freeze({module:'WorkspaceCorpus',oldCount:847,literals:706,dataValues:802,requests:45,
    sourceSha:'3d1ca649b6b4dbdb5ec314909eaa1724e8d0ce3f7be7b44e405ee2252edffad0',
    vectorSha:'70e4ec78bc68029b81943fc2e0dcf4ae320c0e12ddf43d5fc4f1e8def0560147',
    helpers:22,referenceDepth:16,maximum:1104665}),
  query:Object.freeze({module:'WorkspaceQueryCorpus',oldCount:1573,literals:1055,dataValues:1510,requests:62,
    sourceSha:'8fd2f0b2946ebe658264b49cdfc1d79596166b5cc7c27200f7e6dbc309e30c30',
    vectorSha:'6ab01a1bbcfe4a1d270addc11578dee78e30ade05d8b5f99f6183ed7ede27394',
    helpers:29,referenceDepth:40,maximum:1166280}),
});
function profile(key){assert(Object.hasOwn(profiles,key),'closed corpus profile');return profiles[key];}
function regular(path){
  const stat=lstatSync(path);assert(stat.isFile()&&stat.size<=4*1024**2,'bounded regular corpus authority');
  assert.equal(realpathSync(path),resolve(path),'nonaliased corpus authority');
  const bytes=readFileSync(path);assert.equal(bytes.length,stat.size,'corpus authority size unchanged');return bytes;
}
export function decode(source){
  assert.equal(typeof source,'string');assert(Buffer.byteLength(source)<=4*1024**2,'bounded corpus source');
  const matches=[...source.matchAll(/\\semanticdata\{([^\n]+)\}\n/g)];
  assert.equal(matches.length,1,'one semantic corpus');const match=matches[0],data=JSON.parse(match[1]);
  assert.equal(canonical(data),match[1],'canonical corpus JSON');
  assert.deepEqual(Object.keys(data),['declarations','spec']);assert.equal(data.spec,'lexlean/semantic-module/1');
  assert(Array.isArray(data.declarations)&&data.declarations.length<=1700,'bounded corpus declaration inventory');
  return {data,json:match[1],wrapper:source.replace(match[1],'{}')};
}
const metadata=d=>Object.fromEntries(Object.entries(d).filter(([k])=>k!=='body'));
const call=name=>({kind:'call',function:{name},arguments:[]});
const append=(left,right)=>({kind:'primitive',operation:'append',result:{kind:'bytes'},arguments:[left,right]});
function balanced(parts){
  assert(parts.length>0);if(parts.length===1)return parts[0];const mid=Math.floor(parts.length/2);
  return append(balanced(parts.slice(0,mid)),balanced(parts.slice(mid)));
}
function evaluator(declarations,key){
  const spec=profile(key),defs=new Map(),cache=new Map();let total=0;
  for(const d of declarations){assert.equal(typeof d.name,'string');assert(!defs.has(d.name),'unique corpus declaration');defs.set(d.name,d);}
  function expand(name,active=[]){
    assert(!active.includes(name),'nonrecursive fixture data');
    assert(active.length<spec.referenceDepth,'bounded fixture reference depth');
    if(cache.has(name))return cache.get(name);
    const d=defs.get(name);assert(d,'required data definition');assert.equal(d.kind,'definition');
    assert.deepEqual(d.parameters,[]);assert.deepEqual(d.result,{kind:'bytes'});
    const result=term(d.body,[...active,name]);total+=result.length;assert(total<=192*1024**2,'bounded complete data expansion');
    cache.set(name,result);return result;
  }
  function term(x,active=[],depth=0){
    assert(x&&typeof x==='object'&&!Array.isArray(x),'closed fixture data');
    assert(depth<=32,'bounded fixture concatenation depth');let result;
    if(x.kind==='bytes'){
      assert.deepEqual(Object.keys(x).sort(),['hex','kind']);assert.equal(typeof x.hex,'string');
      assert.match(x.hex,/^(?:[0-9a-f]{2})*$/);assert(x.hex.length<=512,'bounded literal C initializer');
      result=Buffer.from(x.hex,'hex');
    }else if(x.kind==='call'){
      assert.deepEqual(Object.keys(x).sort(),['arguments','function','kind']);
      assert.deepEqual(x.arguments,[]);assert.deepEqual(Object.keys(x.function),['name']);
      result=expand(x.function.name,active);
    }else{
      assert.deepEqual(Object.keys(x).sort(),['arguments','kind','operation','result']);
      assert.equal(x.kind,'primitive');assert.equal(x.operation,'append');
      assert.deepEqual(x.result,{kind:'bytes'});assert.equal(x.arguments.length,2);
      result=Buffer.concat(x.arguments.map(v=>term(v,active,depth+1)));
    }
    assert(result.length<=spec.maximum,'bounded expanded fixture bytes');return result;
  }
  return {expand,defs};
}
export function factor(source,key){
  const spec=profile(key);assert.equal(hash(source),spec.sourceSha,'exact original corpus source authority');
  const decoded=decode(source),original=decoded.data.declarations;
  assert.equal(original.length,spec.oldCount);const names=new Set(original.map(d=>d.name));
  assert.equal(names.size,original.length);
  const additions=[],zero=new Map();
  function declare(length,body){
    const name='fixtureZeroRun'+length;assert(!names.has(name),'no old helper renaming');
    additions.push({kind:'definition',name,parameters:[],result:{kind:'bytes'},body});zero.set(length,name);return call(name);
  }
  function run(length){
    assert(Number.isSafeInteger(length)&&length>0&&length<=256,'bounded constant run');
    if(zero.has(length))return call(zero.get(length));
    if(length===1)return declare(1,{kind:'bytes',hex:'00'});
    if((length&(length-1))===0){const half=run(length/2);return declare(length,append(half,half));}
    const parts=[];for(let bit=7;bit>=0;bit--)if(length&(1<<bit))parts.push(run(1<<bit));
    return declare(length,balanced(parts));
  }
  const changed=original.map(d=>{
    if(d.body.kind!=='bytes')return d;
    assert.match(d.body.hex,/^(?:[0-9a-f]{2})*$/);assert(d.body.hex.length<=512);
    const bytes=Buffer.from(d.body.hex,'hex'),parts=[];let position=0;
    const latin=bytes.toString('latin1');for(const match of latin.matchAll(/\x00{16,}/g)){
      if(position<match.index)parts.push({kind:'bytes',hex:bytes.subarray(position,match.index).toString('hex')});
      parts.push(run(match[0].length));position=match.index+match[0].length;
    }
    if(!parts.length)return d;
    if(position<bytes.length)parts.push({kind:'bytes',hex:bytes.subarray(position).toString('hex')});
    return {...d,body:balanced(parts)};
  });
  assert.equal(additions.length,spec.helpers,'exact additional closed helper inventory');
  const old=evaluator(original,key),next=evaluator([...additions,...changed],key);
  const rows=original.map((d,index)=>{
    assert.equal(canonical(metadata(d)),canonical(metadata(changed[index])),'old declaration metadata');
    const bytes=d.kind==='definition'&&d.parameters.length===0&&d.result.kind==='bytes'?old.expand(d.name):null;
    if(bytes)assert(bytes.equals(next.expand(d.name)),'exact old data value');
    return {name:d.name,metadata_sha256:hash(canonical(metadata(d))),body_sha256:hash(canonical(d.body)),
      literal:d.body.kind==='bytes',value:bytes?{bytes:bytes.length,sha256:hash(bytes)}:null};
  });
  assert.equal(rows.filter(r=>r.value).length,spec.dataValues);
  assert.equal(rows.filter(r=>r.literal).length,spec.literals);
  const requests=original.filter(d=>d.name.startsWith('request'));
  assert.equal(requests.length,spec.requests);
  const vectorBytes=requests.map(d=>{const id=d.name.slice(7);
    return id+'\t'+old.expand(d.name).toString('hex')+'\t'+old.expand('response'+id).toString('hex')+'\n';}).join('');
  assert.equal(hash(vectorBytes),spec.vectorSha,'original ordered complete corpus');
  return {source:source.replace(decoded.json,canonical({...decoded.data,declarations:[...additions,...changed]})),
    baseline:{module:spec.module,old_source_sha256:spec.sourceSha,wrapper_sha256:hash(decoded.wrapper),
      old:rows,zero_helpers:additions}};
}
let baselineCache;
export function baseline(){
  if(!baselineCache){const bytes=regular(baselinePath);assert.equal(hash(bytes),BASELINE_SHA256,'frozen original corpus manifest');
    const parsed=JSON.parse(bytes);assert.equal(canonical(parsed)+'\n',bytes.toString());
    assert.equal(parsed.spec,'prismpm/workspace-corpus-zero-blocks/1');
    assert.equal(parsed.source_revision,'5adbcd9460556d476fd141c4364cecf582e2bca0');
    assert.deepEqual(Object.keys(parsed.modules),['query','workspace']);baselineCache=parsed;}
  return baselineCache;
}
export function zeroHelperNames(key){profile(key);return baseline().modules[key].zero_helpers.map(d=>d.name);}
export function validateFactoredCorpus(source,key){
  const spec=profile(key),authority=baseline().modules[key],{data,wrapper}=decode(source);
  assert.equal(hash(wrapper),authority.wrapper_sha256,'original corpus wrapper');
  assert.equal(authority.old.length,spec.oldCount);assert.equal(authority.zero_helpers.length,spec.helpers);
  assert.equal(data.declarations.length,spec.oldCount+spec.helpers,'complete original and zero-block closure');
  assert.equal(canonical(data.declarations.slice(0,spec.helpers)),canonical(authority.zero_helpers),'exact zero-block definitions');
  const old=data.declarations.slice(spec.helpers);const values=evaluator(data.declarations,key);
  const used=new Set();
  function mark(x){
    if(x&&typeof x==='object'){
      if(x.kind==='call'&&!x.function.module)used.add(x.function.name);
      for(const value of Object.values(x)){if(Array.isArray(value))value.forEach(mark);else mark(value);}
    }
  }
  for(let i=0;i<old.length;i++){
    const d=old[i],a=authority.old[i];assert.equal(d.name,a.name,'original ordered declaration identity');
    assert.equal(hash(canonical(metadata(d))),a.metadata_sha256,'original declaration metadata');
    if(!a.literal)assert.equal(hash(canonical(d.body)),a.body_sha256,'original nonliteral definition body');
    if(a.value){
      const bytes=values.expand(d.name);assert.equal(bytes.length,a.value.bytes,'exact old data length');
      assert.equal(hash(bytes),a.value.sha256,'exact old data value');
      if(a.literal)assert(bytes.length<=256,'old bounded literal helper value');
    }
    mark(d.body);
  }
  // New helper bodies are frozen; all must additionally be reachable from old data.
  const queue=[...used];for(let i=0;i<queue.length;i++){
    const d=values.defs.get(queue[i]);assert(d,'closed constant references');
    const before=used.size;mark(d.body);if(used.size!==before)for(const n of used)if(!queue.includes(n))queue.push(n);
  }
  for(const d of authority.zero_helpers)assert(used.has(d.name),'complete used zero-block closure');
  const requests=old.filter(d=>d.name.startsWith('request'));assert.equal(requests.length,spec.requests);
  assert.equal(hash(requests.map(d=>{const id=d.name.slice(7);return id+'\t'+values.expand(d.name).toString('hex')+'\t'+values.expand('response'+id).toString('hex')+'\n';}).join('')),spec.vectorSha,'exact original ordered vector digest');
  return {old:spec.oldCount,helpers:spec.helpers,dataValues:spec.dataValues,requests:spec.requests,vectorSha:spec.vectorSha};
}
function main(){
  assert(process.argv.length===3&&['--write','--check'].includes(process.argv[2]),'use --write or --check only');
  const paths=Object.entries(profiles).map(([key,spec])=>[key,join(root,'stdlib/src/Foundation/Browser/V1',spec.module+'.lex.tex')]);
  if(process.argv[2]==='--write'){
    assert(!lstatSync(baselinePath,{throwIfNoEntry:false}),'original baseline must be new');
    // Close both original sources before any write. This cannot mint new expectations from modified input.
    const rows=paths.map(([key,path])=>({key,path,original:regular(path)}));
    const prepared=rows.map(row=>({...row,...factor(row.original.toString(),row.key)}));
    const manifest={spec:'prismpm/workspace-corpus-zero-blocks/1',source_revision:'5adbcd9460556d476fd141c4364cecf582e2bca0',
      modules:Object.fromEntries(prepared.map(p=>[p.key,p.baseline]))};
    const bytes=canonical(manifest)+'\n';
    for(const p of prepared)assert(regular(p.path).equals(p.original),'original source unchanged before mechanical write');
    writeFileSync(baselinePath,bytes,{flag:'wx'});
    for(const p of prepared)writeFileSync(p.path,p.source);
    console.log(JSON.stringify({scope:'mechanical source rewrite only',baseline_sha256:hash(bytes),
      models:prepared.map(p=>({module:p.baseline.module,old:p.baseline.old.length,helpers:p.baseline.zero_helpers.length,sha256:hash(p.source)}))}));
  }else console.log(JSON.stringify(paths.map(([key,path])=>validateFactoredCorpus(regular(path).toString(),key))));
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url))main();
