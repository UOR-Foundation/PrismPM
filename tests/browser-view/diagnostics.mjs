// Exact shipping guards receive negative-only exports in an isolated test page.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import {withBrowser} from '../../sdk/browser/browser-test-server.mjs';
import {repository} from './compile.mjs';
import {corpus} from './corpus.mjs';
import {verifyTextBoundary} from './text.mjs';

export async function verifyDiagnostics(build){
  verifyTextBoundary();
  const files=['view-dom.mjs','view-error.mjs','view-host.mjs'];
  const sources=Object.fromEntries(files.map(name=>[name,readFileSync(join(repository,'sdk/browser',name),'utf8')]));
  const register=readFileSync(join(repository,'model/browser-view-diagnostics.toml'),'utf8');
  const registered=[...register.matchAll(/^code = "([a-z-]+)"$/gm)].map(m=>m[1]);
  const sites=[...new Set(Object.values(sources).flatMap(source=>[...source.matchAll(/(?:fail|ViewHostError)\('([a-z-]+)'/g)].map(m=>m[1])))].sort();
  assert.deepEqual(sites,registered,'complete exact View diagnostic sites');
  const suffix='\nexport const __negativeBoundary=Object.freeze({interpreter,plan,effect,admittedPage});\n';
  assert.ok(!sources['view-host.mjs'].includes('__negativeBoundary'));
  const rejectionVectors=corpus().filter(v=>v.response.length===1).filter((v,index,all)=>all.findIndex(other=>other.response[0]===v.response[0])===index);
  assert.equal(rejectionVectors.length,14);
  const result=await withBrowser(async({browser,baseURL})=>{
    const page=await browser.newPage();
    for(const name of files)await page.route('**/'+name,r=>r.fulfill({contentType:'text/javascript',body:sources[name]+(name==='view-host.mjs'?suffix:'')}));
    await page.goto(baseURL);
    return page.evaluate(async({modules,labels,rejections})=>{
      const api=await import('./view-host.mjs'),dom=await import('./view-dom.mjs'),guard=api.__negativeBoundary;
      const {openStore}=await import('./store.mjs'),{createIdentity}=await import('./identity.mjs');
      const compiled=Object.fromEntries(await Promise.all(Object.entries(modules).map(async([name,bytes])=>[name,await WebAssembly.compile(new Uint8Array(bytes))])));
      const cases=[],codes=new Set();
      async function reject(label,code,operation,detail=null){let error;try{await operation();}catch(value){error=value;}
        if(!(error instanceof api.ViewHostError)||error.code!==code||error.detail!==detail||error.cause!==undefined||Object.keys(error).sort().join(',')!=='code,detail,name')throw Error(label+': expected '+code+'/'+detail+', got '+error?.code+'/'+error?.detail);
        cases.push(label);codes.add(code);
      }
      await reject('private effect fields','invalid-effect-result',()=>guard.admittedPage({}));
      await reject('closed guest module','invalid-generated-module',()=>guard.interpreter({}));
      await reject('private over-cap request','request-limit',()=>guard.interpreter(compiled.View)(new Uint8Array(133729)));
      await reject('private generated output','invalid-generated-output',()=>guard.plan(new Uint8Array()));
      await reject('modeled labels','invalid-labels',()=>dom.captureLabels(new Uint8Array()));
      await reject('private render root','invalid-root',()=>dom.makeRenderer({}, {},()=>{}));
      for(const row of rejections)await reject('actual generated rejection '+row.detail,'model-rejected',()=>guard.interpreter(compiled.View)(new Uint8Array(row.request)),row.detail);
      const Instance=WebAssembly.Instance;
      try{
        WebAssembly.Instance=class{constructor(...args){const real=new Instance(...args);return{exports:{...real.exports,holo_run(){throw new WebAssembly.RuntimeError('private guest payload');}}};}};
        await reject('actual guest trap','generated-execution-failed',()=>guard.interpreter(compiled.View)(new Uint8Array([255])));
      }finally{WebAssembly.Instance=Instance;}
      const root=document.createElement('main');document.body.append(root);
      const bootstrap={viewModule:compiled.View,commandModule:compiled.Command,queryModule:compiled.Query,journalModule:compiled.Journal,headName:'workspace',root,labels:new Uint8Array(labels)};
      await reject('bootstrap storage failure','host-unavailable',()=>api.openWorkspaceView({...bootstrap,store:{loadIdentity:async()=>{throw {private:'must not escape'};}}}));
      const store=await openStore('view-diagnostic-'+crypto.randomUUID());
      try{
        await store.saveIdentity(await createIdentity());
        const view=await api.openWorkspaceView({...bootstrap,store});
        try{
          await reject('public dispatch arity','invalid-input',()=>view.dispatch());
          await reject('public dispatch cap','invalid-input',()=>view.dispatch(new Uint8Array(4099)));
          view.close();await reject('public terminal dispatch','view-closed',()=>view.dispatch(new Uint8Array([1])));
        }finally{view.close();}
      }finally{store.close();root.remove();}
      return{cases,codes:[...codes].sort()};
    },{modules:Object.fromEntries(Object.entries(build.bytes).map(([name,bytes])=>[name,[...bytes]])),labels:[...build.labels],rejections:rejectionVectors.map(v=>({request:[...v.request],detail:v.response[0]}))});
  });
  assert.deepEqual(result.codes,registered,'every actual View diagnostic is reached');return result;
}
