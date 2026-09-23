// Independent test-only input authoring and explicit expected decisions.
import assert from 'node:assert/strict';
export const bytes=(size,value=1)=>new Uint8Array(size).fill(value);
export function encode(value) {
  const head=(major,size)=>{if(size<24)return Buffer.from([major*32+size]);if(size<256)return Buffer.from([major*32+24,size]);const b=Buffer.alloc(size<65536?3:5);b[0]=major*32+(size<65536?25:26);if(size<65536)b.writeUInt16BE(size,1);else b.writeUInt32BE(size,1);return b;};
  const item=x=>{if(Number.isInteger(x)&&x>=0&&x<=0xffffffff)return head(0,x);if(typeof x==='string'){const b=Buffer.from(x);return Buffer.concat([head(3,b.length),b]);}if(x instanceof Uint8Array)return Buffer.concat([head(2,x.length),x]);assert.ok(Array.isArray(x));return Buffer.concat([head(4,x.length),...x.map(item)]);};
  return new Uint8Array(item(value));
}
const digest='sha256:'+'a'.repeat(64), app=bytes(32,1), manifestId=bytes(32,2), session=bytes(32,3), artifact=bytes(32,4), requested=bytes(32,5),effective=bytes(32,6),key=bytes(65,7);key[0]=4;
const signature=bytes(64,8), clone=structuredClone;
export const base=()=>{
 const manifest=[app,manifestId,[67108864,67108864,16384],[
  ['digest',[2]],['guest',[0,[artifact,'Fixture.echoBytes','fixture/1',3,3,16]]],['random',[1]],
  ['sign',[3,[key,'budget/sign/1']]],['store',[5,['budget-store',3,4096,64]]],['verify',[4,[key,'budget/verify/1']]],
 ]];
 return [[app,manifestId,requested,effective,manifest[3].map(([name])=>[name,3])],manifest,requested,effective,session,[app,manifestId,session,0,'digest',[2,bytes(1)]]].map(value=>clone(value));
};
const vector=(id,value,expected)=>({id,request:encode(value),response:Uint8Array.of(expected)});
const select=(value,resource,operation)=>{value[5][4]=resource;value[5][5]=operation;return value;};
const payloadOperation=(kind,size)=>kind===0?[0,[artifact,'Fixture.echoBytes','fixture/1',bytes(size)]]:kind===1?[1,size]:kind===4?[4,[bytes(size),signature]]:[kind,bytes(size)];
const row=(value,name)=>value[0][4].find(item=>item[0]===name);
export function corpus(){
 const out=[],add=(id,value,expected)=>out.push(vector(id,value,expected));
 add('Base',base(),1);
 {const value=base();value[5][3]=0xfffffffe;add('MaximumOperationCounter',value,1);}
 for(const size of[128,129]){const value=base(),name='digest'.padEnd(size,'x');value[0][4][0][0]=name;value[1][3][0][0]=name;value[5][4]=name;add('ResourceName'+size,value,size===128?1:0);}
 for(const [name,kind]of [['guest',0],['random',1],['digest',2],['sign',3],['verify',4]]){
  for(const size of[0,1,3,4])add(name+'Length'+size,select(base(),name,payloadOperation(kind,size)),size<=3&&(size>0||kind!==1)?1:0);
  for(const maximum of[0,1,3,4,65536,65537,1048576,1048577,0xffffffff]){
   const value=base();row(value,name)[1]=maximum;select(value,name,payloadOperation(kind,1));
   const accepted=kind===0?maximum===3:maximum>0&&maximum<=(kind===1?65536:1048576);
   add(name+'Budget'+maximum,value,accepted?1:0);
  }
 }
 for(const operation of[[5,digest],[6,'head'],[7,['head',[0],digest,[bytes(3)]]]])add('StoreKind'+operation[0],select(base(),'store',operation),1);
 add('StoreEmptyCommit',select(base(),'store',[7,['head',[0],digest,[]]]),0);
 for(const maximum of[0,1,2,4,1048577]){const value=base();row(value,'store')[1]=maximum;add('StoreMismatch'+maximum,value,0);}
 for(const [name,change]of[
  ['BudgetApplication',v=>{v[0][0][0]^=1;}],['BudgetManifest',v=>{v[0][1][0]^=1;}],
  ['RequestedPolicy',v=>{v[2]=bytes(32,9);}],['EffectivePolicy',v=>{v[3]=bytes(32,9);}],
  ['RequestApplication',v=>{v[5][0]=bytes(32,9);}],['RequestManifest',v=>{v[5][1]=bytes(32,9);}],['RequestSession',v=>{v[5][2]=bytes(32,9);}],
  ['CounterOverflow',v=>{v[5][3]=0xffffffff;}],['UnknownResource',v=>{v[5][4]='absent';}],
  ['MissingRow',v=>{v[0][4].pop();}],['ExtraRow',v=>{v[0][4].push(['zzz',1]);}],['DuplicateRow',v=>{v[0][4][1]=clone(v[0][4][0]);}],
  ['BudgetOrder',v=>{v[0][4].reverse();}],['GrantOrder',v=>{v[1][3].reverse();}],['BothOrder',v=>{v[0][4].reverse();v[1][3].reverse();}],
  ['EmptyRows',v=>{v[0][4]=[];}],['EmptyBoth',v=>{v[0][4]=[];v[1][3]=[];}],['WrongRowName',v=>{v[0][4][1][0]='foreign';}],
  ['DuplicateGrant',v=>{v[1][3][1]=clone(v[1][3][0]);}],['ManifestInvalidLimit',v=>{v[1][2][0]=0;}],
  ['InvalidUnusedGuest',v=>{v[1][3][1][1][1][0]=bytes(31);}],['InvalidUnusedStore',v=>{v[1][3][4][1][1][2]=0;}],
  ['InvalidUnusedKey',v=>{v[1][3][3][1][1][0][0]=3;}],['InvalidUnusedContext',v=>{v[1][3][3][1][1][1]='bad..context';}],
 ]){const value=base();change(value);add(name,value,0);}
 for(const at of[0,1,2,3])for(const size of[0,31,33]){const value=base();value[0][at]=bytes(size);add('BudgetRef'+at+'Width'+size,value,0);}
 for(const at of[2,3,4])for(const size of[0,31,33]){const value=base();value[at]=bytes(size);add('ContextRef'+at+'Width'+size,value,0);}
 const operations=[payloadOperation(0,1),payloadOperation(1,1),payloadOperation(2,1),payloadOperation(3,1),payloadOperation(4,1),[5,digest],[6,'head'],[7,['head',[0],digest,[bytes(1)]]]];
 for(const [resource,accepted]of [['guest',[0]],['random',[1]],['digest',[2]],['sign',[3]],['verify',[4]],['store',[5,6,7]]])for(const operation of operations)if(!accepted.includes(operation[0]))add('Cross'+resource+operation[0],select(base(),resource,operation),0);
 for(const [name,change]of[
  ['GuestArtifact',v=>{v[5][5][1][0]=bytes(32,9);}],['GuestEntry',v=>{v[5][5][1][1]='Fixture.other';}],['GuestProtocol',v=>{v[5][5][1][2]='other/1';}],
 ]){const value=select(base(),'guest',payloadOperation(0,1));change(value);add(name,value,0);}
 for(const size of[0,63,65])add('SignatureWidth'+size,select(base(),'verify',[4,[bytes(1),bytes(size)]]),size>64?2:0);
 for(const [name,operation]of[
  ['ObjectInvalid',[5,'invalid']],['HeadInvalid',[6,'bad/name']],['CommitDuplicate',[7,['head',[0],digest,[bytes(1),bytes(1)]]]],
  ['CommitTooMany',[7,['head',[0],digest,Array.from({length:17},(_,i)=>bytes(1,i))]]],['CommitOverObject',[7,['head',[0],digest,[bytes(4)]]]],
 ])add(name,select(base(),'store',operation),name==='CommitTooMany'?2:0);
 const encoded=encode(base());out.push({id:'Trailing',request:Uint8Array.from([...encoded,0]),response:Uint8Array.of(2)});
 for(const [id,request]of [['Empty',[]],['Indefinite',[0x9f,0xff]],['ShortArray',[0x86]],['Nonminimal',[0x98,6,...encoded.slice(1)]]])out.push({id,request:Uint8Array.from(request),response:Uint8Array.of(2)});
 assert.equal(new Set(out.map(row=>row.id)).size,out.length);return out;
}
export function maximumCorpus(){
 const out=[],add=(id,value,expected)=>out.push(vector(id,value,expected));
 for(const [name,kind,maximum]of [['guest',0,2097152],['random',1,65536],['digest',2,1048576],['sign',3,1048576],['verify',4,1048576]])for(const delta of[0,1]){
  const value=base();row(value,name)[1]=maximum;if(kind===0){value[1][3][1][1][1][3]=maximum;value[1][3][1][1][1][5]=64;}
  select(value,name,payloadOperation(kind,maximum+delta));add('Maximum'+name+(delta?'Over':''),value,delta?(kind===1?0:2):1);
 }
 for(const [name,kind,maximum]of [['random',1,65536],['digest',2,1048576],['sign',3,1048576],['verify',4,1048576]]){
  const value=base();row(value,name)[1]=maximum-1;select(value,name,payloadOperation(kind,maximum));add('MaximumAdmittedOneOverBudget'+name,value,0);
 }
 const value=base();value[1][3]=Array.from({length:64},(_,i)=>['r'+String(i).padStart(3,'0').padEnd(127,'x'),[2]]);
 value[0][4]=value[1][3].map(([name])=>[name,1048576]);select(value,value[1][3][63][0],[2,bytes(1048576)]);add('Maximum64Resources',value,1);
 for(const position of[0,31,63]){const wrong=clone(value);wrong[0][4][position][1]=0;add('MaximumBadRow'+position,wrong,0);}
 const extra=clone(value);extra[0][4].push(['zzzz',1]);add('Maximum65BudgetRows',extra,0);
 const grantExtra=clone(extra);grantExtra[1][3].push(['zzzz',[2]]);add('Maximum65GrantRows',grantExtra,2);
 for(const delta of[0,1]){
  const commit=base();row(commit,'store')[1]=1048576;commit[1][3][4][1][1][1]=1048576;
  select(commit,'store',[7,['h'.repeat(128),[1,digest],digest,Array.from({length:16},(_,i)=>bytes(1048576+(i===15?delta:0),i))]]);
  add('MaximumCommit16Objects'+(delta?'Over':''),commit,delta?2:1);
 }
 for(const delta of[0,1]){
  const combined=clone(value),maximum=1048576-delta;
  combined[1][3][63][1]=[5,['n'.repeat(128),maximum,4096,64]];combined[0][4][63][1]=maximum;
  select(combined,combined[1][3][63][0],[7,['h'.repeat(128),[1,digest],digest,Array.from({length:16},(_,i)=>bytes(maximum+(delta&&i===15?1:0),i))]]);
  add('Maximum64ResourcesCommit16Objects'+(delta?'OneOver':''),combined,delta?0:1);
 }
 const frame=bytes(67108864,0);frame.set(encode(base()));out.push({id:'MaximumRejectedFrame',request:frame,response:Uint8Array.of(2)});
 return out;
}
export const overFrame=()=>({id:'OverFrame',request:bytes(67108865),response:Uint8Array.of(2)});
