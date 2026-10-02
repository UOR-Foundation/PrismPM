// Read-only Docker-compatible registry credential selection. No login,
// credential mutation, or secret-bearing command-line argument is performed.
import assert from 'node:assert/strict';
import {constants, closeSync, fstatSync, lstatSync, openSync, readSync, realpathSync} from 'node:fs';
import {join, resolve} from 'node:path';
import {caseKeys, parseJson} from './metadata-layer.mjs';

function configBytes(path) {
  const entry = lstatSync(path, {throwIfNoEntry:false});
  if (!entry) return null;
  assert(entry.isFile() && entry.nlink === 1, 'registry configuration must be a singly linked regular file');
  assert.equal(realpathSync(path), resolve(path), 'aliased registry configuration refused');
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    const before = fstatSync(fd, {bigint:true});
    assert(before.isFile() && before.nlink === 1n && before.size <= 1024n * 1024n, 'registry configuration exceeds bound or changed');
    const bytes = Buffer.alloc(Number(before.size)); let offset = 0;
    while (offset < bytes.length) {
      const size = readSync(fd, bytes, offset, bytes.length - offset, null); assert(size > 0, 'registry configuration shortened'); offset += size;
    }
    assert.equal(readSync(fd, Buffer.alloc(1), 0, 1, null), 0, 'registry configuration grew');
    for (const after of [fstatSync(fd, {bigint:true}), lstatSync(path, {bigint:true})])
      for (const field of ['dev','ino','size','mode','nlink','mtimeNs','ctimeNs']) assert.equal(after[field], before[field], 'registry configuration changed');
    assert.equal(realpathSync(path), resolve(path), 'registry configuration became aliased');
    return bytes;
  } finally {closeSync(fd);}
}

function credential(entry) {
  assert(entry && typeof entry === 'object' && !Array.isArray(entry), 'registry credentials must be an object');
  caseKeys(entry, ['auth','identitytoken','registrytoken','username','password','serveraddress','email']);
  const present = ['auth','identitytoken','registrytoken'].filter(key => entry[key] !== undefined && entry[key] !== '');
  assert(present.length <= 1, 'ambiguous registry credential kinds');
  if (entry.registrytoken !== undefined && entry.registrytoken !== '') {
    assert(typeof entry.registrytoken === 'string' && entry.registrytoken.length <= 16384
      && /^[A-Za-z0-9._~+/-]+=*$/.test(entry.registrytoken), 'bounded registry bearer token required');
    return {kind:'bearer', secret:entry.registrytoken};
  }
  if (entry.identitytoken !== undefined && entry.identitytoken !== '') {
    assert(entry.auth === undefined || entry.auth === '', 'ambiguous registry credential kinds');
    assert(typeof entry.identitytoken === 'string' && entry.identitytoken.length <= 16384, 'registry refresh token exceeds bound');
    return {kind:'refresh', secret:entry.identitytoken};
  }
  if (entry.auth === undefined || entry.auth === '') return null;
  assert(typeof entry.auth === 'string' && entry.auth.length <= 32768 && /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(entry.auth),
    'canonical bounded registry auth required');
  const bytes = Buffer.from(entry.auth, 'base64');
  assert(bytes.toString('base64') === entry.auth, 'noncanonical registry auth encoding');
  const text = new TextDecoder('utf-8', {fatal:true}).decode(bytes), at = text.indexOf(':');
  assert(at > 0 && at <= 256 && text.length - at - 1 > 0 && text.length - at - 1 <= 16384, 'bounded registry username/password required');
  const username = text.slice(0, at), secret = text.slice(at + 1);
  assert(!/[\r\n]/.test(username), 'registry username contains a line break');
  return {kind:'basic', username, secret};
}

export function dockerCredentialProvider({directory, runHelper} = {}) {
  const selected = directory ?? process.env.DOCKER_CONFIG ?? (process.env.HOME && join(process.env.HOME, '.docker'));
  assert(typeof selected === 'string' && selected.startsWith('/') && resolve(selected) === selected, 'absolute registry configuration directory required');
  return async (origin, signal) => {
    signal.throwIfAborted();
    const url = new URL(origin); assert.equal(url.origin, origin);
    const bytes = configBytes(join(selected, 'config.json'));
    if (bytes === null) return null;
    let config;
    try {config = parseJson(bytes);} catch {throw new Error('invalid registry configuration JSON');}
    caseKeys(config, ['auths','credsStore','credHelpers']);
    const hub = url.protocol === 'https:' && url.port === '' && ['docker.io','registry-1.docker.io'].includes(url.hostname);
    const keys = [url.host, origin, origin + '/'];
    if (hub) for (const key of ['docker.io','https://docker.io','https://docker.io/',
      'registry-1.docker.io','https://registry-1.docker.io','https://registry-1.docker.io/','https://index.docker.io/v1/'])
      if (!keys.includes(key)) keys.push(key);
    const matching = map => {
      assert(map && typeof map === 'object' && !Array.isArray(map), 'registry configuration map required');
      const values = keys.filter(key => Object.hasOwn(map, key)).map(key => [key, map[key]]);
      if (values.length > 1) assert(values.every(([, value]) => JSON.stringify(value) === JSON.stringify(values[0][1])),
        'ambiguous registry credential aliases');
      return values[0];
    };
    const helper = config.credHelpers === undefined ? undefined : matching(config.credHelpers);
    const store = helper?.[1] ?? config.credsStore;
    if (store !== undefined && store !== '') {
      assert(typeof store === 'string' && /^[a-z0-9][a-z0-9._-]{0,63}$/.test(store), 'invalid registry credential helper');
      assert.equal(typeof runHelper, 'function', 'SDK-verified credential helper runner required');
      const defaultServer = hub ? 'https://index.docker.io/v1/' : url.host;
      const result = await runHelper('docker-credential-' + store, helper?.[0] ?? defaultServer, signal);
      signal.throwIfAborted();
      if (result === null) return null; // exact helper not-found outcome only
      assert(result && typeof result === 'object' && !Array.isArray(result)
        && Object.keys(result).sort().join(',') === 'Secret,Username', 'closed registry helper response required');
      if (result.Username === '<token>') {
        assert(typeof result.Secret === 'string' && result.Secret.length > 0, 'empty registry helper token');
        return credential({identitytoken:result.Secret});
      }
      assert(typeof result.Username === 'string' && typeof result.Secret === 'string', 'registry helper fields must be strings');
      assert(result.Username.length > 0 && result.Username.length <= 256 && !/[:\r\n]/.test(result.Username), 'invalid registry helper username');
      return credential({auth:Buffer.from(result.Username + ':' + result.Secret).toString('base64')});
    }
    const entry = config.auths === undefined ? undefined : matching(config.auths);
    signal.throwIfAborted();
    return entry ? credential(entry[1]) : null;
  };
}
