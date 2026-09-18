// Private SDK binding only. All read/admission/pagination decisions are generated.
import {BrowserEffectError,bytesCopy,digestBytes,validateIdentity} from './identity.mjs';
import {openJournal} from './journal.mjs';
class QueryAdapterError extends Error{
 constructor(code,detail=null){super(code);this.name='QueryAdapterError';this.code=code;this.detail=detail;}
}
const fail=code=>new QueryAdapterError(code);
const storageFailureCodes=new Set(['invalid-input','crypto-unavailable','identity-corrupt','identity-exists','head-conflict','missing-object','object-corrupt','store-limit','store-policy-mismatch','store-closed','storage-blocked','storage-quota','storage-unavailable']);
function storageFailure(error){
 let code;try{if(error instanceof BrowserEffectError)code=error.code;}catch{/* Exception accessors are not trusted. */}
 return storageFailureCodes.has(code)?new BrowserEffectError(code):fail('storage-outcome-unknown');
}
const maximumOutstanding=2;
const same=(a,b)=>a.length===b.length&&a.every((x,i)=>x===b[i]);
const concatenate=(...parts)=>{const bytes=new Uint8Array(parts.reduce((n,p)=>n+p.length,0));let at=0;for(const part of parts){bytes.set(part,at);at+=part.length;}return bytes;};
const u16=n=>new Uint8Array([n>>>8,n&255]),u24=n=>new Uint8Array([n>>>16,n>>>8&255,n&255]);
const read16=(b,at)=>b[at]*256+b[at+1],read24=(b,at)=>b[at]*65536+b[at+1]*256+b[at+2];
const hashBytes=value=>{if(typeof value!=='string'||!/^sha256:[0-9a-f]{64}$/.test(value))throw fail('invalid-effect-result');return Uint8Array.from(value.slice(7).match(/../g),x=>parseInt(x,16));};
function interpreter(module){
 if(!(module instanceof WebAssembly.Module)||WebAssembly.Module.imports(module).length!==0)throw fail('invalid-generated-module');
 return request=>{
  if(request.length>1166279)throw fail('request-limit');
  try{const instance=new WebAssembly.Instance(module,{}),{memory,holo_alloc:allocate,holo_run:run}=instance.exports;
   if(!(memory instanceof WebAssembly.Memory)||typeof allocate!=='function'||typeof run!=='function')throw fail('invalid-generated-module');
   const pointer=allocate(request.length)>>>0;if(pointer+request.length>memory.buffer.byteLength)throw fail('invalid-generated-output');
   new Uint8Array(memory.buffer,pointer,request.length).set(request);
   const packed=BigInt.asUintN(64,run(pointer,request.length)),offset=Number(packed>>32n),length=Number(packed&0xffffffffn);
   if(length>66803||offset+length>memory.buffer.byteLength||memory.buffer.byteLength>512*65536)throw fail('invalid-generated-output');
   return new Uint8Array(memory.buffer,offset,length).slice();
  }catch(error){if(error instanceof QueryAdapterError)throw error;throw fail('generated-execution-failed');}
 };
}
function capture(value){
 try{
  if(value===null||typeof value!=='object'||Object.getPrototypeOf(value)!==Object.prototype)throw fail('invalid-input');
  const descriptors=Object.getOwnPropertyDescriptors(value),keys=Reflect.ownKeys(descriptors);
  if(keys.some(k=>typeof k!=='string')||keys.sort().join(',')!=='cursor,table,workspace')throw fail('invalid-input');
  for(const key of keys)if(!Object.hasOwn(descriptors[key],'value')||!descriptors[key].enumerable)throw fail('invalid-input');
  const table=descriptors.table.value,workspace=bytesCopy(descriptors.workspace.value,32),cursor=bytesCopy(descriptors.cursor.value,135);
  if(!Number.isInteger(table)||table<0||table>255||workspace.length!==32)throw fail('invalid-input');
  return concatenate([table],workspace,u16(cursor.length),cursor);
 }catch{throw fail('invalid-input');}
}
function page(bytes){
 if(bytes.length===1&&bytes[0]>=1&&bytes[0]<=13)throw new QueryAdapterError('query-rejected',bytes[0]);
 if(bytes.length<76||bytes[0]!==0||bytes[1]>1)throw fail('invalid-generated-output');
 const cursorLength=read16(bytes,71),rowsAt=73+cursorLength;
 if(![0,135].includes(cursorLength)||rowsAt+3>bytes.length)throw fail('invalid-generated-output');
 const rowLength=read24(bytes,rowsAt);
 if(bytes.length!==rowsAt+3+rowLength||bytes[70]>16)throw fail('invalid-generated-output');
 return Object.freeze({table:bytes[1],workspace:bytes.slice(2,34),headId:bytes.slice(34,66),total:read16(bytes,66),offset:read16(bytes,68),count:bytes[70],cursor:bytes.slice(73,rowsAt),rows:bytes.slice(rowsAt+3)});
}
class Queries{
 #identity;#session;#journal;#call;#closed=false;#outstanding=0;#active=false;#pending=[];
 constructor(module,identity,session,journal){this.#call=interpreter(module);this.#identity=identity;this.#session=session;this.#journal=journal;}
 #open(){if(this.#closed)throw fail('adapter-closed');}
 #reserve(){this.#open();if(this.#outstanding>=maximumOutstanding)throw fail('adapter-busy');this.#outstanding++;}
 #queue(operation){return new Promise((resolve,reject)=>{this.#pending.push({operation,resolve,reject});this.#drain();});}
 #drain(){if(this.#active||this.#pending.length===0)return;const entry=this.#pending.shift();this.#active=true;
  const finish=()=>{this.#outstanding--;this.#active=false;this.#drain();};
  Promise.resolve().then(()=>{this.#open();return entry.operation();}).then(value=>{finish();entry.resolve(value);},error=>{finish();entry.reject(error);});
 }
 close(){if(arguments.length!==0)throw fail('invalid-input');this.#closed=true;const pending=this.#pending;this.#pending=[];for(const entry of pending){this.#outstanding--;entry.operation=null;entry.reject(fail('adapter-closed'));}}
 query(value){
  let intent,reserved=false;
  try{if(arguments.length!==1)throw fail('invalid-input');this.#reserve();reserved=true;intent=capture(value);this.#open();}
  catch(error){if(reserved)this.#outstanding--;return Promise.reject(error);}
  return this.#queue(async()=>{
   this.#open();await this.#journal.refresh();this.#open();
   const {head,state}=this.#journal.snapshot(),principal=hashBytes(this.#identity.principal),session=this.#session.slice();
   const headId=hashBytes(await digestBytes(head));this.#open();
   // Re-authenticate after the asynchronous digest, before disclosing any rows.
   await this.#journal.refresh();this.#open();const current=this.#journal.snapshot();
   if(!same(head,current.head)||!same(state,current.state))throw fail('query-context-changed');
   const request=concatenate([80,87,81,1],principal,session,headId,u24(head.length),u24(state.length),head,state,u16(intent.length),intent);
   const result=page(this.#call(request));this.#open();return result;
  });
 }
}
// Only trusted bootstrap supplies verified modules and storage. Public callers
// receive query/close only; no snapshot/head/state or caller-selected identity.
export async function openQueries({queryModule,journalModule,store,headName}){
 interpreter(queryModule);let identity;
 try{identity=await store.loadIdentity();}catch(error){throw storageFailure(error);}
 if(identity===null)throw fail('identity-missing');const captured=await validateIdentity(identity);
 let session;try{session=crypto.getRandomValues(new Uint8Array(32));}catch{throw new BrowserEffectError('crypto-unavailable');}
 const journal=await openJournal(journalModule,store,headName);
 return Object.freeze(new Queries(queryModule,captured,session,journal));
}
