// Acceptance-only command composition. Host code frames model arguments and
// performs actual effects; it never constructs event or envelope bytes.
export async function commandBrowserFixture({commandBytes,journalBytes,vectors}){
 const {createIdentity,signBytes,digestBytes,bytesCopy}=await import('/identity.mjs');
 const {openStore}=await import('/store.mjs');const {openJournal}=await import('/journal.mjs');
 const commandModule=await WebAssembly.compile(new Uint8Array(commandBytes)),journalModule=await WebAssembly.compile(new Uint8Array(journalBytes));
 const empty=()=>new Uint8Array(),hex=value=>Array.from(value,b=>b.toString(16).padStart(2,'0')).join('');
 const unhex=value=>Uint8Array.from(value.match(/../g)??[],b=>parseInt(b,16));
 const bytes=hash=>unhex(hash.slice(7)),text=value=>new TextEncoder().encode(value);
 const check=(condition,label)=>{if(!condition)throw Error(label);};
 const join=(...parts)=>{const result=new Uint8Array(parts.reduce((s,p)=>s+p.length,0));let offset=0;for(const part of parts){result.set(part,offset);offset+=part.length;}return result;};
 const u16=n=>new Uint8Array([n>>>8,n&255]),u24=n=>new Uint8Array([n>>>16,n>>>8&255,n&255]);
 const read24=(b,at)=>b[at]*65536+b[at+1]*256+b[at+2];
 const fail=code=>Object.assign(Error(code),{code});
 const same=(a,b)=>hex(a)===hex(b),sameState=(a,b)=>same(a.head,b.head)&&same(a.state,b.state);
 const calls=[],cases=[];let maximum=0;
 const Instance=WebAssembly.Instance;
 WebAssembly.Instance=class{
  constructor(...args){const instance=new Instance(...args),exports={...instance.exports};
   exports.holo_run=(pointer,length)=>{const request=hex(new Uint8Array(exports.memory.buffer,pointer,length));const result=instance.exports.holo_run(pointer,length),packed=BigInt.asUintN(64,result),offset=Number(packed>>32n),size=Number(packed&0xffffffffn);calls.push({kind:args[0]===commandModule?'Command':'Journal',request,response:hex(new Uint8Array(exports.memory.buffer,offset,size))});return result;};
   return {exports};
  }
 };
 function generated(request){
  check(request.length<=139873,'command request cap');
  const instance=new WebAssembly.Instance(commandModule,{}),p=instance.exports.holo_alloc(request.length);
  new Uint8Array(instance.exports.memory.buffer,p,request.length).set(request);
  const packed=BigInt.asUintN(64,instance.exports.holo_run(p,request.length)),offset=Number(packed>>32n),length=Number(packed&0xffffffffn);
  check(length<=74243&&offset+length<=instance.exports.memory.buffer.byteLength,'command output bounds');
  maximum=Math.max(maximum,instance.exports.memory.buffer.byteLength);check(maximum<=64*65536,'command memory cap');
  return new Uint8Array(instance.exports.memory.buffer,offset,length).slice();
 }
 check(vectors.length===75,'complete modeled Chromium corpus');
 for(const vector of vectors)for(let repeat=0;repeat<2;repeat++){
  const input=unhex(vector.request);
  if(input.length>139873){
   check(vector.id==='OverRequestAllocationCap','only declared over-cap vector');let trapped=false;
   try{new WebAssembly.Instance(commandModule,{}).exports.holo_alloc(input.length);}catch(error){trapped=error instanceof WebAssembly.RuntimeError;}
   check(trapped,'actual Chromium input allocation trap');
  }else check(hex(generated(input))===vector.response,'Chromium vector '+vector.id);
 }
 cases.push('all75 literal vectors twice in Chromium with real allocation boundary');
 for(const name of ['PrepareGenesis','HashGenesis','SignGenesis']){
  const input=unhex(vectors.find(v=>v.id===name).request);
  for(let length=0;length<input.length;length++)check(hex(generated(input.slice(0,length)))==='01','truncated '+name+' at '+length);
 }
 cases.push('every truncated prefix of complete genesis preparation/hash/sign framing');
 check(hex(generated(new Uint8Array(139873)))==='01','at-cap input reaches actual typed decoder');
 const bounded=new WebAssembly.Instance(commandModule,{}),initial=bounded.exports.memory.buffer.byteLength/65536;
 check(initial<=64,'initial guest pages');bounded.exports.memory.grow(64-initial);let memoryRefused=false;
 try{bounded.exports.memory.grow(1);}catch(error){memoryRefused=error instanceof RangeError;}
 check(memoryRefused,'actual compiled64-page maximum');cases.push('exact input cap and compiled64-page memory maximum are enforced');
 function plan(response){
  if(!response.length)throw fail('invalid-generated-output');if(response[0]!==0)throw fail('command-model-'+hex(response));
  const length=read24(response,1),at=4+length;check(at+2<=response.length,'closed command plan');
  const effectLength=response[at]*256+response[at+1];check(at+2+effectLength===response.length,'closed command effect');
  return {pending:response.slice(4,at),effect:response.slice(at+2)};
 }
 const completion=(operation,pending,head,nonce,key,input,result)=>join([operation],u24(pending.length),pending,u24(head.length),head,nonce,key,u16(input.length),input,u16(result.length),result);
 let owned=await createIdentity();const contributor=await createIdentity(),reader=await createIdentity(),outsider=await createIdentity();
 const namespace='command-browser-'+crypto.randomUUID(),workspace=crypto.getRandomValues(new Uint8Array(32)),store=await openStore(namespace);
 await store.saveIdentity(owned);owned=await store.loadIdentity();check(owned!==null,'persisted selected identity');let reopenedStore;
 const journal=await openJournal(journalModule,store,'workspace');await journal.refresh();
 const completed=[];
 // Public input has no selected identity, journal state, pending, token or
 // receipt field. Test-only hooks below can mutate actual effect results.
 function bind(identity,hooks={}){
  let tail=Promise.resolve();
  return Object.freeze({submit(value){
   let captured;
   try{
    if(arguments.length!==1||!value||typeof value!=='object'||Object.keys(value).sort().join(',')!=='action,body,workspace'||!Number.isInteger(value.action)||value.action<0||value.action>255)throw fail('command-input');
    captured={workspace:bytesCopy(value.workspace,32),action:value.action,body:bytesCopy(value.body,4096)};if(captured.workspace.length!==32)throw fail('command-input');
   }catch(error){return Promise.reject(error);}
   const queued=tail.then(async()=>{
    const nonce=crypto.getRandomValues(new Uint8Array(32)),key=identity.publicKey.slice();
    const head=journal.snapshot().head,principal=hooks.principal??bytes(identity.principal);
    const raw=join(nonce,hooks.key??key,principal,captured.workspace,u24(head.length),head,[captured.action],u16(captured.body.length),captured.body);
    let current=plan(generated(join([0],raw)));
    let hashInput=current.effect.slice(),hash=bytes(await digestBytes(hashInput));
    if(hooks.hash)hash=hooks.hash(hash);if(hooks.hashInput)hashInput=hooks.hashInput(hashInput);
    if(hooks.afterHash)await hooks.afterHash();
    current=plan(generated(completion(1,current.pending,journal.snapshot().head,nonce,hooks.key??key,hashInput,hash)));
    let signInput=current.effect.slice(),signature=await signBytes(identity,hooks.context??'prismpm/workspace-event/1',signInput);
    if(hooks.signature)signature=hooks.signature(signature);if(hooks.signInput)signInput=hooks.signInput(signInput);
    if(hooks.afterSign)await hooks.afterSign();
    current=plan(generated(completion(2,current.pending,journal.snapshot().head,nonce,hooks.key??key,signInput,signature)));
    completed.push({pending:current.pending.slice(),envelope:current.effect.slice(),nonce:nonce.slice(),key:key.slice(),signInput,signature});
    // No trusted boolean or claimed completion can replace this actual
    // cryptographic authentication and bound durable generated Journal append.
    return journal.append(current.effect);
   });tail=queued.catch(()=>{});return queued;
  }});
 }
 const owner=bind(owned),member=bind(contributor),readOnly=bind(reader),foreign=bind(outsider);
 const command=(action,body=empty(),space=workspace)=>({action,body,workspace:space});
 async function reject(promise,code,label){let caught;try{await promise;}catch(error){caught=error;}check(caught?.code===code,label+': '+caught?.code);cases.push(label);}
 async function unchanged(promise,code,label){const before=journal.snapshot();await reject(promise,code,label);check(sameState(before,journal.snapshot()),label+' promoted state');}
 try{
  await unchanged(owner.submit(command(4,text('before genesis'))),'command-model-02','post requires a generated genesis head');
  await owner.submit(command(0));cases.push('generated genesis, real identity hash and P-256 signature, actual atomic commit');
  await unchanged(owner.submit(command(0)),'command-model-02','existing head cannot prepare another genesis');
  await owner.submit(command(1,bytes(contributor.principal)));await owner.submit(command(2,bytes(reader.principal)));
  await member.submit(command(4,text('contributor message')));await owner.submit(command(4,text('🧭'.repeat(1024))));cases.push('generated contributor/reader grants and maximum UTF-8 post');
  await unchanged(readOnly.submit(command(4,text('reader denied'))),'model-rejected','Journal alone denies reader posting');
  await unchanged(foreign.submit(command(1,bytes(outsider.principal))),'model-rejected','Journal alone denies outsider grant');
  await unchanged(member.submit(command(2,bytes(outsider.principal))),'model-rejected','Journal alone denies contributor grant');
  await owner.submit(command(3,bytes(contributor.principal)));
  await unchanged(member.submit(command(4,text('revoked denied'))),'model-rejected','Journal alone denies revoked member posting');
  await unchanged(owner.submit(command(3,bytes(owned.principal))),'model-rejected','Journal alone preserves immutable owner');
  await unchanged(owner.submit({...command(4,text('forged')),pending:new Uint8Array([0])}),'command-input','public caller cannot supply pending or completion');
  const other=crypto.getRandomValues(new Uint8Array(32));await unchanged(owner.submit(command(4,text('wrong workspace'),other)),'command-model-04','captured workspace differs from authenticated head');
  const flip=value=>{const next=value.slice();next[0]^=1;return next;};
  await unchanged(bind(owned,{hash:flip}).submit(command(4,text('bad hash'))),'event-id-mismatch','actual hash mutation is rejected by Journal authentication');
  await unchanged(bind(owned,{signature:flip}).submit(command(4,text('bad signature'))),'signature-invalid','actual signature mutation is rejected by Journal authentication');
  await unchanged(bind(owned,{context:'prismpm/workspace-event/2'}).submit(command(4,text('bad context'))),'signature-invalid','real signature under wrong context is rejected');
  await unchanged(bind(owned,{principal:bytes(reader.principal)}).submit(command(4,text('bad principal'))),'author-mismatch','real owner key cannot claim another principal');
  const malformedKey=new Uint8Array(65);malformedKey[0]=4;
  await unchanged(bind(owned,{key:malformedKey}).submit(command(4,text('bad curve point'))),'invalid-input','syntactic uncompressed key is not curve validity');
  await unchanged(bind(owned,{hashInput:flip}).submit(command(4,text('changed preimage'))),'command-model-09','hash completion must match exact generated preimage');
  await unchanged(bind(owned,{signInput:flip}).submit(command(4,text('changed unsigned'))),'command-model-09','signature completion must match exact generated unsigned bytes');
  for(const phase of ['afterHash','afterSign']){
   let concurrent;
   const raced=bind(owned,{[phase]:async()=>{await owner.submit(command(4,text('concurrent '+phase)));concurrent=journal.snapshot();}});
   await reject(raced.submit(command(4,text('stale '+phase))),'command-model-08','head change at '+phase+' refuses stale completion');
   check(sameState(concurrent,journal.snapshot()),'stale command displaced actual concurrent commit');
  }
  const captured=text('captured input'),original=captured.slice(),pending=owner.submit(command(4,captured));captured.fill(0xff);await pending;
  const last=completed.at(-1),effect=last.envelope;
  check(same(effect.slice(-original.length),original),'caller input changed while hashing/signing');cases.push('capture raw intent before first asynchronous effect');
  for(const phase of [1,2]){const answer=generated(completion(phase,last.pending,journal.snapshot().head,last.nonce,last.key,last.signInput,last.signature));check(hex(answer)==='06','terminal pending reused');}
  cases.push('generated terminal phase cannot be reused for hash or signature completion');
  const expected=journal.snapshot();store.close();reopenedStore=await openStore(namespace);const retained=await reopenedStore.loadIdentity();check(retained.principal===owned.principal,'persisted identity survives store reopening');
  const reopened=await openJournal(journalModule,reopenedStore,'workspace');await reopened.refresh();check(sameState(expected,reopened.snapshot()),'complete authenticated reopen differs');cases.push('complete durable Journal replay matches generated command history');
  return {cases,calls,maximum,count:new DataView(expected.head.buffer).getUint16(36)};
 }finally{store.close();reopenedStore?.close();WebAssembly.Instance=Instance;}
}
