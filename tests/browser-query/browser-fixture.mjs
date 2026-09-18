export async function queryBrowserFixture({queryBytes,journalBytes,commandBytes,probeBytes,vectors}){
 const {openQueries}=await import('/queries.mjs'),{openCommands}=await import('/commands.mjs');
 const {createIdentity}=await import('/identity.mjs'),{openStore}=await import('/store.mjs');
 const queryModule=await WebAssembly.compile(new Uint8Array(queryBytes)),journalModule=await WebAssembly.compile(new Uint8Array(journalBytes)),commandModule=await WebAssembly.compile(new Uint8Array(commandBytes));
 const probeModule=await WebAssembly.compile(new Uint8Array(probeBytes));
 const empty=()=>new Uint8Array(),text=s=>new TextEncoder().encode(s),hex=b=>Array.from(b,n=>n.toString(16).padStart(2,'0')).join(''),unhex=s=>Uint8Array.from(s.match(/../g)??[],n=>parseInt(n,16));
 const check=(condition,label)=>{if(!condition)throw Error(label);},calls=[],cases=[],stores=[],adapters=[];const diagnostics=new Set();let maximum=0;
 const Instance=WebAssembly.Instance;
 WebAssembly.Instance=class{constructor(...args){const real=new Instance(...args),exports={...real.exports};exports.holo_run=(pointer,length)=>{
  const request=hex(new Uint8Array(exports.memory.buffer,pointer,length)),result=real.exports.holo_run(pointer,length),packed=BigInt.asUintN(64,result),offset=Number(packed>>32n),size=Number(packed&0xffffffffn);
  calls.push({kind:args[0]===queryModule?'Query':args[0]===commandModule?'Command':args[0]===probeModule?'Decode':'Journal',request,response:hex(new Uint8Array(exports.memory.buffer,offset,size))});if(args[0]===queryModule)maximum=Math.max(maximum,exports.memory.buffer.byteLength);return result;
 };return{exports};}};
 let restore=()=>{};
 const wrapStore=(store,overrides)=>({loadIdentity:()=>store.loadIdentity(),readHead:(...args)=>store.readHead(...args),readObject:(...args)=>store.readObject(...args),commit:(...args)=>store.commit(...args),...overrides});
 async function rejects(promise,code,detail){let caught;try{await promise;}catch(error){caught=error;}check(caught?.code===code,'expected '+code+', got '+caught?.code);if(detail!==undefined)check(caught.detail===detail,'expected detail '+detail+', got '+caught.detail);if(caught.name==='QueryAdapterError')diagnostics.add(caught.code);return caught;}
 async function fixture(){const namespace='queries-'+crypto.randomUUID(),store=await openStore(namespace);stores.push(store);const identity=await createIdentity();await store.saveIdentity(identity);const workspace=crypto.getRandomValues(new Uint8Array(32));
  const commands=await openCommands({commandModule,journalModule,store,headName:'workspace'});adapters.push(commands);
  const submit=(action,body=empty())=>commands.submit({action,body,workspace});await submit(0);
  const open=async(selected=store)=>{const adapter=await openQueries({queryModule,journalModule,store:selected,headName:'workspace'});adapters.push(adapter);return adapter;};const query=await open();
  return{namespace,store,identity,workspace,commands,submit,open,query,intent:(table=0,cursor=empty(),space=workspace)=>({table,cursor,workspace:space})};
 }
 function digestGate(){const original=crypto.subtle.digest,bound=original.bind(crypto.subtle);let entered,release,heads=0;const ready=new Promise(r=>{entered=r;}),pause=new Promise(r=>{release=r;});
  crypto.subtle.digest=async(...args)=>{const bytes=new Uint8Array(args[1]),isHead=bytes.length>=38&&hex(bytes.subarray(0,4))==='50574a01';if(isHead)heads++;const owned=isHead&&heads===2;const result=await bound(...args);if(owned){entered();await pause;}return result;};
  restore=()=>{crypto.subtle.digest=original;};return{ready,release,restore:()=>{restore();restore=()=>{};}};
 }
 try{
  check(vectors.length===62,'all62 literal cases');
  for(const vector of vectors)for(let repeat=0;repeat<2;repeat++){
   const request=unhex(vector.request),instance=new WebAssembly.Instance(queryModule,{});
   if(request.length>1166279){check(vector.id==='OverAllocation','known overcap');let caught;try{instance.exports.holo_alloc(request.length);}catch(error){caught=error;}check(caught instanceof WebAssembly.RuntimeError,'actual query allocation trap');continue;}
   const pointer=instance.exports.holo_alloc(request.length);new Uint8Array(instance.exports.memory.buffer,pointer,request.length).set(request);const packed=BigInt.asUintN(64,instance.exports.holo_run(pointer,request.length));check(hex(new Uint8Array(instance.exports.memory.buffer,Number(packed>>32n),Number(packed&0xffffffffn)))===vector.response,'literal '+vector.id);
  }
  cases.push('all62 vectors twice, complete64-member and256-message traversal');
  for(let value=0;value<258;value++)for(let repeat=0;repeat<2;repeat++){const bytes=value<256?new Uint8Array([value]):value===256?empty():new Uint8Array([1,2]),instance=new WebAssembly.Instance(probeModule,{}),pointer=instance.exports.holo_alloc(bytes.length);new Uint8Array(instance.exports.memory.buffer,pointer,bytes.length).set(bytes);const packed=BigInt.asUintN(64,instance.exports.holo_run(pointer,bytes.length)),actual=hex(new Uint8Array(instance.exports.memory.buffer,Number(packed>>32n),Number(packed&0xffffffffn)));check(actual==='00'+(value<256?value:0).toString(16).padStart(2,'0'),'exact independent octet oracle '+value);check(instance.exports.memory.buffer.byteLength<=16*65536,'diagnostic probe bound');}
  cases.push('exhaustive256octets and closed empty/long probe twice in Chromium');
  {
   const f=await fixture();check(typeof f.query.snapshot==='undefined'&&typeof f.query.refresh==='undefined','no public raw snapshot escape');
   for(const name of['state','head','principal','session','headId','authenticated','receipt'])await rejects(f.query.query({...f.intent(),[name]:true}),'invalid-input');
   let read=false;const getter={...f.intent()};Object.defineProperty(getter,'workspace',{enumerable:true,get(){read=true;return f.workspace;}});await rejects(f.query.query(getter),'invalid-input');check(!read,'public getter read');
   await rejects(f.query.query({...f.intent(),[Symbol('principal')]:true}),'invalid-input');
   const input=f.intent();input.workspace=input.workspace.slice();const pending=f.query.query(input);input.workspace.fill(0);const page=await pending;
   check(page.total===1&&page.count===1&&page.cursor.length===0&&hex(page.rows.subarray(0,32))===f.identity.principal.slice(7)&&page.rows[32]===0,'owner-only member page / synchronous capture');
   check(Object.keys(page).sort().join(',')==='count,cursor,headId,offset,rows,table,total,workspace','only admitted page fields');
   page.rows.fill(0);page.workspace.fill(0);page.headId.fill(0);const again=await f.query.query(f.intent());check(again.rows[0]!==0||again.rows.some(x=>x!==0),'detached page bytes');
   const messages=await f.query.query(f.intent(1));check(messages.total===0&&messages.count===0&&messages.rows.length===0&&messages.cursor.length===0,'empty messages complete');
   cases.push('closed intent, no snapshot escape, synchronous capture and detached pages');
  }
  {
   const f=await fixture(),loaded=await f.store.loadIdentity(),mutable={...loaded,publicKey:loaded.publicKey.slice()};
   const query=await f.open(wrapStore(f.store,{loadIdentity:async()=>mutable}));mutable.publicKey.fill(0);mutable.principal='sha256:'+'00'.repeat(32);mutable.privateKey=null;
   check((await query.query(f.intent())).total===1,'identity must be captured');
   const wrong=await createIdentity();await rejects(f.open(wrapStore(f.store,{loadIdentity:async()=>({...loaded,privateKey:wrong.privateKey})})),'identity-corrupt');
   cases.push('possessed persisted identity capture, wrong private key rejected');
  }
  {
   const f=await fixture(),loaded=await f.store.loadIdentity(),mutable={...loaded,publicKey:loaded.publicKey.slice()};
   const original=crypto.subtle.digest;let entered,release,claimed=false;const ready=new Promise(r=>{entered=r;}),pause=new Promise(r=>{release=r;});
   crypto.subtle.digest=async(...args)=>{const bytes=new Uint8Array(args[1]),owned=!claimed&&bytes.length===65&&bytes[0]===4;if(owned)claimed=true;const result=await original.apply(crypto.subtle,args);if(owned){entered();await pause;}return result;};restore=()=>{crypto.subtle.digest=original;};
   const pending=f.open(wrapStore(f.store,{loadIdentity:async()=>mutable}));await ready;mutable.publicKey.fill(0);mutable.principal='sha256:'+'00'.repeat(32);mutable.privateKey=null;release();const query=await pending;restore();restore=()=>{};
   check((await query.query(f.intent())).total===1,'identity capture before validation await');
   cases.push('mutable persisted key/principal across actual bootstrap digest stays captured');
  }
  {
   const f=await fixture(),reader=await createIdentity(),contributor=await createIdentity(),outsider=await createIdentity();
   await f.submit(1,unhex(contributor.principal.slice(7)));await f.submit(2,unhex(reader.principal.slice(7)));
   for(let i=0;i<17;i++)await f.submit(4,text('message '+i));
   const as=identity=>f.open(wrapStore(f.store,{loadIdentity:async()=>identity}));
   const r=await as(reader),c=await as(contributor),foreign=await as(outsider);
   const first=await r.query(f.intent(1));check(first.total===17&&first.count===16&&first.cursor.length===135,'reader first page');
   const second=await r.query(f.intent(1,first.cursor));check(second.total===17&&second.count===1&&second.offset===16&&second.cursor.length===0,'reader final page');
   check((await c.query(f.intent(1))).count===16,'contributor admitted');await rejects(foreign.query(f.intent(1)),'query-rejected',6);
   await rejects(r.query(f.intent(0,first.cursor)),'query-rejected',11);
   await rejects(c.query(f.intent(1,first.cursor)),'query-rejected',9);
   const reopened=await as(reader);await rejects(reopened.query(f.intent(1,first.cursor)),'query-rejected',9);
   await f.submit(3,unhex(reader.principal.slice(7)));await rejects(r.query(f.intent(1,first.cursor)),'query-rejected',6);
   const ownerFirst=await f.query.query(f.intent(1));await f.submit(4,text('new revision'));await rejects(f.query.query(f.intent(1,ownerFirst.cursor)),'query-rejected',8);
   cases.push('real signatures/replay roles, pagination, private session, revoke and stale cursor');
  }
  {
   const f=await fixture(),secondStore=await openStore(f.namespace);stores.push(secondStore);const peer=await openCommands({commandModule,journalModule,store:secondStore,headName:'workspace'});adapters.push(peer);
   const gate=digestGate(),pending=f.query.query(f.intent(1));await gate.ready;await peer.submit({action:4,body:text('cross-tab winner'),workspace:f.workspace});gate.release();await rejects(pending,'query-context-changed');gate.restore();
   check((await f.query.query(f.intent(1))).total===1,'fresh current head after stale rejection');
   cases.push('actual cross-tab commit during digest rejects before any rows escape');
  }
  {
   const f=await fixture(),gate=digestGate(),first=f.query.query(f.intent());await gate.ready;
   const oversized=new Uint8Array(136);Object.defineProperty(oversized,'byteLength',{value:0});
   const beforeCalls=calls.length,rejected=f.query.query(f.intent(1,oversized));let prompt=false;
   try{
    const outcome=await Promise.race([rejected.then(()=>null,error=>error),new Promise(resolve=>setTimeout(()=>resolve({code:'capture-timeout'}),1000))]);
    check(outcome?.code==='invalid-input','shadowed query bytes must reject before queuing, got '+outcome?.code);
    check(calls.length===beforeCalls,'rejected query reached generated execution');prompt=true;
   }finally{if(!prompt){gate.release();await Promise.allSettled([first,rejected]);gate.restore();}}
   const body=f.intent(1),queued=f.query.query(body);body.workspace=body.workspace.slice();body.workspace.fill(0);
   let touched=false;const hostile=new Proxy({}, {getPrototypeOf(){touched=true;throw Error('input read while saturated');}});await rejects(f.query.query(hostile),'adapter-busy');check(!touched,'busy admission before input inspection');gate.release();await first;check((await queued).total===0,'accepted queued input retained by value');gate.restore();
   await rejects(f.query.query(f.intent(255)),'query-rejected',13);check((await f.query.query(f.intent())).total===1,'failure releases slot');
   cases.push('bounded admission, queued capture, success/failure release');
  }
  {
   const f=await fixture(),gate=digestGate(),first=f.query.query(f.intent());await gate.ready;const queued=f.query.query(f.intent(1));f.query.close();await rejects(queued,'adapter-closed');gate.release();await rejects(first,'adapter-closed');gate.restore();await rejects(f.query.query(f.intent()),'adapter-closed');
   cases.push('close releases queued read and refuses late active disclosure');
  }
  {
   const f=await fixture();await f.submit(4,text('immutable'));let corrupt=false;
   const query=await f.open(wrapStore(f.store,{readObject:async(...args)=>{const result=await f.store.readObject(...args);if(corrupt){const bytes=result.slice();bytes[bytes.length-1]^=1;return bytes;}return result;}}));
   corrupt=true;let rejected=false;try{await query.query(f.intent(1));}catch(error){check(error.code==='object-corrupt','typed authenticated replay rejection: '+error.code);rejected=true;}check(rejected,'tampered replay must reject');
   cases.push('tampered stored envelope cannot supply a synthetic authenticated context');
  }
  {
   const f=await fixture();await f.submit(4,text('private'));let unavailable=false;
   const query=await f.open(wrapStore(f.store,{readObject:async(...args)=>{if(unavailable)throw Error('secret-storage-payload');return f.store.readObject(...args);}}));
   unavailable=true;const error=await rejects(query.query(f.intent(1)),'storage-outcome-unknown');check(!String(error).includes('secret-storage-payload')&&error.cause===undefined,'unknown storage details must remain private');
   unavailable=false;check((await query.query(f.intent(1))).total===1,'next query must do complete authenticated replay');
   cases.push('failed replay discloses no partial rows; explicit new query replays completely');
  }
  return{cases,calls,maximum,diagnostics:[...diagnostics].sort()};
 }finally{restore();WebAssembly.Instance=Instance;for(const a of adapters)try{a.close();}catch{}for(const s of stores)try{s.close();}catch{}}
}
