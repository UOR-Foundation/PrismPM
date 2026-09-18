import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { chmodSync, cpSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, renameSync, rmdirSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import test from 'node:test';
import { inventoryRuntimeDigest, runOfficial, runtimeTreeDigest, verifyRuntime } from './launcher.mjs';

const installed = resolve(process.env.PRISMPM_ASYNCAPI_TEST_RUNTIME ?? '/opt/prismpm/asyncapi-official/scripts');
// Explicit test-only pristine capture, never a production inventory substitute.
const pristineDigest = runtimeTreeDigest(installed);

function disposableCopy(source, destination) {
  // Installed SDK directories are read-only. Only our disposable mutation
  // fixture needs writable directories; the real installation stays intact.
  mkdirSync(destination, {recursive: true, mode: 0o700});
  for (const name of readdirSync(source)) {
    const input = join(source, name), output = join(destination, name);
    if (lstatSync(input).isDirectory()) disposableCopy(input, output);
    else cpSync(input, output, {verbatimSymlinks: true});
  }
}

test('the owned runtime runs all 89 unchanged upstream examples with parser 3.6.0', () => {
  const result = verifyRuntime(installed, pristineDigest);
  assert.equal(result.parser, '@asyncapi/parser/3.6.0');
  assert.equal(result.runtimeLock, '5bd20ce206d3b3b76a7034951c9e19e15291c54eec0a1424139b465c980f1205');
  const output = runOfficial(installed, pristineDigest).toString('utf8');
  assert.equal(output.split('\n').filter(line => line.endsWith(' is valid.')).length, 89);
  assert.ok(output.includes('Number of examples extracted: 89'));
  assert.doesNotMatch(output, /Validation failed|Error in /);
});

test('the owning launcher rejects changed source, lock, parser, missing modules and shadow resolution', () => {
  const scratch = mkdtempSync(join(tmpdir(), 'prismpm-asyncapi-runtime-'));
  try {
    const root = join(scratch, 'official/scripts');
    disposableCopy(dirname(installed), dirname(root));
    // Mutations affect only a disposable copy of the actually installed graph.
    const script = `import {verifyRuntime} from ${JSON.stringify(new URL('./launcher.mjs', import.meta.url).href)};\ntry { process.stdout.write(JSON.stringify(verifyRuntime(process.argv[1], process.argv[2]))); } catch (error) { process.stderr.write(error.message); process.exitCode = 1; }`;
    const invoke = (environment = {}) => execFileSync(process.execPath, ['--input-type=module', '-e', script, root, pristineDigest],
      {stdio: 'pipe', encoding: 'utf8', env: {...process.env, ...environment}});
    assert.match(invoke(), /@asyncapi\/parser\/3\.6\.0/);
    const rewrite = (relative, change, expected) => {
      const path = join(root, relative), original = readFileSync(path);
      chmodSync(path, 0o600);
      try {
        writeFileSync(path, change(original));
        assert.throws(() => invoke(), error => error.status !== 0 && expected.test(error.stderr));
      } finally { writeFileSync(path, original); }
    };
    rewrite('validation/embedded-examples-validation.js', bytes => Buffer.concat([bytes, Buffer.from('\n')]), /upstream source digest/);
    rewrite('validation/package-lock.json', () => '{}', /upstream source digest/);
    rewrite('package-lock.json', () => '{}', /owned runtime lock digest/);
    rewrite('package.json', () => '{}', /owned runtime manifest digest/);
    rewrite('node_modules/@asyncapi/parser/package.json', bytes => JSON.stringify({...JSON.parse(bytes), version: '3.6.3'}), /installed package version/);
    const parserEntry = JSON.parse(readFileSync(join(root, 'node_modules/@asyncapi/parser/package.json'))).main;
    rewrite(`node_modules/@asyncapi/parser/${parserEntry}`, bytes => Buffer.concat([bytes, Buffer.from('\n')]), /installed runtime byte digest/);
    const parser = join(root, 'node_modules/@asyncapi/parser');
    renameSync(parser, join(scratch, 'removed-parser'));
    try { assert.throws(() => invoke(), error => /installed package closure/.test(error.stderr)); }
    finally { renameSync(join(scratch, 'removed-parser'), parser); }
    for (const location of [join(root, 'validation/node_modules'), join(dirname(root), 'node_modules')]) {
      assert.equal(existsSync(location), false);
      mkdirSync(location);
      try { assert.throws(() => invoke(), error => /shadow module directory/.test(error.stderr)); }
      finally { rmdirSync(location); }
    }
    assert.throws(() => invoke({NODE_PATH: scratch}), error => /ambient module configuration/.test(error.stderr));
    assert.throws(() => invoke({NODE_OPTIONS: '--no-warnings'}), error => /ambient module configuration/.test(error.stderr));
    const foreign = join(root, 'node_modules/unregistered-package'); mkdirSync(foreign);
    try {
      writeFileSync(join(foreign, 'package.json'), '{"name":"unregistered-package","version":"1.0.0"}');
      assert.throws(() => invoke(), error => /installed package closure/.test(error.stderr));
    } finally { rmSync(foreign, {recursive: true}); }
    assert.equal(runOfficial(root, pristineDigest).toString('utf8').split('\n')
      .filter(line => line.endsWith(' is valid.')).length, 89, 'restored real upstream replay must pass');
  } finally { rmSync(scratch, {recursive: true, force: true}); }
});

test('fixed inventory binding rejects absent, duplicate, changed, extra-field or noncanonical runtime rows', () => {
  const row = {digest: pristineDigest, id: 'asyncapi-embedded-runtime', kind: 'oracle', version: '@asyncapi/parser/3.6.0'};
  const document = artifacts => Buffer.from(`${JSON.stringify({artifacts, commands: [], schema: 'prismpm/sdk-inventory/1'})}\n`);
  assert.equal(inventoryRuntimeDigest(document([row])), pristineDigest);
  for (const rows of [[], [row, row], [{...row, id: 'other'}], [{...row, kind: 'dependency-lock'}],
    [{...row, version: '@asyncapi/parser/3.6.3'}], [{...row, digest: 'latest'}], [{...row, path: '/tmp/runtime'}]]) {
    assert.throws(() => inventoryRuntimeDigest(document(rows)));
  }
  assert.throws(() => inventoryRuntimeDigest(Buffer.from(document([row]).toString().replace('sdk-inventory/1', 'sdk-inventory/2'))));
  assert.throws(() => inventoryRuntimeDigest(document([row]).subarray(0, -1)));
  assert.throws(() => inventoryRuntimeDigest(Buffer.from(document([row]).toString()
    .replace('"id":"asyncapi-embedded-runtime"', '"id":"other","id":"asyncapi-embedded-runtime"'))));
  const invalidUtf8 = document([row]).toString().replace('"commands":[]', '"commands":["INVALID_UTF8"]');
  const [before, after] = invalidUtf8.split('INVALID_UTF8');
  assert.throws(() => inventoryRuntimeDigest(Buffer.concat([Buffer.from(before), Buffer.from([0xff]), Buffer.from(after)])), /not canonical/);
  const scratch = mkdtempSync(join(tmpdir(), 'prismpm-asyncapi-relocated-'));
  try {
    cpSync(new URL('./launcher.mjs', import.meta.url), join(scratch, 'launcher.mjs'));
    assert.throws(() => execFileSync(process.execPath, [join(scratch, 'launcher.mjs'), '--check'],
      {stdio: 'pipe'}), error => /fixed installed runtime path/.test(error.stderr.toString()));
  } finally { rmSync(scratch, {recursive: true, force: true}); }
});

test('runtime byte framing matches inventory and allows only in-tree executable aliases', () => {
  const scratch = mkdtempSync(join(tmpdir(), 'prismpm-asyncapi-tree-'));
  try {
    const modules = join(scratch, 'node_modules'); mkdirSync(join(modules, '.bin'), {recursive: true});
    mkdirSync(join(modules, 'fixture')); writeFileSync(join(modules, 'fixture/cli.js'), 'fixture bytes');
    symlinkSync('../fixture/cli.js', join(modules, '.bin/tool'));
    const expected = createHash('sha256');
    for (const path of ['.bin/tool', 'fixture/cli.js']) {
      const name = Buffer.from(path), content = Buffer.from('fixture bytes');
      const nameLength = Buffer.alloc(8); nameLength.writeBigUInt64BE(BigInt(name.length));
      const byteLength = Buffer.alloc(8); byteLength.writeBigUInt64BE(BigInt(content.length));
      expected.update(nameLength).update(name).update(byteLength).update(content);
    }
    assert.equal(runtimeTreeDigest(scratch), `sha256:${expected.digest('hex')}`);
    rmSync(join(modules, '.bin/tool'));
    writeFileSync(join(scratch, 'outside.js'), 'fixture bytes');
    symlinkSync('../../outside.js', join(modules, '.bin/tool'));
    assert.throws(() => runtimeTreeDigest(scratch), /escapes installed graph/);
    rmSync(join(modules, '.bin/tool')); symlinkSync('tool', join(modules, '.bin/tool'));
    assert.throws(() => runtimeTreeDigest(scratch));
  } finally { rmSync(scratch, {recursive: true, force: true}); }
});

test('the shell entry rejects ambient preloads before any Node code can execute', () => {
  const scratch = mkdtempSync(join(tmpdir(), 'prismpm-asyncapi-preload-'));
  try {
    const guard = readFileSync(new URL('./launcher.sh', import.meta.url), 'utf8');
    const javascript = readFileSync(new URL('./launcher.mjs', import.meta.url));
    assert.ok(guard.includes(createHash('sha256').update(javascript).digest('hex')),
      'inventoried shell must authenticate its fixed JavaScript bytes before loading');
    assert.ok(guard.indexOf('/usr/bin/sha256sum --check --strict') < guard.indexOf('exec /usr/local/bin/node'));
    const marker = join(scratch, 'preload-ran'), preload = join(scratch, 'preload.cjs');
    writeFileSync(preload, `require('node:fs').writeFileSync(${JSON.stringify(marker)}, 'unsafe');`);
    for (const environment of [{NODE_OPTIONS: `--require ${preload}`}, {NODE_PATH: scratch}]) {
      assert.throws(() => execFileSync('/bin/sh', [new URL('./launcher.sh', import.meta.url).pathname, '--check'],
        {stdio: 'pipe', env: {...process.env, ...environment}}),
      error => /ambient module configuration/.test(error.stderr.toString()));
      assert.equal(existsSync(marker), false);
    }
  } finally { rmSync(scratch, {recursive: true, force: true}); }
});
