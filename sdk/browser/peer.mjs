// Host-only WebRTC transport: manual signaling, no discovery, STUN, TURN, or
// identity authority. Applications must authenticate every received object.
const VERSION = 'prismpm/browser-peer/1';
const CHANNEL = 'prismpm-opaque-v1';
const HEADER = 8;
const SIGNAL_MAXIMUM = 65_536;
const SDP_MAXIMUM = 32_768;
const MAGIC = [0x50, 0x50, 0x01, 0x00];

export class PeerTransportError extends Error {
  constructor(code, message) {
    super(message);
    this.name = 'PeerTransportError';
    this.code = code;
  }
}

function failure(code, message) { return new PeerTransportError(code, message); }

function boundedInteger(value, minimum, maximum, name) {
  if (!Number.isSafeInteger(value) || value < minimum || value > maximum) {
    throw failure('PEER_CONFIG', `Invalid ${name} bound`);
  }
  return value;
}

function description(sdp) {
  if (typeof sdp !== 'string' || sdp.length > SDP_MAXIMUM || !sdp.endsWith('\r\n') ||
      !/^[\x20-\x7e\r\n]+$/.test(sdp)) throw failure('PEER_SIGNAL', 'Invalid SDP encoding or bound');
  const lines = sdp.slice(0, -2).split('\r\n');
  if (lines.some(line => /[\r\n]/.test(line)) || lines[0] !== 'v=0' ||
      lines.filter(line => line.startsWith('m=')).length !== 1 ||
      !lines.some(line => /^m=application [0-9]+ UDP\/DTLS\/SCTP webrtc-datachannel$/.test(line))) {
    throw failure('PEER_SIGNAL', 'Only a single DTLS data channel description is supported');
  }
  const known = /^(?:v=0|o=- [0-9]+ [0-9]+ IN IP[46] [0-9a-fA-F:.]+|s=-|t=0 0|c=IN IP[46] [0-9a-fA-F:.]+|m=application [0-9]+ UDP\/DTLS\/SCTP webrtc-datachannel|a=group:BUNDLE [A-Za-z0-9_-]+|a=extmap-allow-mixed|a=msid-semantic: WMS(?: \*)?|a=mid:[A-Za-z0-9_-]+|a=ice-ufrag:[A-Za-z0-9+/]+|a=ice-pwd:[A-Za-z0-9+/]+|a=ice-options:trickle|a=fingerprint:sha-256 (?:[0-9A-F]{2}:){31}[0-9A-F]{2}|a=setup:(?:actpass|active|passive)|a=sctp-port:[0-9]+|a=max-message-size:[0-9]+|a=end-of-candidates)$/;
  for (const line of lines) {
    if (line.startsWith('a=candidate:')) {
      // Only locally gathered host candidates are accepted; no relay endpoints
      // or caller-supplied ICE-server configuration crosses this boundary.
      if (!/^a=candidate:[A-Za-z0-9+/]+ 1 (?:udp|tcp) [0-9]+ (?:[0-9a-fA-F:.]+|[a-z0-9-]+\.local) [0-9]+ typ host(?: (?:generation [0-9]+|network-id [0-9]+|network-cost [0-9]+|ufrag [A-Za-z0-9+/]+|tcptype (?:active|passive|so)))*$/.test(line)) {
        throw failure('PEER_SIGNAL', 'Unsupported ICE candidate');
      }
    } else if (!known.test(line)) throw failure('PEER_SIGNAL', 'Unsupported SDP field');
  }
  return sdp;
}

function envelope(type, session, sdp) {
  return JSON.stringify({version: VERSION, type, session, sdp: description(sdp)});
}

function parseSignal(text, type) {
  if (typeof text !== 'string' || text.length === 0 || text.length > SIGNAL_MAXIMUM) {
    throw failure('PEER_SIGNAL', 'Invalid signaling encoding or bound');
  }
  let value;
  try { value = JSON.parse(text); } catch { throw failure('PEER_SIGNAL', 'Malformed signaling document'); }
  if (!value || Array.isArray(value) || value.version !== VERSION || value.type !== type ||
      typeof value.session !== 'string' || !/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(value.session) ||
      text !== envelope(type, value.session, value.sdp)) {
    throw failure('PEER_SIGNAL', 'Noncanonical or incompatible signaling document');
  }
  return value;
}

export class BrowserPeer {
  #connection;
  #channel;
  #session;
  #state = 'idle';
  #terminalError;
  #deadline;
  #cancel = new Set();
  #onMessage;
  #onState;
  #onError;
  #maxMessageBytes;
  #maxBufferedBytes;
  #connectTimeoutMs;
  #signalTimeoutMs;
  #ready;
  #resolveReady;
  #rejectReady;
  #queue = [];
  #queuedBytes = 0;
  #receiving = false;

  constructor(options) {
    if (!options || Object.getPrototypeOf(options) !== Object.prototype ||
        Object.keys(options).some(key => !['onMessage', 'onState', 'onError', 'maxMessageBytes', 'maxBufferedBytes', 'connectTimeoutMs', 'signalTimeoutMs'].includes(key)) ||
        typeof options.onMessage !== 'function' ||
        (options.onState !== undefined && typeof options.onState !== 'function') ||
        (options.onError !== undefined && typeof options.onError !== 'function')) {
      throw failure('PEER_CONFIG', 'Expected closed transport options and a receive callback');
    }
    this.#maxMessageBytes = boundedInteger(options.maxMessageBytes ?? 65_536, 1, 1_048_576, 'message');
    this.#maxBufferedBytes = boundedInteger(options.maxBufferedBytes ?? 262_144, this.#maxMessageBytes + HEADER, 4_194_304, 'buffer');
    this.#connectTimeoutMs = boundedInteger(options.connectTimeoutMs ?? 60_000, 100, 600_000, 'connection timeout');
    this.#signalTimeoutMs = boundedInteger(options.signalTimeoutMs ?? 10_000, 100, 60_000, 'signaling timeout');
    this.#onMessage = options.onMessage;
    this.#onState = options.onState ?? (() => {});
    this.#onError = options.onError ?? (() => {});
    this.#ready = new Promise((resolve, reject) => {
      this.#resolveReady = resolve;
      this.#rejectReady = reject;
    });
    // A consumer may inspect events without awaiting readiness.
    this.#ready.catch(() => {});
  }

  get state() { return this.#state; }
  get bufferedAmount() { return this.#channel?.bufferedAmount ?? 0; }

  whenConnected() {
    return this.#terminalError ? Promise.reject(this.#terminalError) : this.#ready;
  }

  #expect(expected) {
    if (this.#terminalError) throw this.#terminalError;
    if (this.#state !== expected) throw failure('PEER_STATE', 'Operation does not match the signaling state');
  }

  #setState(state) {
    this.#state = state;
    try {
      Promise.resolve(this.#onState(state)).catch(() => this.#fail(failure('PEER_CALLBACK', 'Transport state callback failed')));
    } catch { this.#fail(failure('PEER_CALLBACK', 'Transport state callback failed')); }
  }

  #begin(state) {
    if (typeof globalThis.RTCPeerConnection !== 'function' || typeof globalThis.crypto?.randomUUID !== 'function') {
      throw failure('PEER_UNAVAILABLE', 'Secure browser WebRTC is unavailable');
    }
    this.#connection = new RTCPeerConnection({iceServers: [], iceCandidatePoolSize: 0, bundlePolicy: 'max-bundle'});
    this.#connection.onconnectionstatechange = () => {
      if (['failed', 'disconnected', 'closed'].includes(this.#connection.connectionState) && !this.#terminalError) {
        this.#fail(failure('PEER_DISCONNECTED', 'Peer connection ended'));
      }
    };
    this.#connection.ondatachannel = event => this.#attach(event.channel);
    this.#deadline = setTimeout(() => this.#fail(failure('PEER_TIMEOUT', 'Peer connection deadline exceeded')), this.#connectTimeoutMs);
    this.#setState(state);
    if (this.#terminalError) throw this.#terminalError;
  }

  async #gather() {
    if (this.#terminalError) throw this.#terminalError;
    if (this.#connection.iceGatheringState === 'complete') return;
    await new Promise((resolve, reject) => {
      const finish = error => {
        clearTimeout(timer);
        this.#connection.removeEventListener('icegatheringstatechange', changed);
        this.#cancel.delete(finish);
        if (error) reject(error); else resolve();
      };
      const changed = () => { if (this.#connection.iceGatheringState === 'complete') finish(); };
      const timer = setTimeout(() => finish(failure('PEER_TIMEOUT', 'ICE gathering deadline exceeded')), this.#signalTimeoutMs);
      this.#cancel.add(finish);
      this.#connection.addEventListener('icegatheringstatechange', changed);
      changed();
    });
  }

  #operation(promise) {
    // Browsers may leave an in-flight SDP operation unresolved after close().
    // Our session cancellation must settle it independently of the browser.
    return new Promise((resolve, reject) => {
      const cancel = error => { this.#cancel.delete(cancel); reject(error); };
      this.#cancel.add(cancel);
      promise.then(value => {
        this.#cancel.delete(cancel);
        if (this.#terminalError) reject(this.#terminalError); else resolve(value);
      }, error => { this.#cancel.delete(cancel); reject(error); });
      if (this.#terminalError) cancel(this.#terminalError);
    });
  }

  async createOffer() {
    this.#expect('idle');
    try {
      this.#begin('offering');
      this.#session = crypto.randomUUID();
      this.#attach(this.#connection.createDataChannel(CHANNEL, {ordered: true, protocol: VERSION}));
      const offer = await this.#operation(this.#connection.createOffer());
      await this.#operation(this.#connection.setLocalDescription(offer));
      await this.#gather();
      this.#expect('offering');
      const result = envelope('offer', this.#session, this.#connection.localDescription.sdp);
      this.#setState('awaiting-answer');
      if (this.#terminalError) throw this.#terminalError;
      return result;
    } catch (error) { throw this.#fail(error); }
  }

  async acceptOffer(text) {
    this.#expect('idle');
    const offer = parseSignal(text, 'offer');
    try {
      this.#begin('answering');
      this.#session = offer.session;
      await this.#operation(this.#connection.setRemoteDescription({type: 'offer', sdp: offer.sdp}));
      const answer = await this.#operation(this.#connection.createAnswer());
      await this.#operation(this.#connection.setLocalDescription(answer));
      await this.#gather();
      this.#expect('answering');
      const result = envelope('answer', this.#session, this.#connection.localDescription.sdp);
      this.#setState('connecting');
      if (this.#terminalError) throw this.#terminalError;
      return result;
    } catch (error) { throw this.#fail(error); }
  }

  async acceptAnswer(text) {
    this.#expect('awaiting-answer');
    const answer = parseSignal(text, 'answer');
    if (answer.session !== this.#session) throw failure('PEER_SIGNAL', 'Answer belongs to a different session');
    this.#setState('connecting');
    try {
      if (this.#terminalError) throw this.#terminalError;
      await this.#operation(this.#connection.setRemoteDescription({type: 'answer', sdp: answer.sdp}));
      if (this.#terminalError) throw this.#terminalError;
    } catch (error) { throw this.#fail(error); }
  }

  #attach(channel) {
    if (this.#channel || this.#terminalError || channel.label !== CHANNEL || channel.protocol !== VERSION ||
        !channel.ordered || channel.negotiated || channel.maxPacketLifeTime !== null || channel.maxRetransmits !== null) {
      channel.close();
      this.#fail(failure('PEER_CHANNEL', 'Unexpected or unreliable data channel'));
      return;
    }
    this.#channel = channel;
    channel.binaryType = 'arraybuffer';
    channel.onopen = () => {
      if (this.#terminalError) return;
      clearTimeout(this.#deadline);
      this.#setState('open');
      if (!this.#terminalError) this.#resolveReady();
    };
    channel.onclose = () => this.#fail(failure('PEER_DISCONNECTED', 'Peer data channel closed'));
    channel.onerror = () => this.#fail(failure('PEER_CHANNEL', 'Peer data channel failed'));
    channel.onmessage = event => this.#receive(event.data);
  }

  send(bytes) {
    this.#expect('open');
    if (!(bytes instanceof Uint8Array) || !(bytes.buffer instanceof ArrayBuffer) || bytes.byteLength > this.#maxMessageBytes) {
      throw failure('PEER_MESSAGE', 'Expected a bounded byte message');
    }
    const size = bytes.byteLength + HEADER;
    const negotiated = this.#connection.sctp?.maxMessageSize;
    if (negotiated > 0 && size > negotiated) throw failure('PEER_MESSAGE', 'Message exceeds negotiated SCTP bound');
    if (this.#channel.bufferedAmount + size > this.#maxBufferedBytes) {
      throw failure('PEER_BACKPRESSURE', 'Peer send buffer is full');
    }
    const frame = new Uint8Array(size);
    frame.set(MAGIC);
    new DataView(frame.buffer).setUint32(4, bytes.byteLength, false);
    try { frame.set(bytes, HEADER); } catch { throw failure('PEER_MESSAGE', 'Message buffer is detached'); }
    try { this.#channel.send(frame); } catch { throw this.#fail(failure('PEER_CHANNEL', 'Peer send failed')); }
  }

  #receive(data) {
    if (this.#terminalError) return;
    if (!(data instanceof ArrayBuffer) || data.byteLength < HEADER || data.byteLength > this.#maxMessageBytes + HEADER) {
      this.#fail(failure('PEER_MESSAGE', 'Invalid or oversized peer frame'));
      return;
    }
    const bytes = new Uint8Array(data);
    if (MAGIC.some((value, index) => bytes[index] !== value) || new DataView(data).getUint32(4, false) !== data.byteLength - HEADER) {
      this.#fail(failure('PEER_MESSAGE', 'Invalid peer frame header'));
      return;
    }
    if (this.#queuedBytes + data.byteLength > this.#maxBufferedBytes) {
      this.#fail(failure('PEER_BACKPRESSURE', 'Peer receive buffer is full'));
      return;
    }
    this.#queue.push(bytes);
    this.#queuedBytes += data.byteLength;
    void this.#drain();
  }

  async #drain() {
    if (this.#receiving) return;
    this.#receiving = true;
    try {
      while (this.#queue.length && !this.#terminalError) {
        const frame = this.#queue.shift();
        await this.#onMessage(frame.slice(HEADER));
        this.#queuedBytes -= frame.byteLength;
      }
    } catch { this.#fail(failure('PEER_CALLBACK', 'Peer receive callback failed')); }
    finally { this.#receiving = false; }
  }

  #finish(error, state) {
    if (this.#terminalError) return this.#terminalError;
    this.#terminalError = error;
    clearTimeout(this.#deadline);
    for (const cancel of this.#cancel) cancel(error);
    this.#queue.length = 0;
    if (this.#channel) {
      this.#channel.onopen = this.#channel.onclose = this.#channel.onerror = this.#channel.onmessage = null;
      this.#channel.close();
    }
    if (this.#connection) {
      this.#connection.onconnectionstatechange = this.#connection.ondatachannel = null;
      this.#connection.close();
    }
    this.#rejectReady(error);
    this.#setState(state);
    return error;
  }

  #fail(error) {
    if (this.#terminalError) return this.#terminalError;
    const typed = error instanceof PeerTransportError ? error : failure('PEER_CHANNEL', 'WebRTC operation failed');
    this.#finish(typed, 'failed');
    try { Promise.resolve(this.#onError(typed)).catch(() => {}); }
    catch { /* Reporting cannot reopen a failed transport. */ }
    return typed;
  }

  close() { this.#finish(failure('PEER_CLOSED', 'Peer session is closed'), 'closed'); }
}
