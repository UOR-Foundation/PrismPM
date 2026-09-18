import assert from 'node:assert/strict';
import {test} from 'node:test';
import {withBrowser} from './browser-test-server.mjs';

async function participant(t, {browser, baseURL}, options = {}) {
  const context = await browser.newContext();
  t.after(() => context.close());
  const page = await context.newPage();
  const unexpected = [];
  context.on('request', request => {
    if (!request.url().startsWith(baseURL)) unexpected.push(request.url());
  });
  t.after(() => assert.deepEqual(unexpected, []));
  page.setDefaultTimeout(10_000);
  await page.goto(baseURL);
  await page.evaluate(async options => {
    const {BrowserPeer} = await import('/peer.mjs');
    globalThis.received = [];
    globalThis.states = [];
    globalThis.errors = [];
    globalThis.peer = new BrowserPeer({
      ...options,
      onMessage: bytes => received.push([...bytes]),
      onState: state => states.push(state),
      onError: error => errors.push(error.code),
    });
  }, options);
  return page;
}

function peerTest(name, run) {
  test(name, {timeout: 15_000}, t => withBrowser(environment =>
    run(options => participant(t, environment, options))));
}

async function connect(left, right) {
  const offer = await left.evaluate(() => peer.createOffer());
  const answer = await right.evaluate(offer => peer.acceptOffer(offer), offer);
  await left.evaluate(answer => peer.acceptAnswer(answer), answer);
  await Promise.all([left.evaluate(() => peer.whenConnected()), right.evaluate(() => peer.whenConnected())]);
  return {offer, answer};
}

async function rawSender(left, right, options = {}) {
  const offer = await left.evaluate(async options => {
    globalThis.rawConnection = new RTCPeerConnection({iceServers: []});
    globalThis.rawChannel = rawConnection.createDataChannel('prismpm-opaque-v1', {
      ordered: true, protocol: 'prismpm/browser-peer/1', ...options,
    });
    await rawConnection.setLocalDescription(await rawConnection.createOffer());
    if (rawConnection.iceGatheringState !== 'complete') await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Raw peer ICE timeout')), 3000);
      rawConnection.onicegatheringstatechange = () => {
        if (rawConnection.iceGatheringState === 'complete') { clearTimeout(timeout); resolve(); }
      };
    });
    return JSON.stringify({version: 'prismpm/browser-peer/1', type: 'offer', session: crypto.randomUUID(), sdp: rawConnection.localDescription.sdp});
  }, options);
  const answer = await right.evaluate(offer => peer.acceptOffer(offer), offer);
  await left.evaluate(answer => rawConnection.setRemoteDescription({type: 'answer', sdp: JSON.parse(answer).sdp}), answer);
  await Promise.all([left.waitForFunction(() => rawChannel.readyState === 'open'), right.evaluate(() => peer.whenConnected())]);
}

peerTest('two separate browser contexts exchange opaque reliable ordered bytes without a signaling backend', async participant => {
  const left = await participant();
  const right = await participant();
  const {offer, answer} = await connect(left, right);
  assert.equal(JSON.parse(offer).session, JSON.parse(answer).session);
  assert.equal(JSON.parse(offer).type, 'offer');
  assert.equal(JSON.parse(answer).type, 'answer');
  await left.evaluate(() => {
    peer.send(new Uint8Array([0, 1, 127, 128, 255]));
    peer.send(new Uint8Array());
    peer.send(new Uint8Array([3]));
  });
  await right.waitForFunction(() => received.length === 3);
  assert.deepEqual(await right.evaluate(() => received), [[0, 1, 127, 128, 255], [], [3]]);
  await right.evaluate(() => peer.send(new Uint8Array([42])));
  await left.waitForFunction(() => received.length === 1);
  assert.deepEqual(await left.evaluate(() => received), [[42]]);
  await left.evaluate(() => peer.send(new Uint8Array(65_536).fill(255)));
  await right.waitForFunction(() => received.length === 4);
  assert.equal(await right.evaluate(() => received[3].length === 65_536 && received[3].every(value => value === 255)), true);
});

peerTest('closed sessions refuse operations and signal waiting callers', async participant => {
  const page = await participant();
  assert.deepEqual(await page.evaluate(async () => {
    const waiting = peer.whenConnected().catch(error => error.code);
    peer.close();
    peer.close();
    const results = [await waiting, peer.state];
    for (const operation of [() => peer.send(new Uint8Array()), () => peer.createOffer(), () => peer.acceptOffer('{}')]) {
      try { await operation(); } catch (error) { results.push(error.code); }
    }
    return results;
  }), ['PEER_CLOSED', 'closed', 'PEER_CLOSED', 'PEER_CLOSED', 'PEER_CLOSED']);
});

peerTest('rejects malformed, noncanonical, oversized, endpoint-bearing and media signaling before connection', async participant => {
  const left = await participant();
  const right = await participant();
  const offer = await left.evaluate(() => peer.createOffer());
  const result = await right.evaluate(async text => {
    const signal = JSON.parse(text);
    const mutations = [
      '', null, '{', 'x'.repeat(65_537), `${text}\n`, JSON.stringify(signal, null, 2),
      JSON.stringify({...signal, iceServers: [{urls: 'stun:unapproved.invalid'}]}),
      text.replace('"version":', '"version":"duplicate","version":'),
      JSON.stringify({...signal, version: 'unknown'}),
      JSON.stringify({...signal, type: 'answer'}),
      JSON.stringify({...signal, session: 'invalid'}),
      JSON.stringify({...signal, sdp: 'x'.repeat(32_769)}),
      JSON.stringify({...signal, sdp: signal.sdp.replaceAll('\r\n', '\n')}),
      JSON.stringify({...signal, sdp: `${signal.sdp}u=https://unapproved.invalid\r\n`}),
      JSON.stringify({...signal, sdp: `${signal.sdp}a=ice-server:stun:unapproved.invalid\r\n`}),
      JSON.stringify({...signal, sdp: signal.sdp.replace('typ host', 'typ relay')}),
      JSON.stringify({...signal, sdp: `${signal.sdp}m=audio 9 UDP/TLS/RTP/SAVPF 111\r\n`}),
    ];
    const outcomes = [];
    for (const value of mutations) {
      try { await peer.acceptOffer(value); outcomes.push('accepted'); }
      catch (error) { outcomes.push(error.code); }
    }
    return {outcomes, state: peer.state};
  }, offer);
  assert.deepEqual(result.outcomes, Array(17).fill('PEER_SIGNAL'));
  assert.equal(result.state, 'idle');
  const answer = await right.evaluate(offer => peer.acceptOffer(offer), offer);
  await left.evaluate(answer => peer.acceptAnswer(answer), answer);
  await Promise.all([left.evaluate(() => peer.whenConnected()), right.evaluate(() => peer.whenConnected())]);
});

peerTest('answers are bound to the offer session and stale negotiation is rejected', async participant => {
  const left = await participant();
  const right = await participant();
  const offer = await left.evaluate(() => peer.createOffer());
  const answer = await right.evaluate(offer => peer.acceptOffer(offer), offer);
  assert.deepEqual(await left.evaluate(async answer => {
    const other = JSON.parse(answer);
    other.session = crypto.randomUUID();
    const outcomes = [];
    for (const operation of [() => peer.createOffer(), () => peer.acceptOffer('{}'), () => peer.send(new Uint8Array()), () => peer.acceptAnswer(JSON.stringify(other))]) {
      try { await operation(); outcomes.push('accepted'); } catch (error) { outcomes.push(error.code); }
    }
    return {outcomes, state: peer.state};
  }, answer), {outcomes: ['PEER_STATE', 'PEER_STATE', 'PEER_STATE', 'PEER_SIGNAL'], state: 'awaiting-answer'});
  await left.evaluate(answer => peer.acceptAnswer(answer), answer);
  await left.evaluate(() => peer.whenConnected());
  assert.equal(await left.evaluate(async answer => {
    try { await peer.acceptAnswer(answer); } catch (error) { return error.code; }
  }, answer), 'PEER_STATE');
});

peerTest('unanswered offers expire and release waiting callers', async participant => {
  const page = await participant({connectTimeoutMs: 500});
  await page.evaluate(() => peer.createOffer());
  assert.equal(await page.evaluate(() => peer.whenConnected().catch(error => error.code)), 'PEER_TIMEOUT');
  assert.deepEqual(await page.evaluate(() => ({state: peer.state, errors})), {state: 'failed', errors: ['PEER_TIMEOUT']});
});

peerTest('closing while gathering rejects the operation and does not reopen the session', async participant => {
  const page = await participant();
  assert.deepEqual(await page.evaluate(async () => {
    const offer = peer.createOffer().catch(error => error.code);
    peer.close();
    return {outcome: await offer, state: peer.state};
  }), {outcome: 'PEER_CLOSED', state: 'closed'});
});

peerTest('closing an active session tears down the actual remote data channel', async participant => {
  const left = await participant();
  const right = await participant();
  await connect(left, right);
  await left.evaluate(() => peer.close());
  await right.waitForFunction(() => peer.state === 'failed');
  const errors = await right.evaluate(() => errors);
  assert.equal(errors.length, 1);
  // Chromium may report the SCTP abort before the channel-close event.
  assert.ok(['PEER_DISCONNECTED', 'PEER_CHANNEL'].includes(errors[0]));
  assert.equal(await right.evaluate(() => peer.whenConnected().catch(error => error.code)), errors[0]);
});

peerTest('outbound bounds and backpressure reject without silently dropping accepted messages', async participant => {
  const left = await participant({maxMessageBytes: 16, maxBufferedBytes: 24});
  const right = await participant({maxMessageBytes: 16, maxBufferedBytes: 24});
  await connect(left, right);
  assert.deepEqual(await left.evaluate(() => {
    const errors = [];
    for (const value of ['not-bytes', new DataView(new ArrayBuffer(1)), new Uint8Array(17)]) {
      try { peer.send(value); errors.push('accepted'); } catch (error) { errors.push(error.code); }
    }
    const detached = new Uint8Array(1);
    structuredClone(detached, {transfer: [detached.buffer]});
    try { peer.send(detached); errors.push('accepted'); } catch (error) { errors.push(error.code); }
    peer.send(new Uint8Array(16).fill(1));
    try { peer.send(new Uint8Array()); errors.push('accepted'); } catch (error) { errors.push(error.code); }
    return {errors, buffered: peer.bufferedAmount, state: peer.state};
  }), {errors: ['PEER_MESSAGE', 'PEER_MESSAGE', 'PEER_MESSAGE', 'PEER_MESSAGE', 'PEER_BACKPRESSURE'], buffered: 24, state: 'open'});
  await right.waitForFunction(() => received.length === 1);
  assert.deepEqual(await right.evaluate(() => received), [Array(16).fill(1)]);
  await left.waitForFunction(() => peer.bufferedAmount === 0);
  await left.evaluate(() => peer.send(new Uint8Array([2])));
  await right.waitForFunction(() => received.length === 2);
});

peerTest('transport options are closed and prohibit alternate ICE configuration', async participant => {
  const page = await participant();
  assert.deepEqual(await page.evaluate(async () => {
    const {BrowserPeer} = await import('/peer.mjs');
    const variants = [undefined, {}, {onMessage: 'wrong'}, {iceServers: []}, {maxMessageBytes: 0},
      {maxMessageBytes: 1.5}, {maxMessageBytes: 1_048_577}, {maxBufferedBytes: 1},
      {maxBufferedBytes: 4_194_305}, {connectTimeoutMs: 0}, {signalTimeoutMs: Infinity}, {onState: false}, {onError: false}];
    return variants.map(options => {
      try { new BrowserPeer(options === undefined ? undefined : {onMessage: () => {}, ...options}); return 'accepted'; }
      catch (error) { return error.code; }
    });
  }), ['PEER_CONFIG', 'accepted', ...Array(11).fill('PEER_CONFIG')]);
});

for (const [name, input] of [
  ['text instead of binary', 'not binary'],
  ['truncated header', [0x50, 0x50, 1]],
  ['wrong version', [0x50, 0x50, 2, 0, 0, 0, 0, 0]],
  ['invalid reserved byte', [0x50, 0x50, 1, 1, 0, 0, 0, 0]],
  ['inconsistent length', [0x50, 0x50, 1, 0, 0, 0, 0, 1]],
  ['trailing byte', [0x50, 0x50, 1, 0, 0, 0, 0, 0, 1]],
  ['oversized payload', [0x50, 0x50, 1, 0, 0, 0, 0, 17, ...Array(17).fill(0)]],
]) {
  peerTest(`real remote ${name} fails closed without delivery`, async participant => {
    const left = await participant();
    const right = await participant({maxMessageBytes: 16, maxBufferedBytes: 24});
    await rawSender(left, right);
    await left.evaluate(input => rawChannel.send(typeof input === 'string' ? input : new Uint8Array(input)), input);
    await right.waitForFunction(() => peer.state === 'failed');
    assert.deepEqual(await right.evaluate(() => ({received, errors})), {received: [], errors: ['PEER_MESSAGE']});
    await left.waitForFunction(() => rawChannel.readyState === 'closed');
  });
}

peerTest('asynchronous receive callbacks are ordered and their pending bytes are bounded', async participant => {
  const left = await participant();
  const right = await participant();
  await right.evaluate(async () => {
    peer.close();
    const {BrowserPeer} = await import('/peer.mjs');
    globalThis.peer = new BrowserPeer({
      maxMessageBytes: 16, maxBufferedBytes: 24,
      onMessage: bytes => {
        received.push([...bytes]);
        return new Promise(resolve => { globalThis.releaseReceive = resolve; });
      },
      onError: error => errors.push(error.code),
    });
  });
  await rawSender(left, right);
  await left.evaluate(() => {
    for (const value of [1, 2, 3]) rawChannel.send(new Uint8Array([0x50, 0x50, 1, 0, 0, 0, 0, 1, value]));
  });
  await right.waitForFunction(() => peer.state === 'failed');
  assert.deepEqual(await right.evaluate(() => ({received, errors})), {received: [[1]], errors: ['PEER_BACKPRESSURE']});
  await right.evaluate(() => releaseReceive());
  assert.deepEqual(await right.evaluate(() => received), [[1]]);
});

peerTest('receive callback failure is typed and closes the actual peer session', async participant => {
  const left = await participant();
  const right = await participant();
  await right.evaluate(async () => {
    peer.close();
    const {BrowserPeer} = await import('/peer.mjs');
    globalThis.peer = new BrowserPeer({
      onMessage: async () => { throw new Error('Application callback rejection'); },
      onError: error => errors.push(error.code),
    });
  });
  await connect(left, right);
  await left.evaluate(() => peer.send(new Uint8Array([1])));
  await right.waitForFunction(() => peer.state === 'failed');
  assert.deepEqual(await right.evaluate(() => errors), ['PEER_CALLBACK']);
});

for (const [name, options] of [
  ['unordered', {ordered: false}],
  ['lossy', {maxRetransmits: 0}],
  ['unknown protocol', {protocol: 'unrecognized'}],
]) {
  peerTest(`real remote ${name} data channel is rejected`, async participant => {
    const left = await participant();
    const right = await participant();
    await assert.rejects(rawSender(left, right, options), /Unexpected or unreliable data channel/);
    assert.deepEqual(await right.evaluate(() => ({state: peer.state, errors})), {state: 'failed', errors: ['PEER_CHANNEL']});
  });
}

peerTest('an undeclared second remote channel terminates the session', async participant => {
  const left = await participant();
  const right = await participant();
  await rawSender(left, right);
  await left.evaluate(() => rawConnection.createDataChannel('undeclared'));
  await right.waitForFunction(() => peer.state === 'failed');
  assert.deepEqual(await right.evaluate(() => errors), ['PEER_CHANNEL']);
});

peerTest('asynchronous receivers complete in wire order', async participant => {
  const left = await participant();
  const right = await participant();
  await right.evaluate(async () => {
    peer.close();
    const {BrowserPeer} = await import('/peer.mjs');
    globalThis.completed = [];
    globalThis.peer = new BrowserPeer({
      onMessage: async bytes => {
        received.push([...bytes]);
        if (bytes[0] === 1) await new Promise(resolve => { globalThis.releaseReceive = resolve; });
        completed.push(bytes[0]);
      },
      onError: error => errors.push(error.code),
    });
  });
  await connect(left, right);
  await left.evaluate(() => {
    peer.send(new Uint8Array([1]));
    peer.send(new Uint8Array([2]));
  });
  await right.waitForFunction(() => received.length === 1);
  assert.deepEqual(await right.evaluate(() => completed), []);
  await right.evaluate(() => releaseReceive());
  await right.waitForFunction(() => completed.length === 2);
  assert.deepEqual(await right.evaluate(() => ({received, completed, state: peer.state})), {received: [[1], [2]], completed: [1, 2], state: 'open'});
});

peerTest('asynchronous state and error callback faults cannot escape session teardown', async participant => {
  const page = await participant();
  const uncaught = [];
  page.on('pageerror', error => uncaught.push(error.message));
  assert.deepEqual(await page.evaluate(async () => {
    peer.close();
    const {BrowserPeer} = await import('/peer.mjs');
    globalThis.peer = new BrowserPeer({
      onMessage: () => {},
      onState: async () => { throw new Error('State callback fault'); },
      onError: async () => { throw new Error('Error callback fault'); },
    });
    const outcome = await peer.createOffer().catch(error => error.code);
    await new Promise(resolve => setTimeout(resolve, 0));
    return {outcome, state: peer.state};
  }), {outcome: 'PEER_CALLBACK', state: 'failed'});
  assert.deepEqual(uncaught, []);
});
