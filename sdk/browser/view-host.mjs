// Trusted SDK bootstrap only; generated models own application decisions.
import {bytesCopy, validateIdentity} from './identity.mjs';
import {openCommands} from './commands.mjs';
import {openQueries} from './queries.mjs';
import {captureLabels, makeRenderer, presentation} from './view-dom.mjs';
import {ViewHostError, fail} from './view-error.mjs';
export {ViewHostError};
const empty = () => new Uint8Array();
const same = (a, b) => a.length === b.length && a.every((byte, i) => byte === b[i]);
const cat = (...parts) => { const bytes = new Uint8Array(parts.reduce((size, part) => size + part.length, 0)); let at = 0; for (const part of parts) { bytes.set(part, at); at += part.length; } return bytes; };
const u16 = value => Uint8Array.of(value >>> 8, value & 255);
const u24 = value => Uint8Array.of(value >>> 16, value >>> 8 & 255, value & 255);
const read16 = (bytes, at) => bytes[at] * 256 + bytes[at + 1];
const read24 = (bytes, at) => bytes[at] * 65536 + bytes[at + 1] * 256 + bytes[at + 2];
const request = (operation, state, suffix = empty()) => cat([operation], u24(state.length), state, suffix);
function interpreter(module) {
  if (!(module instanceof WebAssembly.Module) || WebAssembly.Module.imports(module).length !== 0) throw fail('invalid-generated-module');
  return input => {
    if (input.length > 133728) throw fail('request-limit');
    try {
      const instance = new WebAssembly.Instance(module, {}), {memory, holo_alloc: allocate, holo_run: run} = instance.exports;
      if (!(memory instanceof WebAssembly.Memory) || typeof allocate !== 'function' || typeof run !== 'function') throw fail('invalid-generated-module');
      const pointer = allocate(input.length) >>> 0;
      if (pointer + input.length > memory.buffer.byteLength) throw fail('invalid-generated-output');
      new Uint8Array(memory.buffer, pointer, input.length).set(input);
      const packed = BigInt.asUintN(64, run(pointer, input.length)), at = Number(packed >> 32n), length = Number(packed & 0xffffffffn);
      if (length > 71055 || at + length > memory.buffer.byteLength || memory.buffer.byteLength > 128 * 65536) throw fail('invalid-generated-output');
      const output = new Uint8Array(memory.buffer, at, length).slice();
      if (output.length === 1 && output[0] >= 1 && output[0] <= 14) throw new ViewHostError('model-rejected', output[0]);
      return output;
    } catch (error) { if (error instanceof ViewHostError) throw error; throw fail('generated-execution-failed'); }
  };
}
function plan(bytes) {
  if (bytes.length < 6 || bytes[0] !== 0) throw fail('invalid-generated-output');
  const length = read24(bytes, 1), at = 4 + length;
  if (length < 79 || length > 66882 || at + 2 > bytes.length) throw fail('invalid-generated-output');
  const effectLength = read16(bytes, at);
  if (effectLength > 4167 || at + 2 + effectLength !== bytes.length) throw fail('invalid-generated-output');
  return {state: bytes.slice(4, at), effect: bytes.slice(at + 2)};
}
function effect(bytes, state, session) {
  if (bytes.length < 70 || bytes.length !== 70 + read16(bytes, 68) || bytes[34] > 2 || bytes[67] > 1
    || !same(bytes.subarray(0, 32), session) || !same(bytes.subarray(0, 34), cat(state.subarray(4, 36), state.subarray(68, 70)))
    || !same(bytes.subarray(35, 67), state.subarray(36, 68)) || bytes[67] !== state[71] || bytes[34] !== state[72]
    || !same(bytes.subarray(70), state.subarray(75, 75 + read16(state, 73)))) throw fail('invalid-generated-output');
  return {binding: bytes.slice(0, 35), kind: bytes[34], workspace: bytes.slice(35, 67), table: bytes[67], input: bytes.slice(70)};
}
function admittedPage(page) {
  // Exact fixed Query ABI framing only; the generated View validates all rows.
  try {
    const workspace = bytesCopy(page.workspace, 32), head = bytesCopy(page.headId, 32), cursor = bytesCopy(page.cursor, 135), rows = bytesCopy(page.rows, 66592);
    if (workspace.length !== 32 || head.length !== 32 || ![0, 135].includes(cursor.length)
      || ![0, 1].includes(page.table) || !Number.isInteger(page.count) || page.count < 0 || page.count > 16
      || !Number.isInteger(page.total) || page.total < 0 || page.total > 256
      || !Number.isInteger(page.offset) || page.offset < 0 || page.offset > 256) throw 0;
    return cat([0, page.table], workspace, head, u16(page.total), u16(page.offset), [page.count], u16(cursor.length), cursor, u24(rows.length), rows);
  } catch { throw fail('invalid-effect-result'); }
}
function failure(error, kind, commands) {
  let code, detail, barrier = true;
  try { code = error?.code; detail = error?.detail; } catch { /* No exception payload crosses the boundary. */ }
  if (kind !== 1) return code === 'query-rejected' || code === 'query-context-changed' ? 1 : 4;
  try { barrier = commands.status().requiresRefresh !== false; } catch { return 4; }
  if (code === 'storage-outcome-unknown' || code === 'commit-outcome-unknown' || code === 'storage-result-mismatch') return 3;
  if (code === 'head-conflict' || code === 'storage-rejected' && detail === 33 || code === 'command-rejected' && detail === 8) return 2;
  if (code === 'command-rejected' || code === 'model-rejected' || code === 'storage-rejected' && [34, 35].includes(detail)) return barrier ? 5 : 1;
  return 4;
}
class View {
  #call; #state; #session; #commands; #queries; #renderer; #closed = false; #active = null;
  constructor(module, session, commands, queries, root, labels) {
    this.#call = interpreter(module); this.#session = session; this.#commands = commands; this.#queries = queries;
    try {
      this.#renderer = makeRenderer(root, labels, bytes => this.dispatch(bytes));
      this.#promote(this.#call(cat([0], session)));
    } catch { throw this.#abort(); }
  }
  #promote(raw) {
    try {
    const next = plan(raw), display = presentation(this.#call(request(3, next.state)));
    const declared = next.effect.length ? effect(next.effect, next.state, this.#session) : null;
    this.#state = next.state; this.#renderer.render(display);
    if (display.phase === 3) {
      this.#closed = true; this.#active = null; this.#commands.close(); this.#queries.close(); this.#renderer.close();
    }
    return declared;
    } catch { throw this.#abort(); }
  }
  #terminate(failed) {
    this.#closed = true; this.#active = null;
    let unavailable = false;
    for (const operation of [() => this.#commands.close(), () => this.#queries.close(), () => this.#renderer.close(failed)]) {
      try { operation(); } catch { unavailable = true; }
    }
    if (unavailable) throw fail('host-unavailable');
  }
  #abort() {
    try { this.#terminate(true); } catch { /* The private terminal flag precedes all teardown effects. */ }
    return fail('host-unavailable');
  }
  #close() {
    let failed = true;
    try { this.#promote(this.#call(request(1, this.#state, Uint8Array.of(6)))); failed = false; }
    catch { throw fail('host-unavailable'); }
    finally { this.#terminate(failed); }
  }
  async #execute(declared, token) {
    if (token.started) return;
    token.started = true;
    let outcome = 0, payload = empty();
    try {
      if (declared.kind === 0) payload = admittedPage(await this.#queries.query({table: declared.table, workspace: declared.workspace, cursor: declared.input}));
      else if (declared.kind === 1) await this.#commands.submit({action: declared.input[0], workspace: declared.workspace, body: declared.input.slice(1)});
      else await this.#commands.refresh();
    } catch (error) { outcome = failure(error, declared.kind, this.#commands); payload = empty(); }
    // Only this privately owned completion may settle its still-live operation.
    if (this.#closed || this.#active !== token) return;
    try {
      this.#active = null;
      const completion = cat(declared.binding, [outcome], u24(payload.length), payload);
      const next = this.#promote(this.#call(request(2, this.#state, completion)));
      if (next !== null) throw fail('invalid-generated-output');
    } catch { throw this.#abort(); }
  }
  dispatch(value) {
    let captured;
    try {
      if (arguments.length !== 1) throw fail('invalid-input');
      if (this.#closed) throw fail('view-closed');
      try { captured = bytesCopy(value, 4098); } catch { throw fail('invalid-input'); }
      if (this.#closed) throw fail('view-closed');
      if (captured.length === 1 && captured[0] === 6) { this.#close(); return Promise.resolve(); }
      const declared = this.#promote(this.#call(request(1, this.#state, captured)));
      if (!declared) return Promise.resolve();
      const token = {started: false}; this.#active = token;
      return this.#execute(declared, token);
    } catch (error) {
      try {
        if (error instanceof ViewHostError && ['invalid-input','view-closed'].includes(error.code) && error.detail === null)
          return Promise.reject(fail(error.code));
        if (error instanceof ViewHostError && error.code === 'model-rejected' && Number.isInteger(error.detail) && error.detail >= 1 && error.detail <= 14)
          return Promise.reject(new ViewHostError('model-rejected',error.detail));
      } catch { /* Hostile exception objects cannot control this boundary. */ }
      return Promise.reject(this.#abort());
    }
  }
  close() {
    if (arguments.length !== 0) throw fail('invalid-input');
    if (!this.#closed) this.#close();
  }
}
export async function openWorkspaceView({viewModule, commandModule, queryModule, journalModule, store, headName, root, labels}) {
  const capturedLabels = captureLabels(labels); interpreter(viewModule);
  let commands, queries;
  try {
    // One possessed identity is captured before either private adapter opens.
    const identity = await validateIdentity(await store.loadIdentity());
    const selected = {loadIdentity: async () => identity, readHead: (...args) => store.readHead(...args),
      readObject: (...args) => store.readObject(...args), commit: (...args) => store.commit(...args)};
    commands = await openCommands({commandModule, journalModule, store: selected, headName});
    queries = await openQueries({queryModule, journalModule, store: selected, headName});
    const session = crypto.getRandomValues(new Uint8Array(32));
    const view = new View(viewModule, session, commands, queries, root, capturedLabels);
    return Object.freeze({dispatch: function (...args) { return view.dispatch(...args); }, close: function (...args) { return view.close(...args); }});
  } catch { commands?.close(); queries?.close(); throw fail('host-unavailable'); }
}
