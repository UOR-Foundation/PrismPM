import assert from 'node:assert/strict';
import {execFile, execFileSync} from 'node:child_process';
import {mkdtempSync, readFileSync, rmSync} from 'node:fs';
import {createServer} from 'node:https';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {promisify} from 'node:util';
import test from 'node:test';
import {sha} from './metadata-layer.mjs';

async function fixture(t) {
  const directory = mkdtempSync(join(tmpdir(), 'metadata-tls-'));
  t.after(() => rmSync(directory, {recursive:true, force:true}));
  const key = join(directory,'key.pem'), cert = join(directory,'cert.pem');
  execFileSync('/usr/bin/openssl', ['req','-x509','-newkey','rsa:2048','-nodes','-days','1',
    '-subj','/CN=SDK transport test only','-addext','subjectAltName=IP:127.0.0.2,IP:127.0.0.3',
    '-keyout',key,'-out',cert], {timeout:15000,maxBuffer:65536,stdio:['ignore','pipe','pipe']});
  const tls = {key:readFileSync(key),cert:readFileSync(cert)};
  const server = async (host, handler) => {
    const instance = createServer(tls, handler);
    await new Promise(resolve => instance.listen(0, host, resolve));
    t.after(async () => {instance.closeAllConnections(); await new Promise(resolve => instance.close(resolve));});
    return host + ':' + instance.address().port;
  };
  const run = async (authority, {trusted = true, insecure = false} = {}) => {
    const reference = authority + '/sdk@' + sha('TLS artifact');
    const program = `
      const {createRegistryTransport} = await import(${JSON.stringify(new URL('./metadata-transport.mjs',import.meta.url).href)});
      const reference = ${JSON.stringify(reference)};
      const transport = createRegistryTransport(reference, async () => ({kind:'bearer',secret:'synthetic-access'}));
      process.stdout.write(await transport({kind:'blob',reference,maximum:1024,timeout_ms:3000}));`;
    const env = {...process.env}; delete env.NODE_EXTRA_CA_CERTS; delete env.NODE_TLS_REJECT_UNAUTHORIZED;
    if (trusted) env.NODE_EXTRA_CA_CERTS = cert;
    if (insecure) env.NODE_TLS_REJECT_UNAUTHORIZED = '0';
    return promisify(execFile)(process.execPath, ['--input-type=module','-e',program], {env,timeout:5000,maxBuffer:65536});
  };
  return {server,run};
}

test('real TLS validates server identity and strips registry credentials across redirect origins', async t => {
  const {server,run} = await fixture(t);
  let targetCalls = 0, sourceCalls = 0, authority;
  const target = await server('127.0.0.3', (req,res) => {
    targetCalls++; assert.equal(req.headers.authorization, undefined); res.end('TLS artifact');
  });
  authority = await server('127.0.0.2', (req,res) => {
    sourceCalls++;
    if (req.headers.authorization === undefined) {
      res.writeHead(401, {'www-authenticate':'Bearer realm="https://' + authority + '/token"'}); res.end();
    } else {
      assert.equal(req.headers.authorization, 'Bearer synthetic-access');
      res.writeHead(307, {location:'https://' + target + '/artifact'}); res.end();
    }
  });
  assert.equal((await run(authority)).stdout, 'TLS artifact');
  assert.equal(sourceCalls,2); assert.equal(targetCalls,1);
  await assert.rejects(run(authority,{trusted:false}));
  assert.equal(sourceCalls,2, 'untrusted TLS must fail before HTTP or credential exchange');
  await assert.rejects(run(authority,{insecure:true}), /disabled TLS verification refused/);
  assert.equal(sourceCalls,2, 'TLS bypass must fail before network I/O');
});

test('real TLS rejects insecure redirects and redirect-origin authentication demands', async t => {
  const {server,run} = await fixture(t);
  for (const mode of ['downgrade','foreign-auth']) {
    let targetCalls = 0;
    const target = await server('127.0.0.3', (req,res) => {
      targetCalls++; assert.equal(req.headers.authorization, undefined);
      res.writeHead(401, {'www-authenticate':'Basic realm="must not receive registry credentials"'}); res.end();
    });
    const authority = await server('127.0.0.2', (_,res) => {
      res.writeHead(307, {location:(mode === 'downgrade' ? 'http://' : 'https://') + target + '/artifact'}); res.end();
    });
    await assert.rejects(run(authority), mode === 'downgrade' ? /insecure redirect/ : /redirect target cannot request/);
    assert.equal(targetCalls,mode === 'downgrade' ? 0 : 1);
  }
});
