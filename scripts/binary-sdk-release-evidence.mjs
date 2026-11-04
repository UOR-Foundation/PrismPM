// Preserve the original binary readers' evidence; this packager grants no acceptance.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {constants, chmodSync, closeSync, fstatSync, lstatSync, mkdirSync,
  mkdtempSync, openSync, readSync, readdirSync, realpathSync, rmSync, unlinkSync, writeFileSync} from 'node:fs';
import {basename, dirname, join, resolve, sep} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {canonical, checkAccepted, hash, inventoryEvidence, verifyResult, verifySource} from './binary-sdk-check.mjs';

const repository = resolve(dirname(fileURLToPath(import.meta.url)), '..');
export const binaryEvidenceLimits = Object.freeze({file: 256 * 1024 ** 2, total: 1024 ** 3,
  entries: 16384, directories: 512, depth: 64});
const identityFields = ['dev', 'ino', 'mode', 'nlink', 'size', 'mtimeNs', 'ctimeNs'];
const validateProof = checkAccepted;
const sameIdentity = (actual, expected) => {
  for (const field of identityFields) assert.equal(actual[field], expected[field], 'binary evidence custody changed');
};
const inside = (path, root) => path === root || path.startsWith(root + sep);

function contextNames(context) {
  assert.deepEqual(Object.keys(context).sort(), ['architecture', 'image_reference', 'language', 'source_revision']);
  assert.match(context.source_revision, /^[0-9a-f]{40}$/);
  assert.match(context.image_reference, /^ghcr\.io\/uor-foundation\/prismpm-sdk@sha256:[0-9a-f]{64}$/);
  assert(['amd64', 'arm64'].includes(context.architecture));
  assert(['1.1', '1.2'].includes(context.language));
  const evidence = `sdk-${context.architecture}-binary-sdk${context.language === '1.2' ? '-language-1.2' : ''}`;
  return {evidence, log: evidence + '.log'};
}

function readHeld(path, maximum) {
  assert.equal(realpathSync(path), path, 'binary evidence path alias');
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    const identity = fstatSync(fd, {bigint: true});
    assert(identity.isFile() && identity.nlink === 1n && identity.size <= BigInt(maximum),
      'bounded independent regular binary evidence file required');
    const bytes = Buffer.alloc(Number(identity.size));
    for (let offset = 0; offset < bytes.length;) {
      const count = readSync(fd, bytes, offset, bytes.length - offset, null);
      assert(count > 0, 'binary evidence shortened'); offset += count;
    }
    assert.equal(readSync(fd, Buffer.alloc(1), 0, 1, null), 0, 'binary evidence grew');
    sameIdentity(fstatSync(fd, {bigint: true}), identity);
    sameIdentity(lstatSync(path, {bigint: true}), identity);
    assert.equal(realpathSync(path), path, 'binary evidence path changed');
    return {bytes, identity};
  } finally {closeSync(fd);}
}

function captureRaw(directory, names) {
  assert.equal(realpathSync(directory), directory, 'binary artifact root alias');
  const files = new Map(), directories = [];
  let entries = 0, total = 0;
  function walk(relative, depth) {
    assert(depth <= binaryEvidenceLimits.depth, 'binary evidence depth limit');
    assert(++entries <= binaryEvidenceLimits.entries, 'binary evidence entry limit');
    const path = relative ? join(directory, relative) : directory;
    assert.equal(realpathSync(path), path, 'binary evidence directory alias');
    const before = lstatSync(path, {bigint: true});
    if (before.isDirectory()) {
      assert(directories.length < binaryEvidenceLimits.directories, 'binary evidence directory limit');
      const fd = openSync(path, constants.O_RDONLY | constants.O_DIRECTORY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
      const row = {path, relative, identity: before, fd, names: null};
      directories.push(row); sameIdentity(fstatSync(fd, {bigint: true}), before);
      row.names = readdirSync(path).sort();
      assert(row.names.length <= binaryEvidenceLimits.entries - entries, 'binary evidence entry limit');
      for (const name of row.names) {
        assert(/^[A-Za-z0-9_.-]{1,127}$/.test(name) && !['.', '..'].includes(name), 'closed binary evidence path');
        walk(relative ? relative + '/' + name : name, depth + 1);
      }
      sameIdentity(lstatSync(path, {bigint: true}), before);
      sameIdentity(fstatSync(fd, {bigint: true}), before);
    } else {
      assert(before.isFile() && !before.isSymbolicLink(), 'binary evidence must be a regular file or directory');
      const row = readHeld(path, Math.min(binaryEvidenceLimits.file, binaryEvidenceLimits.total - total));
      total += row.bytes.length; assert(total <= binaryEvidenceLimits.total, 'complete binary evidence exceeds one GiB');
      files.set(relative, row);
    }
  }
  try {
    walk('', 0);
    assert.deepEqual(directories[0].names, [names.evidence, names.log].sort(), 'exact downloaded binary artifact members required');
    const gate = directories.find(row => row.relative === names.evidence);
    assert(gate, 'binary gate directory required');
    assert.deepEqual(gate.names, ['inventory.json', 'proof', 'result.json', 'source.json'], 'complete original binary gate directory required');
    assert(directories.some(row => row.relative === names.evidence + '/proof'), 'original proof directory required');
    assert(files.has(names.log), 'original binary gate log required');
    return {directory, files, directories, total};
  } catch (error) {
    for (const row of directories) closeSync(row.fd);
    throw error;
  }
}

function recheck(raw) {
  // All directories remain held until publication, including otherwise empty ones.
  for (const row of raw.directories) {
    assert.equal(realpathSync(row.path), row.path, 'binary evidence directory path changed');
    sameIdentity(fstatSync(row.fd, {bigint: true}), row.identity);
    sameIdentity(lstatSync(row.path, {bigint: true}), row.identity);
    assert.deepEqual(readdirSync(row.path).sort(), row.names, 'binary evidence members changed');
  }
  for (const [relative, row] of raw.files) {
    const actual = readHeld(join(raw.directory, relative), binaryEvidenceLimits.file);
    sameIdentity(actual.identity, row.identity);
    assert(actual.bytes.equals(row.bytes), 'binary evidence bytes changed');
  }
}

function authorityAt(sourceRoot) {
  const files = ['vendor/lean4-prod/lean.tar', 'lean-toolchain'].map(relative => {
    const path = join(sourceRoot, relative);
    return {path, ...readHeld(path, relative === 'lean-toolchain' ? 16384 : binaryEvidenceLimits.file)};
  });
  return {files, value: {archive_sha256: hash(files[0].bytes), toolchain: files[1].bytes.toString('utf8').trim()}};
}

function checkAuthority(authority) {
  for (const row of authority.files) {
    const actual = readHeld(row.path, binaryEvidenceLimits.file);
    sameIdentity(actual.identity, row.identity);
    assert(actual.bytes.equals(row.bytes), 'current compiler authority changed');
  }
}

function validateOriginal(raw, context, sourceRoot, names, authority) {
  const bytes = name => {
    const selected = raw.files.get(names.evidence + '/' + name);
    assert(selected, 'missing original binary evidence: ' + name); return selected.bytes;
  };
  const source = JSON.parse(bytes('source.json'));
  assert.equal(source.revision, context.source_revision, 'binary source revision differs');
  verifySource(sourceRoot, source);
  const sourceSha256 = hash(canonical(source));
  const inventory = inventoryEvidence(bytes('inventory.json'), context.source_revision);
  const result = JSON.parse(bytes('result.json'));
  verifyResult(result, context.image_reference, context.source_revision, context.architecture,
    sourceSha256, inventory.sha256, context.language);
  const receiptBytes = bytes('proof/receipt.json'), receipt = JSON.parse(receiptBytes);
  assert(receiptBytes.equals(Buffer.from(canonical(receipt))), 'canonical original proof receipt required');
  assert.equal(receipt.build_id, result.build_id, 'proof/result build identity differs');
  assert.equal(receipt.attestation_id, result.attestation_id, 'proof/result attestation identity differs');
  validateProof(join(raw.directory, names.evidence, 'proof'), receipt, authority.value, context.language);
  // The proof reader may invoke the source-owned semantic helper. Recheck the
  // current source after it returns; compiler pins remain held independently.
  verifySource(sourceRoot, source);
  checkAuthority(authority);
  return {source_sha256: sourceSha256, inventory_sha256: inventory.sha256, receipt};
}

// Read-only USTAR validation. No archive paths or contents are extracted/run.
function verifyArchive(path, expectedFiles, expectedDirectories, {fd: existing, nlink = 1n} = {}) {
  assert.equal(realpathSync(path), path, 'binary archive path alias');
  const fd = existing ?? openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    const identity = fstatSync(fd, {bigint: true});
    const maximum = binaryEvidenceLimits.total + 16 * 1024 ** 2 + (binaryEvidenceLimits.entries + 2) * 1024;
    assert(identity.isFile() && identity.nlink === nlink && identity.size <= BigInt(maximum)
      && identity.size >= 1024n && identity.size % 512n === 0n, 'bounded independent USTAR required');
    let offset = 0;
    const exact = size => {
      const bytes = Buffer.alloc(size);
      for (let at = 0; at < size;) {
        const count = readSync(fd, bytes, at, size - at, offset + at); assert(count > 0, 'binary tar shortened'); at += count;
      }
      offset += size; assert(offset <= Number(identity.size), 'binary tar grew'); return bytes;
    };
    const text = bytes => {
      const end = bytes.indexOf(0), length = end < 0 ? bytes.length : end;
      assert(bytes.subarray(length).every(value => value === 0), 'USTAR field padding');
      assert(bytes.subarray(0, length).every(value => value >= 32 && value < 127), 'USTAR ASCII field');
      return bytes.subarray(0, length).toString('ascii');
    };
    const octal = bytes => {
      const value = bytes.toString('latin1'); assert(/^ *[0-7]+[\0 ]*$/.test(value), 'USTAR octal field');
      const number = Number.parseInt(value.trim().replace(/\0.*$/, ''), 8);
      assert(Number.isSafeInteger(number) && number >= 0); return number;
    };
    const seen = new Set(); let ended = false;
    while (offset < Number(identity.size)) {
      const header = exact(512);
      if (header.every(value => value === 0)) {
        const remaining = Number(identity.size) - offset;
        assert(remaining >= 512 && remaining <= 9728 && exact(remaining).every(value => value === 0), 'complete USTAR terminators required');
        ended = true; break;
      }
      assert(seen.size < binaryEvidenceLimits.entries + 1, 'binary tar entry limit');
      const checksum = header.reduce((sum, value, index) => sum + (index >= 148 && index < 156 ? 32 : value), 0);
      assert.equal(octal(header.subarray(148, 156)), checksum, 'binary tar checksum');
      assert.equal(header.subarray(257, 265).toString('latin1'), 'ustar\0' + '00');
      assert(header.subarray(500).every(value => value === 0), 'USTAR extension refused');
      assert.equal(text(header.subarray(157, 257)), '', 'binary tar links refused');
      const prefix = text(header.subarray(345, 500)), rawName = (prefix ? prefix + '/' : '') + text(header.subarray(0, 100));
      const type = header[156], directory = type === 53;
      assert(type === 0 || type === 48 || directory, 'binary tar links/special entries refused');
      const name = directory && rawName.endsWith('/') ? rawName.slice(0, -1) : rawName;
      assert(name && !name.startsWith('/') && !name.split('/').some(part => !part || ['.', '..'].includes(part)), 'binary tar traversal refused');
      assert(!seen.has(name), 'duplicate binary tar entry');
      if (name.includes('/')) assert(seen.has(name.slice(0, name.lastIndexOf('/'))), 'binary tar ancestor missing');
      seen.add(name);
      const size = octal(header.subarray(124, 136));
      assert.equal(octal(header.subarray(100, 108)), directory ? 0o555 : 0o444, 'binary tar modes');
      assert.equal(octal(header.subarray(108, 116)), 0); assert.equal(octal(header.subarray(116, 124)), 0);
      assert.equal(octal(header.subarray(136, 148)), 0);
      if (directory) {
        assert(expectedDirectories.has(name) && size === 0, 'unexpected binary tar directory');
      } else {
        const expected = expectedFiles.get(name); assert(expected, 'unexpected binary tar file');
        assert.equal(size, expected.size, 'binary tar file size');
        const digest = createHash('sha256');
        for (let left = size; left > 0;) {const count = Math.min(left, 65536); digest.update(exact(count)); left -= count;}
        assert.equal(digest.digest('hex'), expected.sha256, 'binary tar file digest');
        const padding = (512 - size % 512) % 512;
        assert(exact(padding).every(value => value === 0), 'binary tar payload padding');
      }
    }
    assert(ended); assert.equal(seen.size, expectedFiles.size + expectedDirectories.size, 'complete binary archive graph');
    for (const name of [...expectedFiles.keys(), ...expectedDirectories]) assert(seen.has(name), 'binary archive member missing');
    assert.equal(readSync(fd, Buffer.alloc(1), 0, 1, offset), 0);
    sameIdentity(fstatSync(fd, {bigint: true}), identity); sameIdentity(lstatSync(path, {bigint: true}), identity);
    assert.equal(realpathSync(path), path);
    return {fd, identity};
  } catch (error) {if (existing === undefined) closeSync(fd); throw error;}
}

export function packBinaryEvidence(directory, output, context, sourceRoot = repository) {
  context = {...context};
  const names = contextNames(context);
  directory = resolve(directory); output = resolve(output); sourceRoot = resolve(sourceRoot);
  assert.equal(realpathSync(sourceRoot), sourceRoot, 'binary source root alias');
  assert(!inside(output, directory), 'binary archive cannot replace or extend its input');
  const parent = dirname(output), parentIdentity = lstatSync(parent, {bigint: true});
  assert(parentIdentity.isDirectory() && realpathSync(parent) === parent, 'binary archive parent alias');
  assert(!lstatSync(output, {throwIfNoEntry: false}), 'fresh binary archive required');
  const raw = captureRaw(directory, names);
  let stage, stageIdentity, parentFd, archiveOwner, published = false;
  const stagedDirectories = [];
  try {
    parentFd = openSync(parent, constants.O_RDONLY | constants.O_DIRECTORY | constants.O_NOFOLLOW);
    for (const field of ['dev', 'ino', 'mode']) assert.equal(fstatSync(parentFd, {bigint: true})[field], parentIdentity[field]);
    const authority = authorityAt(sourceRoot);
    const original = validateOriginal(raw, context, sourceRoot, names, authority);
    recheck(raw);
    const manifest = {schema: 'prismpm/binary-sdk-gate-evidence/1', scope: 'original-binary-gate-evidence-only',
      ...context, ...original, source_authority: authority.value,
      directories: raw.directories.map(row => row.relative).filter(Boolean).sort(),
      files: [...raw.files.keys()].sort().map(path => ({path, size: raw.files.get(path).bytes.length, sha256: hash(raw.files.get(path).bytes)})),
      unclaimed: ['new-acceptance', 'sdk-release', 'crates.io-publication', 'ecosystem-acceptance', 'foundry-readiness', 'deployment']};
    stage = mkdtempSync(join(parent, '.binary-sdk-evidence-'));
    stageIdentity = lstatSync(stage, {bigint: true});
    const root = join(stage, 'evidence'); mkdirSync(root, {mode: 0o700}); stagedDirectories.push(root);
    for (const relative of manifest.directories) {
      const path = join(root, relative); mkdirSync(path, {mode: 0o700}); stagedDirectories.push(path);
    }
    for (const [relative, row] of raw.files) {
      const path = join(root, relative); writeFileSync(path, row.bytes, {flag: 'wx', mode: 0o444}); chmodSync(path, 0o444);
    }
    const manifestPath = join(root, 'evidence-manifest.json');
    const manifestBytes = Buffer.from(canonical(manifest) + '\n');
    assert(manifestBytes.length <= 16 * 1024 ** 2, 'binary evidence manifest limit');
    writeFileSync(manifestPath, manifestBytes, {flag: 'wx', mode: 0o444}); chmodSync(manifestPath, 0o444);
    for (const path of stagedDirectories) chmodSync(path, 0o555);
    const archive = join(stage, 'evidence.tar');
    const tarEnvironment = {...process.env, LC_ALL: 'C'};
    delete tarEnvironment.TAR_OPTIONS;
    execFileSync('/usr/bin/tar', ['--format=ustar', '--sort=name', '--mtime=@0', '--owner=0', '--group=0', '--numeric-owner',
      '-cf', archive, '-C', root, '--', ...[names.evidence, names.log, 'evidence-manifest.json'].sort()],
    {timeout: 120000, maxBuffer: 1024 * 1024, env: tarEnvironment});
    chmodSync(archive, 0o444);
    const expectedFiles = new Map(manifest.files.map(row => [row.path, row]));
    expectedFiles.set('evidence-manifest.json', {size: manifestBytes.length, sha256: hash(manifestBytes)});
    archiveOwner = verifyArchive(archive, expectedFiles, new Set(manifest.directories));
    validateOriginal(raw, context, sourceRoot, names, authority);
    recheck(raw);
    const currentParent = lstatSync(parent, {bigint: true});
    for (const field of ['dev', 'ino', 'mode']) assert.equal(currentParent[field], parentIdentity[field], 'archive parent changed');
    assert.equal(realpathSync(parent), parent, 'archive parent path changed');
    sameIdentity(fstatSync(archiveOwner.fd, {bigint: true}), archiveOwner.identity);
    sameIdentity(lstatSync(archive, {bigint: true}), archiveOwner.identity);
    // Linux descriptor paths bind both the validated archive inode and original
    // output directory. GNU ln creates the destination exclusively, never replaces.
    execFileSync('/usr/bin/ln', ['--logical', '--no-target-directory', '--',
      `/proc/${process.pid}/fd/${archiveOwner.fd}`, `/proc/${process.pid}/fd/${parentFd}/${basename(output)}`],
    {timeout: 10000, maxBuffer: 1024 * 1024});
    published = true;
    const linked = fstatSync(archiveOwner.fd, {bigint: true});
    for (const field of ['dev', 'ino', 'mode', 'size', 'mtimeNs']) assert.equal(linked[field], archiveOwner.identity[field], 'binary archive changed during publication');
    assert.equal(linked.nlink, 2n, 'binary archive has an unexpected link');
    sameIdentity(lstatSync(output, {bigint: true}), linked);
    const publishedIdentity = verifyArchive(output, expectedFiles, new Set(manifest.directories), {fd: archiveOwner.fd, nlink: 2n}).identity;
    recheck(raw); checkAuthority(authority);
    for (const field of ['dev', 'ino', 'mode']) assert.equal(lstatSync(parent, {bigint: true})[field], parentIdentity[field], 'archive parent changed');
    assert.equal(realpathSync(parent), parent);
    sameIdentity(fstatSync(archiveOwner.fd, {bigint: true}), publishedIdentity);
    sameIdentity(lstatSync(output, {bigint: true}), publishedIdentity);
    assert.equal(realpathSync(output), output, 'published binary archive path changed');
    return manifest;
  } catch (error) {
    if (published) {
      const anchored = `/proc/${process.pid}/fd/${parentFd}/${basename(output)}`;
      const linked = lstatSync(anchored, {throwIfNoEntry: false, bigint: true});
      if (linked?.dev === archiveOwner.identity.dev && linked?.ino === archiveOwner.identity.ino) unlinkSync(anchored);
    }
    throw error;
  } finally {
    for (const row of raw.directories) closeSync(row.fd);
    if (archiveOwner) closeSync(archiveOwner.fd);
    if (parentFd !== undefined) closeSync(parentFd);
    if (stage) {
      const current = lstatSync(stage, {bigint: true});
      assert.equal(current.dev, stageIdentity.dev); assert.equal(current.ino, stageIdentity.ino);
      assert.equal(realpathSync(stage), stage, 'staging root path changed');
      for (const path of stagedDirectories) {
        assert(lstatSync(path).isDirectory() && realpathSync(path) === path, 'staging directory alias'); chmodSync(path, 0o700);
      }
      rmSync(stage, {recursive: true, force: true});
    }
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [directory, output, image_reference, source_revision, architecture, language, ...extra] = process.argv.slice(2);
  assert.equal(extra.length, 0);
  const manifest = packBinaryEvidence(directory, output, {source_revision, image_reference, architecture, language});
  process.stdout.write(canonical(manifest) + '\n');
}
