// Fault injection is acceptance infrastructure, never a generated host input.
export async function exceptions(input) {
  const {openWorkspaceView,ViewHostError}=await import('./view-host.mjs');
  const {createIdentity}=await import('./identity.mjs');
  const {openStore}=await import('./store.mjs');
  const {openCommands}=await import('./commands.mjs');
  const modules=Object.fromEntries(await Promise.all(Object.entries(input.modules).map(async([kind,value])=>[kind,await WebAssembly.compile(new Uint8Array(value))])));
  const assert=(condition,label)=>{if(!condition)throw Error(label);};
  const deferred=()=>{let resolve;const promise=new Promise(done=>resolve=done);return{promise,resolve};};
  const faults=[()=>null,()=>undefined,()=>new Proxy({},{getPrototypeOf(){throw null;},get(){throw undefined;}}),()=>Object.assign(Error('private failure payload'),{cause:{private:'unrendered'}})];
  const cases=[];
  for(const fault of ['trap','model-rejection','render']) {
    const store=await openStore('view-bootstrap-fault-'+crypto.randomUUID());await store.saveIdentity(await createIdentity());
    const root=document.createElement('main');document.body.append(root);
    const Instance=WebAssembly.Instance,create=document.createElement;
    const add=EventTarget.prototype.addEventListener,remove=EventTarget.prototype.removeEventListener;
    const attached=[];let fired=false,mounted=false,modelRejected=false,error;
    try {
      EventTarget.prototype.addEventListener=function(type,listener,options) {
        const result=Reflect.apply(add,this,[type,listener,options]);
        if(this instanceof HTMLElement)attached.push({node:this,type,listener,options});
        return result;
      };
      EventTarget.prototype.removeEventListener=function(type,listener,options) {
        const result=Reflect.apply(remove,this,[type,listener,options]);
        const index=attached.findIndex(entry=>entry.node===this&&entry.type===type&&entry.listener===listener&&entry.options===options);
        if(index!==-1)attached.splice(index,1);return result;
      };
      WebAssembly.Instance=class {
        constructor(module,imports) {
          const real=new Instance(module,imports);if(module!==modules.View)return real;
          return {exports:{...real.exports,holo_run(pointer,length) {
            const request=new Uint8Array(real.exports.memory.buffer,pointer,length);
            if(request[0]!==0||fault==='render')return real.exports.holo_run(pointer,length);
            fired=true;mounted=root.childNodes.length>0;
            if(fault==='trap')throw new WebAssembly.RuntimeError('private initialization payload');
            request.fill(0,1); // Execute the real modeled zero-session rejection.
            const packed=BigInt.asUintN(64,real.exports.holo_run(pointer,length));
            const response=new Uint8Array(real.exports.memory.buffer,Number(packed>>32n),Number(packed&0xffffffffn));
            modelRejected=response.length===1&&response[0]===1;return packed;
          }}};
        }
      };
      document.createElement=function(...args) {
        if(fault==='render'&&root.childNodes.length>0){fired=true;mounted=true;throw Error('private initial render payload');}
        return Reflect.apply(create,this,args);
      };
      try {
        await openWorkspaceView({viewModule:modules.View,commandModule:modules.Command,queryModule:modules.Query,journalModule:modules.Journal,
          store,headName:'bootstrap-fault',root,labels:new Uint8Array(input.labels)});
      } catch(value) {error=value;}
      assert(fired&&mounted,'bootstrap fault reaches mounted renderer '+fault);
      if(fault==='model-rejection')assert(modelRejected,'bootstrap rejection is actual generated zero-session validation');
      assert(error instanceof ViewHostError&&error.code==='host-unavailable','bootstrap failure is private typed host-unavailable '+fault);
      assert(error.message==='host-unavailable'&&error.cause===undefined&&Object.keys(error).sort().join(',')==='code,detail,name','bootstrap payload never escapes');
      assert(root.childNodes.length===0&&attached.length===0,
        'failed bootstrap clears mounted DOM and detaches listeners '+fault+': nodes='+root.childNodes.length+', listeners='+attached.length);
      cases.push('bootstrap-'+fault);
    } finally {
      WebAssembly.Instance=Instance;document.createElement=create;
      EventTarget.prototype.addEventListener=add;EventTarget.prototype.removeEventListener=remove;
      for(const entry of attached)Reflect.apply(remove,entry.node,[entry.type,entry.listener,entry.options]);
      store.close();root.remove();
    }
  }
  for(const asynchronous of [false,true])for(let index=0;index<faults.length;index++) {
    if(input.onlyAsynchronous!==undefined&&asynchronous!==input.onlyAsynchronous)continue;
    const store=await openStore('view-fault-'+crypto.randomUUID());await store.saveIdentity(await createIdentity());
    const workspace=crypto.getRandomValues(new Uint8Array(32)),entered=deferred(),release=deferred();let gate=false,held=false;
    const binding={loadIdentity:()=>store.loadIdentity(),readObject:(...args)=>store.readObject(...args),commit:(...args)=>store.commit(...args),
      readHead:async(...args)=>{if(gate&&!held){held=true;entered.resolve();await release.promise;}return store.readHead(...args);}};
    const commands=await openCommands({commandModule:modules.Command,journalModule:modules.Journal,store:binding,headName:'fault'});
    await commands.submit({action:0,body:new Uint8Array(),workspace});commands.close();
    const root=document.createElement('main');document.body.append(root);
    const view=await openWorkspaceView({viewModule:modules.View,commandModule:modules.Command,queryModule:modules.Query,journalModule:modules.Journal,
      store:binding,headName:'fault',root,labels:new Uint8Array(input.labels)});
    await view.dispatch(new Uint8Array([0,...workspace]));
    const original=document.createElement;let error,settled;
    try {
      if(asynchronous){gate=true;settled=view.dispatch(Uint8Array.of(2)).then(()=>({ok:true}),value=>({value}));await entered.promise;}
      document.createElement=()=>{throw faults[index]();};
      if(asynchronous){release.resolve();const result=await settled;error=result.value;}
      else {try {await view.dispatch(Uint8Array.of(2));}catch(value){error=value;}}
    } finally {document.createElement=original;release.resolve();}
    assert(error instanceof ViewHostError&&error.code==='host-unavailable','promotion failure must be fresh typed host-unavailable '+asynchronous+'/'+index);
    assert(error.message==='host-unavailable'&&error.cause===undefined&&Object.keys(error).sort().join(',')==='code,detail,name','hostile payload never escapes');
    assert(root.childNodes.length===0,'failed promotion clears display');
    let closed;try {await view.dispatch(Uint8Array.of(1));}catch(value){closed=value;}
    assert(closed instanceof ViewHostError&&closed.code==='view-closed','failed promotion terminalizes private host');
    view.close();store.close();root.remove();cases.push((asynchronous?'completion':'dispatch')+index);
  }
  return cases;
}
