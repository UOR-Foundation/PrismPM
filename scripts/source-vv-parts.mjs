// Main-source scheduling retains original owners; installed SDK two-run gates are unchanged.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {closeSync, constants, fstatSync, lstatSync, mkdirSync, openSync, readdirSync,
  realpathSync, unlinkSync, writeFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {captureCommand, releaseContext, selectArtifacts, validateCommand} from './release-gate-evidence.mjs';
import {bootstrapNames, canonical, hash, readOriginal, validateBootstrapBytes} from './sdk-bootstrap-retention.mjs';
import {assembleVvEvidence, parts, validateVvPart} from './vv-assemble.mjs';
import {cargoInventoryParts, validateCargoInventory, validateCargoInventoryUnion} from './vv-cargo-inventory.mjs';

const MiB = 1024 ** 2, totalLimit = 1024 * MiB;
const descriptor = (path, bytes) => ({path, byte_length: bytes.length, sha256: hash(bytes)});
const prefixFor = part => {assert(parts.includes(part), 'closed source V&V part'); return `source-${part}`;};
const identityFields = ['dev', 'ino', 'mode', 'nlink', 'size', 'mtimeNs', 'ctimeNs'];
const sameIdentity = (actual, expected, fields = identityFields) => {
  for (const field of fields) assert.equal(actual[field], expected[field], 'original source evidence custody changed');
};
const absent = path => assert(!lstatSync(path, {throwIfNoEntry: false}), 'fresh source evidence path required');
const registerPath = root => join(root, 'crates/conformance/tests/conformance.rs');
function bindRegister(inventory, expected) {
  assert(Buffer.isBuffer(inventory.get('conformance-register.rs')) && inventory.get('conformance-register.rs').equals(expected),
    'inventory register differs from exact checked-out source');
}
function contextOnly(context) {
  assert.deepEqual(context, releaseContext(context.source_revision, context.architecture, context.image_reference, context.run_id, context.run_attempt));
  assert.equal(context.image_reference, null, 'source-only scheduler context required');
}
function boundedFiles(files) {
  assert(files instanceof Map && files.size <= 16384, 'bounded flat original files required');
  let total = 0;
  for (const [name, bytes] of files) {
    assert.match(name, /^[A-Za-z0-9][A-Za-z0-9._-]{0,254}$/);
    assert(Buffer.isBuffer(bytes) && bytes.length <= 256 * MiB, 'bounded original file required');
    total += bytes.length; assert(total <= totalLimit, 'original evidence total exceeds bound');
  }
}
function inventoryFor(files, part, result) {
  if (!cargoInventoryParts.includes(part)) {assert.equal(result.inventory, null); return new Map();}
  assert.deepEqual(Object.keys(result.inventory).sort(), ['files']);
  const rows = result.inventory.files; assert(Array.isArray(rows) && rows.length > 0 && rows.length <= 1024);
  const prefix = `${prefixFor(part)}-inventory-`, inventory = new Map();
  let total = 0;
  for (const row of rows) {
    assert.equal(typeof row.path, 'string'); assert(row.path.startsWith(prefix));
    const name = row.path.slice(prefix.length); assert.match(name, /^[A-Za-z0-9][A-Za-z0-9._-]{0,126}$/);
    assert(!inventory.has(name), 'duplicate inventory member');
    const bytes = files.get(row.path); assert(Buffer.isBuffer(bytes) && bytes.length <= 16 * MiB);
    total += bytes.length; assert(total <= 128 * MiB, 'inventory total exceeds bound');
    assert.deepEqual(row, descriptor(row.path, bytes)); inventory.set(name, bytes);
  }
  assert.deepEqual([...inventory.keys()], [...inventory.keys()].sort(), 'sorted inventory members required');
  validateCargoInventory(inventory, part, result.source_revision);
  return inventory;
}
export function validateSourcePart(files, part, context) {
  contextOnly(context); boundedFiles(files);
  const prefix = prefixFor(part), raw = files.get(`${prefix}-vv.json`);
  validateVvPart(raw, context.source_revision, part); validateCommand(files, prefix, 'just', ['vv-part', part], 0);
  const bytes = files.get(`${prefix}-result.json`); assert(Buffer.isBuffer(bytes) && bytes.length <= MiB);
  const result = JSON.parse(bytes); assert.equal(bytes.toString(), canonical(result), 'canonical original retention required');
  assert.deepEqual(Object.keys(result).sort(), ['architecture', 'bootstrap', 'image_reference', 'inventory', 'part',
    'run_attempt', 'run_id', 'schema', 'scope', 'source_revision', 'vv']);
  for (const [key, value] of Object.entries(context)) {
    if (key === 'run_attempt') {
      assert.match(result[key], /^[1-9][0-9]{0,19}$/); assert(BigInt(result[key]) <= BigInt(value), 'future producing attempt');
    } else assert.equal(result[key], value, 'foreign original source context');
  }
  assert.equal(result.schema, 'prismpm/source-vv-retention/3');
  assert.equal(result.scope, 'original-source-part-with-executable-inventory-only');
  assert.equal(result.part, part); assert.deepEqual(result.vv, descriptor(`${prefix}-vv.json`, raw));
  if (part === 'source') {
    const bootstrap = new Map(bootstrapNames.map(name => [name, files.get(`run-1-${name}`)]));
    validateBootstrapBytes(bootstrap);
    assert.deepEqual(result.bootstrap, {run: 1, files: [...bootstrap].map(([name, data]) => descriptor(`run-1-${name}`, data))});
  } else assert.equal(result.bootstrap, null);
  return {result, inventory: inventoryFor(files, part, result)};
}
export function assembleSourceParts(files, context, provider, selection) {
  contextOnly(context); boundedFiles(files);
  const owners = parts.map(part => `source-vv-part-${part}`);
  const selected = selectArtifacts(provider, context, owners);
  assert.deepEqual(selection, selected, 'selection must match original provider metadata');
  const expected = [], inventories = new Map();
  for (const [index, part] of parts.entries()) {
    const {result, inventory} = validateSourcePart(files, part, context);
    assert.equal(result.run_attempt, selected.artifacts[index].run_attempt, 'producing attempt differs from selected owner');
    inventories.set(part, inventory);
    const prefix = prefixFor(part);
    expected.push(`${prefix}.json`, `${prefix}.stdout`, `${prefix}.stderr`, `${prefix}-vv.json`, `${prefix}-result.json`,
      ...[...inventory.keys()].map(name => `${prefix}-inventory-${name}`));
    if (part === 'source') expected.push(...bootstrapNames.map(name => `run-1-${name}`));
  }
  assert.deepEqual([...files.keys()].sort(), expected.sort(), 'complete closed original source evidence required');
  validateCargoInventoryUnion(inventories, context.source_revision);
  return assembleVvEvidence(new Map(parts.map(part => [part, files.get(`${prefixFor(part)}-vv.json`)])), context.source_revision);
}

// Keep directory descriptors open across all original reads, then compare both
// names and native identities on a fresh sweep before publishing a receipt.
function holdDirectory(path) {
  assert.equal(realpathSync(path), path, 'directory alias');
  const fd = openSync(path, constants.O_RDONLY | constants.O_DIRECTORY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  const identity = fstatSync(fd, {bigint: true}); assert(identity.isDirectory());
  const verify = () => {
    sameIdentity(fstatSync(fd, {bigint: true}), identity, ['dev', 'ino', 'mode']);
    sameIdentity(lstatSync(path, {bigint: true}), identity, ['dev', 'ino', 'mode']);
    assert.equal(realpathSync(path), path, 'directory custody changed');
  };
  return {path, verify, close: () => closeSync(fd)};
}
function captureFlat(directory, {maximum = 256 * MiB, entries = 16384, total = totalLimit, namePattern = /^[A-Za-z0-9][A-Za-z0-9._-]{0,254}$/} = {}) {
  const held = holdDirectory(directory), records = new Map();
  try {
    const names = readdirSync(directory).sort(); assert(names.length > 0 && names.length <= entries, 'bounded original directory required');
    let size = 0;
    for (const name of names) {
      assert.match(name, namePattern);
      const path = join(directory, name), before = lstatSync(path, {bigint: true});
      assert(before.isFile() && before.nlink === 1n, 'independent regular original required');
      const bytes = readOriginal(path, maximum); size += bytes.length; assert(size <= total, 'original directory total exceeds bound');
      sameIdentity(lstatSync(path, {bigint: true}), before); records.set(name, {before, bytes});
    }
    const directoryIdentity = lstatSync(directory, {bigint: true}); held.verify();
    const verifyIdentity = (extra = []) => {
      held.verify();
      assert.deepEqual(readdirSync(directory).sort(), [...names, ...extra].sort(), 'original member set changed');
      if (!extra.length) sameIdentity(lstatSync(directory, {bigint: true}), directoryIdentity);
      for (const [name, {before}] of records) sameIdentity(lstatSync(join(directory, name), {bigint: true}), before);
    };
    const verify = (extra = []) => {
      held.verify();
      assert.deepEqual(readdirSync(directory).sort(), [...names, ...extra].sort(), 'original member set changed');
      if (!extra.length) sameIdentity(lstatSync(directory, {bigint: true}), directoryIdentity);
      for (const [name, {before, bytes}] of records) {
        const path = join(directory, name); sameIdentity(lstatSync(path, {bigint: true}), before);
        assert(readOriginal(path, maximum).equals(bytes), 'original bytes changed');
        sameIdentity(lstatSync(path, {bigint: true}), before);
      }
      verifyIdentity(extra);
    };
    return {files: new Map([...records].map(([name, record]) => [name, record.bytes])), verify, verifyIdentity, close: held.close};
  } catch (error) {held.close(); throw error;}
}
function holdOriginal(path, maximum) {
  path = resolve(path); const parent = holdDirectory(dirname(path));
  try {
    const before = lstatSync(path, {bigint: true}), bytes = readOriginal(path, maximum);
    sameIdentity(lstatSync(path, {bigint: true}), before);
    const verifyIdentity = () => {parent.verify(); sameIdentity(lstatSync(path, {bigint: true}), before);};
    return {bytes, close: parent.close, verifyIdentity, verify: () => {
      parent.verify(); sameIdentity(lstatSync(path, {bigint: true}), before);
      assert(readOriginal(path, maximum).equals(bytes), 'original metadata bytes changed');
      sameIdentity(lstatSync(path, {bigint: true}), before); parent.verify();
    }};
  } catch (error) {parent.close(); throw error;}
}
function sourceState(root, revision) {
  const env = Object.fromEntries(Object.entries(process.env).filter(([name]) => !name.startsWith('GIT_')));
  Object.assign(env, {GIT_NO_REPLACE_OBJECTS: '1', GIT_OPTIONAL_LOCKS: '0', GIT_TERMINAL_PROMPT: '0', GIT_NO_LAZY_FETCH: '1'});
  const git = args => execFileSync('/usr/bin/git', ['--no-replace-objects', '-c', 'core.fsmonitor=false',
    '-c', 'core.hooksPath=/dev/null', '-c', 'protocol.allow=never', ...args], {cwd: root, env, encoding: 'utf8', timeout: 10000, maxBuffer: MiB}).trim();
  assert.equal(git(['rev-parse', '--show-toplevel']), root, 'exact source worktree required');
  assert.equal(git(['rev-parse', 'HEAD']), revision, 'source revision differs');
  assert.equal(git(['status', '--porcelain']), '', 'clean source worktree required');
  return git(['rev-parse', 'HEAD^{tree}']);
}
function targetFor(root, output, fresh) {
  const target = join(root, 'target');
  if (!lstatSync(target, {throwIfNoEntry: false})) mkdirSync(target, {mode: 0o755});
  assert.equal(realpathSync(target), target);
  output = resolve(output); assert.equal(output, join(target, 'source-vv-evidence'));
  if (fresh) mkdirSync(output, {mode: 0o755});
  assert.equal(realpathSync(output), output); return {target, output};
}
function writeOriginal(path, bytes, writes) {
  writeFileSync(path, bytes, {flag: 'wx', mode: 0o444});
  const identity = lstatSync(path, {bigint: true}); writes.push({path, identity});
}
function removeOwned(writes) {
  for (const {path, identity} of writes.reverse()) {
    const current = lstatSync(path, {throwIfNoEntry: false, bigint: true});
    if (current && current.dev === identity.dev && current.ino === identity.ino && current.isFile() && current.nlink === 1n) unlinkSync(path);
  }
}
async function runPart(root, output, context, part) {
  const prefix = prefixFor(part); contextOnly(context);
  assert.equal(context.architecture, process.arch === 'x64' ? 'amd64' : process.arch);
  const tree = sourceState(root, context.source_revision), paths = targetFor(root, output, true);
  const held = holdDirectory(paths.output), inputs = [], receiptWrites = [];
  try {
    const trustedRegister = cargoInventoryParts.includes(part) ? holdOriginal(registerPath(root), 16 * MiB) : null;
    if (trustedRegister) inputs.push(trustedRegister);
    const record = join(paths.target, `vv-part-${part}.json`);
    const inventoryPath = join(paths.target, `vv-inventory-${part}`);
    absent(record); absent(inventoryPath);
    if (part === 'source') for (const name of bootstrapNames) absent(join(paths.target, name));
    const command = await captureCommand(paths.output, prefix, 'just', ['vv-part', part], root);
    assert.equal(command.status, 0, 'original source part failed'); assert.equal(command.signal, null);
    held.verify();
    const originalVv = holdOriginal(record, 4096); inputs.push(originalVv);
    validateVvPart(originalVv.bytes, context.source_revision, part);
    writeFileSync(join(paths.output, `${prefix}-vv.json`), originalVv.bytes, {flag: 'wx', mode: 0o444});
    let bootstrap = null, inventory = null;
    if (part === 'source') {
      const records = new Map(bootstrapNames.map(name => {
        const original = holdOriginal(join(paths.target, name), 64 * MiB); inputs.push(original); return [name, original.bytes];
      }));
      validateBootstrapBytes(records);
      bootstrap = {run: 1, files: [...records].map(([name, bytes]) => {
        const path = `run-1-${name}`; writeFileSync(join(paths.output, path), bytes, {flag: 'wx', mode: 0o444}); return descriptor(path, bytes);
      })};
    }
    if (cargoInventoryParts.includes(part)) {
      const original = captureFlat(inventoryPath, {maximum: 16 * MiB, entries: 1024, total: 128 * MiB,
        namePattern: /^[A-Za-z0-9][A-Za-z0-9._-]{0,126}$/}); inputs.push(original);
      bindRegister(original.files, trustedRegister.bytes);
      validateCargoInventory(original.files, part, context.source_revision, trustedRegister.bytes);
      inventory = {files: [...original.files].map(([name, bytes]) => {
        const path = `${prefix}-inventory-${name}`; writeFileSync(join(paths.output, path), bytes, {flag: 'wx', mode: 0o444}); return descriptor(path, bytes);
      })};
    } else absent(inventoryPath);
    for (const input of inputs) input.verify();
    assert.equal(sourceState(root, context.source_revision), tree); held.verify();
    writeOriginal(join(paths.output, `${prefix}-result.json`), canonical({schema: 'prismpm/source-vv-retention/3',
      scope: 'original-source-part-with-executable-inventory-only', ...context, part,
      vv: descriptor(`${prefix}-vv.json`, originalVv.bytes), bootstrap, inventory}), receiptWrites);
    const retained = captureFlat(paths.output);
    try {
      const validated = validateSourcePart(retained.files, part, context);
      const expected = [`${prefix}.json`, `${prefix}.stdout`, `${prefix}.stderr`, `${prefix}-vv.json`, `${prefix}-result.json`,
        ...[...validated.inventory.keys()].map(name => `${prefix}-inventory-${name}`),
        ...(part === 'source' ? bootstrapNames.map(name => `run-1-${name}`) : [])];
      assert.deepEqual([...retained.files.keys()].sort(), expected.sort(), 'closed retained source part required');
      retained.verify(); for (const input of inputs) input.verify();
      assert.equal(sourceState(root, context.source_revision), tree); held.verify();
      retained.verifyIdentity(); for (const input of inputs) input.verifyIdentity();
    } finally {retained.close();}
  } catch (error) {removeOwned(receiptWrites); throw error;}
  finally {for (const input of inputs) input.close(); held.close();}
}
function assemblePartFiles(root, output, context, providerPath, selectionPath) {
  contextOnly(context); const tree = sourceState(root, context.source_revision), paths = targetFor(root, output, false);
  const writes = [], held = [];
  try {
    absent(join(paths.output, 'source-vv.json')); absent(join(paths.target, 'vv-evidence.json'));
    const raw = captureFlat(paths.output); held.push(raw);
    const trustedRegister = holdOriginal(registerPath(root), 16 * MiB); held.push(trustedRegister);
    for (const part of cargoInventoryParts) bindRegister(new Map([['conformance-register.rs',
      raw.files.get(`${prefixFor(part)}-inventory-conformance-register.rs`)]]), trustedRegister.bytes);
    const provider = holdOriginal(providerPath, 16 * MiB), selection = holdOriginal(selectionPath, 16 * MiB); held.push(provider, selection);
    assert([...raw.files.values()].reduce((size, bytes) => size + bytes.length, provider.bytes.length + selection.bytes.length) <= totalLimit);
    const receipt = assembleSourceParts(raw.files, context, JSON.parse(provider.bytes), JSON.parse(selection.bytes));
    for (const original of held) original.verify(); assert.equal(sourceState(root, context.source_revision), tree);
    for (const original of held) original.verifyIdentity();
    writeOriginal(join(paths.output, 'source-vv.json'), receipt, writes);
    writeOriginal(join(paths.target, 'vv-evidence.json'), receipt, writes);
    raw.verify(['source-vv.json']); provider.verify(); selection.verify();
    for (const {path, identity} of writes) {sameIdentity(lstatSync(path, {bigint: true}), identity); assert(readOriginal(path, 4096).equals(receipt));}
    assert.equal(sourceState(root, context.source_revision), tree);
    raw.verifyIdentity(['source-vv.json']); trustedRegister.verifyIdentity(); provider.verifyIdentity(); selection.verifyIdentity();
    return receipt;
  } catch (error) {removeOwned(writes); throw error;}
  finally {for (const original of held) original.close();}
}
export async function sourcePartCommand(argv) {
  const [command, rootPath, output, revision, architecture, image, runId, attempt, ...args] = argv;
  assert(['source-part-run', 'source-part-assemble'].includes(command));
  const root = resolve(rootPath); assert.equal(realpathSync(root), root, 'source root alias');
  const context = releaseContext(revision, architecture, image === '-' ? null : image, runId, attempt);
  contextOnly(context);
  if (command === 'source-part-run') {assert.equal(args.length, 1); await runPart(root, output, context, args[0]);}
  else {assert.equal(args.length, 2); assemblePartFiles(root, output, context, ...args);}
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await sourcePartCommand(process.argv.slice(2));
