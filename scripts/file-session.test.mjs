// SDK transport ownership only. Application semantics remain in the compiled guest.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {createServer} from 'node:http';
import test, {before,after} from 'node:test';
const require=createRequire('/opt/prismpm/oracles/package.json');
const {chromium}=require('playwright');
const browserPath=process.arch==='arm64'?'/ms-playwright/chromium_headless_shell-cft-157.0.8099.0/chrome-headless-shell-linux-arm64/chrome-headless-shell':'/ms-playwright/chromium_headless_shell-cft-157.0.8099.0/chrome-headless-shell-linux64/chrome-headless-shell';
const fixture=JSON.parse(readFileSync(new URL('../tests/data/file-session-model-document.json',import.meta.url),'utf8')).application;
const template=readFileSync(new URL('../crates/prismpm/src/workbench/runtime.js',import.meta.url),'utf8');
let browser;
before(async()=>{assert.equal(require('playwright/package.json').version,'1.62.1');browser=await chromium.launch({headless:true,executablePath:browserPath});assert.equal(browser.version(),'157.0.8099.0');});
after(async()=>{await browser?.close();});
function fields(text){let bytes=Buffer.from(text),values=[];while(bytes.length){const at=bytes.indexOf(58);assert(at>=1);const header=bytes.subarray(0,at).toString();assert.match(header,/^(0|[1-9][0-9]*)$/);const n=Number(header);assert(n<=bytes.length-at-1);values.push(new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes.subarray(at+1,at+1+n)));bytes=bytes.subarray(at+1+n);}return values;}
const ready=page=>page.waitForFunction(()=>document.querySelector('#guided-submit')?.disabled===false);
const result=page=>page.waitForFunction(()=>document.querySelector('#result-empty')?.hidden===true);
async function withPage(run,{delay=0,nextRequest=()=>''}={}){
 const view=structuredClone(fixture.workbench);view.session=structuredClone(fixture.session);view.execution.timeout_milliseconds=1000;
 const script=template.replace('__VIEW__',JSON.stringify(view)).replace('__INPUT_LIMIT__','4096').replace('__OUTPUT_LIMIT__','16384').replace('__BINDING__','"./unused.js"').replace('__TRANSPORT__','"hologram"');
 const invocations=[];
 const server=createServer(async(req,res)=>{
  res.setHeader('cache-control','no-store');
  if(req.url==='/app.js'){res.setHeader('content-type','text/javascript');res.end(script);return;}
  if(req.url==='/_hologram/intent'){
   let body='';for await(const chunk of req)body+=chunk;
   const payload=JSON.parse(body).payload;invocations.push(payload);const at=payload.indexOf('\x1f'),prefix=payload.slice(0,at),values=fields(payload.slice(at+1));
   const restore=prefix==='restore',raw=prefix===view.raw.request_prefix,operation=restore?'restore':raw?'raw':'echo';
   const value=restore||raw?values[0]:values[0]+values[1];
   const reject=values.at(-1)==='invalid';
   const answer={version:1,status:reject?'error':'ok',operation,message:'The transport fixture completed.',rows:0,columns:0,cells:[],details:[],request:nextRequest(values),response:reject?'forged-state':value,error:reject?'invalid':''};
   if(delay)await new Promise(resolve=>setTimeout(resolve,delay));
   res.setHeader('content-type','application/json');res.end(JSON.stringify({version:1,outputs:[values.at(-1)==='bad-json'?'{':JSON.stringify(answer)]}));return;
  }
  res.setHeader('content-type','text/html');res.end('<!doctype html><meta charset="utf-8"><title>File session component</title><body><script type="module" src="/app.js"></script></body>');
 });
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const context=await browser.newContext({acceptDownloads:true});const page=await context.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
 try{await page.goto(`http://127.0.0.1:${server.address().port}`);await ready(page);await run({page,invocations,view});assert.deepEqual(errors,[]);}
 finally{await context.close();server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
}
async function invoke(page,value){await page.locator('#field-message').fill(value);await page.locator('#guided-submit').click();await ready(page);}
async function saved(page){const download=page.waitForEvent('download');await page.locator('#save-session').click();return readFileSync(await (await download).path());}
test('file session successful transitions carry state and failed results preserve it',async()=>withPage(async({page,invocations})=>{
 await invoke(page,'first');await result(page);await invoke(page,'second');assert.deepEqual(fields(invocations[1].split('\x1f').slice(1).join('\x1f')),['first','second']);
 assert.equal((await saved(page)).toString(),'firstsecond');await invoke(page,'invalid');assert.equal((await saved(page)).toString(),'firstsecond');await invoke(page,'bad-json');assert.equal((await saved(page)).toString(),'firstsecond');
 assert.equal(await page.locator('#field-state').isVisible(),false);
}));
test('file session imports exact UTF-8 bytes despite textarea normalization',async()=>withPage(async({page,invocations})=>{
 const bytes=Buffer.from('\ufeffZoë\r\n\0\x1f: passage');await page.locator('#file-message').setInputFiles({name:'source.txt',mimeType:'text/plain',buffer:bytes});
 await page.waitForFunction(()=>document.querySelector('#field-message').value.includes('Zoë'));
 await page.locator('#guided-submit').click();await result(page);assert.deepEqual(Buffer.from(fields(invocations[0].slice(invocations[0].indexOf('\x1f')+1))[1]),bytes);assert.deepEqual(await saved(page),bytes);
}));
test('file session user edits invalidate pending file reads',async()=>withPage(async({page,invocations})=>{
 await page.evaluate(()=>{const original=File.prototype.arrayBuffer;File.prototype.arrayBuffer=async function(){await new Promise(r=>setTimeout(r,150));return original.call(this);};});
 await page.locator('#file-message').setInputFiles({name:'late.txt',mimeType:'text/plain',buffer:Buffer.from('stale file')});await page.locator('#field-message').fill('my edit');await page.waitForTimeout(220);assert.equal(await page.locator('#field-message').inputValue(),'my edit');
 await page.locator('#guided-submit').click();await result(page);assert.equal(fields(invocations[0].slice(invocations[0].indexOf('\x1f')+1))[1],'my edit');
}));
test('file session restore is validated and replacement requires explicit confirmation',async()=>withPage(async({page})=>{
 await invoke(page,'unsaved');page.once('dialog',dialog=>dialog.dismiss());await page.locator('#open-session').setInputFiles({name:'session.prism',mimeType:'application/octet-stream',buffer:Buffer.from('replacement')});assert.equal((await saved(page)).toString(),'unsaved');
 page.once('dialog',dialog=>dialog.accept());await page.locator('#open-session').setInputFiles({name:'session.prism',mimeType:'application/octet-stream',buffer:Buffer.from('replacement')});await page.waitForFunction(()=>document.querySelector('#result').textContent==='replacement');assert.equal((await saved(page)).toString(),'replacement');
 await page.locator('#open-session').setInputFiles({name:'bad.prism',mimeType:'application/octet-stream',buffer:Buffer.from('invalid')});await page.waitForFunction(()=>document.querySelector('#result-error').hidden===false);assert.equal((await saved(page)).toString(),'replacement');
}));
test('file session invalid UTF-8 and oversized selected files never dispatch',async()=>withPage(async({page,invocations})=>{
 for(const bytes of [Buffer.from([255]),Buffer.alloc(4097,97)]){await page.locator('#file-message').setInputFiles({name:'bad.txt',mimeType:'text/plain',buffer:bytes});await page.waitForFunction(()=>document.querySelector('#status').dataset.state==='input_error');assert.equal(invocations.length,0);}
 assert.equal(await page.locator('#save-session').isDisabled(),true);
}));
test('file session cancelled computation cannot replace prior state',async()=>withPage(async({page})=>{
 await invoke(page,'kept');await page.locator('#field-message').fill('late');await page.locator('#guided-submit').click();await page.locator('#cancel').click();await page.waitForTimeout(300);assert.equal((await saved(page)).toString(),'kept');
},{delay:180}));
test('file session oversized successful responses preserve prior state',async()=>withPage(async({page})=>{
 await invoke(page,'kept');await invoke(page,'x'.repeat(2048));await page.waitForFunction(()=>document.querySelector('#status').dataset.state==='response_error');assert.equal((await saved(page)).toString(),'kept');
}));
test('file session changes remain visibly unsaved after requesting a download',async()=>withPage(async({page,view})=>{
 await invoke(page,'kept');assert.equal(await page.locator('#session-unsaved').textContent(),view.session.unsaved_label);await saved(page);assert.equal(await page.locator('#session-unsaved').textContent(),view.session.unsaved_label);await page.locator('#reset').click();assert.equal((await saved(page)).toString(),'kept');
}));

test('file session next form prepares exact draft values without dispatching or changing state',async()=>withPage(async({page,invocations,view})=>{
 await invoke(page,'kept');await result(page);
 assert.equal(await page.locator('#next-form').textContent(),view.operations[0].label);
 assert.equal(invocations.length,1);await page.locator('#next-form').click();
 assert.equal(invocations.length,1);assert.equal((await saved(page)).toString(),'kept');
 assert.equal(await page.locator('#field-message').inputValue(),'é\n\0\x1f:');
 await page.locator('#guided-submit').click();await result(page);
 assert.deepEqual(fields(invocations[1].slice(invocations[1].indexOf('\x1f')+1)),['kept','é\r\n\0\x1f:']);
 await page.locator('#field-message').fill('edited');assert.equal(await page.locator('#next-form').isVisible(),false);
},{nextRequest:()=> 'ui-echo-v1\x1f0:7:é\r\n\0\x1f:'}));
test('file session invalid continuation refuses the entire response before replacing state',async()=>withPage(async({page})=>{
 await invoke(page,'kept');await result(page);
 await invoke(page,'bad-next');await page.waitForFunction(()=>document.querySelector('#status').dataset.state==='response_error');
 assert.equal((await saved(page)).toString(),'kept');assert.equal(await page.locator('#next-form').isVisible(),false);
},{nextRequest:values=>values.at(-1)==='bad-next'?'restore\x1f0:':''}));
