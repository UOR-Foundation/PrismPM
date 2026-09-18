// Negative-only access to unchanged private guards; never shipped as an API.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {withBrowser} from '../../sdk/browser/browser-test-server.mjs';
import {captureGeneratedCalls, replayNative} from './replay.mjs';

export async function verifyDiagnostics({wasmBytes, native, work}) {
  const source = readFileSync(new URL('../../sdk/browser/journal.mjs', import.meta.url), 'utf8');
  const registry = readFileSync(new URL('../../model/browser-diagnostics.toml', import.meta.url), 'utf8');
  const codes = [...registry.matchAll(/^code = "([a-z-]+)"$/gm)].map(row => row[1]);
  const used = [...new Set([...source.matchAll(/(?:fail|JournalAdapterError)\('([a-z-]+)'/g)].map(row => row[1]))].sort();
  assert.deepEqual(codes, used, 'complete actual journal-origin diagnostic register');
  assert.equal(codes.length, 18);
  const suffix = '\nexport const __negativeBoundary = Object.freeze({hashBytes, u24, accepted, plan, interpreter});\n';
  assert.ok(!source.includes('__negativeBoundary'), 'test access is absent from shipped adapter');
  const served = source + suffix;
  assert.equal(served.slice(0, -suffix.length), source, 'all production bodies are exact');
  const {cases, generatedCalls} = await withBrowser(async ({browser, baseURL}) => {
    const page = await browser.newPage();
    await page.addInitScript(captureGeneratedCalls);
    await page.route('**/journal.mjs', route => route.fulfill({contentType:'text/javascript', body:served}));
    await page.goto(baseURL);
    return page.evaluate(async wasmBytes => {
      const {openJournal, __negativeBoundary: guard} = await import('/journal.mjs');
      const {createIdentity, digestBytes, signBytes} = await import('/identity.mjs');
      const {openStore} = await import('/store.mjs');
      const cases = [], module = await WebAssembly.compile(new Uint8Array(wasmBytes));
      const fail = message => { throw Error(message); };
      const check = (value, message) => { if (!value) fail(message); };
      const reject = async (label, code, operation) => {
        let error;
        try { await operation(); } catch (caught) { error = caught; }
        check(error?.code === code, `${label}: expected ${code}, received ${error?.code}`);
        cases.push(label); return error;
      };
      for (const [index, value] of [null, undefined, 4, [], 'sha256:00', 'SHA256:'+'0'.repeat(64), 'sha256:'+'g'.repeat(64)].entries()) {
        await reject('private hash guard '+index, 'invalid-hash', () => guard.hashBytes(value));
      }
      for (const [index, value] of [-1, 16777216, 1.5, NaN, '1', undefined].entries()) {
        await reject('private length guard '+index, 'invalid-length', () => guard.u24(value));
      }
      for (const [index, value] of [[], [0], [0,0,0,1,0,0,0], [0,1,0,39,0,0,0], [0,0,0,0,16,202,172]].entries()) {
        await reject('private output guard '+index, 'invalid-generated-output', () => guard.plan(new Uint8Array(value)));
      }
      await reject('private empty result guard', 'invalid-generated-output', () => guard.accepted(new Uint8Array()));
      await reject('public module shape', 'invalid-generated-module', () => openJournal({}, {}, 'workspace'));
      await reject('public head name', 'invalid-input', () => openJournal(module, {}, '../workspace'));
      await reject('real guest preallocation request bound', 'request-limit', () => guard.interpreter(module)(new Uint8Array(1235981)));
      const hex = value => Array.from(value, byte => byte.toString(16).padStart(2,'0')).join('');
      const unhex = value => Uint8Array.from(value.match(/../g), byte => parseInt(byte,16));
      const join = (...rows) => { const bytes = new Uint8Array(rows.reduce((n,row)=>n+row.length,0)); let at=0; for(const row of rows){bytes.set(row,at);at+=row.length;}return bytes; };
      const raw = guard.interpreter(module);
      function projection(op, envelope) {const result=raw(join([6,op],envelope));check(result[0]===0,'real generated projection');return result.slice(1);}
      const identity = await createIdentity();
      async function genesis(author) {
        const event = new Uint8Array(134);event[0]=1;event.fill(9,2,34);event.fill(1,34,66);event.set(author,98);
        const envelope=join([80,87,69,1],identity.publicKey,new Uint8Array(64),event);
        envelope.set(unhex((await digestBytes(projection(2,envelope))).slice(7)),167);
        envelope.set(await signBytes(identity,'prismpm/workspace-event/1',projection(1,envelope)),69);
        return envelope;
      }
      const valid = await genesis(unhex(identity.principal.slice(7)));
      const store = await openStore('journal-diagnostics-actual');
      const mismatchedStore = await openStore('journal-diagnostics-mismatched');
      try {
        const actual=await openJournal(module,store,'workspace');await actual.append(valid);await actual.refresh();
        await reject('real signature cannot claim another author','author-mismatch', async()=>actual.append(await genesis(new Uint8Array(32).fill(8))));
        const mismatched=await openJournal(module,{
          readHead:(...args)=>mismatchedStore.readHead(...args),readObject:(...args)=>mismatchedStore.readObject(...args),
          commit:async request=>{await mismatchedStore.commit(request);return 'sha256:'+'0'.repeat(64);},
        },'workspace');
        await reject('real commit wrong acknowledgment','storage-result-mismatch',()=>mismatched.append(valid));
        check(mismatched.snapshot().state.length===0,'wrong acknowledgment promoted state');
        await reject('wrong acknowledgment replay barrier','replay-required',()=>mismatched.append(valid));
        await mismatched.refresh();check(mismatched.snapshot().state.length>0,'real committed state was not recovered');
        cases.push('wrong acknowledgment authenticates durable recovery');
        for(const [index,value]of [null,'private exception',{get code(){throw Error('private getter');}}].entries()) {
          const error=await reject('unknown storage value '+index,'storage-outcome-unknown',()=>openJournal(module,{readHead:async()=>{throw value;}},'workspace'));
          check(error.name==='JournalAdapterError'&&error.message==='storage-outcome-unknown'&&error.detail===null&&!Object.hasOwn(error,'cause'),'unknown exception retained');
        }
        const known=await reject('known storage namespace sanitized','storage-unavailable',()=>openJournal(module,{readHead:async()=>{throw Object.assign(Error('private exception'),{code:'storage-unavailable',payload:'private'});}},'workspace'));
        check(known.name==='BrowserEffectError'&&known.message==='storage-unavailable'&&!Object.hasOwn(known,'payload')&&!Object.hasOwn(known,'cause'),'known storage payload leaked');
      } finally {store.close();mismatchedStore.close();}
      return {cases,generatedCalls:globalThis.__generatedJournalCalls};
    },Array.from(wasmBytes));
  });
  assert.equal(cases.length,30);
  await replayNative('diagnostics',generatedCalls,wasmBytes,{native,work});
  return {cases, sourceSha256:createHash('sha256').update(source).digest('hex')};
}
