import assert from 'node:assert/strict';
import { createPublicKey, verify, webcrypto } from 'node:crypto';
import test from 'node:test';
import {runInNewContext} from 'node:vm';
import { BrowserEffectError, createIdentity, identityPrincipal, signBytes, verifyBytes, validateIdentity } from './identity.mjs';

globalThis.crypto ??= webcrypto;

test('randomness uses fresh bounded byte buffers and the WebCrypto provider exactly once', async () => {
  const {randomBytes, MAX_RANDOM_BYTES} = await import('./identity.mjs');
  assert.equal(MAX_RANDOM_BYTES, 65536);
  const original = Object.getOwnPropertyDescriptor(globalThis, 'crypto');
  let calls = 0;
  const seen = [];
  const provider = {getRandomValues(bytes) {
    assert.equal(this, provider);
    assert.equal(Object.getPrototypeOf(bytes), Uint8Array.prototype);
    assert.ok(!seen.includes(bytes));
    seen.push(bytes); calls++;
    for (let i = 0; i < bytes.length; i++) bytes[i] = (i + calls) % 256;
    return bytes;
  }};
  try {
    Object.defineProperty(globalThis, 'crypto', {configurable: true, value: provider});
    for (const size of [1, 32, 65536]) {
      const bytes = randomBytes(size);
      assert.equal(bytes.length, size);
      assert.deepEqual(bytes, Uint8Array.from({length: size}, (_, i) => (i + calls) % 256));
    }
    assert.equal(calls, 3);
  } finally { Object.defineProperty(globalThis, 'crypto', original); }
  const first = randomBytes(32), second = randomBytes(32);
  assert.equal(first.length, 32);
  assert.notEqual(first.buffer, second.buffer);
  // This is a smoke check, not statistical entropy certification.
  assert.notDeepEqual(first, second);
});

test('randomness rejects invalid sizes before touching the provider or coercing objects', async () => {
  const {randomBytes} = await import('./identity.mjs');
  const original = Object.getOwnPropertyDescriptor(globalThis, 'crypto');
  let touched = 0;
  try {
    Object.defineProperty(globalThis, 'crypto', {configurable: true, get() { touched++; throw Error('provider accessed'); }});
    for (const size of [undefined, null, false, true, 0, -0, -1, 1.5, NaN, Infinity,
      65537, Number.MAX_SAFE_INTEGER, 32n, '32', Symbol('size'),
      {valueOf() { touched++; throw Error('coercion'); }}]) {
      assert.throws(() => randomBytes(size), {code: 'invalid-input'});
    }
    assert.equal(touched, 0);
  } finally { Object.defineProperty(globalThis, 'crypto', original); }
});

test('randomness fails closed without exposing provider errors or using a fallback', async () => {
  const {randomBytes} = await import('./identity.mjs');
  const original = Object.getOwnPropertyDescriptor(globalThis, 'crypto');
  const mathRandom = Math.random;
  let fallbackCalls = 0;
  try {
    Math.random = () => { fallbackCalls++; throw Error('insecure fallback'); };
    for (const provider of [undefined, {}, {getRandomValues: 42},
      {get getRandomValues() { throw Error('private provider detail'); }},
      {getRandomValues() { throw new Proxy({}, {get() { throw Error('read thrown object'); }}); }}]) {
      Object.defineProperty(globalThis, 'crypto', {configurable: true, value: provider});
      assert.throws(() => randomBytes(32), error => {
        assert.ok(error instanceof BrowserEffectError);
        assert.equal(error.code, 'crypto-unavailable');
        assert.equal(error.message, 'crypto-unavailable');
        assert.deepEqual(Object.keys(error).sort(), ['code', 'name']);
        assert.equal(Object.hasOwn(error, 'cause'), false);
        return true;
      });
    }
    assert.equal(fallbackCalls, 0);
  } finally {
    Math.random = mathRandom;
    Object.defineProperty(globalThis, 'crypto', original);
  }
});

test('nonextractable identity proves possession without assigning organization roles', async () => {
  assert.equal(await checkIntrinsicKeys(), 12);
  assert.equal(await checkPrototypeLifecycle(), 3);
  const identity = await createIdentity();
  assert.match(identity.principal, /^sha256:[0-9a-f]{64}$/);
  assert.equal(identity.publicKey.length, 65);
  assert.equal(identity.privateKey.extractable, false);
  assert.equal(await identityPrincipal(identity.publicKey), identity.principal);
  const payload = new TextEncoder().encode('a portable signed record');
  const signature = await signBytes(identity, 'test.records/1', payload);
  assert.equal(signature.length, 64);
  assert.equal(await verifyBytes(identity.publicKey, 'test.records/1', payload, signature), true);
  await validateIdentity(identity);
  await assert.rejects(crypto.subtle.exportKey('pkcs8', identity.privateKey));
  assert.deepEqual(Object.keys(identity).sort(), ['principal', 'privateKey', 'publicKey']);
});

test('signatures cannot cross contexts, authors, or bytes', async () => {
  const first = await createIdentity();
  const second = await createIdentity();
  const bytes = Uint8Array.of(0, 255, 42);
  const signature = await signBytes(first, 'test.records/1', bytes);
  assert.equal(await verifyBytes(first.publicKey, 'test.records/2', bytes, signature), false);
  assert.equal(await verifyBytes(second.publicKey, 'test.records/1', bytes, signature), false);
  assert.equal(await verifyBytes(first.publicKey, 'test.records/1', Uint8Array.of(0, 255, 43), signature), false);
  signature[0] ^= 1;
  assert.equal(await verifyBytes(first.publicKey, 'test.records/1', bytes, signature), false);
  await assert.rejects(validateIdentity({ ...first, privateKey: second.privateKey }), { code: 'identity-corrupt' });
  await assert.rejects(validateIdentity({ ...first, principal: second.principal }), { code: 'identity-corrupt' });
});

test('cryptographic boundary rejects malformed and excessive input before dispatch', async () => {
  assert.equal(await checkIntrinsicBytes(runInNewContext('({Uint8Array, SharedArrayBuffer})')), 23);
  const identity = await createIdentity();
  for (const context of ['', 'bad\0domain', 'x'.repeat(129), '../scope', 12]) {
    await assert.rejects(signBytes(identity, context, new Uint8Array()), { code: 'invalid-input' });
  }
  for (const bytes of [[], new Uint8Array(1048577), new Uint16Array(1)]) {
    await assert.rejects(signBytes(identity, 'test/1', bytes), { code: 'invalid-input' });
  }
  await assert.rejects(verifyBytes(new Uint8Array(65), 'test/1', new Uint8Array(), new Uint8Array(64)), { code: 'invalid-input' });
  await assert.rejects(verifyBytes(identity.publicKey, 'test/1', new Uint8Array(), new Uint8Array(63)), { code: 'invalid-input' });
  const bytes = new Uint8Array(1048576);
  const signature = await signBytes(identity, 'test/1', bytes);
  assert.equal(await verifyBytes(identity.publicKey, 'test/1', bytes, signature), true);
  const hostileCode = new BrowserEffectError('identity-corrupt');
  Object.defineProperty(hostileCode, 'code', {get() { throw new Error('untrusted getter'); }});
  const unavailable = Object.assign(new BrowserEffectError('crypto-unavailable'), {payload: 'untrusted'});
  for (const thrown of [null, undefined, 0, false, 'untrusted', {},
    {code: 'crypto-unavailable', payload: 'untrusted'}, hostileCode,
    new Proxy({}, {getPrototypeOf() { throw new Error('untrusted prototype'); }}), unavailable]) {
    await assert.rejects(validateIdentity(new Proxy({}, {ownKeys() { throw thrown; }})), error => {
      assert.ok(error instanceof BrowserEffectError);
      assert.notEqual(error, thrown);
      assert.equal(error.code, thrown === unavailable ? 'crypto-unavailable' : 'identity-corrupt');
      assert.deepEqual(Object.keys(error).sort(), ['code', 'name']);
      assert.equal(Object.hasOwn(error, 'cause'), false);
      return true;
    });
  }
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'crypto');
  try {
    Object.defineProperty(globalThis, 'crypto', {configurable: true, value: {}});
    await assert.rejects(validateIdentity(identity), {code: 'crypto-unavailable'});
  } finally {
    if (descriptor) Object.defineProperty(globalThis, 'crypto', descriptor);
    else delete globalThis.crypto;
  }
  assert.equal((await validateIdentity(identity)).privateKey, identity.privateKey);
});

test('signed input is captured before asynchronous key operations', async () => {
  const identity = await createIdentity();
  const bytes = Uint8Array.of(7, 8, 9);
  const operation = signBytes(identity, 'test/1', bytes);
  bytes[0] = 0;
  const signature = await operation;
  assert.equal(await verifyBytes(identity.publicKey, 'test/1', Uint8Array.of(7, 8, 9), signature), true);
});

test('identity validation captures its private key before any asynchronous work', async () => {
  const original = await createIdentity();
  const replacement = await createIdentity();
  let reads = 0;
  const identity = { publicKey: original.publicKey, principal: original.principal,
    get privateKey() { return ++reads === 1 ? original.privateKey : replacement.privateKey; } };
  const validated = await validateIdentity(identity);
  assert.equal(reads, 1);
  assert.equal(validated.privateKey, original.privateKey);
});

test('independent native verifier checks the exact domain-separated wire format', async () => {
  const identity = await createIdentity();
  const payload = Uint8Array.of(0, 255, 7);
  const signature = await signBytes(identity, 'test/1', payload);
  const key = createPublicKey({ format: 'jwk', key: {
    kty: 'EC', crv: 'P-256',
    x: Buffer.from(identity.publicKey.slice(1, 33)).toString('base64url'),
    y: Buffer.from(identity.publicKey.slice(33)).toString('base64url'),
  } });
  const message = Buffer.concat([Buffer.from('prismpm/browser-signature/1\0'),
    Buffer.from([0, 6]), Buffer.from('test/1'), Buffer.from(payload)]);
  assert.equal(verify('sha256', message, { key, dsaEncoding: 'ieee-p1363' }, signature), true);
  assert.equal(verify('sha256', payload, { key, dsaEncoding: 'ieee-p1363' }, signature), false);
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
  const prior=Object.getOwnPropertyDescriptor(nativeLayer,'algorithm');let nativeReads=0;
  try {
    Object.defineProperty(nativeLayer,'algorithm',{configurable:true,get(){nativeReads++;throw Error('changed native metadata');}});
    await reject(()=>signBytes(prototypeIdentity,'test/1',new Uint8Array()),'changed native metadata descriptor');
    if(nativeReads!==0)throw Error('changed native metadata descriptor executed');
  } finally {if(prior)Object.defineProperty(nativeLayer,'algorithm',prior);else delete nativeLayer.algorithm;}
  return checked;
}

async function checkIntrinsicBytes(realm) {
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
