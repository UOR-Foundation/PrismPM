export async function adapterFixture({commandBytes,journalBytes,vectors}){
  const {openCommands}=await import('/commands.mjs');
  const {openJournal}=await import('/journal.mjs');
  const {createIdentity,BrowserEffectError}=await import('/identity.mjs');
  const {openStore}=await import('/store.mjs');
  const commandModule=await WebAssembly.compile(new Uint8Array(commandBytes)),journalModule=await WebAssembly.compile(new Uint8Array(journalBytes));
  const hex=b=>Array.from(b,n=>n.toString(16).padStart(2,'0')).join('');
  const unhex=s=>Uint8Array.from(s.match(/../g)??[],n=>parseInt(n,16));
  const text=s=>new TextEncoder().encode(s),empty=()=>new Uint8Array();
  const same=(a,b)=>hex(a.head)===hex(b.head)&&hex(a.state)===hex(b.state);
  const check=(condition,label)=>{if(!condition)throw Error(label);};
  const calls=[],cases=[],stores=[],adapters=[],observerStores=new WeakMap();const diagnostics=new Set();let maximum=0;
  async function observe(adapter){const journal=await openJournal(journalModule,observerStores.get(adapter),'workspace');return journal.snapshot();}
  const Instance=WebAssembly.Instance;
  WebAssembly.Instance=class{
    constructor(...args){const real=new Instance(...args),exports={...real.exports};
      exports.holo_run=(pointer,length)=>{const request=hex(new Uint8Array(exports.memory.buffer,pointer,length));const result=real.exports.holo_run(pointer,length),packed=BigInt.asUintN(64,result),offset=Number(packed>>32n),size=Number(packed&0xffffffffn);calls.push({kind:args[0]===commandModule?'Command':'Journal',request,response:hex(new Uint8Array(exports.memory.buffer,offset,size))});if(args[0]===commandModule)maximum=Math.max(maximum,exports.memory.buffer.byteLength);return result;};return {exports};
    }
  };
  let restoreEffect=()=>{};
  const headCount=state=>state.head.length?new DataView(state.head.buffer,state.head.byteOffset,state.head.byteLength).getUint16(36):0;
  async function rejects(promise,code,detail){let caught;try{await promise;}catch(error){caught=error;}check(caught?.code===code,'expected '+code+', got '+caught?.code);if(detail!==undefined)check(caught.detail===detail,'expected detail '+detail);if(caught.name==='CommandAdapterError')diagnostics.add(caught.code);return caught;}
  async function fixture(){
    const namespace='commands-'+crypto.randomUUID(),store=await openStore(namespace);stores.push(store);
    const identity=await createIdentity();await store.saveIdentity(identity);
    const workspace=crypto.getRandomValues(new Uint8Array(32));
    const open=async(selected=store)=>{const adapter=await openCommands({commandModule,journalModule,store:selected,headName:'workspace'});adapters.push(adapter);observerStores.set(adapter,store);return adapter;};
    const adapter=await open();const command=(action,body=empty(),space=workspace)=>({action,body,workspace:space});
    return {namespace,store,identity,workspace,open,adapter,command};
  }
  function wrapStore(store,overrides){return {loadIdentity:()=>store.loadIdentity(),readHead:(...a)=>store.readHead(...a),readObject:(...a)=>store.readObject(...a),commit:(...a)=>store.commit(...a),...overrides};}
  function effectGate(method,{change,fail=false}={}){
    const original=crypto.subtle[method],bound=original.bind(crypto.subtle);let entered,release,claimed=false;
    const ready=new Promise(resolve=>{entered=resolve;}),pause=new Promise(resolve=>{release=resolve;});
    crypto.subtle[method]=async(...args)=>{
      const data=new Uint8Array(args[method==='digest'?1:2]),prefix=text('prismpm/browser-signature/1\0');
      const match=data.length>prefix.length&&prefix.every((byte,index)=>data[index]===byte);
      const owned=match&&!claimed;if(owned)claimed=true;
      const actual=await bound(...args);if(!owned)return actual;
      entered();await pause;if(fail)throw Error('private effect payload must not leak');return change?change(actual):actual;
    };
    restoreEffect=()=>{crypto.subtle[method]=original;};
    return {ready,release,restore:()=>{restoreEffect();restoreEffect=()=>{};}};
  }
  try{
    check(vectors.length===75,'complete75 corpus');
    for(const row of vectors)for(let repeat=0;repeat<2;repeat++){
      const request=unhex(row.request),instance=new WebAssembly.Instance(commandModule,{});
      if(request.length>139873){check(row.id==='OverRequestAllocationCap','declared overcap');let caught;try{instance.exports.holo_alloc(request.length);}catch(error){caught=error;}check(caught instanceof WebAssembly.RuntimeError,'actual allocation trap');continue;}
      const pointer=instance.exports.holo_alloc(request.length);new Uint8Array(instance.exports.memory.buffer,pointer,request.length).set(request);
      const packed=BigInt.asUintN(64,instance.exports.holo_run(pointer,request.length));check(hex(new Uint8Array(instance.exports.memory.buffer,Number(packed>>32n),Number(packed&0xffffffffn)))===row.response,'literal '+row.id);
    }
    cases.push('all75 command vectors twice, unchanged64-page guest');
    {
      const f=await fixture(),{adapter,command}=f;
      await adapter.submit(command(0));const before=(await observe(adapter));
      for(const extra of ['actor','head','state','pending','authenticated','receipt'])await rejects(adapter.submit({...command(4,text('no')), [extra]:true}),'invalid-input');
      const getter={...command(4,text('no'))};let read=false;Object.defineProperty(getter,'body',{get(){read=true;return text('no');},enumerable:true});await rejects(adapter.submit(getter),'invalid-input');check(!read,'getter executed');
      const symbols={...command(4,text('no')),[Symbol('actor')]:true};await rejects(adapter.submit(symbols),'invalid-input');
      check(same(before,(await observe(adapter))),'invalid public input changed state');
      const body=text('immutable submitted body'),expected=body.slice(),space=f.workspace.slice();const pending=adapter.submit(command(4,body,space));body.fill(0x78);space.fill(0);await pending;
      check(hex((await observe(adapter)).state).includes(hex(expected)),'synchronous input copy');
      const status=adapter.status();check(Object.isFrozen(status)&&!Object.hasOwn(status,'head')&&!Object.hasOwn(status,'state')&&typeof adapter.snapshot==='undefined','command status cannot disclose raw state');
      cases.push('closed own-data input, synchronous capture and status-only output');
    }
    {
      const f=await fixture(),loaded=await f.store.loadIdentity(),mutable={...loaded,publicKey:loaded.publicKey.slice()};
      const adapter=await f.open(wrapStore(f.store,{loadIdentity:async()=>mutable}));const principal=adapter.status().principal;
      mutable.publicKey.fill(0);mutable.principal='sha256:'+'00'.repeat(32);mutable.privateKey=null;
      await adapter.submit(f.command(0));check(adapter.status().principal===principal,'captured identity replaced');
      const wrong=await createIdentity();await rejects(f.open(wrapStore(f.store,{loadIdentity:async()=>({...loaded,privateKey:wrong.privateKey})})),'identity-corrupt');
      await rejects(f.open(wrapStore(f.store,{loadIdentity:async()=>null})),'identity-missing');
      cases.push('private validated persisted identity is immutable and mismatched keys rejected');
    }
    {
      const f=await fixture();await f.adapter.submit(f.command(0));
      async function participant(){const store=await openStore('selected-'+crypto.randomUUID());stores.push(store);const identity=await createIdentity();await store.saveIdentity(identity);return{store,identity};}
      const member=await participant(),reader=await participant(),outsider=await participant();
      await f.adapter.submit(f.command(1,unhex(member.identity.principal.slice(7))));
      await f.adapter.submit(f.command(2,unhex(reader.identity.principal.slice(7))));
      const selected=async p=>f.open(wrapStore(f.store,{loadIdentity:()=>p.store.loadIdentity()}));
      const contributor=await selected(member),readOnly=await selected(reader),foreign=await selected(outsider);
      await contributor.submit(f.command(4,text('actual contributor')));await readOnly.refresh();await foreign.refresh();
      await rejects(readOnly.submit(f.command(4,text('reader denied'))),'model-rejected');
      await rejects(foreign.submit(f.command(1,unhex(outsider.identity.principal.slice(7)))),'model-rejected');
      await f.adapter.refresh();await f.adapter.submit(f.command(4,text('🧭'.repeat(1024))));
      await f.adapter.submit(f.command(3,unhex(member.identity.principal.slice(7))));await contributor.refresh();
      await rejects(contributor.submit(f.command(4,text('revoked denied'))),'model-rejected');
      await rejects(f.adapter.submit(f.command(3,unhex(f.identity.principal.slice(7)))),'model-rejected');
      await f.adapter.refresh();check(headCount((await observe(f.adapter)))===6,'all five generated actions and role decisions');
      cases.push('all five generated actions, persisted principals, role admission and maximum UTF-8 body');
    }
    for(const method of ['digest','sign']){
      const f=await fixture();await f.adapter.submit(f.command(0));const second=await openStore(f.namespace);stores.push(second);const peer=await f.open(second);
      const gate=effectGate(method),pending=f.adapter.submit(f.command(4,text('stale '+method)));await gate.ready;
      await peer.submit(f.command(4,text('winning '+method)));const winner=(await observe(peer));gate.release();await rejects(pending,'command-rejected',8);gate.restore();
      check(same(winner,(await observe(f.adapter))),'stale effect displaced durable winner');
      const rejected=calls.filter(call=>call.kind==='Command').at(-1);
      check(rejected.request.startsWith(method==='digest'?'01':'02')&&rejected.response==='08','stale rejection must occur at the actual '+method+' completion');
      cases.push('cross-tab durable head change during '+method+' rejects stale completion');
    }
    {
      const f=await fixture();await f.adapter.submit(f.command(0));
      const result=await Promise.all([f.adapter.submit(f.command(4,text('queued1'))),f.adapter.submit(f.command(4,text('queued2')))]);
      check(result.every(ack=>ack.committed===true&&!Object.hasOwn(ack,'state')&&!Object.hasOwn(ack,'head'))&&headCount(await observe(f.adapter))===3,'serialized private queue');
      const ordered=hex((await observe(f.adapter)).state);check(ordered.indexOf(hex(text('queued1')))<ordered.indexOf(hex(text('queued2'))),'queued command order');
      const reopenStore=await openStore(f.namespace);stores.push(reopenStore);const reopened=await f.open(reopenStore);check(same((await observe(reopened)),(await observe(f.adapter))),'reopen authenticated replay');
      cases.push('serialized commands and complete durable reload');
    }
    for(const method of ['digest','sign']){
      const f=await fixture();await f.adapter.submit(f.command(0));const before=(await observe(f.adapter));
      const gate=effectGate(method,{fail:true}),pending=f.adapter.submit(f.command(4,text('failed effect')));await gate.ready;gate.release();const error=await rejects(pending,'crypto-unavailable');gate.restore();
      check(error.message==='crypto-unavailable'&&!Object.hasOwn(error,'cause'),'effect error leaked');check(same(before,(await observe(f.adapter))),'failed crypto promoted state');
      await f.adapter.submit(f.command(4,text('explicit retry')));check(headCount((await observe(f.adapter)))===2,'explicit retry failed');
      cases.push(method+' failure has no durable effect or automatic retry');
    }
    for(const method of ['digest','sign']){
      const f=await fixture();await f.adapter.submit(f.command(0));const before=(await observe(f.adapter));
      const gate=effectGate(method,{change:result=>{const changed=new Uint8Array(result.slice(0));changed[0]^=1;return changed.buffer;}}),pending=f.adapter.submit(f.command(4,text('forged effect')));await gate.ready;gate.release();await rejects(pending,method==='digest'?'event-id-mismatch':'signature-invalid');gate.restore();
      check(same(before,(await observe(f.adapter))),'forged crypto promoted');await rejects(f.adapter.submit(f.command(4,text('blocked'))),'refresh-required');await f.adapter.refresh();check(same(before,(await observe(f.adapter))),'forged crypto persisted');
      cases.push('actual '+method+' result forgery fails authenticated Journal');
    }
    {
      const f=await fixture();await f.adapter.submit(f.command(0));let commits=0;
      const adapter=await f.open(wrapStore(f.store,{commit:async()=>{commits++;throw new BrowserEffectError('storage-quota');}}));const before=(await observe(adapter));
      await rejects(adapter.submit(f.command(4,text('quota'))),'storage-rejected',35);await rejects(adapter.submit(f.command(4,text('no retry'))),'refresh-required');
      check(commits===1&&same(before,(await observe(adapter))),'known failed commit retried/promoted');await adapter.refresh();check(same(before,(await observe(adapter))),'failed commit persisted');
      cases.push('known storage rejection requires explicit refresh without retry');
    }
    {
      const f=await fixture();await f.adapter.submit(f.command(0));let commits=0;
      const adapter=await f.open(wrapStore(f.store,{commit:async(...args)=>{commits++;await f.store.commit(...args);throw Error('secret unknown acknowledgement');}}));const before=(await observe(adapter));
      const error=await rejects(adapter.submit(f.command(4,text('durable but unacknowledged'))),'storage-outcome-unknown');check(error.message==='storage-outcome-unknown'&&!Object.hasOwn(error,'cause'),'unknown error leaked');
      check(adapter.status().requiresRefresh&&!same(before,await observe(adapter)),'unknown durable commit must retain refresh barrier');await rejects(adapter.submit(f.command(4,text('no retry'))),'refresh-required');check(commits===1,'unknown commit retried');
      await adapter.refresh();check(headCount((await observe(adapter)))===2,'durable unknown commit not recovered');
      cases.push('unknown post-commit outcome preserves barrier and replay recovers durability');
    }
    for(const method of ['digest','sign']){
      const f=await fixture();await f.adapter.submit(f.command(0));const before=(await observe(f.adapter));
      const gate=effectGate(method),pending=f.adapter.submit(f.command(4,text('closed before write')));await gate.ready;f.adapter.close();gate.release();await rejects(pending,'adapter-closed');gate.restore();
      await rejects(f.adapter.submit(f.command(4,text('late'))),'adapter-closed');const reopened=await f.open();check(same(before,(await observe(reopened))),'closed before append wrote data');
      cases.push('close during '+method+' rejects late completion before storage');
    }
    {
      const f=await fixture();await f.adapter.submit(f.command(0));let enter,release,commits=0;const ready=new Promise(r=>{enter=r;}),hold=new Promise(r=>{release=r;});
      const adapter=await f.open(wrapStore(f.store,{commit:async(...args)=>{commits++;const result=await f.store.commit(...args);enter();await hold;return result;}}));
      const pending=adapter.submit(f.command(4,text('committed while closing')));await ready;adapter.close();release();await rejects(pending,'commit-outcome-unknown');
      check(commits===1,'close retried pending commit');const reopened=await f.open();check(headCount((await observe(reopened)))===2,'close falsely rolled back actual durable commit');
      cases.push('close during real durable commit reports uncertainty and reopen recovers');
    }
    {
      const f=await fixture();await f.adapter.submit(f.command(0));let broken=false;
      const adapter=await f.open(wrapStore(f.store,{readHead:async(...args)=>{if(broken)throw Error('private read failure');return f.store.readHead(...args);}}));const before=(await observe(adapter));broken=true;
      await rejects(adapter.refresh(),'storage-outcome-unknown');check(same(before,(await observe(adapter))),'failed refresh promoted');await rejects(adapter.submit(f.command(4,text('blocked'))),'refresh-required');broken=false;await adapter.refresh();check(same(before,(await observe(adapter))),'recovered replay changed state');
      cases.push('failed replay preserves state and requires explicit recovery');
    }
    {
      const f=await fixture();await f.adapter.submit(f.command(0));let enter,release;
      const ready=new Promise(r=>{enter=r;}),hold=new Promise(r=>{release=r;});
      const adapter=await f.open(wrapStore(f.store,{commit:async(...args)=>{enter();await hold;return f.store.commit(...args);}}));
      const peerStore=await openStore(f.namespace);stores.push(peerStore);const peer=await f.open(peerStore);
      const before=(await observe(adapter)),pending=adapter.submit(f.command(4,text('conflicting candidate')));await ready;
      await peer.submit(f.command(4,text('durable CAS winner')));release();await rejects(pending,'storage-rejected',33);
      check(adapter.status().requiresRefresh&&hex((await observe(adapter)).state).includes(hex(text('durable CAS winner')))&&!hex((await observe(adapter)).state).includes(hex(text('conflicting candidate'))),'CAS conflict must retain durable winner and refresh barrier');await rejects(adapter.submit(f.command(4,text('blocked'))),'refresh-required');
      await adapter.refresh();check(same((await observe(peer)),(await observe(adapter))),'CAS recovery lost winner');
      cases.push('actual post-signature CAS conflict preserves winner without merge or retry');
    }
    {
      const f=await fixture();await f.adapter.submit(f.command(0));let broken=false,reads=0;
      const adapter=await f.open(wrapStore(f.store,{readObject:async(...args)=>{if(broken&&++reads===2)throw Error('private replay payload');return f.store.readObject(...args);}}));
      const before=(await observe(adapter));await f.adapter.submit(f.command(4,text('new retained event')));broken=true;
      await rejects(adapter.refresh(),'storage-outcome-unknown');check(adapter.status().requiresRefresh&&!same(before,await observe(adapter)),'partial replay must retain refresh barrier without exposing rows');await rejects(adapter.submit(f.command(4,text('blocked'))),'refresh-required');
      broken=false;await adapter.refresh();check(same((await observe(f.adapter)),(await observe(adapter))),'complete replay did not recover');
      cases.push('mid-history object read failure cannot promote partial replay');
    }
    {
      const f=await fixture();await f.adapter.submit(f.command(0));const gate=effectGate('digest'),first=f.adapter.submit(f.command(4,text('active')));await gate.ready;
      const oversized=new Uint8Array(4097);Object.defineProperty(oversized,'byteLength',{value:0});
      const beforeCalls=calls.length,rejected=f.adapter.submit(f.command(4,oversized));let prompt=false;
      try{
        const outcome=await Promise.race([rejected.then(()=>null,error=>error),new Promise(resolve=>setTimeout(()=>resolve({code:'capture-timeout'}),1000))]);
        check(outcome?.code==='invalid-input','shadowed command bytes must reject before queuing, got '+outcome?.code);
        check(calls.length===beforeCalls,'rejected command reached generated execution');prompt=true;
      }finally{if(!prompt){gate.release();await Promise.allSettled([first,rejected]);gate.restore();}}
      const body=text('captured queued body'),expected=body.slice(),second=f.adapter.submit(f.command(4,body));body.fill(0x78);
      let accessed=false;const unretained=new Proxy(f.command(4,text('busy')),{getOwnPropertyDescriptor(...args){accessed=true;return Reflect.getOwnPropertyDescriptor(...args);}});
      const third=f.adapter.submit(unretained);let passed=false;
      try{
        const outcome=await Promise.race([third.then(()=>null,error=>error),new Promise(resolve=>setTimeout(()=>resolve({code:'admission-timeout'}),1000))]);
        check(outcome?.name==='CommandAdapterError'&&outcome?.code==='adapter-busy','saturated submit must reject adapter-busy before retention, got '+outcome?.code);diagnostics.add(outcome.code);
        await rejects(f.adapter.refresh(),'adapter-busy');check(!accessed,'busy caller was inspected/copied');passed=true;
      }finally{gate.release();await Promise.allSettled([first,second,third]);gate.restore();}
      check(passed&&headCount((await observe(f.adapter)))===3,'bounded active/queued completion');
      check(hex((await observe(f.adapter)).state).includes(hex(expected)),'accepted queued bytes were mutable');
      await f.adapter.submit(f.command(4,text('capacity released')));check(headCount((await observe(f.adapter)))===4,'capacity not released after success');
      cases.push('two-operation admission rejects before capture and releases after success');
    }
    {
      const f=await fixture();await f.adapter.submit(f.command(0));const gate=effectGate('digest',{fail:true}),first=f.adapter.submit(f.command(4,text('failed active')));await gate.ready;
      const second=f.adapter.submit(f.command(4,text('queued survives'))),failure=rejects(first,'crypto-unavailable');gate.release();await failure;await second;gate.restore();
      await f.adapter.submit(f.command(4,text('post-failure capacity')));check(headCount((await observe(f.adapter)))===3,'failure leaked capacity or retried');
      cases.push('failed active operation releases capacity without automatic retry');
    }
    {
      const f=await fixture();await f.adapter.submit(f.command(0));const before=(await observe(f.adapter)),gate=effectGate('digest'),first=f.adapter.submit(f.command(4,text('active close')));await gate.ready;
      const second=f.adapter.submit(f.command(4,text('queued close')));f.adapter.close();let released=false;
      try{
        const outcome=await Promise.race([second.then(()=>null,error=>error),new Promise(resolve=>setTimeout(()=>resolve({code:'close-timeout'}),1000))]);
        check(outcome?.code==='adapter-closed','close must release queued input without waiting for active effect');released=true;
      }finally{gate.release();await Promise.allSettled([first,second]);gate.restore();}
      check(released,'close queue release');await rejects(first,'adapter-closed');const reopened=await f.open();check(same(before,(await observe(reopened))),'closed queue reached storage');
      cases.push('close immediately rejects queued work while owned active effect settles');
    }
    check(maximum===50*65536,'complete corpus measured peak: '+maximum+' bytes ('+maximum/65536+' pages)');return{cases,calls,maximum,diagnostics:[...diagnostics].sort()};
  }finally{restoreEffect();for(const adapter of adapters)adapter.close();for(const store of stores)store.close();WebAssembly.Instance=Instance;}
}
