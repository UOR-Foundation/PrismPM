// Acceptance-only browser journeys. Never shipped as a presentation service.
import {openPresentation} from './presentation-dom.mjs';
import {encodeWire, decodePresentation, decodeIntent, PresentationError, PRESENTATION_MAXIMUM} from './presentation-wire.mjs';
import {maximumShape} from './maximum-fixtures.mjs';

const check = (condition, message) => { if (!condition) throw Error(message); };
const hex = bytes => Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
const equal = (a, b) => a.length === b.length && a.every((value, index) => value === b[index]);
const tick = () => new Promise(resolve => setTimeout(resolve, 0));
const deferred = () => { let resolve; const promise = new Promise(done => { resolve = done; }); return {promise, resolve}; };
const clone = value => structuredClone(value);
const labels = [
  '\uFEFFBrand <img src=x onerror="globalThis.presentationInjected=1">', 'Caption', 'Choice',
  'First', 'Form', 'Heading', 'Input', 'Multiline', 'Navigation', 'Option A', 'Option B',
  'Second', 'Status', 'Column',
].map((text, index) => ({id: 'label' + String(index).padStart(2, '0'), text}));

function expectedFrame(revision = 1) {
  return [1, revision, 0, 13, 1, 6, [
    [0, [0, 0]], [1, [1, 8]], [1, [3, 2, 5]],
    [1, [4, '\uFEFFdynamic <script>globalThis.presentationInjected=2</script> 😀']],
    [1, [2, 4]], [5, [5, 6, true, true, 64, 'initial', 0]],
    [5, [6, 7, true, false, 128, '\uFEFFfirst\r\nsecond', 0]],
    [5, [7, 2, true, true, 10, [[10, 9], [20, 10]], 0]],
    [5, [8, 3, 101, true, false, [6]]],
    [5, [8, 11, 202, true, true, [6, 7, 8]]],
    [1, [9, 1, [13], [['<svg onload="globalThis.presentationInjected=3">'], ['\uFEFFtable 😀']]]],
  ]];
}

export async function runFixture(input) {
  check(input && ((Array.isArray(input.wire) && Array.isArray(input.fixture) && Array.isArray(input.labels))
    || input.adapterOnly === true), 'actual source fixture/catalogue and codec or explicit component-only mode');
  const cases = [], calls = [], views = [], roots = [], intents = [];
  let module, fixtureModule, labelsModule, sourceLabels = labels, maxMemory = 0;
  if (input.wire) {
    module = await WebAssembly.compile(new Uint8Array(input.wire));
    check(WebAssembly.Module.imports(module).length === 0, 'actual codec is import-free');
    fixtureModule = await WebAssembly.compile(new Uint8Array(input.fixture));
    check(WebAssembly.Module.imports(fixtureModule).length === 0, 'actual source fixture is import-free');
    labelsModule = await WebAssembly.compile(new Uint8Array(input.labels));
    check(WebAssembly.Module.imports(labelsModule).length === 0, 'actual source catalogue is import-free');
  }
  function execute(module, bytes, role) {
    const {exports: {memory, holo_alloc, holo_run}} = new WebAssembly.Instance(module, {});
    const pointer = holo_alloc(bytes.length) >>> 0;
    new Uint8Array(memory.buffer, pointer, bytes.length).set(bytes);
    const result = BigInt.asUintN(64, holo_run(pointer, bytes.length));
    const at = Number(result >> 32n), length = Number(result & 0xffffffffn);
    check(length <= PRESENTATION_MAXIMUM && at + length <= memory.buffer.byteLength, 'bounded actual codec output');
    const output = new Uint8Array(memory.buffer, at, length).slice();
    calls.push({role, request: hex(bytes), response: hex(output)});
    maxMemory = Math.max(maxMemory, memory.buffer.byteLength);
    return output;
  }
  if (labelsModule) {
    const actual = execute(labelsModule, new Uint8Array(), 'labels');
    sourceLabels = JSON.parse(new TextDecoder('utf-8', {fatal: true, ignoreBOM: true}).decode(actual));
    check(JSON.stringify(sourceLabels) === JSON.stringify(labels), 'complete catalogue equals genuine source-produced labels');
  }
  function frame(revision = 1) {
    const expected = expectedFrame(revision);
    if (!fixtureModule) return expected;
    const actual = execute(fixtureModule, encodeWire([1, revision, 0, 6]), 'fixture');
    check(equal(actual, encodeWire(expected)), 'complete baseline presentation equals actual modeled fixture output');
    return clone(decodePresentation(actual));
  }
  function checked(bytes, accepted = true) {
    if (!module) return bytes;
    const output = execute(module, bytes, 'wire');
    check(equal(bytes, output) === accepted, 'actual generated frame/intent acceptance');
    return bytes;
  }
  const bytes = value => checked(encodeWire(value));
  const paint = (view, value) => view.render(bytes(value));
  function failure(operation, code) {
    let error;
    try { operation(); } catch (value) { error = value; }
    check(error instanceof PresentationError && (code === undefined || error.code === code),
      'expected presentation refusal ' + code + ', observed ' + error?.code);
    check(Object.keys(error).sort().join(',') === 'code,name' && error.cause === undefined, 'closed diagnostic has no raw error payload');
  }
  function open(dispatch = raw => {
    const intent = decodeIntent(checked(raw)); intents.push(intent);
    const next = frame(intent[1] + 1); next[5] = 0; return bytes(next);
  }, options = {}) {
    const root = document.createElement('main'); document.body.append(root); roots.push(root);
    const listeners = new Map(), add = root.addEventListener.bind(root), remove = root.removeEventListener.bind(root);
    root.addEventListener = (type, listener, ...rest) => { listeners.set(type, listener); add(type, listener, ...rest); };
    root.removeEventListener = (type, listener, ...rest) => { if (listeners.get(type) === listener) listeners.delete(type); remove(type, listener, ...rest); };
    const view = openPresentation({root, labels: sourceLabels, dispatch, maximum: PRESENTATION_MAXIMUM,
      requestMaximum: PRESENTATION_MAXIMUM, ...options}); views.push(view);
    return {root, view, listeners, input: () => root.querySelector('input'), form: () => root.querySelector('form'),
      textarea: () => root.querySelector('textarea'), select: () => root.querySelector('select'),
      buttons: () => [...root.querySelectorAll('button')], alert: () => root.querySelector('[role=alert]')?.textContent};
  }
  const edit = (control, value) => { control.value = value; control.dispatchEvent(new Event('input', {bubbles: true})); };
  const phase = (value, state) => {
    value[2] = state; value[5] = 0;
    for (const [, content] of value[6]) {
      if ([5, 6, 7].includes(content[0])) content[2] = false;
      if (content[0] === 8) content[3] = false;
    }
    return value;
  };
  try {
    const first = open(); paint(first.view, frame());
    check(Object.isFrozen(first.view) && Object.keys(first.view).sort().join(',') === 'close,render', 'closed presentation API');
    check(first.root.querySelector('section').getAttribute('aria-label') === labels[0].text, 'catalogue preserves BOM and hostile text');
    check(first.root.querySelector('h2').textContent === 'Heading', 'generated semantic heading');
    check(first.root.querySelector('nav').getAttribute('aria-label') === 'Navigation', 'generated semantic navigation');
    check(first.root.querySelector('caption').textContent === 'Caption'
      && first.root.querySelector('th').scope === 'col', 'semantic table caption and column');
    check(first.root.textContent.includes('\uFEFFdynamic <script>') && first.root.textContent.includes('\uFEFFtable 😀'), 'dynamic Unicode is exact text');
    check(!first.root.querySelector('img,svg,script,a,[style],[href],[src],[id]')
      && globalThis.presentationInjected === undefined, 'text-only closed DOM has no executable sink');
    check(first.input().labels[0].textContent === 'Input' && first.select().labels[0].textContent.includes('Choice'), 'native associated labels');
    check(document.activeElement === first.input() && first.root.querySelector('[role=status]').getAttribute('aria-live') === 'polite', 'generated focus/live');
    cases.push('semantic-safe-text-catalogue-focus-live');

    const before = first.root.innerHTML, originalControl = first.input();
    for (const mutate of [
      value => { value[6][0][1][1] = 255; },
      value => { value[3] = 256; },
      value => { value[6][7][1][5][0][1] = 255; },
      value => { value[6][10][1][2][0] = 255; },
    ]) {
      const invalid = frame(2); mutate(invalid); failure(() => first.view.render(bytes(invalid)), 'labels');
      check(first.root.innerHTML === before && first.input() === originalControl, 'catalogue failure precedes all DOM mutation');
    }
    for (const mutate of [
      value => { value[6][5][1][0] = 10; },
      value => { value[6][0][0] = 1; },
      value => { value[6][9][1][5] = [7, 6]; },
      value => { value[2] = 1; },
    ]) {
      const invalid = frame(2); mutate(invalid);
      failure(() => first.view.render(checked(encodeWire(invalid), false)));
      check(first.root.innerHTML === before && first.input() === originalControl, 'shape/lifecycle rejection precedes mutation');
    }
    const altered = frame(); altered[6][3][1][1] = 'changed equal revision';
    failure(() => first.view.render(bytes(altered)), 'stale');
    failure(() => first.view.render(bytes(frame(0))), 'stale');
    first.view.render(bytes(frame()));
    check(first.input() === originalControl && first.root.innerHTML === before, 'byte-identical repeat is idempotent');
    cases.push('full-validation-stale-and-secret-rejection-before-mutation');

    edit(first.input(), 'retained 😀'); first.input().focus(); first.input().setSelectionRange(1, 4);
    const next = frame(2); next[5] = 0; paint(first.view, next);
    check(first.input() === originalControl && first.input().value === 'retained 😀', 'keyed edits survive unchanged generated defaults');
    check(document.activeElement === originalControl && originalControl.selectionStart === 1 && originalControl.selectionEnd === 4, 'keyed focus and selection survive');
    next[1] = 3; next[6][5][1][5] = 'source reset'; paint(first.view, next);
    check(first.input().value === 'source reset', 'changed generated default resets edits');
    check(document.activeElement === originalControl && originalControl.selectionStart === 'source reset'.length
      && originalControl.selectionEnd === 'source reset'.length, 'same-kind reset retains focus without restoring stale draft selection');
    const pending = phase(frame(4), 1); pending[4] = 2; paint(first.view, pending);
    check(first.root.getAttribute('aria-busy') === 'true'
      && [...first.root.querySelectorAll('input,textarea,select,button')].every(node => node.disabled), 'pending controls are natively disabled');
    check(first.root.querySelector('[role=status]').getAttribute('aria-live') === 'assertive', 'assertive live mode');
    const pendingSubmits = intents.length;
    first.form().dispatchEvent(new SubmitEvent('submit', {bubbles: true, cancelable: true,
      submitter: first.buttons()[1]})); await tick();
    check(intents.length === pendingSubmits && first.alert() === 'Unable to submit this request.',
      'programmatic submit cannot bypass modeled pending phase');
    paint(first.view, phase(frame(5), 2));
    check(first.root.getAttribute('aria-busy') === 'false', 'replay-required is not fabricated pending');
    paint(first.view, phase(frame(6), 3)); failure(() => first.view.render(bytes(frame(7))), 'closed');
    const kinds = open(); paint(kinds.view, frame());
    const newKind = frame(2); newKind[5] = 0; newKind[6][5][1][0] = 6; paint(kinds.view, newKind);
    const changedControl = kinds.root.querySelector('[data-presentation-node="6"] textarea');
    check(document.activeElement !== changedControl, 'changed control kind cannot inherit implicit focus');
    newKind[1] = 3; newKind[5] = 6; paint(kinds.view, newKind);
    check(document.activeElement === changedControl, 'explicit modeled focus can select a changed control kind');
    cases.push('keyed-edit-focus-reset-and-modeled-lifecycle');

    const emptyDraft = (revision, epoch) => {
      const value = frame(revision); value[5] = 0;
      for (const [, content] of value[6]) {
        if ([5, 6, 7].includes(content[0])) content[6] = epoch;
        if ([5, 6].includes(content[0])) content[5] = '';
      }
      return value;
    };
    const sent = [], reset = open(raw => { sent.push(decodeIntent(checked(raw))); return bytes(emptyDraft(2, 1)); });
    paint(reset.view, emptyDraft(1, 0)); edit(reset.input(), 'sent message'); edit(reset.textarea(), 'sent body');
    reset.buttons()[1].click(); await tick();
    check(sent.length === 1 && reset.input().value === '' && reset.textarea().value === '', 'modeled draft epoch clears sent fields to the identical empty default');
    edit(reset.input(), 'organization A draft'); edit(reset.textarea(), 'A notes'); edit(reset.select(), '20');
    paint(reset.view, emptyDraft(3, 1));
    check(reset.input().value === 'organization A draft' && reset.textarea().value === 'A notes'
      && reset.select().value === '20', 'ordinary revisions retain the same draft epoch');
    paint(reset.view, emptyDraft(4, 2));
    check(reset.input().value === '' && reset.textarea().value === '' && reset.select().value === '10', 'new organization epoch resets unchanged defaults and selection');
    const oldReply = deferred(), context = open(raw => { checked(raw); return oldReply.promise; });
    paint(context.view, emptyDraft(1, 0)); edit(context.input(), 'old organization'); context.buttons()[0].click();
    paint(context.view, emptyDraft(2, 1)); edit(context.input(), 'new organization draft');
    oldReply.resolve(bytes(emptyDraft(3, 0))); await tick();
    check(context.input().value === 'new organization draft', 'higher-revision result from stale draft context cannot overwrite current draft');
    cases.push('source-owned-draft-reset-context-and-stale-response');

    const exactDefaults = [], defaults = open(raw => {
      exactDefaults.push(decodeIntent(checked(raw))); return bytes(frame(exactDefaults.length + 1));
    });
    const original = frame(); original[6][5][1][5] = '\uFEFFone\r\ntwo\nthree'; paint(defaults.view, original);
    defaults.buttons()[1].click(); await tick();
    check(exactDefaults[0][3][0][1] === '\uFEFFone\r\ntwo\nthree'
      && exactDefaults[0][3][1][1] === '\uFEFFfirst\r\nsecond', 'untouched defaults retain exact LF/CRLF/BOM bytes');
    edit(defaults.input(), 'edited\nline'); edit(defaults.textarea(), '\uFEFFedited\r\nline');
    const actualInput = defaults.input().value, actualMultiline = defaults.textarea().value;
    defaults.buttons()[1].click(); await tick();
    check(exactDefaults[1][3][0][1] === actualInput && exactDefaults[1][3][1][1] === actualMultiline, 'edited intent uses actual browser text without invented normalization');
    cases.push('exact-default-bytes-and-native-edited-values');

    let submits = 0;
    const bounded = open(raw => { submits++; checked(raw); return bytes(frame(2)); }); paint(bounded.view, frame());
    for (const value of ['', 'é'.repeat(33), '\ud800']) {
      edit(bounded.input(), value); bounded.buttons()[0].click(); await tick();
      check(submits === 0 && bounded.alert() === 'Unable to submit this request.', 'required/UTF8 byte/surrogate rejection prevents dispatch');
    }
    const small = open(() => { throw Error('must not dispatch'); }, {requestMaximum: 1}); paint(small.view, frame());
    small.buttons()[0].click(); await tick(); check(small.alert() === 'Unable to submit this request.', 'exact primary request maximum enforced');
    const selected = [], choices = open(raw => { const intent = decodeIntent(checked(raw)); selected.push(intent); return bytes(frame(intent[1] + 1)); });
    paint(choices.view, frame()); edit(choices.select(), '20'); const preserved = frame(2); preserved[5] = 0; paint(choices.view, preserved);
    check(choices.select().value === '20', 'keyed selection survives same generated default');
    choices.buttons()[1].click(); await tick();
    check(JSON.stringify(selected[0][3].map(field => field[0])) === '[6,7,8]' && selected[0][3][2][1] === 20, 'intent captures exact modeled field order and chosen identifier');
    const changed = frame(4); changed[6][7][1][4] = 20; paint(choices.view, changed);
    check(choices.select().value === '20', 'generated selection is applied');
    edit(choices.select(), ''); choices.buttons()[1].click(); await tick(); check(selected.length === 1, 'required no-selection rejects');
    cases.push('required-utf8-request-bounds-and-choice-bindings');

    const waiting = deferred(); let asynchronous = 0, asyncView;
    asyncView = open(raw => { checked(raw); asynchronous++; asyncView.buttons()[0].click(); return waiting.promise; });
    paint(asyncView.view, frame()); asyncView.buttons()[0].click(); asyncView.buttons()[0].click();
    check(asynchronous === 1, 'synchronous reentrant and pending duplicate submissions are serialized');
    paint(asyncView.view, frame(3)); const afterNew = asyncView.root.innerHTML;
    waiting.resolve(bytes(frame(2))); await tick();
    check(asyncView.root.querySelector('section') && asyncView.input().value === 'initial'
      && asyncView.root.innerHTML.replace('Unable to submit this request.', '') === afterNew, 'late older result cannot replace newer manually rendered frame');
    check(asyncView.alert() === 'Unable to submit this request.', 'stale async result uses fixed safe alert');
    const throws = open(() => { throw Error('private secret that must not appear'); }); paint(throws.view, frame());
    throws.buttons()[0].click(); await tick(); check(throws.alert() === 'Unable to submit this request.'
      && !throws.root.textContent.includes('private secret'), 'dispatcher errors never echo details');
    cases.push('serialized-reentrancy-stale-results-and-private-diagnostics');

    const late = deferred(); let closedCalls = 0;
    const closing = open(raw => { checked(raw); closedCalls++; return late.promise; }); paint(closing.view, frame());
    const heldButton = closing.buttons()[0]; heldButton.click(); closing.view.close();
    check(closing.listeners.size === 0, 'terminal close removes each actual native listener');
    const replacement = openPresentation({root: closing.root, labels, dispatch: () => { throw Error('not used'); },
      maximum: PRESENTATION_MAXIMUM, requestMaximum: PRESENTATION_MAXIMUM}); views.push(replacement);
    paint(replacement, frame(9)); const reopened = closing.root.innerHTML;
    closing.view.close(); late.resolve(bytes(frame(2))); heldButton.click(); await tick();
    check(closing.root.innerHTML === reopened && closedCalls === 1, 'closed view and late result cannot disturb a new owner of the root');
    failure(() => closing.view.render(bytes(frame(10))), 'closed');
    cases.push('terminal-close-listeners-late-result-and-root-ownership');

    const brands = open(); const protectedBytes = bytes(frame());
    for (const name of ['buffer', 'byteLength', 'byteOffset', 'length']) Object.defineProperty(protectedBytes, name, {get() { throw Error('shadow getter evaluated'); }});
    brands.view.render(protectedBytes);
    const detached = bytes(frame(2)); structuredClone(detached.buffer, {transfer: [detached.buffer]});
    const brandsBefore = brands.root.innerHTML; failure(() => brands.view.render(detached), 'bytes');
    if (typeof SharedArrayBuffer === 'function') {
      const shared = new Uint8Array(new SharedArrayBuffer(8)); Object.defineProperty(shared, 'buffer', {value: new ArrayBuffer(8)});
      failure(() => brands.view.render(shared), 'bytes');
    } else throw Error('isolated real browser SharedArrayBuffer is required');
    failure(() => brands.view.render({}), 'bytes'); failure(() => brands.view.render(bytes(frame(2)), true), 'options');
    check(brands.root.innerHTML === brandsBefore, 'native byte-brand failures do not mutate DOM');
    cases.push('native-byte-brands-detached-shared-and-closed-arity');

    let invoked = 0; const hostile = Object.defineProperty({}, 'labels', {get() { invoked++; return labels; }});
    failure(() => openPresentation(hostile)); check(invoked === 0, 'option accessors never execute');
    for (const bad of [[...labels].reverse(), [{id: 'same', text: 'a'}, {id: 'same', text: 'b'}],
      [{id: 'valid', text: '\ud800'}], [{id: 'valid', text: 'é'.repeat(2049)}], [{id: 'valid', text: 'line\nbreak'}]]) {
      failure(() => openPresentation({root: brands.root, labels: bad, dispatch() {}, maximum: 1000, requestMaximum: 1000}));
      check(brands.root.innerHTML === brandsBefore, 'bad catalogue rejected without DOM mutation');
    }
    const accessorLabel = Object.defineProperty({id: 'valid'}, 'text', {get() { invoked++; return 'x'; }});
    failure(() => openPresentation({root: brands.root, labels: [accessorLabel], dispatch() {}, maximum: 1000, requestMaximum: 1000}));
    check(invoked === 0, 'catalogue accessors never execute');
    const capturedLabels = clone(sourceLabels), catalogueView = open(undefined, {labels: capturedLabels});
    capturedLabels[0].text = 'mutated caller catalogue'; capturedLabels.reverse();
    paint(catalogueView.view, frame());
    check(catalogueView.root.querySelector('section').getAttribute('aria-label') === sourceLabels[0].text,
      'complete source catalogue is captured before caller mutation');
    cases.push('closed-options-and-immutable-catalogue-capture');

    const maximal = [1, 1, 0, 0, 0, 0, []], structure = maximal[6];
    for (let id = 1; id <= 16; id++) structure.push([id - 1, [0, 0]]);
    structure.push([0, [2, 4]]);
    for (let id = 18; id <= 33; id++) structure.push([17, [5, 6, true, false, 1, 'x', 0]]);
    structure.push([17, [7, 2, true, false, 1, Array.from({length: 256}, (_, index) => [index + 1, 9]), 0]]);
    for (let id = 1; id <= 64; id++) structure.push([17, [8, 3, id, true, id === 1, Array.from({length: 16}, (_, index) => index + 18)]]);
    structure.push([0, [9, 1, Array(16).fill(13), Array.from({length: 256}, () => Array(16).fill('x'))]]);
    while (structure.length < 256) structure.push([0, [4, 'x']]);
    const allBounds = open(); paint(allBounds.view, maximal);
    check(allBounds.root.querySelectorAll('[data-presentation-node]').length === 256
      && allBounds.buttons().length === 64 && allBounds.select().options.length === 257
      && allBounds.root.querySelectorAll('tbody td').length === 4096
      && allBounds.root.querySelectorAll('th').length === 16, 'combined node/depth/action/binding/choice/table maxima are actually rendered');
    const allBoundsBefore = allBounds.root.innerHTML;
    for (const mutate of [
      value => { value[6].push([0, [4, 'overflow']]); },
      value => { value[6][16][0] = 16; },
      value => { value[6][33][1][5].push([257, 9]); },
      value => { value[6][255] = [17, [8, 3, 65, true, false, []]]; },
    ]) {
      const invalid = clone(maximal); invalid[1] = 2; mutate(invalid);
      failure(() => allBounds.view.render(checked(encodeWire(invalid), false)));
      check(allBounds.root.innerHTML === allBoundsBefore, 'combined-bound rejection is atomic');
    }
    cases.push('combined-structural-maxima-and-overruns');

    const keyboardCalls = [], keyboard = open(raw => {
      const intent = decodeIntent(checked(raw)); keyboardCalls.push(intent);
      const next = frame(intent[1] + 1); next[5] = 0; return bytes(next);
    }); paint(keyboard.view, frame());
    keyboard.root.dataset.keyboardPresentation = '';
    globalThis.__presentationJourney = {
      count: () => keyboardCalls.length,
      finish() {
        check(keyboardCalls.length === 2 && keyboardCalls[0][2] === 202 && keyboardCalls[1][2] === 101,
          'actual keyboard uses declared default and explicitly selected button');
        cases.push('actual-native-keyboard-default-and-button-submission');
        for (const view of views) view.close(); for (const root of roots) root.remove();
        return {cases, calls, maxMemory, modelChecked: Boolean(module && fixtureModule && labelsModule), keyboard: keyboardCalls};
      },
    };
    return {cases, modelChecked: Boolean(module && fixtureModule && labelsModule)};
  } catch (error) {
    for (const view of views) view.close(); for (const root of roots) root.remove();
    throw error;
  }
}

export async function runMaximumFixture(input) {
  check(Array.isArray(input?.wire), 'maximum acceptance requires actual generated codec');
  const module = await WebAssembly.compile(new Uint8Array(input.wire));
  check(WebAssembly.Module.imports(module).length === 0, 'actual maximum codec is import-free');
  const {exports: {memory, holo_alloc, holo_run}} = new WebAssembly.Instance(module, {});
  const shape = maximumShape(input.id);
  const textLength = PRESENTATION_MAXIMUM - encodeWire(shape.frame).length - 4;
  const text = 'x'.repeat(textLength); shape.setText(text);
  const request = encodeWire(shape.frame);
  check(request.length === PRESENTATION_MAXIMUM, 'whole-frame maximum includes canonical CBOR framing');
  const pointer = holo_alloc(request.length) >>> 0;
  new Uint8Array(memory.buffer, pointer, request.length).set(request);
  const packed = BigInt.asUintN(64, holo_run(pointer, request.length));
  const at = Number(packed >> 32n), length = Number(packed & 0xffffffffn);
  check(length === request.length && at + length <= memory.buffer.byteLength, 'actual generated maximum result is bounded');
  const response = new Uint8Array(memory.buffer, at, length);
  check(equal(request, response), 'actual generated maximum round-trip preserves all bytes');
  const digest = async bytes => hex(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)));
  const requestDigest = await digest(request), responseDigest = await digest(response);
  const root = document.createElement('main'); document.body.append(root);
  const view = openPresentation({root, labels, dispatch() { throw Error('maximum display never dispatches'); },
    maximum: PRESENTATION_MAXIMUM, requestMaximum: PRESENTATION_MAXIMUM});
  try {
    view.render(request);
    const content = root.querySelector(shape.selector);
    check(content.textContent === text && content.textContent.length === textLength, 'actual DOM preserves full admitted maximum text');
    view.render(request); check(root.querySelector(shape.selector) === content, 'maximum identical repeat is idempotent');
    let rejected = false;
    try { view.render(new Uint8Array(PRESENTATION_MAXIMUM + 1)); }
    catch (error) { rejected = error instanceof PresentationError && error.code === 'bytes'; }
    check(rejected && root.querySelector(shape.selector) === content, 'over-budget bytes reject before changing maximum DOM');
    return {schema: 'prismpm/private-presentation-maximum/1', id: input.id, frame_length: request.length,
      text_length: textLength, request_sha256: requestDigest, response_sha256: responseDigest,
      maximum_memory: memory.buffer.byteLength, modelChecked: true};
  } finally { view.close(); root.remove(); }
}
