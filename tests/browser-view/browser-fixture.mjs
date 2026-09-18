// Acceptance only: actual generated models and crypto/IDB; no shipping hooks.
export async function runFixture(input) {
  const {openWorkspaceView, ViewHostError} = await import('./view-host.mjs');
  const {openCommands} = await import('./commands.mjs');
  const {createIdentity, BrowserEffectError} = await import('./identity.mjs');
  const {openStore} = await import('./store.mjs');
  const modules = Object.fromEntries(await Promise.all(Object.entries(input.modules).map(async ([key, bytes]) => [key, await WebAssembly.compile(new Uint8Array(bytes))])));
  const labels = new Uint8Array(input.labels), names = JSON.parse(new TextDecoder().decode(labels));
  const check = (value, message) => { if (!value) throw Error(message); };
  const text = value => new TextEncoder().encode(value), empty = () => new Uint8Array();
  const hex = bytes => Array.from(bytes, n => n.toString(16).padStart(2, '0')).join('');
  const unhex = value => Uint8Array.from(value.match(/../g), x => parseInt(x, 16));
  const concat = (...parts) => new Uint8Array(parts.flatMap(part => Array.from(part)));
  const stores = [], views = [], adapters = [], cases = [], calls = [], maximum = {};
  const Instance = WebAssembly.Instance;
  WebAssembly.Instance = class {
    constructor(module, imports) {
      const real = new Instance(module, imports), exports = {...real.exports};
      const kind = Object.keys(modules).find(key => modules[key] === module);
      exports.holo_run = (pointer, length) => {
        const request = hex(new Uint8Array(exports.memory.buffer, pointer, length));
        const output = BigInt.asUintN(64, real.exports.holo_run(pointer, length));
        calls.push({kind, request, response: hex(new Uint8Array(exports.memory.buffer, Number(output >> 32n), Number(output & 0xffffffffn)))});
        maximum[kind] = Math.max(maximum[kind] ?? 0, exports.memory.buffer.byteLength); return output;
      };
      return {exports};
    }
  };
  const deferred = () => { let resolve; const promise = new Promise(done => { resolve = done; }); return {promise, resolve}; };
  async function rejects(operation, code, detail) {
    let error; try { await operation; } catch (value) { error = value; }
    check(error instanceof ViewHostError && error.code === code, 'expected private host ' + code + ', got ' + error?.code);
    if (detail !== undefined) check(error.detail === detail, 'expected model detail ' + detail + ', got ' + error.detail);
    check(error.cause === undefined && Object.keys(error).sort().join(',') === 'code,detail,name', 'no exception payload');
  }
  async function fixture() {
    const namespace = 'view-host-' + crypto.randomUUID(), owner = await openStore(namespace), reader = await openStore(namespace + '-reader');
    stores.push(owner, reader); const first = await createIdentity(), second = await createIdentity();
    await owner.saveIdentity(first); await reader.saveIdentity(second);
    const workspace = crypto.getRandomValues(new Uint8Array(32)); let commits = 0;
    const hooks = {beforeRead: null, afterCommit: null, commitError: null};
    const binding = keys => ({loadIdentity: () => keys.loadIdentity(),
      readHead: async (...args) => { if (hooks.beforeRead) await hooks.beforeRead(); return owner.readHead(...args); },
      readObject: (...args) => owner.readObject(...args),
      commit: async (...args) => { if (hooks.commitError) return hooks.commitError(...args); const value = await owner.commit(...args); commits++; if (hooks.afterCommit) await hooks.afterCommit(); return value; }});
    const command = await openCommands({commandModule: modules.Command, journalModule: modules.Journal, store: binding(owner), headName: 'workspace'}); adapters.push(command);
    await command.submit({action: 0, body: empty(), workspace});
    const open = async (keys = owner, custom = {}) => {
      const root = document.createElement('main'); document.body.append(root);
      const view = await openWorkspaceView({viewModule: modules.View, commandModule: modules.Command, queryModule: modules.Query,
        journalModule: modules.Journal, store: binding(keys), headName: 'workspace', root, labels, ...custom}); views.push(view);
      return {view, root, select: () => view.dispatch(concat([0], workspace)),
        command: (action, body = empty()) => view.dispatch(concat([4, action], body)),
        refresh: () => view.dispatch(Uint8Array.of(5)), members: () => view.dispatch(Uint8Array.of(1)), messages: () => view.dispatch(Uint8Array.of(2)),
        status: () => root.querySelector('[data-slot=status]').textContent,
        rows: () => [...root.querySelectorAll('tbody tr')].map(row => [...row.cells].map(cell => cell.textContent))};
    };
    return {namespace, owner, reader, first, second, workspace, command, open, hooks, commits: () => commits};
  }
  try {
    const f = await fixture(), owner = await f.open();
    check(Object.isFrozen(owner.view) && Object.keys(owner.view).sort().join(',') === 'close,dispatch', 'closed private API');
    for (const key of ['state', 'session', 'snapshot', 'queries', 'commands', 'complete', 'principal']) check(owner.view[key] === undefined, 'private ' + key);
    const selecting = owner.select(); check(owner.status() === names.pending, 'synchronous model pending'); await selecting;
    check(owner.rows().length === 1 && owner.rows()[0][0] === f.first.principal.slice(7), 'actual admitted owner row');
    check(document.activeElement === owner.root.querySelector('[data-slot=result]'), 'modeled result focus');
    check(owner.root.querySelector('[data-slot=status]').getAttribute('aria-live') === 'polite', 'modeled live polite');
    check(owner.root.querySelector('h1').textContent === names.title && !owner.root.querySelector('img,script'), 'modeled labels are text, not HTML');
    check(globalThis.labelExecuted === undefined, 'hostile label did not execute');
    cases.push('private bootstrap, actual admitted rows, semantic labels/focus/live and text-only modeled labels');

    for (const [index, bad] of [empty(), new Uint8Array(4099), {}, null, new Proxy({}, {getPrototypeOf() { throw null; }})].entries()) {
      if (index === 0) await rejects(owner.view.dispatch(bad), 'model-rejected', 3);
      else await rejects(owner.view.dispatch(bad), 'invalid-input');
    }
    await rejects(owner.view.dispatch(Uint8Array.of(1), true), 'invalid-input');
    const oversized = new Uint8Array(4099); Object.defineProperty(oversized, 'byteLength', {value: 0});
    await rejects(owner.view.dispatch(oversized), 'invalid-input');
    check(typeof SharedArrayBuffer === 'function', 'real isolated Chromium shared-buffer boundary is available');
    const shared = new Uint8Array(new SharedArrayBuffer(33)); shared.set(concat([0], f.workspace));
    Object.defineProperty(shared, 'buffer', {value: new ArrayBuffer(33)});
    await rejects(owner.view.dispatch(shared), 'invalid-input');
    const shadowed = concat([0], f.workspace);
    for(const name of ['buffer','byteLength','byteOffset','length']) Object.defineProperty(shadowed,name,{get(){throw null;}});
    await owner.view.dispatch(shadowed);
    const captured = concat([0], f.workspace), capturedWork = owner.view.dispatch(captured);
    structuredClone(captured.buffer, {transfer: [captured.buffer]}); await capturedWork;
    check(owner.rows()[0][0] === f.first.principal.slice(7), 'synchronous detached public intent capture');
    const badLabels = [text('{}'), text(JSON.stringify({...names, extra: 'x'})), text(JSON.stringify({...names, title: 'x'.repeat(257)})),
      text(new TextDecoder().decode(labels).replace('{', '{"action":"duplicate",')), new Uint8Array(8193), Uint8Array.of(255), concat([0xef,0xbb,0xbf],labels)];
    for (const bad of badLabels) await rejects(f.open(f.owner, {labels: bad}), 'invalid-labels');
    const canonical = value => text(JSON.stringify(Object.fromEntries(Object.keys(value).sort().map(key => [key,value[key]]))));
    const exact = {...names};
    for(const key of Object.keys(exact).filter(key=>key!=='spec')) {
      const remaining = 8192-canonical(exact).length;
      exact[key] += 'a'.repeat(Math.min(remaining,256-text(exact[key]).length));
      if(canonical(exact).length===8192)break;
    }
    check(canonical(exact).length===8192,'exact full labels artifact boundary constructed');
    const maximal = await f.open(f.owner,{labels:canonical(exact)}); maximal.view.close();
    const unicode = await f.open(f.owner,{labels:canonical({...names,title:'é'.repeat(128)})});
    check(unicode.root.querySelector('h1').textContent==='é'.repeat(128),'exact256 UTF8 bytes preserved');unicode.view.close();
    await rejects(f.open(f.owner,{labels:canonical({...names,title:'é'.repeat(129)})}),'invalid-labels');
    cases.push('bounded closed labels/public intent, hostile accessors, duplicate JSON and detached capture');

    await owner.command(1, unhex(f.second.principal.slice(7)));
    check(owner.status() === names.replay && owner.rows().length === 0, 'success requires explicit replay and clears rows');
    await rejects(owner.messages(), 'model-rejected', 5); await owner.refresh(); await owner.members();
    check(owner.rows().length === 2 && owner.rows()[1][1] === names.contributor, 'actual contributor grant');
    const participant = await f.open(f.reader); await participant.select();
    const hostile = '\uFEFF<img src=x onerror="globalThis.messageExecuted=true"> & <script>bad()</script> ☃';
    await participant.command(4, text(hostile)); await participant.refresh(); await participant.messages();
    check(participant.rows()[0][2] === hostile && !participant.root.querySelector('img,script'), 'admitted message content remains exact text');
    check(globalThis.messageExecuted === undefined, 'hostile body did not execute');
    const staleCommits = f.commits();
    await owner.command(3, unhex(f.second.principal.slice(7)));
    check(owner.status() === names.conflict && f.commits() === staleCommits, 'actual stale head rejects revoke without committing');
    await owner.refresh(); await owner.command(3, unhex(f.second.principal.slice(7))); await owner.refresh();
    await participant.messages();
    check(participant.status() === names.rejected && participant.rows().length === 0, 'current revocation clears and rejects new disclosure');
    await owner.command(2, unhex(f.second.principal.slice(7))); await owner.refresh(); await participant.messages();
    check(participant.rows()[0][2] === hostile, 'reader admission after regrant');
    await participant.refresh();
    const beforeRejected = f.commits(); await participant.command(4, text('reader must not write'));
    check(participant.status() === names.rejected && f.commits() === beforeRejected, 'known modeled role rejection has no durable write');
    await rejects(participant.messages(), 'model-rejected', 5);
    check(participant.root.querySelector('[data-slot=status]').getAttribute('aria-live') === 'assertive', 'rejected replay barrier is assertive');
    await participant.refresh(); await participant.messages(); check(participant.rows()[0][2] === hostile, 'explicit recovery after known rejection barrier');
    cases.push('two possessed identities, all grant/revoke/post roles, read revocation and real requiresRefresh rejection barrier');

    owner.view.close(); participant.view.close(); f.owner.close(); f.reader.close();
    const persistedOwner = await openStore(f.namespace), persistedReader = await openStore(f.namespace + '-reader'); stores.push(persistedOwner, persistedReader);
    check((await persistedOwner.loadIdentity()).principal === f.first.principal && (await persistedReader.loadIdentity()).principal === f.second.principal, 'independent identities persisted');
    const root = document.createElement('main'); document.body.append(root);
    const reopened = await openWorkspaceView({viewModule: modules.View, commandModule: modules.Command, queryModule: modules.Query, journalModule: modules.Journal,
      store: {loadIdentity: () => persistedReader.loadIdentity(), readHead: (...args) => persistedOwner.readHead(...args), readObject: (...args) => persistedOwner.readObject(...args), commit: (...args) => persistedOwner.commit(...args)}, headName: 'workspace', root, labels}); views.push(reopened);
    await reopened.dispatch(concat([0], f.workspace)); await reopened.dispatch(Uint8Array.of(2));
    check(root.querySelector('tbody tr td:last-child').textContent === hostile, 'reopen replays actual persisted admitted content');
    cases.push('close/reopen preserves both actual nonextractable identities and authenticated persisted content');

    const maximumFixture = await fixture(), maximumView = await maximumFixture.open(); await maximumView.select();
    for (let index = 0; index < 17; index++) {
      await maximumView.command(4, text(String.fromCharCode(65 + index).repeat(4096))); await maximumView.refresh();
    }
    await maximumView.messages(); check(maximumView.rows().length === 16 && maximumView.rows().every(row => text(row[2]).length === 4096), 'complete16 maximum UTF8 bodies, no truncation');
    check(maximumView.root.querySelector('[data-slot=paging]').textContent === names.total + ': 17; ' + names.offset + ': 0', 'exact first-page totals');
    await maximumView.view.dispatch(Uint8Array.of(3)); check(maximumView.rows().length === 1 && maximumView.rows()[0][2] === 'Q'.repeat(4096), 'Next consumes only modeled private cursor');
    await rejects(maximumView.view.dispatch(Uint8Array.of(3)), 'model-rejected', 9);
    await maximumView.command(0); check(maximumView.status() === names.rejected, 'duplicate genesis rejected by generated model'); await maximumView.refresh();
    cases.push('all five commands, full16x4096 presentation, exact paging and final-page rejection');

    const race = await fixture(), raceView = await race.open(); await raceView.select();
    const entered = deferred(), release = deferred(); let once = false;
    race.hooks.beforeRead = async () => { if (!once) { once = true; entered.resolve(); await release.promise; } };
    const pending = raceView.messages(); await entered.promise;
    await rejects(raceView.members(), 'model-rejected', 4);
    [...raceView.root.querySelectorAll('button')].find(node => node.textContent === names.members).dispatchEvent(new MouseEvent('click', {bubbles: true}));
    raceView.view.close(); raceView.view.close();
    check(raceView.status() === names.closed && raceView.rows().length === 0, 'close immediately clears pending UI');
    release.resolve(); await pending;
    check(raceView.status() === names.closed && raceView.rows().length === 0 && raceView.root.querySelector('[data-slot=diagnostic]').textContent === '', 'late completion cannot reopen or diagnose closed UI');
    await rejects(raceView.members(), 'view-closed'); race.hooks.beforeRead = null;
    for (const intentClose of [false, true]) {
      const faulty = await fixture(), faultyView = await faulty.open(); await faultyView.select();
      const started = deferred(), finished = deferred(); let held = false;
      faulty.hooks.beforeRead = async () => { if (!held) { held = true; started.resolve(); await finished.promise; } };
      const reading = faultyView.messages(); await started.promise;
      const create = document.createElement; document.createElement = () => { throw new Proxy({}, {get() { throw null; }}); };
      try {
        await rejects(intentClose ? faultyView.view.dispatch(Uint8Array.of(6)) : Promise.resolve().then(() => faultyView.view.close()), 'host-unavailable');
      } finally { document.createElement = create; finished.resolve(); }
      await reading; faultyView.view.close();
      check(faultyView.root.childNodes.length === 0, 'failed terminal rendering clears DOM and suppresses late promotion');
      await rejects(faultyView.members(), 'view-closed');
    }
    cases.push('single-flight, idempotent close, retained pending read and late completion suppression');

    const durable = await fixture(), durableView = await durable.open(); await durableView.select();
    const commitEntered = deferred(), commitRelease = deferred();
    durable.hooks.afterCommit = async () => { commitEntered.resolve(); await commitRelease.promise; };
    const writing = durableView.command(4, text('already durable')); await commitEntered.promise; durableView.view.close(); commitRelease.resolve(); await writing;
    check(durableView.status() === names.closed && durableView.rows().length === 0, 'late durable completion cannot reopen'); durable.hooks.afterCommit = null;
    const afterClose = await durable.open(); await afterClose.select(); await afterClose.messages();
    check(afterClose.rows()[0][2] === 'already durable' && durable.commits() === 2, 'close never claims rollback or repeats a durable effect');
    cases.push('close during actual durable commit preserves committed bytes without late promotion or retry');

    const uncertain = await fixture(), uncertainView = await uncertain.open(); await uncertainView.select();
    uncertain.hooks.commitError = async (...args) => { await uncertain.owner.commit(...args); throw new Proxy({}, {get() { throw Error('private backend payload'); }}); };
    await uncertainView.command(4, text('uncertain actual commit'));
    check(uncertainView.status() === names.unknown && uncertainView.rows().length === 0, 'unknown actual durable outcome stays distinct');
    check(!uncertainView.root.textContent.includes('private backend payload'), 'hostile error payload not rendered');
    uncertain.hooks.commitError = null; await uncertainView.refresh(); await uncertainView.messages();
    check(uncertainView.rows()[0][2] === 'uncertain actual commit', 'uncertain outcome requires actual authenticated replay');
    uncertain.hooks.beforeRead = async () => { throw null; }; await uncertainView.messages();
    check(uncertainView.status() === names.unavailable && uncertainView.rows().length === 0, 'hostile query exception clears rows and reports modeled unavailable');
    uncertain.hooks.beforeRead = null; await uncertainView.messages(); check(uncertainView.rows().length === 1, 'new query genuinely replays after recovery');
    cases.push('hostile storage outcomes, actual uncertain commit, no payload leak and explicit authenticated recovery');

    // A live fixture retained only by acceptance infrastructure for actual keyboard actions.
    const keyboard = await fixture(), keyboardView = await keyboard.open();
    globalThis.__viewKeyboardFixture = {view: keyboardView.view, root: keyboardView.root, workspace: hex(keyboard.workspace), commits: keyboard.commits,
      finish: () => { WebAssembly.Instance = Instance; return {calls, maximum}; }};
    return {cases, calls, maximum, names, keyboardWorkspace: hex(keyboard.workspace)};
  } catch (error) {
    for (const view of views) try { view.close(); } catch {};
    WebAssembly.Instance = Instance;
    throw error;
  }
}
