import assert from 'node:assert/strict';
import {createHash, createPublicKey, verify} from 'node:crypto';
import test from 'node:test';
import {withBrowser} from './browser-test-server.mjs';

async function inBrowser(operation) {
  return withBrowser(async ({browser, baseURL}) => {
    const page = await browser.newPage();
    const failures = [];
    page.on('pageerror', error => failures.push(error.message));
    page.on('request', request => {
      if (!request.url().startsWith(baseURL)) failures.push(request.url());
    });
    await page.route(baseURL, route => route.fulfill({status:200,contentType:'text/html',headers:{'Cross-Origin-Opener-Policy':'same-origin','Cross-Origin-Embedder-Policy':'require-corp'},body:'<!doctype html><title>Isolated browser acceptance</title>'}));
    await page.goto(baseURL);
    const result = await page.evaluate(operation);
    await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 0)));
    assert.deepEqual(failures, []);
    return result;
  });
}

test('Chromium random bytes use native bounded fresh buffers', {timeout: 30000}, async () => {
  const result = await inBrowser(async () => {
    const {randomBytes, MAX_RANDOM_BYTES} = await import('./identity.mjs');
    const a = randomBytes(32), b = randomBytes(32);
    const maximum = randomBytes(MAX_RANDOM_BYTES);
    return {bound: MAX_RANDOM_BYTES, sizes: [randomBytes(1).length, a.length, maximum.length],
      fresh: a.buffer !== b.buffer, differ: a.some((value, index) => value !== b[index]),
      native: Object.getPrototypeOf(a) === Uint8Array.prototype};
  });
  assert.deepEqual(result, {bound: 65536, sizes: [1, 32, 65536], fresh: true, differ: true, native: true});
});

test('Chromium random boundary rejects input before effects and sanitizes unavailable providers', {timeout: 30000}, async () => {
  const result = await inBrowser(async () => {
    const {randomBytes} = await import('./identity.mjs');
    const original = Object.getOwnPropertyDescriptor(globalThis, 'crypto');
    const codes = []; let calls = 0;
    try {
      Object.defineProperty(globalThis, 'crypto', {configurable: true, get() { calls++; throw Error('private error'); }});
      for (const size of [0, -1, 1.5, 65537, NaN, Infinity, null, '32', 32n,
        {valueOf() { calls++; throw Error('coercion'); }}]) {
        try { randomBytes(size); codes.push('accepted'); } catch (error) { codes.push(error.code); }
      }
      if (calls !== 0) throw Error('invalid input reached provider');
      try { randomBytes(32); codes.push('accepted'); }
      catch (error) {
        codes.push(error.code);
        if (error.message !== 'crypto-unavailable' || Object.hasOwn(error, 'cause')) throw Error('provider leak');
      }
    } finally {
      if (original) Object.defineProperty(globalThis, 'crypto', original); else delete globalThis.crypto;
    }
    return {codes, calls, restored: randomBytes(32).length};
  });
  assert.deepEqual(result, {codes: [...Array(10).fill('invalid-input'), 'crypto-unavailable'], calls: 1, restored: 32});
});

test('Chromium identity is nonextractable and its signature verifies independently', {timeout: 30000}, async () => {
  assert.equal(await inBrowser(checkIntrinsicKeys), 14);
  assert.equal(await inBrowser(checkPrototypeLifecycle), 3);
  const result = await inBrowser(async () => {
    const {createIdentity, signBytes, verifyBytes, validateIdentity} = await import('./identity.mjs');
    const identity = await createIdentity();
    await validateIdentity(identity);
    const payload = Uint8Array.of(0, 255, 7);
    const signature = await signBytes(identity, 'test/1', payload);
    let exportFailure;
    try { await crypto.subtle.exportKey('pkcs8', identity.privateKey); }
    catch (error) { exportFailure = error.name; }
    return {publicKey: [...identity.publicKey], signature: [...signature],
      principal: identity.principal, keys: Object.keys(identity).sort(),
      extractable: identity.privateKey.extractable, exportFailure,
      valid: await verifyBytes(identity.publicKey, 'test/1', payload, signature)};
  });
  assert.deepEqual(result.keys, ['principal', 'privateKey', 'publicKey']);
  assert.equal(result.extractable, false);
  assert.equal(result.exportFailure, 'InvalidAccessError');
  assert.equal(result.valid, true);
  const publicKey = Buffer.from(result.publicKey);
  assert.equal(publicKey.length, 65);
  assert.equal(result.principal, `sha256:${createHash('sha256').update(publicKey).digest('hex')}`);
  const key = createPublicKey({format: 'jwk', key: {kty: 'EC', crv: 'P-256',
    x: publicKey.subarray(1, 33).toString('base64url'),
    y: publicKey.subarray(33).toString('base64url')}});
  const payload = Buffer.from([0, 255, 7]);
  const message = Buffer.concat([Buffer.from('prismpm/browser-signature/1\0'),
    Buffer.from([0, 6]), Buffer.from('test/1'), payload]);
  assert.equal(result.signature.length, 64);
  assert.equal(verify('sha256', message, {key, dsaEncoding: 'ieee-p1363'}, Buffer.from(result.signature)), true);
  assert.equal(verify('sha256', payload, {key, dsaEncoding: 'ieee-p1363'}, Buffer.from(result.signature)), false);
});

test('Chromium rejects changed signature contexts, authors, payloads, and signatures', {timeout: 30000}, async () => {
  const result = await inBrowser(async () => {
    const {createIdentity, signBytes, verifyBytes, validateIdentity} = await import('./identity.mjs');
    const first = await createIdentity(), second = await createIdentity();
    const payload = Uint8Array.of(7, 8, 9);
    const signature = await signBytes(first, 'test.records/1', payload);
    const results = [await verifyBytes(first.publicKey, 'test.records/1', payload, signature),
      await verifyBytes(first.publicKey, 'test.records/2', payload, signature),
      await verifyBytes(second.publicKey, 'test.records/1', payload, signature),
      await verifyBytes(first.publicKey, 'test.records/1', Uint8Array.of(7, 8, 0), signature)];
    signature[0] ^= 1;
    results.push(await verifyBytes(first.publicKey, 'test.records/1', payload, signature));
    for (const identity of [{...first, privateKey: second.privateKey}, {...first, principal: second.principal}]) {
      try { await validateIdentity(identity); results.push('accepted'); }
      catch (error) { results.push(error.code); }
    }
    return results;
  });
  assert.deepEqual(result, [true, false, false, false, false, 'identity-corrupt', 'identity-corrupt']);
});

test('Chromium captures signing, verification, and identity inputs before asynchronous work', {timeout: 30000}, async () => {
  const result = await inBrowser(async () => {
    const {createIdentity, signBytes, verifyBytes, validateIdentity} = await import('./identity.mjs');
    const first = await createIdentity(), second = await createIdentity();
    const payload = Uint8Array.of(7, 8, 9), signingIdentity = {...first};
    const signing = signBytes(signingIdentity, 'test/1', payload);
    signingIdentity.privateKey = second.privateKey;
    payload.fill(0);
    const signature = await signing;
    const valid = await verifyBytes(first.publicKey, 'test/1', Uint8Array.of(7, 8, 9), signature);
    const publicKey = new Uint8Array(first.publicKey), capturedPayload = Uint8Array.of(7, 8, 9);
    const verifying = verifyBytes(publicKey, 'test/1', capturedPayload, signature);
    publicKey.fill(0); capturedPayload.fill(0); signature.fill(0);
    const mutable = {...first, publicKey: new Uint8Array(first.publicKey)};
    const validating = validateIdentity(mutable);
    mutable.privateKey = second.privateKey; mutable.principal = second.principal; mutable.publicKey.fill(0);
    const retained = await validating;
    return {valid, capturedVerification: await verifying, retainedPrincipal: retained.principal === first.principal,
      retainedKey: retained.privateKey === first.privateKey, retainedBytes: [...retained.publicKey],
      expectedBytes: [...first.publicKey]};
  });
  assert.equal(result.valid, true);
  assert.equal(result.capturedVerification, true);
  assert.equal(result.retainedPrincipal, true);
  assert.equal(result.retainedKey, true);
  assert.deepEqual(result.retainedBytes, result.expectedBytes);
});

test('Chromium enforces malformed input, key, and exact payload bounds', {timeout: 30000}, async () => {
  assert.equal(await inBrowser(async () => {
    const frame=document.createElement('iframe');document.body.append(frame);
    return await (async function checkIntrinsicBytes(realm) {
  const {bytesCopy}=await import('./identity.mjs');
  let checked=0;
  const equal=(actual,expected,label)=>{if(actual.length!==expected.length||actual.some((b,i)=>b!==expected[i]))throw Error(label);checked++;};
  const reject=(value,maximum,label)=>{let caught;try{bytesCopy(value,maximum);}catch(error){caught=error;}if(caught?.code!=='invalid-input')throw Error(label+' must reject invalid-input');checked++;};
  const oversize=new Uint8Array(4097);Object.defineProperty(oversize,'byteLength',{value:0});
  reject(oversize,4096,'shadowed byteLength');
  const wrong=new Uint16Array(2);Object.defineProperty(wrong,Symbol.toStringTag,{value:'Uint8Array'});
  reject(wrong,4,'forged typed-array tag');
  reject(Object.create(Uint8Array.prototype),4,'prototype-only array');
  reject(new Proxy(new Uint8Array(4),{}),4,'typed-array proxy');
  for(const factory of [()=>new SharedArrayBuffer(4),()=>new realm.SharedArrayBuffer(4)]) {
    const shared=new Uint8Array(factory());Object.defineProperty(shared,'buffer',{value:new ArrayBuffer(4)});
    reject(shared,4,'shadowed shared backing');
  }
  const detached=new Uint8Array(4);structuredClone(detached.buffer,{transfer:[detached.buffer]});reject(detached,4,'detached view');
  const growable=new ArrayBuffer(8,{maxByteLength:16}),outOfBounds=new Uint8Array(growable,4,4);
  if(typeof growable.resize!=='function')throw Error('pinned runtime must support resizable buffers');
  growable.resize(2);reject(outOfBounds,4,'out-of-bounds resizable view');
  class Subclass extends Uint8Array {}
  const unusual=new Subclass([1,2,3,4]);let touched=0;
  for(const key of ['buffer','byteLength','length','constructor',Symbol.iterator])Object.defineProperty(unusual,key,{get(){touched++;throw Error('untrusted metadata read');}});
  Object.defineProperty(Subclass,Symbol.species,{get(){touched++;throw Error('untrusted species read');}});
  const copied=bytesCopy(unusual,4);equal(copied,[1,2,3,4],'subclass exact bytes');
  if(touched!==0||Object.getPrototypeOf(copied)!==Uint8Array.prototype)throw Error('copy executed untrusted metadata');
  unusual[0]=9;equal(copied,[1,2,3,4],'copy retained source alias');
  equal(bytesCopy(new realm.Uint8Array([9,8,7]),3),[9,8,7],'cross-realm exact bytes');
  const validResizable=new ArrayBuffer(4,{maxByteLength:8}),view=new Uint8Array(validResizable);view.set([4,3,2,1]);
  const stable=bytesCopy(view,4);validResizable.resize(0);equal(stable,[4,3,2,1],'resizable copy alias');
  for(const maximum of [-1,NaN,Infinity,1.5,'4'])reject(new Uint8Array(0),maximum,'invalid bound');
  for(const size of [0,32,65,135,4096,1048576]){const value=new Uint8Array(size);if(size)value[size-1]=255;const copy=bytesCopy(value,size);equal(copy,value,'exact bound '+size);if(copy===value)throw Error('copy aliases source');}
  return checked;
})(frame.contentWindow);
  }), 23);
  const result = await inBrowser(async () => {
    const {BrowserEffectError, createIdentity, signBytes, verifyBytes, validateIdentity} = await import('./identity.mjs');
    const identity = await createIdentity(), codes = [];
    const reject = async operation => {
      try { await operation(); codes.push('accepted'); }
      catch (error) { codes.push(error.code ?? error.name); }
    };
    for (const context of ['', 'bad\0domain', '../scope', 'x'.repeat(129), 12]) {
      await reject(() => signBytes(identity, context, new Uint8Array()));
    }
    for (const bytes of [[], new Uint16Array(1), new Uint8Array(1048577)]) {
      await reject(() => signBytes(identity, 'test/1', bytes));
    }
    await reject(() => verifyBytes(new Uint8Array(65), 'test/1', new Uint8Array(), new Uint8Array(64)));
    await reject(() => verifyBytes(identity.publicKey, 'test/1', new Uint8Array(), new Uint8Array(63)));
    const detached = Uint8Array.of(1);
    structuredClone(detached.buffer, {transfer: [detached.buffer]});
    await reject(() => signBytes(identity, 'test/1', detached));
    await reject(() => signBytes({...identity, privateKey: {}}, 'test/1', new Uint8Array()));
    const extractable = await crypto.subtle.generateKey({name: 'ECDSA', namedCurve: 'P-256'}, true, ['sign', 'verify']);
    await reject(() => signBytes({...identity, privateKey: extractable.privateKey}, 'test/1', new Uint8Array()));
    const maximum = new Uint8Array(1048576);
    const signature = await signBytes(identity, 'test/1', maximum);
    const hostileCode = new BrowserEffectError('identity-corrupt');
    Object.defineProperty(hostileCode, 'code', {get() { throw new Error('untrusted getter'); }});
    const unavailable = Object.assign(new BrowserEffectError('crypto-unavailable'), {payload: 'untrusted'});
    const trapped = [];
    for (const thrown of [null, undefined, 0, false, 'untrusted', {},
      {code: 'crypto-unavailable', payload: 'untrusted'}, hostileCode,
      new Proxy({}, {getPrototypeOf() { throw new Error('untrusted prototype'); }}), unavailable]) {
      try { await validateIdentity(new Proxy({}, {ownKeys() { throw thrown; }})); trapped.push(false); }
      catch (error) {
        trapped.push(error instanceof BrowserEffectError && error !== thrown
          && error.code === (thrown === unavailable ? 'crypto-unavailable' : 'identity-corrupt')
          && Object.keys(error).sort().join(',') === 'code,name' && !Object.hasOwn(error, 'cause'));
      }
    }
    const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'crypto');
    try {
      Object.defineProperty(globalThis, 'crypto', {configurable: true, value: {}});
      await reject(() => validateIdentity(identity));
    } finally {
      if (descriptor) Object.defineProperty(globalThis, 'crypto', descriptor);
      else delete globalThis.crypto;
    }
    return {codes, trapped, restored: (await validateIdentity(identity)).privateKey === identity.privateKey,
      maximumValid: await verifyBytes(identity.publicKey, 'test/1', maximum, signature)};
  });
  assert.deepEqual(result.codes, [...Array(11).fill('invalid-input'), ...Array(2).fill('identity-corrupt'), 'crypto-unavailable']);
  assert.deepEqual(result.trapped, Array(10).fill(true));
  assert.equal(result.restored, true);
  assert.equal(result.maximumValid, true);
});


async function checkIntrinsicKeys() {
  const {createIdentity,identityPrincipal,signBytes,validateIdentity}=await import('./identity.mjs');
  let checked=0;
  const reject=async(operation,label)=>{let caught;try{await operation();}catch(error){caught=error;}if(caught?.code!=='identity-corrupt')throw Error(label+' must reject identity-corrupt');checked++;};
  const extractable=await crypto.subtle.generateKey({name:'ECDSA',namedCurve:'P-256'},true,['sign','verify']);
  const actual=Object.getOwnPropertyDescriptor(CryptoKey.prototype,'extractable').get;
  if(!Reflect.apply(actual,extractable.privateKey,[]))throw Error('negative key is not extractable');
  // Independently establish actual key extractability without exposing key bytes.
  if((await crypto.subtle.exportKey('pkcs8',extractable.privateKey)).byteLength===0)throw Error('extractable key export failed');
  Object.defineProperty(extractable.privateKey,'extractable',{value:false});
  const publicKey=new Uint8Array(await crypto.subtle.exportKey('raw',extractable.publicKey));
  const identity={privateKey:extractable.privateKey,publicKey,principal:await identityPrincipal(publicKey)};
  await reject(()=>signBytes(identity,'test/1',new Uint8Array()),'shadowed extractability signing');
  await reject(()=>validateIdentity(identity),'shadowed extractability possession');
  const wrongCurve=await crypto.subtle.generateKey({name:'ECDSA',namedCurve:'P-384'},false,['sign','verify']);
  Object.defineProperty(wrongCurve.privateKey,'algorithm',{value:{name:'ECDSA',namedCurve:'P-256'}});
  await reject(()=>signBytes({privateKey:wrongCurve.privateKey},'test/1',new Uint8Array()),'shadowed curve');
  const secret=await crypto.subtle.generateKey({name:'HMAC',hash:'SHA-256'},false,['sign']);
  Object.defineProperties(secret,{type:{value:'private'},algorithm:{value:{name:'ECDSA',namedCurve:'P-256'}}});
  await reject(()=>signBytes({privateKey:secret},'test/1',new Uint8Array()),'shadowed secret key type');
  const valid=await createIdentity();let touched=0;
  for(const property of ['type','extractable','algorithm','usages'])Object.defineProperty(valid.privateKey,property,{get(){touched++;throw Error('untrusted key metadata');}});
  await reject(()=>signBytes(valid,'test/1',Uint8Array.of(1)),'shadow key metadata');
  if(touched!==0)throw Error('untrusted key metadata executed');
  await reject(()=>signBytes({privateKey:Object.create(CryptoKey.prototype)},'test/1',new Uint8Array()),'prototype-only key');
  await reject(()=>signBytes({privateKey:new Proxy(valid.privateKey,{})},'test/1',new Uint8Array()),'proxied key');

  const inherited=await createIdentity();let inheritedReads=0;
  Object.setPrototypeOf(inherited.privateKey,Object.create(CryptoKey.prototype,{usages:{get(){inheritedReads++;throw Error('inherited metadata');}}}));
  await reject(()=>signBytes(inherited,'test/1',new Uint8Array()),'inherited key metadata');
  if(inheritedReads!==0)throw Error('inherited key metadata executed');
  for(const field of ['algorithm','usages']) {
    const item=await createIdentity(),get=Object.getOwnPropertyDescriptor(CryptoKey.prototype,field).get;
    const metadata=Reflect.apply(get,item.privateKey,[]);let metadataReads=0;
    Object.defineProperty(metadata,field==='algorithm'?'name':'0',{get(){metadataReads++;throw Error('nested metadata');}});
    if(Reflect.apply(get,item.privateKey,[])===metadata)await reject(()=>signBytes(item,'test/1',new Uint8Array()),'nested '+field+' metadata');
    else {if((await signBytes(item,'test/1',new Uint8Array())).length!==64)throw Error('fresh native metadata');checked++;}
    if(metadataReads!==0)throw Error('nested metadata executed');
  }
  const cachedCurve=await crypto.subtle.generateKey({name:'ECDSA',namedCurve:'P-384'},false,['sign','verify']);
  Reflect.apply(Object.getOwnPropertyDescriptor(CryptoKey.prototype,'algorithm').get,cachedCurve.privateKey,[]).namedCurve='P-256';
  await reject(()=>signBytes({privateKey:cachedCurve.privateKey},'test/1',new Uint8Array()),'actual signature shape');

  const prototypeIdentity=await createIdentity(),nativeLayer=Object.getPrototypeOf(prototypeIdentity.privateKey);
  let proxyReads=0;
  await reject(()=>signBytes({privateKey:new Proxy(prototypeIdentity.privateKey,{get(){proxyReads++;throw Error('forged native brand');}})},'test/1',new Uint8Array()),'native key proxy brand');
  if(proxyReads!==0)throw Error('Chromium native key brand ran proxy trap');
  const forged=Object.create(nativeLayer,Object.getOwnPropertyDescriptors(prototypeIdentity.privateKey));
  await reject(()=>signBytes({privateKey:forged},'test/1',new Uint8Array()),'copied native key wrapper');
  const prior=Object.getOwnPropertyDescriptor(nativeLayer,'algorithm');let nativeReads=0;
  try {
    Object.defineProperty(nativeLayer,'algorithm',{configurable:true,get(){nativeReads++;throw Error('changed native metadata');}});
    await reject(()=>signBytes(prototypeIdentity,'test/1',new Uint8Array()),'changed native metadata descriptor');
    if(nativeReads!==0)throw Error('changed native metadata descriptor executed');
  } finally {if(prior)Object.defineProperty(nativeLayer,'algorithm',prior);else delete nativeLayer.algorithm;}
  return checked;
}


async function checkPrototypeLifecycle() {
  const {createIdentity,signBytes,verifyBytes}=await import('./identity.mjs');
  const original=globalThis.crypto,descriptor=Object.getOwnPropertyDescriptor(globalThis,'crypto');
  const first=await createIdentity(),second=await createIdentity();
  const provider=()=>Object.fromEntries(['digest','importKey','generateKey','exportKey','sign','verify'].map(name=>[name,original.subtle[name].bind(original.subtle)]));
  const install=subtle=>Object.defineProperty(globalThis,'crypto',{configurable:true,value:{subtle,getRandomValues:original.getRandomValues.bind(original)}});
  const rejected=async(promise,code)=>{let error;try{await promise;}catch(value){error=value;}if(error?.code!==code)throw Error('expected '+code+', got '+error?.code);};
  try {
    const recovering=provider();let attempts=0;
    recovering.generateKey=async(...args)=>{if(++attempts===1)throw Error('private provider failure');return original.subtle.generateKey(...args);};
    install(recovering);await rejected(signBytes(first,'test/1',new Uint8Array()),'crypto-unavailable');
    const recovered=await signBytes(first,'test/1',new Uint8Array());
    if(attempts!==2||!await verifyBytes(first.publicKey,'test/1',new Uint8Array(),recovered))throw Error('reference prototype failure must recover');
    const delayed=provider();let entered,release,generated=0,touched=0;
    const ready=new Promise(resolve=>{entered=resolve;}),wait=new Promise(resolve=>{release=resolve;});
    delayed.generateKey=async(...args)=>{generated++;const pair=await original.subtle.generateKey(...args);entered();await wait;return pair;};
    install(delayed);
    const a=signBytes(first,'test/1',Uint8Array.of(1)),b=signBytes(second,'test/1',Uint8Array.of(2));
    await ready;
    Object.setPrototypeOf(first.privateKey,Object.create(Object.getPrototypeOf(first.privateKey),{usages:{get(){touched++;throw Error('late prototype metadata');}}}));
    release();await rejected(a,'identity-corrupt');
    const signature=await b;
    if(generated!==1||touched!==0||!await verifyBytes(second.publicKey,'test/1',Uint8Array.of(2),signature))throw Error('shared prototype lookup or late key capture failed');
    return 3;
  } finally {if(descriptor)Object.defineProperty(globalThis,'crypto',descriptor);else delete globalThis.crypto;}
}
