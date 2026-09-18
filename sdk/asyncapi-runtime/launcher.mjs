#!/usr/local/bin/node
// Prism-owned execution closure; upstream source and historical lock stay intact.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { closeSync, constants, existsSync, fstatSync, lstatSync, openSync, readSync, readdirSync, realpathSync, statSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

export const runtimeLockDigest = '5bd20ce206d3b3b76a7034951c9e19e15291c54eec0a1424139b465c980f1205';
const manifestDigest = '89f3a5feea766b7d899f5224452bfd0f725aa00c17f241da2cf0818081437d93';
const upstream = {
  '../spec/asyncapi.md': '983a9c0ccb35412d4f6f254ab1ae9ea09a560664b1e41efb0109b12d0cd91869',
  'validation/base-doc-combined.json': '9caedb8e4506b2456d5842f3aeb1097790838717c2bf512fc9157f032cac3635',
  'validation/embedded-examples-validation.js': '41b52f9ac08146dfea623a9a411805be3c1897cd3d61a52d599b51766e305268',
  'validation/package.json': manifestDigest,
  'validation/package-lock.json': 'a8de3ba500b77dc3cc5059e88fd536e4da0e95e53c1db5ea89b89808b157d8ba',
  'validation/user-create.avsc': '4fb2b0edb207b2696627537408180805022174355fd3d3e1709530353207d6dc',
  'validation/validation_examples.log': '28d0ec2bfb5d049084d9f3b61730f735f22b2c177bec957a5425c3488ca7d4dd',
};
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const installedRoot = '/opt/prismpm/asyncapi-official/scripts';
const inventoryPath = '/opt/prismpm/share/inventory.json';

function regular(path, limit = 32 * 1024 * 1024) {
  assert.ok(lstatSync(path).isFile() && realpathSync(path) === resolve(path), `non-regular runtime file: ${path}`);
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW);
  try {
    const metadata = fstatSync(fd);
    assert.ok(metadata.isFile() && metadata.size <= limit, 'runtime file exceeds bound');
    const bytes = Buffer.alloc(metadata.size + 1);
    let length = 0, count;
    while ((count = readSync(fd, bytes, length, bytes.length - length, length)) > 0) length += count;
    assert.equal(length, metadata.size, 'runtime file changed during bounded read');
    return bytes.subarray(0, length);
  } finally { closeSync(fd); }
}

// Identical length-framed byte ordering to generate-inventory.mjs. Source
// layout checks alone do not establish package-code integrity.
export function runtimeTreeDigest(root) {
  root = realpathSync(join(root, 'node_modules'));
  const digest = createHash('sha256');
  let count = 0, total = 0;
  function visit(directory, prefix = '') {
    assert.ok(prefix.split('/').length <= 128, 'runtime tree depth exceeds bound');
    for (const name of readdirSync(directory).sort((a, b) => Buffer.from(a).compare(Buffer.from(b)))) {
      assert.ok(++count <= 65_536, 'runtime tree exceeds bound');
      const path = join(directory, name), relative = prefix ? `${prefix}/${name}` : name;
      const target = realpathSync(path), metadata = lstatSync(path);
      assert.ok(target.startsWith(`${root}${sep}`), 'runtime tree link escapes installed graph');
      if (metadata.isSymbolicLink()) {
        assert.ok(relative.split('/').at(-2) === '.bin' && statSync(target).isFile(), 'runtime tree link is not an in-graph executable alias');
      }
      if (metadata.isDirectory()) visit(path, relative);
      else {
        const bytes = regular(target), encoded = Buffer.from(relative);
        total += bytes.length;
        assert.ok(total <= 512 * 1024 * 1024, 'runtime tree exceeds bound');
        const size = Buffer.alloc(8); size.writeBigUInt64BE(BigInt(encoded.length));
        const length = Buffer.alloc(8); length.writeBigUInt64BE(BigInt(bytes.length));
        digest.update(size).update(encoded).update(length).update(bytes);
      }
    }
  }
  visit(root);
  return `sha256:${digest.digest('hex')}`;
}

export function inventoryRuntimeDigest(bytes) {
  assert.ok(bytes.length <= 8 * 1024 * 1024, 'SDK inventory exceeds bound');
  const value = JSON.parse(bytes);
  const canonical = item => Array.isArray(item) ? item.map(canonical)
    : item && typeof item === 'object' ? Object.fromEntries(Object.keys(item).sort((a, b) => Buffer.from(a).compare(Buffer.from(b)))
      .map(key => [key, canonical(item[key])])) : item;
  assert.deepEqual(bytes, Buffer.from(`${JSON.stringify(canonical(value))}\n`), 'SDK inventory is not canonical');
  assert.deepEqual(Object.keys(value).sort(), ['artifacts', 'commands', 'schema']);
  assert.equal(value.schema, 'prismpm/sdk-inventory/1');
  assert.ok(Array.isArray(value.artifacts) && Array.isArray(value.commands));
  const matches = value.artifacts.filter(row => row.id === 'asyncapi-embedded-runtime');
  assert.equal(matches.length, 1, 'exact installed AsyncAPI runtime inventory row required');
  const row = matches[0];
  assert.deepEqual(Object.keys(row).sort(), ['digest', 'id', 'kind', 'version']);
  assert.equal(row.kind, 'oracle');
  assert.equal(row.version, '@asyncapi/parser/3.6.0');
  assert.match(row.digest, /^sha256:[0-9a-f]{64}$/);
  return row.digest;
}

function installedPackages(root) {
  const packages = new Map();
  function directory(path, prefix) {
    assert.ok(lstatSync(path).isDirectory() && realpathSync(path) === resolve(path), 'non-regular module directory');
    for (const name of readdirSync(path).sort()) {
      if (name === '.bin' || name === '.package-lock.json') continue;
      assert.ok(!name.startsWith('.'), 'unregistered module directory entry');
      const target = join(path, name), relative = `${prefix}/${name}`;
      if (name.startsWith('@')) { directory(target, relative); continue; }
      assert.ok(lstatSync(target).isDirectory() && realpathSync(target) === resolve(target), 'non-regular installed package');
      const manifest = JSON.parse(regular(join(target, 'package.json')));
      packages.set(relative, manifest);
      if (existsSync(join(target, 'node_modules'))) directory(join(target, 'node_modules'), `${relative}/node_modules`);
    }
  }
  directory(join(root, 'node_modules'), 'node_modules');
  return packages;
}

export function verifyRuntime(root, expectedTreeDigest, environment = process.env) {
  root = resolve(root);
  assert.ok(!environment.NODE_OPTIONS && !environment.NODE_PATH, 'ambient module configuration is forbidden');
  assert.equal(realpathSync(root), root, 'runtime directory must not be a symlink');
  assert.equal(sha(regular(join(root, 'package.json'))), manifestDigest, 'owned runtime manifest digest differs');
  const lockBytes = regular(join(root, 'package-lock.json'));
  assert.equal(sha(lockBytes), runtimeLockDigest, 'owned runtime lock digest differs');
  for (const [path, digest] of Object.entries(upstream)) {
    assert.equal(sha(regular(join(root, path))), digest, `upstream source digest differs: ${path}`);
  }
  assert.deepEqual(readdirSync(join(root, 'validation')).sort(),
    Object.keys(upstream).filter(path => path.startsWith('validation/')).map(path => path.slice(11)).sort(),
    'upstream validation file closure differs (shadow module directory forbidden)');
  // No local/ancestor module directory may override or fill gaps in the graph.
  for (let parent = dirname(root); ; parent = dirname(parent)) {
    assert.ok(!existsSync(join(parent, 'node_modules')), `shadow module directory: ${parent}`);
    if (parent === dirname(parent)) break;
  }
  const lock = JSON.parse(lockBytes);
  assert.equal(lock.lockfileVersion, 3);
  const expected = Object.keys(lock.packages).filter(path => path !== '').sort();
  const installed = installedPackages(root);
  assert.deepEqual([...installed.keys()].sort(), expected, 'installed package closure differs');
  for (const path of expected) {
    assert.equal(installed.get(path).version, lock.packages[path].version, `installed package version differs: ${path}`);
  }
  assert.match(expectedTreeDigest, /^sha256:[0-9a-f]{64}$/);
  assert.equal(runtimeTreeDigest(root), expectedTreeDigest, 'installed runtime byte digest differs from SDK inventory');
  const require = createRequire(join(root, 'validation/embedded-examples-validation.js'));
  const direct = ['@asyncapi/parser', 'js-yaml', 'json-merge-patch', 'jsonpointer'];
  for (const name of direct) {
    const expected = join(root, 'node_modules', name);
    assert.ok(realpathSync(require.resolve(name)).startsWith(`${expected}${sep}`), `module resolution escaped owned runtime: ${name}`);
  }
  assert.equal(installed.get('node_modules/@asyncapi/parser').version, '3.6.0', 'official parser version differs');
  return {parser: '@asyncapi/parser/3.6.0', runtimeLock: runtimeLockDigest};
}

export function runOfficial(root, expectedTreeDigest) {
  verifyRuntime(root, expectedTreeDigest);
  return execFileSync(process.execPath, [join(root, 'validation/embedded-examples-validation.js')], {
    cwd: join(root, 'validation'), timeout: 120_000, maxBuffer: 16 * 1024 * 1024,
    env: {HOME: '/tmp', LANG: 'C.UTF-8', LC_ALL: 'C.UTF-8', PATH: '/usr/local/bin:/usr/bin:/bin'},
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

const ownFile = fileURLToPath(import.meta.url);
if (process.argv[1] && realpathSync(process.argv[1]) === ownFile) {
  try {
    const root = dirname(ownFile);
    assert.equal(root, installedRoot, 'production launcher must use the fixed installed runtime path');
    assert.ok(process.argv.length === 2 || (process.argv.length === 3 && process.argv[2] === '--check'),
      'usage: asyncapi-official [--check]');
    // No environment/argument override and no absent-inventory fallback. The
    // retained exact SDK image/lock authenticates this inventory; its actual
    // installed package bytes must match before loading upstream code.
    const expected = inventoryRuntimeDigest(regular(inventoryPath, 8 * 1024 * 1024));
    if (process.argv[2] === '--check') process.stdout.write(`${JSON.stringify(verifyRuntime(root, expected))}\n`);
    else process.stdout.write(runOfficial(root, expected));
  } catch (error) {
    process.stderr.write(`AsyncAPI runtime rejected: ${error.message}\n`);
    if (error.stderr) process.stderr.write(error.stderr);
    process.exitCode = 1;
  }
}
