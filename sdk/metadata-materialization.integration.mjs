// Real Docker materialization, not SDK release/native-execution acceptance.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {randomUUID} from 'node:crypto';
import {mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, utimesSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, resolve} from 'node:path';
import {decodeMetadataLayer, label, limits, parseConfig, parseManifest, paths, profile, sha} from './metadata-layer.mjs';
import {ociFixtureManifest} from './oci-test-fixture.mjs';

const work = mkdtempSync(join(tmpdir(), 'prismpm-metadata-materialization-'));
const nonce = process.argv[2] ?? randomUUID(), tags = new Set(), containers = new Set();
assert.match(nonce,/^[0-9a-f]{8}-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}$/);
const failAfter = process.argv[3];
assert(failAfter === undefined || ['build','create'].includes(failAfter));
const ownership = 'org.prismpm.test.owner';
const media = 'application/vnd.oci.image.';
const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, {timeout:60000, maxBuffer:4 * 1024 * 1024, ...options});
  assert.ifError(result.error); assert.equal(result.status, 0, command + ': ' + result.stderr.toString());
  return result.stdout;
};
const docker = (...args) => run('/usr/local/bin/docker', args);
const describe = (bytes, mediaType) => ({mediaType, size:bytes.length, digest:sha(bytes)});
const safeArchivePath = path => assert(typeof path === 'string' && /^[A-Za-z0-9_./-]+$/.test(path)
  && !path.startsWith('/') && !path.split('/').some(part => !part || part === '.' || part === '..'), 'unsafe Docker archive member');
const inspectOwned = (kind, name) => {
  const result = spawnSync('/usr/local/bin/docker', [kind,'inspect','--format','{{json .}}',name], {timeout:15000, maxBuffer:1024 * 1024});
  assert.ifError(result.error);
  if (result.status !== 0 && /No such (?:image|container|object):/i.test(result.stderr.toString())) return null;
  assert.equal(result.status, 0, 'cannot reconcile owned ' + kind + ': ' + result.stderr.toString());
  const value = JSON.parse(result.stdout);
  assert.equal(value.Config.Labels[ownership], nonce, 'refusing cleanup of unowned resource');
  if (kind === 'container') assert.equal(value.Name, '/' + name, 'container identity changed');
  return value;
};
let failure;
try {
  for (const architecture of ['amd64', 'arm64']) for (const hostile of ['opt-link','opt-file','prismpm-link','share-link']) {
    const context = join(work, architecture + '-' + hostile);
    mkdirSync(context); const base = join(context, 'base'), metadata = join(context, 'metadata');
    mkdirSync(base); mkdirSync(metadata);
    if (hostile.startsWith('opt-')) {
      if (hostile === 'opt-link') symlinkSync('/outside', join(base, 'opt'));
      else writeFileSync(join(base, 'opt'), 'prior non-directory');
    } else {
      mkdirSync(join(base, 'opt'));
      if (hostile === 'prismpm-link') symlinkSync('/outside', join(base, 'opt/prismpm'));
      else {mkdirSync(join(base, 'opt/prismpm')); symlinkSync('/outside', join(base, 'opt/prismpm/share'));}
    }
    for (const dir of ['opt','opt/prismpm','opt/prismpm/share']) {
      mkdirSync(join(metadata, dir), {mode:0o755}); utimesSync(join(metadata, dir), 0, 0);
    }
    const inventory = Buffer.from(JSON.stringify({scope:'synthetic-materialization-fixture', architecture, hostile}) + '\n');
    const standards = Buffer.from('synthetic standards, not an accepted SDK\n');
    for (const [name, bytes] of [[paths.inventory, inventory],[paths.standards, standards]]) {
      writeFileSync(join(metadata, name), bytes, {mode:0o444}); utimesSync(join(metadata, name), 0, 0);
    }
    // Creating files changes directory mtimes: freeze after all writes.
    for (const dir of ['opt','opt/prismpm','opt/prismpm/share']) utimesSync(join(metadata, dir), 0, 0);
    writeFileSync(join(context, 'Dockerfile'), 'FROM scratch AS base\nCOPY base/ /\nFROM scratch AS metadata\nCOPY metadata/ /\n' +
      'FROM base AS final\nCOPY --link --from=metadata / /\nLABEL ' + label + '="' + profile + '"\nCMD ["/never-executed"]\n');
    const tag = 'prismpm-metadata-test-' + nonce + ':' + architecture + '-' + hostile;
    tags.add(tag);
    docker('build', '--network', 'none', '--provenance=false', '--platform', 'linux/' + architecture,
      '--output', 'type=image,oci-mediatypes=true', '--label', ownership + '=' + nonce, '--tag', tag, context);
    if (failAfter === 'build') throw new Error('deliberate loss of completed build response');
    const inspected = JSON.parse(docker('image','inspect','--format','{{json .}}',tag));
    assert.equal(inspected.Os, 'linux'); assert.equal(inspected.Architecture, architecture);
    const archive = docker('image','save',tag);
    const member = path => {safeArchivePath(path); return run('/usr/bin/tar', ['-xOf','-',path], {input:archive});};
    const saved = JSON.parse(member('manifest.json'));
    assert.equal(saved.length, 1); const record = saved[0];
    const configBytes = member(record.Config), config = JSON.parse(configBytes);
    assert.deepEqual(config.rootfs.diff_ids, inspected.RootFS.Layers);
    assert.equal(record.Layers.length, config.rootfs.diff_ids.length);
    let manifestBytes, terminal;
    if (inspected.Descriptor) {
      // Containerd image IDs name the manifest, not its configuration. Verify
      // the complete saved graph instead of treating the ID as a config hash.
      assert.equal(inspected.Id, inspected.Descriptor.digest);
      const raw = member('blobs/sha256/' + inspected.Id.slice(7));
      assert.equal(sha(raw), inspected.Id); assert.equal(raw.length, inspected.Descriptor.size);
      manifestBytes = ociFixtureManifest(raw);
      const sourceManifest = JSON.parse(manifestBytes);
      assert.equal(sourceManifest.config.digest, sha(configBytes));
      assert.equal(sourceManifest.config.size, configBytes.length);
      terminal = member('blobs/sha256/' + sourceManifest.layers.at(-1).digest.slice(7));
    } else {
      // Classic Docker's save format contains uncompressed layers; it binds
      // those exact bytes through the materialized configuration's DiffIDs.
      assert.equal(sha(configBytes), inspected.Id, 'materialized image configuration differs');
      const layers = record.Layers.map(path => describe(member(path), media + 'layer.v1.tar'));
      assert.deepEqual(layers.map(row => row.digest), config.rootfs.diff_ids);
      manifestBytes = Buffer.from(JSON.stringify({schemaVersion:2, mediaType:media + 'manifest.v1+json',
        config:describe(configBytes, media + 'config.v1+json'), layers}));
      terminal = member(record.Layers.at(-1));
    }
    const admitted = parseManifest(manifestBytes, describe(manifestBytes, media + 'manifest.v1+json'));
    const parsed = parseConfig(configBytes, admitted, {os:'linux', architecture});
    assert(terminal.length <= limits.expanded);
    assert.deepEqual(decodeMetadataLayer(terminal, admitted, parsed), {inventory, standards});
    const name = 'prismpm-metadata-test-' + nonce + '-' + architecture + '-' + hostile;
    containers.add(name);
    const container = docker('create','--name',name,'--label',ownership + '=' + nonce,
      '--network','none','--read-only','--platform','linux/' + architecture,tag).toString().trim();
    if (failAfter === 'create') throw new Error('deliberate loss of completed create response');
    assert.match(container, /^[0-9a-f]{64}$/);
    assert.equal(inspectOwned('container', name).Id, container);
    const copied = join(context, 'copied'); mkdirSync(copied);
    for (const name of Object.values(paths)) docker('cp',container + ':/' + name,join(copied,name.split('/').at(-1)));
    assert.deepEqual(readFileSync(join(copied,'inventory.json')), inventory);
    assert.deepEqual(readFileSync(join(copied,'standards.lock')), standards);
    docker('rm','--volumes',container); containers.delete(name);
    console.log(JSON.stringify({scope:'actual-materialization-only', architecture, hostile,
      config_digest:sha(configBytes), terminal_diff_id:config.rootfs.diff_ids.at(-1), terminal_bytes:terminal.length,
      inventory_digest:sha(inventory), standards_digest:sha(standards)}));
  }
  const official = run('/usr/local/bin/oci-image-schema-conformance', ['-test.v','-test.timeout','30s'],
    {cwd:resolve('standards/oracles/oci-image-1.1.1/schema')}).toString();
  assert.equal((official.match(/^--- PASS:/gm) ?? []).length, 14);
  assert(!/^--- (?:FAIL|SKIP):/m.test(official));
  console.log('PASS 8 actual two-platform hostile-ancestor materializations; 14 pinned official OCI schema tests; no foreign code execution');
  for (const operation of ['build','create']) {
    const selected = randomUUID();
    const result = spawnSync(process.execPath,[resolve('sdk/metadata-materialization.integration.mjs'),selected,operation],
      {timeout:90000,maxBuffer:1024*1024});
    assert.ifError(result.error); assert.equal(result.status,1);
    assert(result.stderr.toString().includes('deliberate loss of completed ' + operation + ' response'));
    for (const [kind,name] of [
      ['image','prismpm-metadata-test-' + selected + ':amd64-opt-link'],
      ['container','prismpm-metadata-test-' + selected + '-amd64-opt-link'],
    ]) assert.equal(inspectOwned(kind,name),null,'uncertain successful creation must be reconciled and cleaned');
  }
  console.log('PASS exact owned-resource cleanup after lost build and create responses');
} catch (error) {
  failure = error;
} finally {
  // Exact per-invocation IDs/tags only. Never prune shared Docker resources.
  const failures = failure ? [failure] : [];
  for (const name of containers) try {
    const owned = inspectOwned('container',name);
    if (owned) docker('rm','--volumes',owned.Id);
  } catch (error) {failures.push(error);}
  for (const tag of tags) try {
    if (inspectOwned('image',tag)) docker('image','rm',tag);
  } catch (error) {failures.push(error);}
  rmSync(work, {recursive:true, force:true});
  if (failures.length) throw new AggregateError(failures, 'materialization or owned cleanup failed');
}
