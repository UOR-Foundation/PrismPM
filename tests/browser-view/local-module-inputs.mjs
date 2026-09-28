// Test-only static ESM closure. Parse declarations without executing modules.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {closeSync, fstatSync, lstatSync, openSync, readFileSync, realpathSync} from 'node:fs';
import {dirname, relative, resolve} from 'node:path';

const parser = 'import {SourceTextModule} from "node:vm";import {readFileSync} from "node:fs";'
  + 'const source=readFileSync(0,"utf8");const module=new SourceTextModule(source);'
  + 'process.stdout.write(JSON.stringify(module.dependencySpecifiers));';
const parserArguments = Object.freeze(['--experimental-vm-modules', '--input-type=module', '-e', parser]);
const parserEnvironment = Object.freeze({NODE_NO_WARNINGS: '1', TZ: 'UTC'});
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const staticImports = new Map();
let cachedBytes = 0;
// Eviction affects only parsing cost, never the admitted source/file domain.
const CACHE_BYTES = 16777216, CACHE_ENTRIES = 256;

function parserIdentity() {
  const path = realpathSync(process.execPath), descriptor = openSync(path, 'r');
  try {
    const before = fstatSync(descriptor, {bigint: true});
    assert.ok(before.isFile() && before.size > 0n && before.size <= 268435456n
      && (before.mode & 0o111n) !== 0n, 'bounded executable static ESM parser');
    const bytes = readFileSync(descriptor), after = fstatSync(descriptor, {bigint: true});
    const identity = stat => [stat.dev, stat.ino, stat.mode, stat.nlink, stat.size, stat.mtimeNs, stat.ctimeNs].map(String);
    assert.deepEqual(identity(after), identity(before), 'static ESM parser changed while captured');
    assert.deepEqual(identity(lstatSync(path, {bigint: true})), identity(before), 'static ESM parser path replaced');
    assert.equal(BigInt(bytes.length), before.size);
    return Object.freeze({path, key: digest(JSON.stringify({identity: identity(before), executable: digest(bytes),
      versions: process.versions, arguments: parserArguments, environment: parserEnvironment}))});
  } finally {closeSync(descriptor);}
}

function dependencies(identity, bytes, path) {
  const key = identity.key + ':' + digest(bytes), cached = staticImports.get(key);
  if (cached && cached.bytes.equals(bytes)) {
    staticImports.delete(key); staticImports.set(key, cached);
    return cached.imports;
  }
  const parsed = spawnSync(identity.path, parserArguments,
    {input: bytes, encoding: 'utf8', env: parserEnvironment, timeout: 10000, maxBuffer: 1048576});
  assert.equal(parsed.error, undefined); assert.equal(parsed.signal, null);
  assert.equal(parsed.status, 0, 'actual static ESM parsing: ' + path + '\n' + parsed.stderr);
  const imports = JSON.parse(parsed.stdout);
  assert.ok(Array.isArray(imports) && imports.every(value => typeof value === 'string'));
  const result = Object.freeze(imports);
  if (cached) {staticImports.delete(key); cachedBytes -= cached.bytes.length;}
  staticImports.set(key, {bytes: Buffer.from(bytes), imports: result}); cachedBytes += bytes.length;
  while (staticImports.size > CACHE_ENTRIES || cachedBytes > CACHE_BYTES) {
    const first = staticImports.keys().next().value;
    cachedBytes -= staticImports.get(first).bytes.length; staticImports.delete(first);
  }
  return result;
}

export function localModuleInputs(repository, entries, readSource) {
  repository = resolve(repository);
  const captured = new Map(), pending = [...entries], identity = parserIdentity();
  while (pending.length) {
    const path = pending.pop();
    assert.equal(typeof path, 'string');
    assert.ok(path.endsWith('.mjs') && path.split('/').every(part => /^[A-Za-z0-9_.-]+$/.test(part)
      && part !== '.' && part !== '..'), 'closed local module path');
    if (captured.has(path)) continue;
    const bytes = readSource(path);
    assert.ok(Buffer.isBuffer(bytes) && bytes.length <= 4194304, 'bounded captured module buffer');
    const source = new TextDecoder('utf-8', {fatal: true}).decode(bytes);
    // A cache hit never suppresses source admission, file reading or resolution.
    const dynamicImport = new RegExp('\\bim' + 'port(?:\\s|/\\*[\\s\\S]*?\\*/|//[^\\n]*(?:\\n|$))*\\(');
    assert.ok(!dynamicImport.test(source), 'dynamic imports require a separately registered owning closure');
    const imports = dependencies(identity, bytes, path);
    captured.set(path, bytes);
    for (const specifier of imports) {
      if (specifier.startsWith('node:')) continue;
      assert.ok(specifier.startsWith('./') || specifier.startsWith('../'), 'no undeclared package or remote import');
      const selected = relative(repository, resolve(repository, dirname(path), specifier));
      assert.ok(!selected.startsWith('../') && selected !== '..' && !selected.startsWith('/'), 'module import cannot escape repository');
      pending.push(selected);
    }
  }
  assert.deepEqual(parserIdentity(), identity, 'static ESM parser changed during closure capture');
  return new Map([...captured].sort(([left], [right]) => left < right ? -1 : left > right ? 1 : 0));
}
