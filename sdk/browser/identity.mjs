// Generic Web Cryptography bindings. Application authority is never inferred
// from possession of a key; the generated model must authorize every effect.
const algorithm = Object.freeze({ name: 'ECDSA', namedCurve: 'P-256' });
const signatureAlgorithm = Object.freeze({ name: 'ECDSA', hash: 'SHA-256' });
const encoder = new TextEncoder();
const domain = encoder.encode('prismpm/browser-signature/1\0');
export const MAX_SIGNED_BYTES = 1048576;
export const MAX_RANDOM_BYTES = 65536;

// Capture native brands before examining caller-controlled objects. Ordinary
// typed-array/key properties can be shadowed without changing their backing data.
const apply = Reflect.apply;
const ByteArray = Uint8Array;
const typedArrayPrototype = Object.getPrototypeOf(ByteArray.prototype);
const byteTag = Object.getOwnPropertyDescriptor(typedArrayPrototype, Symbol.toStringTag)?.get;
const byteLength = Object.getOwnPropertyDescriptor(typedArrayPrototype, 'byteLength')?.get;
const byteBuffer = Object.getOwnPropertyDescriptor(typedArrayPrototype, 'buffer')?.get;
const arrayBufferLength = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, 'byteLength')?.get;
const byteSet = typedArrayPrototype.set;
const hasOwn = Object.prototype.hasOwnProperty;
const prototypeOf = Object.getPrototypeOf;
const descriptorsOf = Object.getOwnPropertyDescriptors;
const keyPrototype = globalThis.CryptoKey?.prototype;
const keyFields = ['type', 'extractable', 'algorithm', 'usages'];
const keyGetters = keyFields.map(field => keyPrototype
  ? Object.getOwnPropertyDescriptor(keyPrototype, field)?.get : undefined);
const providerPrototypes = new WeakMap();

function keyMetadata(algorithmValue, usagesValue) {
  // Providers may cache these otherwise native metadata objects. Read only own
  // data descriptors so a caller cannot turn a cached field into a callback.
  const fields = descriptorsOf(algorithmValue), uses = descriptorsOf(usagesValue);
  return prototypeOf(algorithmValue) === Object.prototype
    && Object.keys(fields).sort().join(',') === 'name,namedCurve'
    && fields.name.value === 'ECDSA' && fields.namedCurve.value === 'P-256'
    && Array.isArray(usagesValue) && prototypeOf(usagesValue) === Array.prototype
    && Object.keys(uses).sort().join(',') === '0,length'
    && uses.length.value === 1 && uses[0].value === 'sign';
}

function validSigningKey(key) {
  const [type, extractable, keyAlgorithm, usages] = keyGetters.map(getter => apply(getter, key, []));
  return type === 'private' && extractable === false && keyMetadata(keyAlgorithm, usages)
    && !keyFields.some(field => apply(hasOwn, key, [field]));
}

function unchangedKeyPrototype(expected) {
  // Only the exact provider-derived layer is allowed above the captured native
  // interface. Inspect descriptors without evaluating any shadowed getter.
  if (expected !== keyPrototype && (prototypeOf(expected) !== keyPrototype
      || keyFields.some(field => apply(hasOwn, expected, [field])))) return false;
  const fields = descriptorsOf(keyPrototype);
  return keyFields.every((field, index) => fields[field]?.get === keyGetters[index]
    && fields[field]?.set === undefined);
}

async function signingPrototype(provider) {
  let pending = providerPrototypes.get(provider);
  if (!pending) {
    pending = Promise.resolve().then(async () => {
      // Node and browsers expose different genuine native prototype chains.
      // Learn the exact one only from the already trusted crypto provider.
      const pair = await provider.generateKey(algorithm, false, ['sign', 'verify']);
      if (!validSigningKey(pair.privateKey)) throw new BrowserEffectError('crypto-unavailable');
      return prototypeOf(pair.privateKey);
    });
    providerPrototypes.set(provider, pending);
  }
  try { return await pending; }
  catch {
    if (providerPrototypes.get(provider) === pending) providerPrototypes.delete(provider);
    throw new BrowserEffectError('crypto-unavailable');
  }
}

function subtle() {
  const provider = globalThis.crypto?.subtle;
  if (!provider || ['digest', 'importKey', 'generateKey', 'exportKey', 'sign', 'verify']
    .some(method => typeof provider[method] !== 'function')) throw new BrowserEffectError('crypto-unavailable');
  return provider;
}

export class BrowserEffectError extends Error {
  constructor(code) {
    super(code);
    this.name = 'BrowserEffectError';
    this.code = code;
  }
}

export function randomBytes(length) {
  // No coercion or provider access precedes the bounded allocation contract.
  if (!Number.isSafeInteger(length) || length < 1 || length > MAX_RANDOM_BYTES) {
    throw new BrowserEffectError('invalid-input');
  }
  try {
    const bytes = new ByteArray(length);
    const provider = globalThis.crypto;
    apply(provider.getRandomValues, provider, [bytes]);
    return bytes;
  } catch {
    throw new BrowserEffectError('crypto-unavailable');
  }
}

export function bytesCopy(value, maximum = MAX_SIGNED_BYTES) {
  try {
    if (!Number.isSafeInteger(maximum) || maximum < 0
        || apply(byteTag, value, []) !== 'Uint8Array') throw new BrowserEffectError('invalid-input');
    // This native getter rejects SharedArrayBuffer even from another realm.
    apply(arrayBufferLength, apply(byteBuffer, value, []), []);
    const length = apply(byteLength, value, []);
    if (length > maximum) throw new BrowserEffectError('invalid-input');
    const copy = new ByteArray(length);
    // Typed-array set uses internal slots, not source iterator/species/getters;
    // it also rejects detached and out-of-bounds resizable views.
    apply(byteSet, copy, [value]);
    return copy;
  } catch {
    throw new BrowserEffectError('invalid-input');
  }
}

function publicBytes(value) {
  const bytes = bytesCopy(value, 65);
  if (bytes.length !== 65 || bytes[0] !== 4) throw new BrowserEffectError('invalid-input');
  return bytes;
}

function signedMessage(context, value) {
  if (typeof context !== 'string' || !/^[a-zA-Z0-9][a-zA-Z0-9._/-]{0,127}$/.test(context)
      || context.includes('..')) throw new BrowserEffectError('invalid-input');
  const bytes = bytesCopy(value);
  const scope = encoder.encode(context);
  const message = new Uint8Array(domain.length + 2 + scope.length + bytes.length);
  message.set(domain);
  new DataView(message.buffer).setUint16(domain.length, scope.length);
  message.set(scope, domain.length + 2);
  message.set(bytes, domain.length + 2 + scope.length);
  return message;
}

export async function digestBytes(value) {
  const bytes = bytesCopy(value);
  try {
    const digest = new Uint8Array(await subtle().digest('SHA-256', bytes));
    return `sha256:${Array.from(digest, byte => byte.toString(16).padStart(2, '0')).join('')}`;
  } catch {
    throw new BrowserEffectError('crypto-unavailable');
  }
}

export async function identityPrincipal(value) {
  const bytes = publicBytes(value);
  const provider = subtle();
  try {
    await provider.importKey('raw', bytes, algorithm, false, ['verify']);
  } catch {
    throw new BrowserEffectError('invalid-input');
  }
  return digestBytes(bytes);
}

export async function createIdentity() {
  try {
    const provider = subtle();
    const pair = await provider.generateKey(algorithm, false, ['sign', 'verify']);
    const publicKey = new Uint8Array(await provider.exportKey('raw', pair.publicKey));
    return Object.freeze({ privateKey: pair.privateKey, publicKey, principal: await digestBytes(publicKey) });
  } catch {
    throw new BrowserEffectError('crypto-unavailable');
  }
}

export async function signBytes(identity, context, value) {
  // Capture the complete message synchronously; callers retain mutable buffers.
  const message = signedMessage(context, value);
  let key;
  try {
    key = identity?.privateKey;
    if (!validSigningKey(key)) {
      throw new BrowserEffectError('identity-corrupt');
    }
  } catch {
    throw new BrowserEffectError('identity-corrupt');
  }
  let provider;
  try { provider = subtle(); }
  catch { throw new BrowserEffectError('crypto-unavailable'); }
  const expectedPrototype = await signingPrototype(provider);
  try {
    // Repeat after the only asynchronous preparation gap: callers still own the
    // key object. No custom prototype/getter may reach the provider's rereads.
    if (prototypeOf(key) !== expectedPrototype || !unchangedKeyPrototype(expectedPrototype)
        || !validSigningKey(key)) {
      throw new BrowserEffectError('identity-corrupt');
    }
  } catch { throw new BrowserEffectError('identity-corrupt'); }
  let signature;
  try {
    signature = new ByteArray(await provider.sign(signatureAlgorithm, key, message));
  } catch {
    throw new BrowserEffectError('crypto-unavailable');
  }
  // Metadata reflection alone cannot establish a provider's actual key curve.
  // P-256 P1363 signatures have exactly two 32-byte integers.
  if (signature.length !== 64) throw new BrowserEffectError('identity-corrupt');
  return signature;
}

export async function verifyBytes(publicKey, context, value, signature) {
  const message = signedMessage(context, value);
  const keyBytes = publicBytes(publicKey);
  const signatureBytes = bytesCopy(signature, 64);
  if (signatureBytes.length !== 64) throw new BrowserEffectError('invalid-input');
  const provider = subtle();
  let key;
  try {
    key = await provider.importKey('raw', keyBytes, algorithm, false, ['verify']);
  } catch {
    throw new BrowserEffectError('invalid-input');
  }
  try {
    return await provider.verify(signatureAlgorithm, key, signatureBytes, message);
  } catch {
    throw new BrowserEffectError('crypto-unavailable');
  }
}

export async function validateIdentity(identity) {
  try {
    if (!identity || Object.keys(identity).sort().join(',') !== 'principal,privateKey,publicKey') {
      throw new BrowserEffectError('identity-corrupt');
    }
    const publicKey = publicBytes(identity.publicKey);
    const principal = identity.principal;
    const privateKey = identity.privateKey;
    const captured = { publicKey, principal, privateKey };
    if (await identityPrincipal(publicKey) !== principal) throw new BrowserEffectError('identity-corrupt');
    const challenge = randomBytes(32);
    const signature = await signBytes(captured, 'key-possession/1', challenge);
    if (!await verifyBytes(publicKey, 'key-possession/1', challenge, signature)) {
      throw new BrowserEffectError('identity-corrupt');
    }
    return Object.freeze(captured);
  } catch (error) {
    let unavailable = false;
    try { unavailable = error instanceof BrowserEffectError && error.code === 'crypto-unavailable'; }
    catch { /* Thrown values and their accessors are untrusted. */ }
    throw new BrowserEffectError(unavailable ? 'crypto-unavailable' : 'identity-corrupt');
  }
}
