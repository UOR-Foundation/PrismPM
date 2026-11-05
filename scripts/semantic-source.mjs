// One reader and writer for the semantic module of a LexLean source, whichever
// surface the source uses. The SDK sources are language-1.2 notation, which has
// no `\semanticdata{...}` line to match; LexLean itself reads a notation module
// back to its data form, so every script that inspects or mutates declarations
// goes through `prismpm-semantic-source`, a thin wrapper over
// `lexlean::sdk::{read_module, write_module, to_notation}`.
//
// Outside an SDK, PRISMPM_SEMANTIC_SOURCE names a source-bootstrap tool; without
// it Cargo runs this repository's helper. Any installed SDK marker instead
// requires the fixed, inventory-bound private asset. Missing assets in older
// SDKs refuse; an environment override cannot replace an installed SDK tool.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import {lstatSync, realpathSync} from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import {readSmall, snapshotFile} from '../sdk/exporter-seed.mjs';
import {encodeInventory} from '../sdk/inventory-metadata.mjs';

const repository = join(dirname(fileURLToPath(import.meta.url)), '..');
const installed = Object.freeze({marker: '/etc/profile.d/prismpm-sdk.sh',
  inventory: '/opt/prismpm/share/inventory.json', root: '/opt/prismpm/share/conformance-root',
  helper: '/opt/prismpm/conformance/bin/prismpm-semantic-source'});
const actualFiles = {stat: path => lstatSync(path, {bigint: true, throwIfNoEntry: false}),
  realpath: realpathSync, read: readSmall, snapshot: snapshotFile};
const identity = ['dev', 'ino', 'uid', 'gid', 'mode', 'size', 'nlink', 'mtimeNs', 'ctimeNs'];

// Private selector test seam: production always uses the fixed paths and real
// filesystem above. No CLI/config/environment option supplies this interface.
export function selectSemanticSourceRuntime(command, root, explicit, files = actualFiles) {
  assert(['read', 'write', 'to-notation', 'runtime-audit'].includes(command));
  const sdk = files.stat(installed.marker) || files.stat(installed.inventory) || files.stat(installed.root)
    || root === installed.root || root.startsWith(installed.root + '/');
  if (!sdk) return {program: explicit || 'cargo', args: explicit ? [command]
    : ['run', '--offline', '--locked', '--quiet', '-p', 'repo-conformance', '--bin', 'prismpm-semantic-source', '--', command],
    verify() {}};
  assert(!explicit || explicit === installed.helper, 'installed SDK refuses an external semantic helper');
  const custody = new Map();
  function capture(path, maximum, executable) {
    assert.equal(files.realpath(path), path, 'installed semantic asset is aliased');
    for (let current = path; ; current = dirname(current)) {
      const entry = files.stat(current);
      assert(entry && entry.uid === 0n && (entry.mode & 0o022n) === 0n,
        'root-owned immutable semantic asset custody required');
      if (current === path) {
        assert(entry.isFile() && entry.nlink === 1n && entry.size > 0n && entry.size <= BigInt(maximum));
        assert.equal(entry.mode & 0o7000n, 0n);
        if (executable) assert((entry.mode & 0o111n) !== 0n);
      } else assert(entry.isDirectory(), 'regular immutable semantic asset ancestor required');
      if (custody.has(current)) {
        for (const key of identity) assert.equal(entry[key], custody.get(current)[key], 'semantic custody changed during admission');
      } else custody.set(current, entry);
      if (current === '/') break;
    }
  }
  capture(installed.inventory, 8 * 1024 * 1024, false);
  capture(installed.helper, 256 * 1024 * 1024, true);
  const inventoryText = files.read(installed.inventory, 8 * 1024 * 1024);
  const inventory = JSON.parse(inventoryText);
  assert.equal(encodeInventory(inventory), inventoryText, 'canonical installed SDK inventory required');
  assert.deepEqual(Object.keys(inventory).sort(), ['artifacts', 'commands', 'schema']);
  assert.equal(inventory.schema, 'prismpm/sdk-inventory/1');
  assert(Array.isArray(inventory.artifacts) && Array.isArray(inventory.commands));
  const rows = inventory.artifacts.filter(row => row.id === 'conformance-semantic-source');
  assert.equal(rows.length, 1, 'one private semantic helper inventory artifact required');
  const row = rows[0];
  assert.deepEqual(Object.keys(row).sort(), ['digest', 'id', 'kind', 'version']);
  assert.equal(row.kind, 'binary'); assert.equal(row.version, '0.4.0');
  assert.match(row.digest, /^sha256:[0-9a-f]{64}$/);
  const checkCustody = () => {
    for (const [path, before] of custody) {
      assert.equal(files.realpath(path), path, 'installed semantic asset ancestor changed');
      const after = files.stat(path);
      assert(after, 'installed semantic asset disappeared');
      for (const key of identity) assert.equal(after[key], before[key], 'installed semantic asset replaced: ' + key);
    }
  };
  const verify = () => {
    checkCustody();
    assert.equal(files.read(installed.inventory, 8 * 1024 * 1024), inventoryText, 'installed semantic inventory changed');
    assert.equal('sha256:' + files.snapshot(installed.helper, 256 * 1024 * 1024).sha256,
      row.digest, 'installed semantic helper differs from inventory');
    checkCustody();
  };
  verify();
  return {program: installed.helper, args: [command], verify};
}

function tool(command, input) {
  const selected = selectSemanticSourceRuntime(command, repository, process.env.PRISMPM_SEMANTIC_SOURCE);
  let output;
  try {
    output = execFileSync(selected.program, selected.args, { cwd: repository, input, encoding: 'utf8', maxBuffer: 1 << 28 });
  } catch (error) {
    try { selected.verify(); } catch (custody) { error.semanticSourceCustodyFailure = custody.message; }
    throw error;
  }
  selected.verify();
  return output;
}

const text = source => (typeof source === 'string' ? source : Buffer.from(source).toString('utf8'));

// The module of `source` (string or bytes) as a JSON value.
export function readSemantic(source) {
  return JSON.parse(tool('read', text(source)));
}

// `source` with its module replaced by `value`, in the surface it already uses.
export function writeSemantic(source, value) {
  return tool('write', JSON.stringify({ source: text(source), value }));
}

// Data-form source text rewritten as language-1.2 notation.
export function toNotation(source) {
  return tool('to-notation', text(source));
}

// Exact policies of the pinned source-owned runtime imported by bound Lean
// output, including the complete transitive module import closure.
export function runtimeAudit(generatedTexts) {
  const texts = generatedTexts.map(source => typeof source === 'string' ? source
    : new TextDecoder('utf-8', { fatal: true }).decode(source));
  return JSON.parse(tool('runtime-audit', JSON.stringify(texts)));
}
