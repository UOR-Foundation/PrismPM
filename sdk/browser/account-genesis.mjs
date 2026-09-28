// Private account declarations. Artifact provenance belongs to the verified
// bootstrap; these facts do not prove key possession or current account authority.
import {bytesCopy, identityPrincipal, digestBytes} from './identity.mjs';
import {inspectEffectModule} from './effects-module.mjs';
import {encodeEffectWire as encode, decodeEffectWire as decode} from './effects-wire.mjs';

const FRAME = 512, ARTIFACT = 67108864, MEMORY = 1073741824;
const apply = Reflect.apply, own = Object.getOwnPropertyDescriptors;
const prototype = Object.getPrototypeOf, keys = Reflect.ownKeys;
const Instance = WebAssembly.Instance, compile = WebAssembly.compile;
const same = (a, b) => a.length === b.length && a.every((byte, index) => byte === b[index]);
const fail = code => new AccountGenesisError(code);
export class AccountGenesisError extends Error {
  constructor(code) {super(code); this.name = 'AccountGenesisError'; this.code = code;}
}
function exact(value, names) {
  if (!value || prototype(value) !== Object.prototype) throw fail('invalid-input');
  const fields = own(value), actual = keys(fields);
  if (actual.length !== names.length || actual.some(key => !names.includes(key)
    || !Object.hasOwn(fields[key], 'value'))) throw fail('invalid-input');
  return Object.fromEntries(names.map(name => [name, fields[name].value]));
}
function copy(value, maximum) {
  try {return bytesCopy(value, maximum);} catch {throw fail('invalid-input');}
}
function canonical(bytes) {
  try {
    const value = decode(bytes);
    if (!same(encode(value), bytes)) throw fail('invalid-input');
    return value;
  } catch {throw fail('invalid-input');}
}
function request(value) {
  let bytes;
  try {bytes = encode(value);} catch {throw fail('invalid-input');}
  if (bytes.length > FRAME) throw fail('invalid-input');
  return bytes;
}
function run(module, input) {
  try {
    const instance = new Instance(module, {});
    const {memory, holo_alloc: allocate, holo_run: execute} = instance.exports;
    if (!(memory instanceof WebAssembly.Memory) || typeof allocate !== 'function'
      || allocate.length !== 1 || typeof execute !== 'function' || execute.length !== 2)
      throw fail('invalid-generated-module');
    const pointer = allocate(input.length) >>> 0;
    if (pointer + input.length > memory.buffer.byteLength) throw fail('invalid-generated-output');
    new Uint8Array(memory.buffer, pointer, input.length).set(input);
    const result = execute(pointer, input.length);
    if (typeof result !== 'bigint') throw fail('invalid-generated-output');
    const packed = BigInt.asUintN(64, result), start = Number(packed >> 32n), length = Number(packed & 0xffffffffn);
    if (length > FRAME || start + length > memory.buffer.byteLength || memory.buffer.byteLength > MEMORY)
      throw fail('invalid-generated-output');
    const value = canonical(new Uint8Array(memory.buffer, start, length).slice());
    if (!Array.isArray(value) || value.length !== 3 || value[0] !== 1)
      throw fail('invalid-generated-output');
    if (value[1] === 1 || value[1] === 2) throw fail('model-rejected');
    if (value[1] !== 0) throw fail('invalid-generated-output');
    return value[2];
  } catch (error) {
    if (error instanceof AccountGenesisError) throw error;
    throw fail('generated-execution-failed');
  }
}

export async function openAccountGenesis(options) {
  if (arguments.length !== 1) throw fail('invalid-input');
  const fields = exact(options, ['wire', 'wireDigest']);
  const wire = copy(fields.wire, ARTIFACT), wireDigest = copy(fields.wireDigest, 32);
  if (wireDigest.length !== 32) throw fail('invalid-input');
  let actual;
  try {actual = new Uint8Array(await crypto.subtle.digest('SHA-256', wire));}
  catch {throw fail('crypto-unavailable');}
  if (!same(actual, wireDigest)) throw fail('artifact-mismatch');
  let module;
  try {
    inspectEffectModule(wire, 16384);
    module = await apply(compile, WebAssembly, [wire]);
  } catch {throw fail('invalid-generated-module');}
  const facts = new WeakMap();
  const declare = async function (options) {
    if (arguments.length !== 1) throw fail('invalid-input');
    const fields = exact(options, ['genesis', 'expectedNamespace']);
    const genesis = copy(fields.genesis, FRAME), expected = copy(fields.expectedNamespace, 32);
    if (expected.length !== 32) throw fail('invalid-input');
    const value = canonical(genesis);
    if (run(module, request([1, 2, expected, value])) !== true) throw fail('invalid-generated-output');
    const projected = run(module, request([1, 1, value]));
    if (!Array.isArray(projected) || projected.length !== 4) throw fail('invalid-generated-output');
    const namespace = copy(projected[0], 32), publicKey = copy(projected[1], 65);
    const canonicalGenesis = copy(projected[2], FRAME), material = copy(projected[3], FRAME);
    if (namespace.length !== 32 || publicKey.length !== 65 || !same(namespace, expected)
      || !same(canonicalGenesis, genesis)) throw fail('invalid-generated-output');
    let keyIdentity, identity;
    try {keyIdentity = await identityPrincipal(publicKey);}
    catch {throw fail('invalid-key');}
    try {
      const digest = await digestBytes(material);
      identity = Uint8Array.from(digest.slice(7).match(/../g), pair => Number.parseInt(pair, 16));
    } catch {throw fail('crypto-unavailable');}
    const handle = Object.freeze(Object.create(null));
    facts.set(handle, {genesis, namespace, publicKey, material, keyIdentity, identity});
    return handle;
  };
  const readFacts = function (handle) {
    if (arguments.length !== 1 || !facts.has(handle)) throw fail('invalid-facts');
    const value = facts.get(handle);
    return Object.freeze({genesis: copy(value.genesis, FRAME), namespace: copy(value.namespace, 32),
      publicKey: copy(value.publicKey, 65), material: copy(value.material, FRAME),
      keyIdentity: value.keyIdentity, identity: copy(value.identity, 32), wireDigest: copy(wireDigest, 32)});
  };
  return Object.freeze(Object.assign(Object.create(null), {declare, readFacts}));
}
