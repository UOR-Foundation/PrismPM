// Generic private host binding, not application policy or read admission.
import {BrowserEffectError,bytesCopy,digestBytes,signBytes,validateIdentity} from './identity.mjs';
import {openJournal} from './journal.mjs';

class CommandAdapterError extends Error {
  constructor(code,detail=null){super(code);this.name='CommandAdapterError';this.code=code;this.detail=detail;}
}
const fail=code=>new CommandAdapterError(code);
const storageFailureCodes=new Set(['invalid-input','crypto-unavailable','identity-corrupt','identity-exists','head-conflict','missing-object','object-corrupt','store-limit','store-policy-mismatch','store-closed','storage-blocked','storage-quota','storage-unavailable']);
function storageFailure(error){
  let code;try{if(error instanceof BrowserEffectError)code=error.code;}catch{/* Exception accessors are not trusted. */}
  return storageFailureCodes.has(code)?new BrowserEffectError(code):fail('storage-outcome-unknown');
}
const context='prismpm/workspace-event/1';
// Host admission budget: one active operation and at most one retained waiter.
// This does not alter any modeled workspace/event/message domain limit.
const outstandingMaximum=2;
const hashBytes=value=>{
  if(typeof value!=='string'||!/^sha256:[0-9a-f]{64}$/.test(value))throw fail('invalid-effect-result');
  return Uint8Array.from(value.slice(7).match(/../g),byte=>parseInt(byte,16));
};
function concatenate(...parts){
  const result=new Uint8Array(parts.reduce((size,part)=>size+part.length,0));
  let offset=0;for(const part of parts){result.set(part,offset);offset+=part.length;}return result;
}
const u16=value=>new Uint8Array([value>>>8,value&255]);
const u24=value=>new Uint8Array([value>>>16,value>>>8&255,value&255]);
const read24=(bytes,at)=>bytes[at]*65536+bytes[at+1]*256+bytes[at+2];
function interpreter(module){
  if(!(module instanceof WebAssembly.Module)||WebAssembly.Module.imports(module).length!==0)throw fail('invalid-generated-module');
  return request=>{
    if(request.length>139873)throw fail('request-limit');
    try{
      const instance=new WebAssembly.Instance(module,{});
      const {memory,holo_alloc:allocate,holo_run:run}=instance.exports;
      if(!(memory instanceof WebAssembly.Memory)||typeof allocate!=='function'||typeof run!=='function')throw fail('invalid-generated-module');
      const pointer=allocate(request.length)>>>0;
      if(pointer+request.length>memory.buffer.byteLength)throw fail('invalid-generated-output');
      new Uint8Array(memory.buffer,pointer,request.length).set(request);
      const packed=BigInt.asUintN(64,run(pointer,request.length)),offset=Number(packed>>32n),length=Number(packed&0xffffffffn);
      if(length>74243||offset+length>memory.buffer.byteLength||memory.buffer.byteLength>64*65536)throw fail('invalid-generated-output');
      return new Uint8Array(memory.buffer,offset,length).slice();
    }catch(error){if(error instanceof CommandAdapterError)throw error;throw fail('generated-execution-failed');}
  };
}
function plan(bytes){
  if(bytes.length===1&&bytes[0]>=1&&bytes[0]<=12)throw new CommandAdapterError('command-rejected',bytes[0]);
  if(bytes.length<6||bytes[0]!==0)throw fail('invalid-generated-output');
  const length=read24(bytes,1),at=4+length;
  if(length>69874||at+2>bytes.length)throw fail('invalid-generated-output');
  const effectLength=bytes[at]*256+bytes[at+1];
  if(effectLength>4363||at+2+effectLength!==bytes.length)throw fail('invalid-generated-output');
  return {pending:bytes.slice(4,at),effect:bytes.slice(at+2)};
}
const completion=(operation,pending,head,nonce,key,input,result)=>concatenate(
  [operation],u24(pending.length),pending,u24(head.length),head,nonce,key,u16(input.length),input,u16(result.length),result);
function capture(value){
  try{
    if(value===null||typeof value!=='object'||Object.getPrototypeOf(value)!==Object.prototype)throw fail('invalid-input');
    const descriptors=Object.getOwnPropertyDescriptors(value),keys=Reflect.ownKeys(descriptors);
    if(keys.some(key=>typeof key!=='string')||keys.sort().join(',')!=='action,body,workspace')throw fail('invalid-input');
    for(const key of keys)if(!Object.hasOwn(descriptors[key],'value')||!descriptors[key].enumerable)throw fail('invalid-input');
    const action=descriptors.action.value,body=bytesCopy(descriptors.body.value,4096),workspace=bytesCopy(descriptors.workspace.value,32);
    if(!Number.isInteger(action)||action<0||action>255||workspace.length!==32)throw fail('invalid-input');
    return {action,body,workspace};
  }catch{throw fail('invalid-input');}
}

class Commands {
  #identity;#journal;#call;#closed=false;#requiresRefresh=false;
  #outstanding=0;#active=false;#pending=[];
  constructor(module,identity,journal){this.#call=interpreter(module);this.#identity=identity;this.#journal=journal;}
  #open(){if(this.#closed)throw fail('adapter-closed');}
  #reserve(){this.#open();if(this.#outstanding>=outstandingMaximum)throw fail('adapter-busy');this.#outstanding++;}
  #queue(operation){
    return new Promise((resolve,reject)=>{this.#pending.push({operation,resolve,reject});this.#drain();});
  }
  #drain(){
    if(this.#active||this.#pending.length===0)return;
    const entry=this.#pending.shift();this.#active=true;
    const finish=()=>{this.#outstanding--;this.#active=false;this.#drain();};
    Promise.resolve().then(()=>{this.#open();return entry.operation();}).then(
      value=>{finish();entry.resolve(value);},error=>{finish();entry.reject(error);});
  }
  async #refresh(){
    this.#open();this.#requiresRefresh=true;
    await this.#journal.refresh();this.#open();this.#requiresRefresh=false;
  }
  status(){
    if(arguments.length!==0)throw fail('invalid-input');this.#open();
    return Object.freeze({principal:this.#identity.principal,requiresRefresh:this.#requiresRefresh});
  }
  refresh(){
    if(arguments.length!==0)return Promise.reject(fail('invalid-input'));
    try{this.#reserve();}catch(error){return Promise.reject(error);}
    return this.#queue(async()=>{await this.#refresh();return this.status();});
  }
  close(){
    if(arguments.length!==0)throw fail('invalid-input');this.#closed=true;
    const pending=this.#pending;this.#pending=[];
    for(const entry of pending){this.#outstanding--;entry.operation=null;entry.reject(fail('adapter-closed'));}
  }
  submit(value){
    let captured,reserved=false;
    try{this.#open();if(arguments.length!==1)throw fail('invalid-input');this.#reserve();reserved=true;captured=capture(value);this.#open();}
    catch(error){if(reserved)this.#outstanding--;return Promise.reject(error);}
    return this.#queue(async()=>{
      this.#open();if(this.#requiresRefresh)throw fail('refresh-required');
      const key=this.#identity.publicKey.slice(),principal=hashBytes(this.#identity.principal),head=this.#journal.snapshot().head;
      let nonce;
      try{nonce=crypto.getRandomValues(new Uint8Array(32));}catch{throw new BrowserEffectError('crypto-unavailable');}
      const raw=concatenate(nonce,key,principal,captured.workspace,u24(head.length),head,[captured.action],u16(captured.body.length),captured.body);
      let current=plan(this.#call(concatenate([0],raw)));
      const hashInput=current.effect.slice(),digest=hashBytes(await digestBytes(hashInput));this.#open();
      await this.#refresh();
      current=plan(this.#call(completion(1,current.pending,this.#journal.snapshot().head,nonce,key,hashInput,digest)));
      const signInput=current.effect.slice(),signature=await signBytes(this.#identity,context,signInput);this.#open();
      await this.#refresh();
      current=plan(this.#call(completion(2,current.pending,this.#journal.snapshot().head,nonce,key,signInput,signature)));
      this.#open();this.#requiresRefresh=true;
      try{
        await this.#journal.append(current.effect);
        if(this.#closed)throw fail('commit-outcome-unknown');
        this.#requiresRefresh=false;return Object.freeze({committed:true,principal:this.#identity.principal,requiresRefresh:false});
      }catch(error){if(this.#closed)throw fail('commit-outcome-unknown');throw error;}
    });
  }
}

// SDK bootstrap only. Product callers receive only this returned private API.
export async function openCommands({commandModule,journalModule,store,headName}){
  interpreter(commandModule);
  let identity;
  try{identity=await store.loadIdentity();}
  catch(error){throw storageFailure(error);}
  if(identity===null)throw fail('identity-missing');
  const captured=await validateIdentity(identity);
  const journal=await openJournal(journalModule,store,headName);
  return Object.freeze(new Commands(commandModule,captured,journal));
}
