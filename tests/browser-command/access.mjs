import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {withBrowser} from '../../sdk/browser/browser-test-server.mjs';
async function fixture({commandBytes,journalBytes}){
 const {openCommands}=await import('/commands.mjs'),{createIdentity}=await import('/identity.mjs'),{openStore}=await import('/store.mjs');
 const commandModule=await WebAssembly.compile(new Uint8Array(commandBytes)),journalModule=await WebAssembly.compile(new Uint8Array(journalBytes));
 const store=await openStore('command-admission-'+crypto.randomUUID()),owner=await createIdentity(),outsider=await createIdentity(),member=await createIdentity();await store.saveIdentity(owner);
 const adapters=[],empty=()=>new Uint8Array(),workspace=crypto.getRandomValues(new Uint8Array(32));
 const wrap=identity=>new Proxy(store,{get(target,key){if(key==='loadIdentity')return async()=>identity;const value=target[key];return typeof value==='function'?value.bind(target):value;}});
 const open=async identity=>{const adapter=await openCommands({commandModule,journalModule,store:wrap(identity),headName:'workspace'});adapters.push(adapter);return adapter;};
 const keys=(value,expected)=>{if(value===null||typeof value!=='object'||Object.keys(value).sort().join(',')!==expected||!Object.isFrozen(value))throw Error('command result disclosed non-status fields');};
 const status=(value,identity)=>{keys(value,'principal,requiresRefresh');if(value.principal!==identity.principal||typeof value.requiresRefresh!=='boolean')throw Error('status identity binding');};
 const denied=async promise=>{try{await promise;}catch(error){if(error.code!=='model-rejected')throw error;return;}throw Error('unauthorized command admitted');};
 const input=(action,body=empty())=>({action,body,workspace});
 try{
  const authority=await open(owner);await authority.submit(input(0));await authority.submit(input(4,new TextEncoder().encode('retained private message')));
  for(const identity of[outsider,member]){
   if(identity===member){await authority.submit(input(1,Uint8Array.from(member.principal.slice(7).match(/../g),x=>parseInt(x,16))));await authority.submit(input(3,Uint8Array.from(member.principal.slice(7).match(/../g),x=>parseInt(x,16))));}
   const adapter=await open(identity);
   if(typeof adapter.snapshot==='function')throw Error('command snapshot disclosed raw state to outsider/revoked principal');
   const methods=Object.getOwnPropertyNames(Object.getPrototypeOf(adapter)).sort().join(',');if(methods!=='close,constructor,refresh,status,submit')throw Error('closed product command methods');
   status(adapter.status(),identity);status(await adapter.refresh(),identity);
   await denied(adapter.submit(input(4,new TextEncoder().encode('denied'))));status(adapter.status(),identity);status(await adapter.refresh(),identity);
   if(adapter.close()!==undefined)throw Error('close disclosed data');
  }
  status(authority.status(),owner);status(await authority.refresh(),owner);
  const ack=await authority.submit(input(4,new TextEncoder().encode('acknowledged')));keys(ack,'committed,principal,requiresRefresh');if(ack.committed!==true||ack.requiresRefresh!==false||ack.principal!==owner.principal)throw Error('commit acknowledgement');
  return{outsider:true,revoked:true,acknowledgement:true};
 }finally{for(const adapter of adapters)adapter.close();store.close();}
}
export async function verifyAccess(build){
 const source=readFileSync(new URL('../../sdk/browser/commands.mjs',import.meta.url),'utf8');
 const bytes=name=>Array.from(name==='guest'?build.wasmBytes:build.journalWasm);
 const invoke=body=>withBrowser(async({browser,baseURL})=>{const page=await browser.newPage();await page.route('**/commands.mjs',r=>r.fulfill({status:200,contentType:'text/javascript',body}));await page.goto(baseURL);return page.evaluate(fixture,{commandBytes:bytes('guest'),journalBytes:bytes('journal-guest')});});
 assert.deepEqual(await invoke(source),{outsider:true,revoked:true,acknowledgement:true});
 const needle='return Object.freeze({principal:this.#identity.principal,requiresRefresh:this.#requiresRefresh});';
 const changed=source.replace(needle,'return Object.freeze({...this.#journal.snapshot(),principal:this.#identity.principal,requiresRefresh:this.#requiresRefresh});');
 assert.notEqual(changed,source);await assert.rejects(invoke(changed),/command result disclosed non-status fields/);
}
