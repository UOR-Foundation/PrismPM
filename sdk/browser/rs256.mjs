// Generic cryptographic effect only. A valid signature does not authenticate
// an issuer, token, mailbox, account, or organization.
import {BrowserEffectError, bytesCopy} from './identity.mjs';

export const MAX_MESSAGE_BYTES = 1048576;
export const MAX_PUBLIC_KEY_BYTES = 2048;
export const MAX_SIGNATURE_BYTES = 1024;
const algorithm = Object.freeze({name: 'RSASSA-PKCS1-v1_5', hash: 'SHA-256'});
const apply = Reflect.apply;

export async function verifyRs256(publicKey, message, signature) {
  // All caller-owned buffers are captured before the first asynchronous gap.
  const keyBytes = bytesCopy(publicKey, MAX_PUBLIC_KEY_BYTES);
  const messageBytes = bytesCopy(message, MAX_MESSAGE_BYTES);
  const signatureBytes = bytesCopy(signature, MAX_SIGNATURE_BYTES);
  if (keyBytes.length === 0 || signatureBytes.length < 256) {
    throw new BrowserEffectError('invalid-input');
  }
  let provider, importKey, verify;
  try {
    provider = globalThis.crypto.subtle;
    importKey = provider.importKey;
    verify = provider.verify;
    if (typeof importKey !== 'function' || typeof verify !== 'function') throw null;
  } catch {
    throw new BrowserEffectError('crypto-unavailable');
  }
  let key;
  try {
    key = await apply(importKey, provider, ['spki', keyBytes, algorithm, false, ['verify']]);
  } catch {
    throw new BrowserEffectError('invalid-input');
  }
  // This key is freshly returned by the trusted provider, never caller-owned.
  // The host's finite 2048..8192-bit range does not replace issuer key policy.
  try {
    const metadata = key.algorithm;
    if (key.type !== 'public' || key.extractable !== false
        || metadata.name !== algorithm.name || metadata.hash.name !== algorithm.hash
        || !Number.isSafeInteger(metadata.modulusLength)
        || metadata.modulusLength < 2048 || metadata.modulusLength > 8192
        || key.usages.length !== 1 || key.usages[0] !== 'verify'
        || signatureBytes.length !== Math.ceil(metadata.modulusLength / 8)) throw null;
  } catch {
    throw new BrowserEffectError('invalid-input');
  }
  try {
    const result = await apply(verify, provider, [algorithm, key, signatureBytes, messageBytes]);
    if (typeof result !== 'boolean') throw null;
    return result;
  } catch {
    throw new BrowserEffectError('crypto-unavailable');
  }
}
