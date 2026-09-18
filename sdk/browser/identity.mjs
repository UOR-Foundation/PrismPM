// Generic Web Cryptography bindings. Application authority is never inferred
// from possession of a key; the generated model must authorize every effect.
const algorithm = Object.freeze({ name: 'ECDSA', namedCurve: 'P-256' });
const signatureAlgorithm = Object.freeze({ name: 'ECDSA', hash: 'SHA-256' });
const encoder = new TextEncoder();
const domain = encoder.encode('prismpm/browser-signature/1\0');
export const MAX_SIGNED_BYTES = 1048576;

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

export function bytesCopy(value, maximum = MAX_SIGNED_BYTES) {
  try {
    if (!(value instanceof Uint8Array)
        || (typeof SharedArrayBuffer !== 'undefined' && value.buffer instanceof SharedArrayBuffer)
        || value.byteLength > maximum) throw new BrowserEffectError('invalid-input');
    return new Uint8Array(value);
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
    if (typeof CryptoKey === 'undefined' || !(key instanceof CryptoKey)
        || key.type !== 'private' || key.extractable !== false
        || key.algorithm?.name !== 'ECDSA' || key.algorithm.namedCurve !== 'P-256'
        || !Array.isArray(key.usages) || key.usages.length !== 1 || key.usages[0] !== 'sign') {
      throw new BrowserEffectError('identity-corrupt');
    }
  } catch {
    throw new BrowserEffectError('identity-corrupt');
  }
  try {
    return new Uint8Array(await subtle().sign(signatureAlgorithm, key, message));
  } catch {
    throw new BrowserEffectError('crypto-unavailable');
  }
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
    let challenge;
    try { challenge = crypto.getRandomValues(new Uint8Array(32)); }
    catch { throw new BrowserEffectError('crypto-unavailable'); }
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
