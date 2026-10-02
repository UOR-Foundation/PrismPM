import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import test from 'node:test';
import {createRegistryTransport, registryReference} from './metadata-transport.mjs';
import {sha} from './metadata-layer.mjs';

async function server(t, handler) {
  const requests = [];
  const instance = createServer((request, response) => {requests.push(request.url); handler(request, response);});
  await new Promise(resolve => instance.listen(0, '127.0.0.1', resolve));
  t.after(async () => {instance.closeAllConnections(); await new Promise(resolve => instance.close(resolve));});
  return {origin:'http://127.0.0.1:' + instance.address().port, authority:'127.0.0.1:' + instance.address().port, requests};
}
const request = (reference, extra = {}) => ({kind:'blob', reference, maximum:1024, timeout_ms:1000, ...extra});

test('real HTTP returns exact binary bytes and requests identity encoding', async t => {
  const bytes = Buffer.from([0,255,10,128,0,1]);
  const host = await server(t, (req, res) => {
    assert.equal(req.headers['accept-encoding'], 'identity');
    assert.equal(req.headers.authorization, undefined);
    res.writeHead(200, {'content-length':bytes.length}); res.end(bytes);
  });
  const reference = host.authority + '/sdk@' + sha(bytes);
  assert.deepEqual(await createRegistryTransport(reference)(request(reference)), bytes);
  assert.equal(host.requests.length, 1);
});

test('actual streamed overflow, excessive declared size, coding and truncation fail closed', async t => {
  for (const mode of ['stream','length','coding','truncated']) {
    const host = await server(t, (_, res) => {
      if (mode === 'length') {res.writeHead(200, {'content-length':'2048'}); res.end('short');}
      if (mode === 'stream') {res.writeHead(200); res.write(Buffer.alloc(700)); res.end(Buffer.alloc(700));}
      if (mode === 'coding') {res.writeHead(200, {'content-encoding':'deflate'}); res.end('not identity');}
      if (mode === 'truncated') {res.writeHead(200, {'content-length':'100'}); res.end('short');}
    });
    const reference = host.authority + '/sdk@' + sha(mode);
    await assert.rejects(createRegistryTransport(reference)(request(reference, {timeout_ms:150})));
    assert.equal(host.requests.length, 1, 'failed requests must not be retried');
  }
});

test('one real request deadline covers a stalled response body and slow headers', async t => {
  for (const mode of ['body','headers']) {
    const host = await server(t, (_, res) => {if (mode === 'body') {res.writeHead(200); res.write('partial');}});
    const reference = host.authority + '/sdk@' + sha(mode), start = performance.now();
    await assert.rejects(createRegistryTransport(reference)(request(reference, {timeout_ms:75})));
    assert(performance.now() - start < 1000, 'deadline must interrupt acquisition');
  }
});

test('redirect count is finite and insecure cross-origin redirects are not followed', async t => {
  const loop = await server(t, (_, res) => {res.writeHead(307, {location:'/again'}); res.end();});
  const reference = loop.authority + '/sdk@' + sha('loop');
  await assert.rejects(createRegistryTransport(reference)(request(reference)), /redirect budget/);
  assert.equal(loop.requests.length, 5);
  const target = await server(t, (_, res) => {res.end('must not be requested');});
  const source = await server(t, (_, res) => {res.writeHead(302, {location:target.origin + '/outside'}); res.end();});
  const ref = source.authority + '/sdk@' + sha('outside');
  await assert.rejects(createRegistryTransport(ref)(request(ref)), /insecure redirect/);
  assert.equal(target.requests.length, 0);
});

test('actual anonymous, basic and refresh token exchanges retain exact pull scope', async t => {
  for (const mode of ['anonymous','basic','refresh']) {
    let host;
    host = await server(t, (req, res) => {
      if (req.url.startsWith('/token')) {
        if (mode === 'refresh') {
          assert.equal(req.method, 'POST'); let body = '';
          req.on('data', bytes => {body += bytes;}); req.on('end', () => {
            const parameters = new URLSearchParams(body);
            assert.equal(parameters.get('scope'), 'repository:sdk:pull');
            assert.equal(parameters.get('grant_type'), 'refresh_token'); assert.equal(parameters.get('refresh_token'), 'synthetic-refresh');
            res.end(JSON.stringify({access_token:'synthetic-access'}));
          });
          return;
        }
        const url = new URL(req.url, host.origin);
        assert.equal(url.searchParams.get('scope'), 'repository:sdk:pull');
        assert.equal(url.searchParams.get('service'), 'fixture-registry');
        assert.equal(req.headers.authorization, mode === 'basic' ? 'Basic ' + Buffer.from('fixture:synthetic-secret').toString('base64') : undefined);
        res.end(JSON.stringify({token:'synthetic-access'})); return;
      }
      if (req.headers.authorization === 'Bearer synthetic-access') {res.end('exact artifact'); return;}
      res.writeHead(401, {'www-authenticate':'Bearer realm="' + host.origin + '/token",service="fixture-registry",scope="repository:sdk:pull"'}); res.end();
    });
    const reference = host.authority + '/sdk@' + sha('exact artifact');
    let resolutions = 0;
    const transport = createRegistryTransport(reference, async (origin, signal) => {
      assert.equal(origin, host.origin); assert(!signal.aborted); resolutions++;
      return mode === 'anonymous' ? null : mode === 'basic' ? {kind:'basic', username:'fixture', secret:'synthetic-secret'}
        : {kind:'refresh', secret:'synthetic-refresh'};
    });
    assert.equal((await transport(request(reference))).toString(), 'exact artifact');
    assert.equal(resolutions, 1); assert.equal(host.requests.length, 3);
  }
});

test('direct Basic authentication performs one scoped retry without exposing credentials in the URL', async t => {
  const authorization = 'Basic ' + Buffer.from('fixture:synthetic-password').toString('base64');
  const host = await server(t, (req, res) => {
    assert(!req.url.includes('synthetic'));
    if (req.headers.authorization === authorization) res.end('private artifact');
    else {res.writeHead(401, {'www-authenticate':'Basic realm="fixture",charset="UTF-8"'}); res.end();}
  });
  const reference = host.authority + '/sdk@' + sha('private artifact');
  const transport = createRegistryTransport(reference, async () => ({kind:'basic',username:'fixture',secret:'synthetic-password'}));
  assert.equal((await transport(request(reference))).toString(), 'private artifact');
  assert.equal(host.requests.length, 2);
});

test('pre-scoped registry tokens are sent only to the registry and never to its token realm', async t => {
  let host;
  host = await server(t, (req, res) => {
    assert(!req.url.startsWith('/token'));
    if (req.headers.authorization === 'Bearer synthetic-access') res.end('private artifact');
    else {res.writeHead(401, {'www-authenticate':'Bearer realm="' + host.origin + '/token"'}); res.end();}
  });
  const reference = host.authority + '/sdk@' + sha('private artifact');
  const transport = createRegistryTransport(reference, async () => ({kind:'bearer', secret:'synthetic-access'}));
  assert.equal((await transport(request(reference))).toString(), 'private artifact');
  assert.equal(host.requests.length, 2);
});

test('ambiguous challenges, expanded scope, oversized token responses and repeated auth are refused', async t => {
  for (const mode of ['duplicate','scope','token-size','token-conflict','auth-repeat']) {
    let host;
    host = await server(t, (req, res) => {
      if (req.url.startsWith('/token')) {
        if (mode === 'token-size') res.end(Buffer.alloc(65537, 32));
        else if (mode === 'token-conflict') res.end(JSON.stringify({token:'one', access_token:'two'}));
        else res.end(JSON.stringify({token:'one'}));
        return;
      }
      let value = 'Bearer realm="' + host.origin + '/token"';
      if (mode === 'duplicate') value += ',realm="' + host.origin + '/other"';
      if (mode === 'scope') value += ',scope="repository:sdk:push,pull"';
      res.writeHead(401, {'www-authenticate':value}); res.end();
    });
    const reference = host.authority + '/sdk@' + sha(mode);
    await assert.rejects(createRegistryTransport(reference)(request(reference)));
    assert.equal(host.requests.length, mode === 'auth-repeat' ? 3 : mode.startsWith('token') ? 2 : 1);
  }
});

test('credential resolution shares the deadline and can never leave the request waiting indefinitely', async t => {
  let host;
  host = await server(t, (_, res) => {res.writeHead(401, {'www-authenticate':'Bearer realm="' + host.origin + '/token"'}); res.end();});
  const reference = host.authority + '/sdk@' + sha('deadline'); let signal;
  await assert.rejects(createRegistryTransport(reference, async (_, selected) => {signal = selected; await new Promise(() => {});})(
    request(reference, {timeout_ms:75})), /credential deadline/);
  assert(signal.aborted); assert.equal(host.requests.length, 1);
});

test('reserved realm query fields cannot override POST token scope or receive any credentials', async t => {
  for (const name of ['scope','service','grant_type','refresh_token','SCOPE']) {
    let host, resolutions = 0;
    host = await server(t, (_, res) => {res.writeHead(401, {'www-authenticate':'Bearer realm="' + host.origin + '/token?' + name + '=conflict"'}); res.end();});
    const reference = host.authority + '/sdk@' + sha('conflict');
    await assert.rejects(createRegistryTransport(reference, async () => {resolutions++; return {kind:'refresh',secret:'synthetic-refresh'};})(
      request(reference)), /reserved OCI authentication query/);
    assert.equal(resolutions, 0); assert.equal(host.requests.length, 1);
  }
});

test('malformed or contradictory token responses do not expose token bytes in diagnostics', async t => {
  for (const body of ['{"token":"synthetic-secret-token",BROKEN}',
    '{"token":"synthetic-secret-token","access_token":"another-secret-token"}']) {
    let host;
    host = await server(t, (req, res) => {
      if (req.url.startsWith('/token')) res.end(body);
      else {res.writeHead(401, {'www-authenticate':'Bearer realm="' + host.origin + '/token"'}); res.end();}
    });
    const reference = host.authority + '/sdk@' + sha('diagnostics');
    await assert.rejects(createRegistryTransport(reference)(request(reference)), error => {
      assert(!String(error.stack).includes('secret-token')); assert(!JSON.stringify(error).includes('secret-token')); return true;
    });
  }
});

test('default registry ports normalize to the same origin without changing pinned references', () => {
  const digest = sha('reference');
  for (const [a, b] of [['example.com:443','example.com'],['localhost:80','localhost'],['127.0.0.1:80','127.0.0.1']]) {
    const first = registryReference(a + '/sdk@' + digest), second = registryReference(b + '/sdk@' + digest);
    assert.equal(first.origin, second.origin); assert.equal(first.repository, a + '/sdk');
  }
});

test('invalid repositories, ports, cross-repository requests and caller bounds are refused', async () => {
  for (const repository of ['example.invalid/a/../b','example.invalid/a//b','example.invalid:0/a','example.invalid:65536/a'])
    assert.throws(() => createRegistryTransport(repository + '@' + sha('test')));
  const reference = 'example.invalid/sdk@' + sha('test'), transport = createRegistryTransport(reference);
  for (const extra of [{maximum:Infinity},{maximum:0},{timeout_ms:45001},{kind:'pull'},
    {reference:'example.invalid/other@' + sha('test')}]) await assert.rejects(transport(request(reference, extra)));
});
