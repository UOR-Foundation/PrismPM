// Real Docker/OCI transport regression using explicitly synthetic test images.
// Every recorded fixture digest hashes a file actually copied into its image;
// these images are NOT SDK releases and no tool execution claim is made.
import assert from 'node:assert/strict';
import {execFile} from 'node:child_process';
import {createHash, randomUUID} from 'node:crypto';
import {mkdtemp, mkdir, readFile, rm, writeFile} from 'node:fs/promises';
import {createServer, connect} from 'node:net';
import {tmpdir} from 'node:os';
import {join, resolve} from 'node:path';
import {promisify} from 'node:util';

const exec = promisify(execFile);
const run = async (program, args) => (await exec(program, args, {timeout: 180_000, maxBuffer: 16 * 1024 * 1024})).stdout;
const docker = async (...args) => run('docker', args);
const sha = bytes => `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
const canonical = value => Array.isArray(value) ? value.map(canonical) : value && typeof value === 'object'
  ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
const encode = value => `${JSON.stringify(canonical(value))}\n`;
const cli = resolve(process.argv[2] ?? 'target/debug/prismpm');
const directory = await mkdtemp(join(tmpdir(), 'prismpm-sdk-update-oci-'));
const nonce = randomUUID();
const registryName = `prismpm-sdk-update-test-${nonce}`;
const configVolume = `${registryName}-config`;
const registryImage = 'ghcr.io/project-zot/zot@sha256:cd2aea942f428630bcb4190542be6abd35e14177aab84fc7ccad0dca8ecb363d';
const imageReferences = new Set();
let registry, proxy, volumeCreated = false;
const connections = new Set();
try {
  await writeFile(join(directory, 'zot.json'), JSON.stringify({distSpecVersion: '1.1.1',
    http: {address: '0.0.0.0', port: 5000, compat: ['docker2s2']},
    log: {level: 'error'}, storage: {rootDirectory: '/tmp/zot'}}));
  await docker('volume', 'create', configVolume); volumeCreated = true;
  registry = (await docker('create', '--name', registryName, '--publish', '127.0.0.1::5000',
    '--read-only', '--tmpfs', '/tmp:rw,nosuid,nodev', '--volume', `${configVolume}:/config`, registryImage, 'serve', '/config/sdk-update-test.json')).trim();
  assert.match(registry, /^[0-9a-f]{64}$/);
  await docker('cp', join(directory, 'zot.json'), `${registry}:/config/sdk-update-test.json`);
  await docker('start', registry);
  const endpoint = (await docker('port', registry, '5000/tcp')).trim();
  assert.match(endpoint, /^127\.0\.0\.1:[0-9]+$/);
  // Docker uses the host loopback endpoint; inside a bridged devcontainer,
  // temporarily forward the same loopback port to this owned registry only.
  if (process.env.REMOTE_CONTAINERS === 'true' || await readFile('/.dockerenv').then(() => true, () => false)) {
    const address = (await docker('inspect', registry, '--format', '{{(index .NetworkSettings.Networks "bridge").IPAddress}}')).trim();
    assert.match(address, /^[0-9.]+$/);
    proxy = createServer(client => {
      const remote = connect(5000, address);
      for (const socket of [client, remote]) { connections.add(socket); socket.on('close', () => connections.delete(socket)); }
      client.on('error', () => remote.destroy()); remote.on('error', () => client.destroy());
      client.pipe(remote); remote.pipe(client);
    });
    await new Promise((resolveListen, reject) => { proxy.once('error', reject); proxy.listen(Number(endpoint.split(':')[1]), '127.0.0.1', resolveListen); });
  }
  let ready = false;
  for (let attempt = 0; attempt < 30; attempt++) {
    try { const response = await fetch(`http://${endpoint}/v2/`); if (response.ok) { ready = true; break; } } catch (_) {}
    await new Promise(resolveWait => setTimeout(resolveWait, 100));
  }
  assert.ok(ready, 'owned test registry did not start');
  const captures = [];
  for (const generation of ['old', 'new']) {
    const standards = Buffer.from(`Synthetic standards-lock transport fixture ${generation}\n`);
    const tags = [];
    for (const architecture of ['amd64', 'arm64']) {
      const context = join(directory, generation, architecture);
      await mkdir(join(context, 'facts'), {recursive: true});
      const artifacts = [];
      for (const kind of ['adapter', 'base-image', 'binary', 'crate', 'dependency-lock', 'oracle', 'schema', 'test-corpus', 'trust-root', 'workflow']) {
        const content = Buffer.from(`Synthetic ${generation}/${architecture}/${kind} bytes\n`);
        const id = `test-${kind}`;
        await writeFile(join(context, 'facts', id), content);
        artifacts.push({id, kind, version: 'test-fixture', digest: sha(content)});
      }
      const commands = [];
      for (const command of ['cargo', 'devcontainer', 'docker', 'just', 'prismpm']) {
        const content = Buffer.from(`Synthetic nonexecuted ${generation}/${architecture}/${command}\n`);
        await writeFile(join(context, 'facts', command), content);
        commands.push({command, executable: `/test-facts/${command}`, sha256: sha(content).slice(7)});
      }
      await writeFile(join(context, 'inventory.json'), encode({schema: 'prismpm/sdk-inventory/1', artifacts, commands}));
      await writeFile(join(context, 'standards.lock'), standards);
      await writeFile(join(context, 'Dockerfile'), 'FROM scratch\nCOPY facts /test-facts\nCOPY inventory.json standards.lock /opt/prismpm/share/\nCMD ["/never-executed-test-fixture"]\n');
      const tag = `${endpoint}/test-sdk-${nonce}:${generation}-${architecture}`;
      imageReferences.add(tag); tags.push(tag);
      // This transport fixture has no release-attestation claims; the locked
      // profile requires exactly its two platform manifests, not extra indexes.
      await docker('build', '--provenance=false', '--platform', `linux/${architecture}`,
        '--output', 'type=image,oci-mediatypes=true', '--tag', tag, context);
      await docker('push', tag);
    }
    const indexTag = `${endpoint}/test-sdk-${nonce}:${generation}`;
    await docker('buildx', 'imagetools', 'create', '--tag', indexTag, ...tags);
    const index = await docker('buildx', 'imagetools', 'inspect', '--raw', indexTag);
    const reference = `${endpoint}/test-sdk-${nonce}@${sha(index)}`;
    const lock = JSON.parse(await run(process.execPath, ['sdk/platform-lock.mjs', 'capture', reference, sha(standards), '/usr/local/bin/docker']));
    assert.equal(lock.sdk_index, index);
    assert.equal(lock.standards_lock, sha(standards));
    for (const platform of lock.platforms) {
      const architecture = platform.platform.split('/')[1];
      const expected = await readFile(join(directory, generation, architecture, 'inventory.json'));
      assert.equal(platform.inventory_digest, sha(expected));
      assert.equal(platform.inventory_document, expected.toString('utf8'));
      assert.deepEqual(platform.inventory, JSON.parse(expected).artifacts);
      imageReferences.add(`${endpoint}/test-sdk-${nonce}@${platform.manifest_digest}`);
    }
    captures.push(lock);
  }
  const project = join(directory, 'project'); await mkdir(project);
  const committed = JSON.stringify(canonical(captures[0])); await writeFile(join(project, 'prismpm.lock'), committed);
  const args = ['--json', '--project', project, 'lock', 'update', '--sdk-image', captures[1].sdk_image,
    '--standards-lock', captures[1].standards_lock];
  const proposal = JSON.parse(await run(cli, args));
  assert.equal(proposal.schema, 'prismpm/sdk-lock-update/2');
  assert.deepEqual(proposal.proposed_lock, captures[1]);
  assert.equal(proposal.changes.length, 4);
  assert.equal(await readFile(join(project, 'prismpm.lock'), 'utf8'), committed);
  args[args.length - 1] = sha('wrong requested standards');
  await assert.rejects(run(cli, args), error => error.stderr.includes('PP5401') || error.stdout.includes('PP5401'));
  assert.equal(await readFile(join(project, 'prismpm.lock'), 'utf8'), committed);
  console.log('PASS actual Docker/OCI two-generation, two-architecture capture; current CLI review proposal; standards mismatch rejection; no lock adoption');
} finally {
  for (const socket of connections) socket.destroy();
  if (proxy) await new Promise(resolveClose => proxy.close(resolveClose));
  if (registry) await docker('rm', '--force', registry);
  if (volumeCreated) await docker('volume', 'rm', configVolume);
  for (const reference of imageReferences) {
    try { await docker('image', 'rm', reference); } catch (error) {
      // Removing the final tag may already remove its digest reference.
      if (!/no such image/i.test(error.stderr ?? '')) throw error;
    }
  }
  await rm(directory, {recursive: true, force: true});
}
