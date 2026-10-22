// Executable regression for the MCP 2026-07-28 server-conformance oracle.
//
// The positive control is the suite's own reference server at the locked commit
// with one line added: that file does not put the 2026-07-28 `resultType`
// discriminator on the streamed tools/call result, so 7 required scenarios fail
// without it. The planted defect is the same server with its listener opened to
// every host, which fails exactly the required dns-rebinding-protection scenario.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const here = fileURLToPath(new URL('.', import.meta.url));
const suiteRoot = process.env.PRISMPM_MCP_SUITE_ROOT ?? '/opt/prismpm/mcp-conformance';
const corpus = process.env.PRISMPM_MCP_CONFORMANCE_CORPUS
  ?? join(here, '../../standards/oracles/mcp-conformance-232a9014');
const oracle = process.env.PRISMPM_MCP_CONFORMANCE_ORACLE ?? join(here, 'mcp-conformance-server.mjs');
const upstream = readFileSync(join(corpus, 'examples/servers/typescript/everything-server.ts'), 'utf8');
const scratch = mkdtempSync(join(tmpdir(), 'mcp-conformance-test-'));

const RESULT_TYPE = {
  from: "        write({ jsonrpc: '2.0', id, result });",
  to: "        (result as any).resultType ??= 'complete';\n        write({ jsonrpc: '2.0', id, result });",
};
const OPEN_LISTENER = {
  from: 'const app = createMcpExpressApp();',
  to: "const app = createMcpExpressApp({ host: '0.0.0.0' });",
};

function replaceOnce(source, { from, to }) {
  assert.equal(source.split(from).length - 1, 1, `${from} occurs once in the locked reference server`);
  return source.replace(from, to);
}

let counter = 0;
function subject(source, manifest = {}, file = 'server.ts') {
  const directory = join(scratch, `subject-${counter++}`);
  mkdirSync(directory);
  writeFileSync(join(directory, file), source);
  writeFileSync(join(directory, 'prismpm-mcp-subject.json'), JSON.stringify({
    version: 'prismpm/mcp-server-subject/1',
    spec: '2026-07-28',
    entry: 'server.ts',
    path: '/mcp',
    modules: 'suite-fixture',
    ...manifest,
  }));
  return directory;
}

function run(directory, root = suiteRoot) {
  return spawnSync(process.execPath, [oracle, '--suite-root', root, directory], { encoding: 'utf8', timeout: 300000 });
}

test('the locked reference server, with the resultType line, passes every required scenario', () => {
  const result = run(subject(replaceOnce(upstream, RESULT_TYPE)));
  assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
  const report = JSON.parse(result.stdout);
  assert.equal(report.valid, true);
  assert.equal(report.scored.scenarios, 37);
  assert.deepEqual(report.scored.failing, []);
  assert.equal(report.not_scored.scenarios, 13);
  assert.equal(report.not_scored.run, 13);
});

test('the locked reference server without the resultType line fails required scenarios', () => {
  const result = run(subject(upstream));
  assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
  assert.ok(JSON.parse(result.stdout).scored.failing.includes('tools-call-simple-text'));
});

test('one required scenario failing is rejected with the suite diagnostic', () => {
  const planted = replaceOnce(replaceOnce(upstream, RESULT_TYPE), OPEN_LISTENER);
  const result = run(subject(planted));
  assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
  assert.deepEqual(JSON.parse(result.stdout).scored.failing, ['dns-rebinding-protection']);
  assert.match(result.stderr, /^dns-rebinding-protection: /m);
});

test('a subject that never listens is rejected without running the suite', () => {
  const result = run(subject('process.exit(3);\n'));
  assert.equal(result.status, 1);
  assert.match(result.stderr, /exited before listening/);
});

test('an unusable subject manifest is rejected', () => {
  const source = 'setInterval(() => {}, 1000);\n';
  for (const [manifest, needle] of [
    [{ spec: '2025-11-25' }, '2026-07-28'],
    [{ entry: '../escape.mjs' }, 'relative'],
    [{ modules: 'anything' }, 'modules'],
    [{ entry: 'server.mjs', modules: 'none' }, 'not a file'],
    [{ path: 'mcp' }, 'absolute URL path'],
  ]) {
    const result = run(subject(source, manifest));
    assert.equal(result.status, 1, JSON.stringify(manifest));
    assert.ok(result.stderr.includes(needle), `${JSON.stringify(manifest)}: ${result.stderr}`);
  }
});

test('requirements that differ from the frozen file stop the oracle before the subject starts', () => {
  const root = join(scratch, 'altered-suite');
  mkdirSync(join(root, 'requirements'), { recursive: true });
  const frozen = readFileSync(join(corpus, 'requirements/2026-07-28.yaml'), 'utf8');
  writeFileSync(join(root, 'requirements/2026-07-28.yaml'), frozen.replace('  - tools-list\n', ''));
  writeFileSync(join(root, 'package.json'), '{}');
  const result = run(subject('setInterval(() => {}, 1000);\n', { modules: 'none', entry: 'server.mjs' }, 'server.mjs'), root);
  assert.equal(result.status, 2);
  assert.match(result.stderr, /differ from the frozen file/);
});

test('an absent suite is a harness fault, not a verdict on the subject', () => {
  const result = run(subject('setInterval(() => {}, 1000);\n'), join(scratch, 'no-suite'));
  assert.equal(result.status, 2);
});
