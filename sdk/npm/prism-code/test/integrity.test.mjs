// A holo whose bytes do not match its own digests is refused when it is opened, not only when it is checked (#583).
// Before, load() returned the application with identities.verified false and dispatch answered, and
// `vectors` passed a holo whose manifest had been changed.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { load, loadArchive, parseArchive, resolveArchive, VerificationError } from '../src/index.mjs';
import { fromFile } from '../src/node.mjs';

const bin = fileURLToPath(new URL('../bin/prism-code.mjs', import.meta.url));
const fixture = fileURLToPath(new URL('../fixtures/Calculator.holo', import.meta.url));
const original = new Uint8Array(readFileSync(fixture));
const encode = (text) => new TextEncoder().encode(text);
const decode = (bytes) => new TextDecoder().decode(bytes);

/** The fixture with one bit flipped at `at`. */
const flipped = (at) => {
  const bytes = original.slice();
  bytes[at] ^= 1;
  return bytes;
};
const APP_MANIFEST_KIND = 15;
/** A byte inside the application manifest, the first section: the archive still parses and the guest still compiles. */
const manifestFlip = () => {
  const section = parseArchive(original).sections.find((s) => s.kind === APP_MANIFEST_KIND);
  assert.ok(section, 'the fixture has an application manifest');
  return flipped(section.offset + 98);
};
/** A byte inside the model document's content, which holds the acceptance vectors. */
const modelFlip = () => {
  const model = parseArchive(original).blobs.find((b) => b.label === resolveArchive(parseArchive(original)).identities.modelKappa);
  return flipped(model.content.byteOffset - original.byteOffset + 104);
};

const refused = (error) => {
  assert.ok(error instanceof VerificationError, `${error?.name}: ${error?.message}`);
  assert.equal(error.check, 'holo.integrity');
  assert.match(error.message, /^the holo does not verify, so it was changed after it was built: /);
  assert.match(error.message, /allowUnverified: true/);
  assert.ok(Array.isArray(error.details.failures) && error.details.failures.length > 0);
  return true;
};

test('the untouched fixture still loads and verifies', async () => {
  const app = await loadArchive(original);
  assert.equal(app.identities.verified, true);
  assert.equal(decode(await app.dispatch(encode('1\tadd\t20\t22'))), '1\tok\t42');
});

test('a holo with a changed manifest is refused by load, loadArchive and fromFile', async () => {
  const bytes = manifestFlip();
  await assert.rejects(loadArchive(bytes), refused);
  await assert.rejects(load(bytes), refused);
  const path = join(mkdtempSync(join(tmpdir(), 'prism-integrity-')), 'changed.holo');
  writeFileSync(path, bytes);
  await assert.rejects(fromFile(path), refused);
  await assert.rejects(load(path), refused);
});

test('a changed model document is refused: its vectors no longer say what the build said', async () => {
  await assert.rejects(loadArchive(modelFlip()), (error) => refused(error) && /section \d+ is labelled blake3:[0-9a-f]{64} but its content hashes to blake3:[0-9a-f]{64}/.test(error.message));
});

test('a changed guest is refused as holo.integrity before it is compiled, not as a compile error', async () => {
  const guest = parseArchive(original).blobs.find((b) => b.label.startsWith('blake3:2adca3e7'));
  const at = guest.content.byteOffset - original.byteOffset + 100;
  await assert.rejects(loadArchive(flipped(at)), refused);
});

test('allowUnverified opens a changed holo, with identities.verified false, to examine it', async () => {
  const app = await loadArchive(manifestFlip(), { allowUnverified: true });
  assert.equal(app.identities.verified, false);
  const path = join(mkdtempSync(join(tmpdir(), 'prism-integrity-')), 'changed.holo');
  writeFileSync(path, manifestFlip());
  assert.equal((await fromFile(path, { allowUnverified: true })).identities.verified, false);
});

test('the CLI refuses a changed holo in vectors, history and generate; run --allow-unverified still runs it', () => {
  const dir = mkdtempSync(join(tmpdir(), 'prism-integrity-cli-'));
  const path = join(dir, 'changed.holo');
  writeFileSync(path, manifestFlip());
  const cli = (args) => spawnSync(process.execPath, [bin, ...args], { encoding: 'utf8', cwd: dir });
  for (const args of [['vectors', path], ['history', path], ['generate', path, '--out', join(dir, 'gen')]]) {
    const result = cli(args);
    assert.notEqual(result.status, 0, `${args[0]} exits non-zero`);
    assert.match(result.stderr, /holo\.integrity|the holo does not verify/, args[0]);
    assert.doesNotMatch(result.stdout, /vectors passed/, args[0]);
  }
  const ran = cli(['run', path, '1\tadd\t1\t1', '--rm', '--allow-unverified']);
  assert.equal(ran.status, 0, ran.stderr);
  assert.equal(ran.stdout, '1\tok\t2');
  assert.match(ran.stderr, /holo\.integrity/);
  const refusedRun = cli(['run', path, '1\tadd\t1\t1', '--rm', '--policy', 'pure-only']);
  assert.notEqual(refusedRun.status, 0);
  assert.equal(refusedRun.stdout, '');
});
