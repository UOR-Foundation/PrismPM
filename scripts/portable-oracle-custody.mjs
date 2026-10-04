import assert from 'node:assert/strict';
import {lstatSync, realpathSync, readdirSync, cpSync, mkdirSync, writeFileSync, symlinkSync} from 'node:fs';
import {dirname, join} from 'node:path';
import {snapshotFile} from '../sdk/exporter-seed.mjs';

const fields = ['dev', 'ino', 'mode', 'size', 'nlink', 'mtimeNs', 'ctimeNs'];
export function capture(selected) {
  const path = realpathSync(selected), before = lstatSync(path, {bigint: true});
  const measurement = snapshotFile(path);
  const identity = Object.fromEntries(fields.map(key => [key, before[key].toString()]));
  const verify = () => {
    assert.equal(realpathSync(selected), path, 'oracle input path changed');
    const current = lstatSync(path, {bigint: true});
    for (const key of fields) assert.equal(current[key].toString(), identity[key], 'oracle input custody changed');
    assert.deepEqual(snapshotFile(path), measurement, 'oracle input bytes changed');
  };
  verify();
  return {selected, path, measurement, identity, verify};
}

export function requireBoundaryCheck(name, diagnostic) {
  const checks = {'wrong-response': 'response-envelope', 'delayed-wrong-response': 'response-envelope',
    duplicate: 'single-invocation', 'delayed-duplicate': 'single-invocation'};
  if (Object.hasOwn(checks, name)) assert.equal(diagnostic.check, checks[name], 'unrelated assertion is not boundary evidence');
}

export function refuseCargoAncestorConfiguration(directory) {
  for (let current = realpathSync(directory); ; current = dirname(current)) {
    for (const name of ['config', 'config.toml']) {
      let present = true;
      try { lstatSync(join(current, '.cargo', name)); }
      catch (error) { if (error.code === 'ENOENT') present = false; else throw error; }
      assert.equal(present, false, 'unowned Cargo ancestor configuration');
    }
    if (dirname(current) === current) return;
  }
}

export function snapshotSourceTree(root, {excludeGitDatabase = false} = {}) {
  const rows = [];
  function visit(directory, prefix) {
    assert.equal(realpathSync(directory), directory, 'source directory alias');
    for (const name of readdirSync(directory).sort()) {
      if (excludeGitDatabase && name === '.git') continue;
      assert(!/[\x00-\x1f\x7f]/.test(name), 'source control-character path');
      assert(rows.length < 32768, 'source entry bound exceeded');
      const path = join(directory, name), relative = prefix + name;
      const stat = lstatSync(path);
      assert(!stat.isSymbolicLink(), 'source alias refused');
      if (stat.isDirectory()) {
        rows.push({path: relative, kind: 'directory'});
        visit(path, relative + '/');
      } else rows.push({path: relative, ...snapshotFile(path)});
    }
  }
  visit(root, '');
  return rows;
}

export function privateGitObjects(sourceCargo, privateCargo,
  name = 'hologram-ab6b9bff1a591920', revision = '2bda6a9a9476872dade705bd61ece4209607f6da') {
  const source = realpathSync(join(sourceCargo, 'git/db', name, 'objects'));
  const destination = join(privateCargo, 'git/db', name);
  mkdirSync(join(destination, 'refs/commit'), {recursive: true});
  cpSync(source, join(destination, 'objects'), {recursive: true, errorOnExist: true, force: false});
  // Download caches legitimately hard-link objects. The private copy must not
  // contain aliases; Git fsck and the locked commit authenticate copied bytes.
  const rows = snapshotSourceTree(join(destination, 'objects'));
  assert(!rows.some(row => row.path.startsWith('info/') && row.kind !== 'directory'),
    'Git object alternates or graft metadata refused');
  writeFileSync(join(destination, 'config'), '[core]\n\tbare = true\n\trepositoryformatversion = 0\n', {flag: 'wx'});
  writeFileSync(join(destination, 'HEAD'), 'ref: refs/heads/master\n', {flag: 'wx'});
  writeFileSync(join(destination, 'refs/commit', revision), revision + '\n', {flag: 'wx'});
  return destination;
}

export function privateRegistryDownloads(sourceCargo, privateCargo) {
  mkdirSync(join(privateCargo, 'registry'), {recursive: true});
  for (const name of ['cache', 'index'])
    symlinkSync(realpathSync(join(sourceCargo, 'registry', name)), join(privateCargo, 'registry', name));
}
