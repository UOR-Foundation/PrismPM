// Bounded OCI Distribution reads. Redirects and authentication share one
// request deadline/budget; binary layers are never HTTP-auto-decompressed.
import assert from 'node:assert/strict';
import {parseJson} from './metadata-layer.mjs';

export function registryReference(reference) {
  assert.match(reference, /^[a-z0-9.-]+(?::[0-9]{1,5})?\/[a-z0-9./_-]+@sha256:[0-9a-f]{64}$/);
  const [repository, digest] = reference.split('@'), [authority, ...parts] = repository.split('/');
  assert(parts.every(part => part && part !== '.' && part !== '..'), 'noncanonical OCI repository');
  const [host, port] = authority.split(':');
  assert(port === undefined || (Number(port) > 0 && Number(port) <= 65535), 'invalid OCI registry port');
  const scheme = host === '127.0.0.1' || host === 'localhost' ? 'http:' : 'https:';
  // Docker 28.4 registry/config.go distinguishes the reference namespace from
  // its V2 transport host. Nondefault ports are independent registry origins.
  const endpoint = host === 'docker.io' && (port === undefined || Number(port) === 443) ? 'registry-1.docker.io' : authority;
  return {repository, path:parts.join('/'), digest, origin:new URL(scheme + '//' + endpoint).origin};
}

function secureUrl(value, origin) {
  const url = new URL(value);
  assert(!url.username && !url.password && !url.hash, 'OCI URL credentials/fragments refused');
  assert(url.protocol === 'https:' || (url.protocol === 'http:' && url.origin === origin
    && ['localhost','127.0.0.1'].includes(url.hostname)), 'OCI insecure redirect or authentication endpoint refused');
  return url;
}

function challenge(value) {
  assert(typeof value === 'string' && value.length <= 4096 && /^Bearer /i.test(value), 'bounded OCI bearer challenge required');
  let rest = value.slice(7); const result = {};
  while (rest) {
    const match = /^\s*([a-z_]+)="([^"\\\r\n]*)"\s*(?:,\s*|$)/i.exec(rest);
    assert(match, 'ambiguous OCI bearer challenge');
    const key = match[1].toLowerCase(); assert(!Object.hasOwn(result, key), 'duplicate OCI challenge field');
    assert(['realm','service','scope','error','error_description'].includes(key), 'unknown OCI challenge field');
    result[key] = match[2]; rest = rest.slice(match[0].length);
  }
  assert(result.realm, 'OCI token realm absent');
  return result;
}

export function createRegistryTransport(reference, credentials = async () => null) {
  assert(process.env.NODE_TLS_REJECT_UNAUTHORIZED !== '0', 'disabled TLS verification refused');
  const registry = registryReference(reference);
  assert.equal(typeof credentials, 'function');
  return async ({kind, reference:requested, maximum, timeout_ms}) => {
    assert(['manifest','blob'].includes(kind), 'closed OCI fetch kind required');
    assert(Number.isSafeInteger(maximum) && maximum > 0 && maximum <= 32 * 1024 * 1024);
    assert(Number.isSafeInteger(timeout_ms) && timeout_ms > 0 && timeout_ms <= 45000);
    const target = registryReference(requested); assert.equal(target.repository, registry.repository, 'OCI repository changed');
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), timeout_ms);
    const signal = controller.signal;
    let requests = 0;
    try {
      const resolveCredential = async () => {
        signal.throwIfAborted();
        let abort;
        const aborted = new Promise((_, reject) => {
          abort = () => reject(new Error('OCI credential deadline exceeded'));
          signal.addEventListener('abort', abort, {once:true});
          if (signal.aborted) abort();
        });
        try {return await Promise.race([credentials(registry.origin, signal), aborted]);}
        finally {signal.removeEventListener('abort', abort);}
      };
      const basic = credential => {
        assert(credential?.kind === 'basic', 'OCI basic credentials required');
        assert(typeof credential.secret === 'string' && credential.secret.length > 0 && credential.secret.length <= 16384,
          'bounded OCI credential required');
        assert(typeof credential.username === 'string' && credential.username.length > 0 && credential.username.length <= 256
          && !/[:\r\n]/.test(credential.username), 'bounded OCI username required');
        return 'Basic ' + Buffer.from(credential.username + ':' + credential.secret).toString('base64');
      };
      const read = async (url, bound, headers, method = 'GET', body) => {
        let current = secureUrl(url, registry.origin), forwarded = {...headers};
        for (let redirects = 0; ; redirects++) {
          assert(++requests <= 16, 'OCI request budget exhausted');
          signal.throwIfAborted();
          const response = await fetch(current, {method, body, signal, redirect:'manual',
            headers:{'Accept-Encoding':'identity', ...forwarded}});
          if ([301,302,303,307,308].includes(response.status)) {
            await response.body?.cancel();
            assert(redirects < 4 && method === 'GET', 'OCI redirect budget or method violation');
            const location = response.headers.get('location'); assert(location && location.length <= 4096, 'bounded OCI redirect location required');
            const next = secureUrl(new URL(location, current).href, registry.origin);
            if (next.origin !== current.origin) delete forwarded.Authorization;
            current = next; continue;
          }
          if (response.status !== 200) {
            const auth = response.headers.get('www-authenticate');
            await response.body?.cancel();
            return {status:response.status, auth, origin:current.origin};
          }
          try {
            assert([null,'identity'].includes(response.headers.get('content-encoding')), 'HTTP content coding refused for exact OCI bytes');
            const length = response.headers.get('content-length');
            if (length !== null) assert(/^[0-9]+$/.test(length) && Number(length) <= bound, 'OCI response length exceeds bound');
            const parts = []; let size = 0;
            for await (const chunk of response.body ?? []) {
              size += chunk.length; assert(size <= bound, 'OCI streamed response exceeds bound'); parts.push(Buffer.from(chunk));
            }
            if (length !== null) assert.equal(size, Number(length), 'OCI response length differs');
            return {status:200, bytes:Buffer.concat(parts), origin:current.origin};
          } catch (error) {
            controller.abort(); throw error;
          }
        }
      };
      const url = registry.origin + '/v2/' + registry.path + '/' + (kind === 'manifest' ? 'manifests/' : 'blobs/') + target.digest;
      const headers = {Accept:kind === 'manifest'
        ? 'application/vnd.oci.image.index.v1+json, application/vnd.oci.image.manifest.v1+json'
        : 'application/octet-stream'};
      let response = await read(url, maximum, headers);
      if (response.status === 401) {
        assert.equal(response.origin, registry.origin, 'redirect target cannot request registry credentials');
        if (typeof response.auth === 'string' && /^Basic /i.test(response.auth)) {
          assert(response.auth.length <= 4096 && /^Basic realm="[^"\\\r\n]*"(?:,\s*charset="UTF-8")?$/i.test(response.auth),
            'bounded unambiguous OCI basic challenge required');
          response = await read(url, maximum, {...headers, Authorization:basic(await resolveCredential())});
          assert.equal(response.status, 200, 'OCI basic authentication failed');
          return response.bytes;
        }
        const facts = challenge(response.auth);
        const scope = 'repository:' + registry.path + ':pull';
        assert(facts.scope === undefined || facts.scope === scope, 'OCI authentication scope differs');
        const realm = secureUrl(facts.realm, registry.origin);
        for (const key of realm.searchParams.keys()) assert(!['scope','service','grant_type','refresh_token','client_id',
          'client_secret','username','password','access_token','token','account'].includes(key.toLowerCase()),
        'reserved OCI authentication query parameter');
        assert(facts.service === undefined || facts.service.length <= 256, 'OCI authentication service exceeds bound');
        // Credentials stay in SDK custody, never command-line arguments.
        const credential = await resolveCredential();
        if (credential?.kind === 'bearer') {
          assert(typeof credential.secret === 'string' && credential.secret.length > 0 && credential.secret.length <= 16384
            && /^[A-Za-z0-9._~+/-]+=*$/.test(credential.secret), 'bounded OCI bearer credential required');
          response = await read(url, maximum, {...headers, Authorization:'Bearer ' + credential.secret});
          assert.equal(response.status, 200, 'OCI bearer authentication failed');
          return response.bytes;
        }
        const tokenHeaders = {Accept:'application/json'};
        const params = new URLSearchParams({scope});
        if (facts.service !== undefined) params.set('service', facts.service);
        let method = 'GET', body;
        if (credential !== null) {
          assert(credential && ['basic','refresh'].includes(credential.kind), 'unsupported OCI credential kind');
          assert(typeof credential.secret === 'string' && credential.secret.length > 0 && credential.secret.length <= 16384,
            'bounded OCI credential required');
          assert(realm.origin === registry.origin || (registry.origin === 'https://registry-1.docker.io'
            && realm.origin === 'https://auth.docker.io'), 'registry credentials cannot cross an unapproved origin');
          if (credential.kind === 'basic') {
            tokenHeaders.Authorization = basic(credential);
          } else {
            method = 'POST'; params.set('grant_type','refresh_token'); params.set('refresh_token',credential.secret);
            tokenHeaders['Content-Type'] = 'application/x-www-form-urlencoded'; body = params.toString();
          }
        }
        if (method === 'GET') for (const [key, value] of params) realm.searchParams.set(key, value);
        const tokenResponse = await read(realm.href, 64 * 1024, tokenHeaders, method, body);
        assert.equal(tokenResponse.status, 200, 'OCI token acquisition failed');
        let token;
        try {token = parseJson(tokenResponse.bytes);} catch {throw new Error('invalid OCI token response');}
        assert(token && typeof token === 'object' && !Array.isArray(token), 'OCI token response must be an object');
        if (token.token !== undefined && token.access_token !== undefined) assert(token.token === token.access_token, 'ambiguous OCI token fields');
        const bearer = token.token ?? token.access_token;
        assert(typeof bearer === 'string' && bearer.length > 0 && bearer.length <= 16384
          && /^[A-Za-z0-9._~+/-]+=*$/.test(bearer), 'bounded OCI bearer token required');
        response = await read(url, maximum, {...headers, Authorization:'Bearer ' + bearer});
      }
      assert.equal(response.status, 200, 'OCI blob acquisition failed');
      return response.bytes;
    } finally {clearTimeout(timer); controller.abort();}
  };
}
