// Private artifact preflight, not a substitute for WebAssembly compilation or
// independently verified entry/protocol provenance. Inspect the same captured
// artifact bytes that are hashed and compiled; never inspect a caller reread.
import {bytesCopy, bytesLength} from './identity.mjs';
import {EFFECT_FRAME_MAXIMUM} from './effects-wire.mjs';

const HEADER = [0, 97, 115, 109, 1, 0, 0, 0];
const MAXIMUM_SECTIONS = 64;
// Data-count precedes code/data despite its numeric section ID.
const ORDER = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 10];
const invalid = () => new TypeError('invalid-effect-module');
export const EFFECT_ARTIFACTS_MAXIMUM = 256 * 1024 * 1024;

// Byte admission only: this neither copies nor authenticates/compiles modules.
// Call with the captured list, then synchronously copy those same byte views.
export function inspectEffectArtifactBudget(wire, guests) {
  if (arguments.length !== 2 || !Array.isArray(guests)
    || Object.getPrototypeOf(guests) !== Array.prototype || guests.length > 64) throw invalid();
  const fields = Object.getOwnPropertyDescriptors(guests);
  if (Reflect.ownKeys(fields).length !== guests.length + 1) throw invalid();
  let total;
  try {
    total = bytesLength(wire, EFFECT_FRAME_MAXIMUM);
    for (let index = 0; index < guests.length; index++) {
      const field = fields[index];
      if (!field || !('value' in field)) throw invalid();
      const length = bytesLength(field.value, EFFECT_FRAME_MAXIMUM);
      if (length > EFFECT_ARTIFACTS_MAXIMUM - total) throw invalid();
      total += length;
    }
  } catch { throw invalid(); }
  return total;
}

export function inspectEffectModule(value, memoryPages) {
  if (arguments.length !== 2 || !Number.isSafeInteger(memoryPages)
    || memoryPages < 1 || memoryPages > 65536) throw invalid();
  let bytes;
  try { bytes = bytesCopy(value, EFFECT_FRAME_MAXIMUM); } catch { throw invalid(); }
  if (bytes.length < HEADER.length || HEADER.some((byte, index) => bytes[index] !== byte)) throw invalid();
  let at = HEADER.length, previous = 0, sections = 0, memory;

  function unsigned(end) {
    let value = 0;
    for (let width = 0; width < 5; width++) {
      if (at >= end) throw invalid();
      const byte = bytes[at++];
      if (width === 4 && byte > 15) throw invalid();
      value += (byte & 127) * 2 ** (width * 7);
      if (byte < 128) {
        if (width > 0 && value < 2 ** (width * 7)) throw invalid();
        return value;
      }
    }
    throw invalid();
  }

  while (at < bytes.length) {
    if (++sections > MAXIMUM_SECTIONS) throw invalid();
    const id = bytes[at++];
    if (id >= ORDER.length || id === 8) throw invalid();
    if (id !== 0) {
      if (ORDER[id] <= previous) throw invalid();
      previous = ORDER[id];
    }
    const length = unsigned(bytes.length);
    if (length > bytes.length - at) throw invalid();
    const end = at + length;
    if (id === 2) {
      // No function, table, memory, global or tag import is executable here.
      if (unsigned(end) !== 0 || at !== end) throw invalid();
    } else if (id === 5) {
      // Exactly one non-shared Wasm32 memory, with an explicit maximum. This
      // prevents initial allocation and memory.grow from exceeding the grant,
      // rather than discovering over-allocation after guest execution.
      if (unsigned(end) !== 1 || unsigned(end) !== 1) throw invalid();
      const initialPages = unsigned(end), maximumPages = unsigned(end);
      if (initialPages > maximumPages || maximumPages > memoryPages || at !== end) throw invalid();
      memory = Object.freeze({initialPages, maximumPages});
    }
    // Compilation validates the opaque type/function/table/global/export,
    // element/code/data and custom bodies. It must follow this preflight before
    // any instantiation; no synthetic fragment establishes guest acceptance.
    at = end;
  }
  if (memory === undefined) throw invalid();
  return memory;
}
