import assert from 'node:assert/strict';
import {mkdtempSync, writeFileSync, rmSync, symlinkSync, linkSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {dockerCredentialProvider} from './metadata-credentials.mjs';

function fixture(t, config) {
  const directory = mkdtempSync(join(tmpdir(), 'metadata-credentials-'));
  t.after(() => rmSync(directory, {recursive:true, force:true}));
  if (config !== undefined) writeFileSync(join(directory, 'config.json'), typeof config === 'string' ? config : JSON.stringify(config), {mode:0o600});
  return directory;
}
const signal = () => new AbortController().signal;
const origin = 'https://ghcr.io';
const auth = (username, secret) => Buffer.from(username + ':' + secret).toString('base64');

test('read-only credentials resolve exact registry aliases and preserve credential kinds', async t => {
  for (const key of ['ghcr.io',origin,origin + '/']) {
    for (const [entry, expected] of [
      [{auth:auth('fixture','synthetic:password')}, {kind:'basic', username:'fixture', secret:'synthetic:password'}],
      [{identitytoken:'synthetic-refresh'}, {kind:'refresh', secret:'synthetic-refresh'}],
      [{registrytoken:'synthetic-access'}, {kind:'bearer', secret:'synthetic-access'}],
    ]) {
      const directory = fixture(t, {auths:{[key]:entry}});
      assert.deepEqual(await dockerCredentialProvider({directory})(origin, signal()), expected);
    }
  }
  for (const config of [undefined, {}, {auths:{'other.example':{auth:auth('other','synthetic-secret')}}}]) {
    assert.equal(await dockerCredentialProvider({directory:fixture(t, config)})(origin, signal()), null);
  }
});

test('ambiguous, malformed, noncanonical and oversized credentials fail without exposing secrets', async t => {
  const secret = 'synthetic-secret-not-for-diagnostics';
  const configs = [
    '{"auths":{"ghcr.io":{"auth":"' + secret + '"},"ghcr.io":{}}}',
    '{"auths":"' + secret,
    {auths:{'ghcr.io':{auth:auth('fixture',secret)}, [origin]:{identitytoken:secret}}},
    {auths:{'ghcr.io':{auth:auth('fixture',secret), identitytoken:secret}}},
    {auths:{'ghcr.io':{registrytoken:secret, identitytoken:secret}}},
    {auths:{'ghcr.io':{auth:'YR=='}}},
    {auths:{'ghcr.io':{auth:auth('',secret)}}},
    {auths:{'ghcr.io':{auth:auth('fixture\n',secret)}}},
    {auths:{'ghcr.io':{registrytoken:secret + '\r\n'}}},
    {auths:{'ghcr.io':{identitytoken:secret.repeat(1024)}}},
    {Auths:{'ghcr.io':{auth:auth('fixture',secret)}}},
    ' '.repeat(1024 * 1024 + 1),
  ];
  for (const config of configs) await assert.rejects(
    dockerCredentialProvider({directory:fixture(t, config)})(origin, signal()),
    error => !String(error.stack).includes(secret));
});

test('credential files reject symlinks and hard links and never fall back to an unrelated registry', async t => {
  for (const link of [symlinkSync, linkSync]) {
    const directory = fixture(t);
    const target = join(directory, 'other.json'); writeFileSync(target, '{}');
    link(target, join(directory, 'config.json'));
    await assert.rejects(dockerCredentialProvider({directory})(origin, signal()), /singly linked/);
  }
});

test('per-registry helpers override global stores and require verified execution', async t => {
  const directory = fixture(t, {credsStore:'global', credHelpers:{'ghcr.io':'scoped'}, auths:{'ghcr.io':{auth:auth('ignored','synthetic')}}});
  let calls = 0;
  const provider = dockerCredentialProvider({directory, runHelper:async (command, server, selected) => {
    calls++; assert.equal(command, 'docker-credential-scoped'); assert.equal(server, 'ghcr.io'); assert(!selected.aborted);
    return {Username:'<token>', Secret:'synthetic-refresh'};
  }});
  assert.deepEqual(await provider(origin, signal()), {kind:'refresh', secret:'synthetic-refresh'});
  assert.equal(calls, 1);
  await assert.rejects(dockerCredentialProvider({directory})(origin, signal()), /verified credential helper/);
  await assert.rejects(dockerCredentialProvider({directory:fixture(t, {credsStore:'../unsafe'})})(origin, signal()), /invalid registry credential helper/);
  assert.equal(await dockerCredentialProvider({directory, runHelper:async () => null})(origin, signal()), null);
  await assert.rejects(dockerCredentialProvider({directory, runHelper:async () => ({Username:'fixture', Secret:'synthetic', Other:1})})(origin, signal()), /closed registry helper/);
  for (const result of [{Username:'bad:name',Secret:'synthetic'}, {Username:'<token>',Secret:''}])
    await assert.rejects(dockerCredentialProvider({directory, runHelper:async () => result})(origin, signal()));
  const controller = new AbortController(); controller.abort();
  await assert.rejects(provider(origin, controller.signal)); assert.equal(calls, 1);
});

test('Docker Hub legacy auth configuration is recognized without other-registry fallback', async t => {
  const directory = fixture(t, {auths:{'https://index.docker.io/v1/':{auth:auth('fixture','synthetic')}}});
  const provider = dockerCredentialProvider({directory});
  assert.deepEqual(await provider('https://registry-1.docker.io', signal()), {kind:'basic', username:'fixture', secret:'synthetic'});
  assert.equal(await provider(origin, signal()), null);
  const helper = dockerCredentialProvider({directory:fixture(t, {credsStore:'fixture'}), runHelper:async (command, server) => {
    assert.equal(command, 'docker-credential-fixture');
    return server === 'https://index.docker.io/v1/' ? {Username:'fixture', Secret:'synthetic'} : null;
  }});
  for (const host of ['docker.io','registry-1.docker.io'])
    assert.deepEqual(await helper('https://' + host, signal()), {kind:'basic', username:'fixture', secret:'synthetic'});
});
