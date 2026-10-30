// The authoring front door (issue #325). The agent and the toolchain are test
// doubles here: this file proves the gates and the bookkeeping around them, not
// that a real agent writes a model. A test's name starts with the check id that
// docs/proposals/325-authoring-front-door.md maps to a MUST of the design.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { spawnSync } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { fromFile } from '../src/node.mjs';
import { inspect } from '../src/inspect.mjs';
import { authorBuild, modelState, replayExamplesGate, vectorMismatch } from '../src/author/build.mjs';
import { authorRequest, modelPathAllowed, processAgent, validateResponse, RESPONSE_SCHEMA } from '../src/author/agent.mjs';
import { checkAuthoring, compareHolos } from '../src/author/check.mjs';
import { falsified, framedDigest, loadSpec, modelDigest, parseExample, parseIntent, readModel, specDigest, theoremName } from '../src/author/spec.mjs';
import { intentVerdicts } from '../src/author/toolchain.mjs';
import { messageRequest, responseFromMessage } from '../src/author/anthropic.mjs';

const here = (path) => fileURLToPath(new URL(path, import.meta.url));
const FIXTURES = here('./fixtures/authoring/');
const HOLO = here('../fixtures/Calculator.holo');
const BIN = here('../bin/prism-code.mjs');
const MODEL_SHA256 = '074921890f5c372c89f4ed794d26dd550231d5e9bf99bf9474cc1a78923292c6';
const IMAGE = `ghcr.io/uor-foundation/prismpm-sdk@sha256:${'c'.repeat(64)}`;
const RECORD = 'Foundation.Bytes::Bytes';
const scaffold = new Map([['lexlean.toml', new TextEncoder().encode('spec = "lexlean/project/1"\n')]]);
const bytes = (text) => new TextEncoder().encode(text);

function project(name = 'calculator') {
  const root = mkdtempSync(join(tmpdir(), 'prism-author-'));
  cpSync(join(FIXTURES, name), root, { recursive: true });
  writeFileSync(join(root, 'prismpm.lock'), JSON.stringify({ schema: 'prismpm/sdk-lock/2', sdk_image: IMAGE }));
  return root;
}

function put(root, path, content) {
  mkdirSync(dirname(join(root, path)), { recursive: true });
  writeFileSync(join(root, path), content);
}

/** The pinned toolchain's effects on a project, without the toolchain. `theorems` picks what the kernel says. */
function fakeToolchain(root, { fail = {}, theorems = (spec) => spec.intent.map((row) => ({ name: `App.Model.${row.theorem}`, result: 'ok', observed: [], policy: { kind: 'none', axioms: [] } })), holo = HOLO } = {}) {
  const calls = [];
  return {
    calls,
    run: async (command, args = []) => {
      calls.push([command, ...args].join(' '));
      if (fail[command]) return { code: fail[command].code ?? 1, output: fail[command].output ?? `${command} failed` };
      const spec = loadSpec(root);
      if (command === 'build') {
        cpSync(holo, join(root, '.prism/build/b/App.holo'));
        put(root, '.prism/build/b/manifest.json', JSON.stringify({ inputs: { model_id: MODEL_SHA256 }, files: [{ path: 'App.holo' }] }));
        if (args.includes('--update')) {
          const lock = JSON.parse(readFileSync(join(root, 'prismpm.lock'), 'utf8'));
          writeFileSync(join(root, 'prismpm.lock'), JSON.stringify({ ...lock, schema: 'prismpm/sdk-lock/3', inputs: { model: { sha256: MODEL_SHA256 }, spec: { basis: 'prism-toml', blake3: spec.digest } } }));
        }
      }
      if (command === 'verify') {
        put(root, '.prism/verified/v/manifest.json', JSON.stringify({ model_sha256: MODEL_SHA256 }));
        put(root, '.prism/verified/v/lexlean-attestation.json', JSON.stringify({ status: 'verified', declarations: theorems(spec) }));
      }
      return { code: 0, output: '' };
    },
  };
}

/** An agent that proves every sentence, or does what `answer` says. */
function fakeAgent(answer = null) {
  const requests = [];
  return {
    adapter: 'test',
    requests,
    turn: async (request) => {
      requests.push(request);
      if (answer) return answer(request, requests.length);
      return {
        schema: RESPONSE_SCHEMA,
        files: [{ path: 'src/App.lex.tex', text: `% model ${requests.length} uses ${RECORD}\n` }],
        sentences: request.spec.intent.map(({ theorem }) => ({ theorem, status: 'proved', statement: 'a statement' })),
        types: request.spec.types.map(({ path }) => ({ source: `${path}#Request`, record: RECORD })),
        usage: { total_tokens: 10 },
      };
    },
  };
}

const build = (root, agent, toolchain, options = {}) => authorBuild({ root, agent, toolchain, scaffold, scaffoldContext: {}, ...options });

test('author.spec.digest: the digest is the one the Rust lock computes', () => {
  // The reference vectors of spec_digest_matches_the_reference_vectors_and_ignores_insertion_order in input_lock.rs.
  assert.equal(specDigest(new Map()), '38270f9a684133fd4a9e08731bd9d24b9e90cc92eb757b6fb548ca33589a361b');
  const files = new Map([['b.lex.tex', bytes('B')], ['a/prismpm.toml', bytes('A\n')]]);
  assert.equal(specDigest(files), 'fc0cc56804659d03cdf3e6a74ab4053a7037a24b4f8c24ab02c050b0c9bad798');
  assert.notEqual(modelDigest(files), specDigest(files), 'a model digest is domain separated');
  assert.equal(framedDigest('x\n', new Map([['a', bytes('1')], ['b', bytes('2')]])), framedDigest('x\n', new Map([['b', bytes('2')], ['a', bytes('1')]])));
});

test('author.spec.digest: the Calculator fixture has the digest the Rust test pins', () => {
  // the_calculator_authoring_fixture_has_the_digest_javascript_pins in input_lock.rs. Stated bytes: eol=lf in .gitattributes.
  assert.equal(loadSpec(join(FIXTURES, 'calculator')).digest, '5487513160c873eb0dcf02a42146fc8bbadc06d1d701ea2b08a3169d4c814d1a');
});

test('author.spec.digest: prism.toml, intent.md, examples and types are covered, and nothing else', () => {
  const root = project();
  const digest = () => loadSpec(root).digest;
  let last = digest();
  for (const [path, change] of [
    ['prism.toml', `${readFileSync(join(root, 'prism.toml'), 'utf8')}\n`],
    ['intent.md', `${readFileSync(join(root, 'intent.md'), 'utf8')}\n`],
    ['types/calculator.ts', '// changed\n'],
    ['examples/99-more.json', '{"input":"a","output":"b"}'],
  ]) {
    writeFileSync(join(root, path), change);
    assert.notEqual(digest(), last, path);
    last = digest();
  }
  for (const path of ['README.md', 'model/src/App.lex.tex', 'prismpm.toml']) {
    put(root, path, 'not the spec');
    assert.equal(digest(), last, path);
  }
  rmSync(root, { recursive: true });
});

test('author.intent.theorem: a sentence maps to one deterministic theorem name', () => {
  const [row] = parseIntent('# heading\n\nadd never overflows when the sum fits.\n');
  assert.equal(row.line, 3);
  assert.equal(row.kind, 'never');
  assert.equal(row.theorem, theoremName('add never overflows when the sum fits.'));
  assert.match(row.theorem, /^intent_add_never_overflows_when_the_sum_[0-9a-f]{8}$/);
  assert.notEqual(theoremName('add never overflows.'), theoremName('add never overflow.'));
});

test('author.intent.grammar: one sentence, one keyword, a period, no repeats', () => {
  for (const [text, message] of [
    ['add is commutative.', /exactly one of always, never or only/],
    ['add always holds and never fails.', /exactly one of/],
    ['add never fails', /ends with a period/],
    ['add never fails. sub never fails.', /one sentence per line/],
    ['add never fails.\nadd never fails.', /repeats line 1/],
  ]) {
    assert.throws(() => parseIntent(text), (error) => error.code === 'author.intent' && message.test(error.message), text);
  }
  assert.equal(parseIntent('calculate never returns `always only never`.')[0].kind, 'never', 'a literal holds no keyword');
});

test('author.examples.wire: a string is its bytes, hex is its bytes, anything else is canonical JSON', () => {
  const text = (value) => new TextDecoder().decode(value);
  assert.equal(text(parseExample('examples/a.json', bytes('{"input":"1\\tadd","output":""}')).request), '1\tadd');
  assert.deepEqual([...parseExample('examples/a.json', bytes('{"input":{"hex":"ff00"},"output":"x"}')).request], [0xff, 0]);
  assert.equal(text(parseExample('examples/a.json', bytes('{"input":{"b":1,"a":2},"output":5}')).request), '{"a":2,"b":1}');
  assert.equal(text(parseExample('examples/a.json', bytes('{"input":{"b":1,"a":2},"output":5}')).response), '5');
  for (const [path, content] of [
    ['examples/a.txt', '{}'],
    ['examples/a.json', 'not json'],
    ['examples/a.json', '{"input":"a"}'],
    ['examples/a.json', '{"input":"a","output":"b","extra":1}'],
    ['examples/a.json', '{"input":{"hex":"XY"},"output":"b"}'],
  ]) {
    assert.throws(() => parseExample(path, bytes(content)), (error) => error.code === 'author.example', content);
  }
});

test('author.examples.vectors: the Calculator examples are byte for byte the reference acceptance vectors', async () => {
  const spec = loadSpec(join(FIXTURES, 'calculator'));
  const reference = await fromFile(HOLO);
  assert.equal(spec.examples.length, 22);
  assert.equal(vectorMismatch(spec.examples, reference.vectors), null);
  const changed = [...spec.examples];
  changed[3] = { ...changed[3], response: bytes('1\tok\t0') };
  assert.equal(vectorMismatch(changed, reference.vectors).example, spec.examples[3].path);
  assert.match(vectorMismatch(spec.examples.slice(1), reference.vectors).reason, /carries 22 acceptance vectors/);
});

test('author.intent.false: the shipped sentences of both reference specs hold on their examples', () => {
  for (const name of ['calculator', 'payment-approval']) {
    const spec = loadSpec(join(FIXTURES, name));
    assert.deepEqual(falsified(spec.intent, spec.examples), [], name);
  }
});

test('author.intent.false: always, never and only each refute on the examples', () => {
  const examples = [{ path: 'examples/a.json', request: bytes('divide 1 0'), response: bytes('error') }];
  const wrong = (sentence) => falsified(parseIntent(sentence), examples).map((row) => row.example);
  assert.deepEqual(wrong('calculate never returns `error`.'), ['examples/a.json']);
  assert.deepEqual(wrong('calculate always returns `ok`.'), ['examples/a.json']);
  assert.deepEqual(wrong('calculate returns `error` only when the input contains `mod`.'), ['examples/a.json']);
  assert.deepEqual(wrong('calculate never returns `error` when the input contains `add`.'), [], 'the condition does not apply');
  assert.deepEqual(wrong('calculate returns `error` only when the input contains `divide`.'), []);
  assert.deepEqual(wrong('calculate never returns a wrong result.'), [], 'plain prose is for the kernel');
  assert.deepEqual(falsified(parseIntent('x never returns `\\t`.'), [{ path: 'e', request: bytes(''), response: bytes('a\tb') }]).length, 1, 'a literal may escape a tab');
});

test('author.intent.false: the negative fixture fails the build by name and never calls the agent', async () => {
  const root = project('negative-intent');
  const agent = fakeAgent();
  const result = await build(root, agent, fakeToolchain(root));
  assert.equal(result.code, 1);
  assert.match(result.text, /calculate never returns `error`\.\s+FALSE/);
  assert.match(result.text, /examples\/13-1-divide-1-0\.json/);
  assert.match(result.text, /The agent was not called/);
  assert.equal(agent.requests.length, 0);
  assert.equal(existsSync(join(root, 'model')), false);
  rmSync(root, { recursive: true });
});

test('author.intent.false: the negative fixture through the command line needs no lock and no Docker', () => {
  const run = spawnSync(process.execPath, [BIN, 'build', '--unstable-author', '--project', join(FIXTURES, 'negative-intent')], { encoding: 'utf8' });
  assert.equal(run.status, 1);
  assert.match(run.stderr, /calculate never returns `error`\./);
});

test('author.lock.record: a passing build pins spec and model next to the agent, in a lock the schema admits', async () => {
  const root = project();
  const agent = fakeAgent();
  const toolchain = fakeToolchain(root);
  const result = await build(root, agent, toolchain, { model: 'a-model-id' });
  assert.equal(result.code, 0, result.text);
  const lock = JSON.parse(readFileSync(join(root, 'prismpm.lock'), 'utf8'));
  const spec = loadSpec(root);
  assert.deepEqual(lock.authoring, { agent: { adapter: 'test', model: 'a-model-id' }, model_tree: { blake3: modelDigest(readModel(root)) }, spec: { blake3: spec.digest } });
  assert.equal(lock.inputs.spec.blake3, lock.authoring.spec.blake3, 'next to the model digest, and equal to what the toolchain computed');
  assert.deepEqual(toolchain.calls, ['check', 'build --update', 'verify']);
  assert.equal(existsSync(join(root, '.prism-author')), false);
  const sidecar = JSON.parse(readFileSync(join(root, 'model/author.json'), 'utf8'));
  assert.equal(sidecar.sentences.length, spec.intent.length);
  assert.equal(readFileSync(join(root, 'prismpm.toml'), 'utf8').includes('lexlean_project = "model/lexlean.toml"'), true);
  rmSync(root, { recursive: true });
});

test('author.report.sentence: the report shows sentences and example counts, never a theorem name', async () => {
  const root = project();
  const result = await build(root, fakeAgent(), fakeToolchain(root));
  const spec = loadSpec(root);
  for (const row of spec.intent) {
    assert.match(result.text, new RegExp(`${row.sentence.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\s+checked`));
    assert.equal(result.text.includes(row.theorem), false);
  }
  assert.match(result.text, /22 of 22 replayed through the guest/);
  rmSync(root, { recursive: true });
});

test('author.cache.hit: an unchanged spec never calls the agent, and a changed one does', async () => {
  const root = project();
  const agent = fakeAgent();
  await build(root, agent, fakeToolchain(root));
  assert.equal(agent.requests.length, 1);
  const lockAfterFirst = readFileSync(join(root, 'prismpm.lock'), 'utf8');
  const again = await build(root, agent, fakeToolchain(root));
  assert.equal(again.code, 0, again.text);
  assert.equal(agent.requests.length, 1, 'no second call');
  assert.match(again.text, /cached from spec/);
  assert.equal(readFileSync(join(root, 'prismpm.lock'), 'utf8'), lockAfterFirst, 'a cached build leaves the lock alone');
  put(root, 'examples/99-more.json', '{"input":"a","output":"b"}');
  const stale = await build(root, agent, fakeToolchain(root), { maxIterations: 1 });
  assert.equal(agent.requests.length, 2, 'a changed spec calls it again');
  assert.equal(stale.code, 1, 'and the fixed test holo no longer matches examples/');
  rmSync(root, { recursive: true });
});

test('author.cache.hit: the pin in the lock is reused, and --no-cache is what spends a new call', async () => {
  const root = project();
  await build(root, fakeAgent(), fakeToolchain(root), { model: 'first-model' });
  const agent = fakeAgent();
  const kept = await build(root, agent, fakeToolchain(root), { model: 'second-model' });
  assert.equal(agent.requests.length, 0, 'a new pin alone does not regenerate');
  assert.match(kept.text, /written by test\/first-model/);
  await build(root, agent, fakeToolchain(root), { noCache: true });
  assert.equal(agent.requests[0].agent.model, 'first-model');
  rmSync(root, { recursive: true });
});

test('author.cache.hit: a lock update that drops the record does not spend an agent call', async () => {
  const root = project();
  await build(root, fakeAgent(), fakeToolchain(root));
  const lock = JSON.parse(readFileSync(join(root, 'prismpm.lock'), 'utf8'));
  delete lock.authoring;
  writeFileSync(join(root, 'prismpm.lock'), JSON.stringify(lock));
  const agent = fakeAgent();
  const toolchain = fakeToolchain(root);
  const result = await build(root, agent, toolchain);
  assert.equal(result.code, 0, result.text);
  assert.equal(agent.requests.length, 0);
  assert.equal(toolchain.calls.includes('build --update'), true);
  assert.notEqual(JSON.parse(readFileSync(join(root, 'prismpm.lock'), 'utf8')).authoring, undefined);
  rmSync(root, { recursive: true });
});

test('author.model.digest: a generated model edited by hand is refused, not overwritten', async () => {
  const root = project();
  await build(root, fakeAgent(), fakeToolchain(root));
  put(root, 'model/src/App.lex.tex', 'edited');
  const agent = fakeAgent();
  await assert.rejects(build(root, agent, fakeToolchain(root)), (error) => error.code === 'author.model' && /changed after it was generated/.test(error.message));
  assert.equal(agent.requests.length, 0);
  assert.equal(readFileSync(join(root, 'model/src/App.lex.tex'), 'utf8'), 'edited');
  rmSync(root, { recursive: true });
});

test('author.level3: a project with no spec files is delegated, and a hand-written model is never overwritten', async () => {
  const level3 = mkdtempSync(join(tmpdir(), 'prism-author-'));
  put(level3, 'prism.toml', '[application]\nname = "hand"\nversion = "0.1.0"\n');
  put(level3, 'model/src/Hand.lex.tex', 'by hand');
  const agent = fakeAgent();
  assert.deepEqual(await build(level3, agent, fakeToolchain(level3)), { delegate: true });
  assert.equal(agent.requests.length, 0);
  assert.equal((await checkAuthoring(HOLO, { root: level3 })).checks[0].id, 'author.level3');
  put(level3, 'intent.md', 'hand never fails.\n');
  put(level3, 'examples/a.json', '{"input":"a","output":"b"}');
  put(level3, 'prismpm.lock', JSON.stringify({ schema: 'prismpm/sdk-lock/2', sdk_image: IMAGE }));
  await assert.rejects(build(level3, agent, fakeToolchain(level3)), (error) => error.code === 'author.model' && /written by hand/.test(error.message));
  assert.equal(readFileSync(join(level3, 'model/src/Hand.lex.tex'), 'utf8'), 'by hand');
  rmSync(level3, { recursive: true });
});

test('author.spec: half a spec, another model source and a missing manifest are refused', () => {
  const root = mkdtempSync(join(tmpdir(), 'prism-author-'));
  assert.throws(() => loadSpec(root), (error) => error.code === 'author.spec');
  put(root, 'prism.toml', '[application]\nname = "a"\nversion = "0.1.0"\n');
  put(root, 'intent.md', 'a never fails.\n');
  assert.throws(() => loadSpec(root), (error) => error.code === 'author.spec' && /examples/.test(error.message));
  put(root, 'examples/a.json', '{"input":"a","output":"b"}');
  put(root, 'types/a.rs', 'struct A;');
  assert.throws(() => loadSpec(root), (error) => error.code === 'author.types');
  rmSync(join(root, 'types'), { recursive: true });
  assert.equal(loadSpec(root).level, 2);
  put(root, 'prism.toml', '[application]\nname = "a"\nversion = "0.1.0"\n[model]\nsource = "src"\n');
  assert.throws(() => loadSpec(root), (error) => error.code === 'author.spec' && /source/.test(error.message));
  rmSync(root, { recursive: true });
});

test('author.intent.unproved: the agent giving up fails the build, names the sentence, offers its explanation', async () => {
  const root = project();
  const sentence = loadSpec(root).intent[1];
  put(root, 'model/author.json', JSON.stringify({ spec: 'an older spec' }));
  put(root, 'model/keep.lex.tex', 'the old model');
  const before = readFileSync(join(root, 'prismpm.lock'), 'utf8');
  const agent = fakeAgent((request) => ({
    schema: RESPONSE_SCHEMA,
    files: [],
    sentences: request.spec.intent.map(({ theorem }) => (theorem === sentence.theorem ? { theorem, status: 'unproved', explanation: 'subtract can overflow at the lower bound' } : { theorem, status: 'proved' })),
    types: [{ source: 'types/calculator.ts#Request', record: RECORD }],
  }));
  const toolchain = fakeToolchain(root);
  const result = await build(root, agent, toolchain);
  assert.equal(result.code, 1);
  assert.match(result.text, new RegExp(`${sentence.sentence.slice(0, 40)}.*UNPROVED`));
  assert.match(result.text, /agent: subtract can overflow at the lower bound/);
  assert.equal(result.text.includes(sentence.theorem), false);
  assert.equal(readFileSync(join(root, 'model/keep.lex.tex'), 'utf8'), 'the old model');
  assert.equal(readFileSync(join(root, 'prismpm.lock'), 'utf8'), before);
  assert.deepEqual(toolchain.calls, [], 'a sentence it gave up on never reaches the toolchain');
  rmSync(root, { recursive: true });
});

test('author.evidence.kernel: the claim is not the proof, and a missing theorem is unproved however the agent answers', async () => {
  const root = project();
  const spec = loadSpec(root);
  const dropped = spec.intent[0];
  const toolchain = fakeToolchain(root, { theorems: (s) => s.intent.filter((row) => row !== s.intent[0]).map((row) => ({ name: `App.Model.${row.theorem}`, result: 'ok', observed: [], policy: { kind: 'none', axioms: [] } })) });
  const agent = fakeAgent();
  const result = await build(root, agent, toolchain, { maxIterations: 2 });
  assert.equal(result.code, 1);
  assert.equal(agent.requests.length, 2);
  assert.match(result.text, new RegExp(`${dropped.sentence.slice(0, 40)}.*UNPROVED`));
  assert.match(result.text, /why: the model has no theorem for this sentence/);
  assert.match(result.text, /1 intent sentence unproved after 2 attempts are spent/);
  assert.equal(agent.requests[1].previous.stage, 'proof');
  assert.match(agent.requests[1].previous.diagnostics, /no theorem/);
  assert.equal(existsSync(join(root, 'model')), false, 'a failed first generation leaves no model');
  assert.equal(existsSync(join(root, '.prism-author/rejected/src/App.lex.tex')), true, 'and keeps the attempt for the reader');
  assert.equal(JSON.parse(readFileSync(join(root, 'prismpm.lock'), 'utf8')).authoring, undefined);
  rmSync(root, { recursive: true });
});

test('author.evidence.kernel: a theorem the kernel refused or that leans on sorry is unproved', () => {
  const intent = parseIntent('a never fails.\nb never fails.\nc never fails.\nd never fails.\n');
  const rows = intentVerdicts(intent, [
    { name: `M.${intent[0].theorem}`, verdict: 'PASSED', axioms: { observed: [] } },
    { name: `M.${intent[1].theorem}`, verdict: 'FAILED', axioms: { observed: [] } },
    { name: `M.${intent[2].theorem}`, verdict: 'PASSED', axioms: { observed: ['sorryAx'] } },
  ]);
  assert.deepEqual(rows.map((row) => row.verdict), ['checked', 'UNPROVED', 'UNPROVED', 'UNPROVED']);
  assert.match(rows[2].why, /sorry/);
  assert.equal(intentVerdicts(intent, null).every((row) => row.verdict === 'UNPROVED'), true, 'no evidence is not a pass');
});

test('author.evidence.kernel: a theorem name must match exactly, not as a suffix of another', () => {
  const [row] = parseIntent('a never fails.\n');
  const rows = intentVerdicts([row], [{ name: `M.other_${row.theorem}`, verdict: 'PASSED', axioms: { observed: [] } }]);
  assert.equal(rows[0].verdict, 'UNPROVED');
});

test('author.iterate: a toolchain failure goes back to the agent, and a later attempt can pass', async () => {
  const root = project();
  const toolchain = fakeToolchain(root);
  const original = toolchain.run;
  let checks = 0;
  toolchain.run = async (command, args) => {
    if (command === 'check' && ++checks === 1) return { code: 1, output: 'PP1001 at line 3' };
    return original(command, args);
  };
  const agent = fakeAgent();
  const result = await build(root, agent, toolchain);
  assert.equal(result.code, 0, result.text);
  assert.equal(agent.requests.length, 2);
  assert.equal(agent.requests[0].previous, null);
  assert.deepEqual([agent.requests[1].previous.stage, agent.requests[1].previous.diagnostics], ['check', 'PP1001 at line 3']);
  assert.match(result.text, /in 2 attempts, 20 tokens/);
  rmSync(root, { recursive: true });
});

test('author.iterate: a malformed answer spends an attempt and is explained; a dead agent stops the build', async () => {
  const root = project();
  const agent = fakeAgent((request, turn) => (turn === 1 ? { schema: 'wrong' } : fakeAgentAnswer(request)));
  const result = await build(root, agent, fakeToolchain(root));
  assert.equal(result.code, 0, result.text);
  assert.match(agent.requests[1].previous.diagnostics, /author-response/);
  const dead = { adapter: 'test', turn: async () => { throw Object.assign(new Error('the agent exited with 1'), { authorCode: 'author.agent' }); } };
  const stopped = await build(root, dead, fakeToolchain(root), { noCache: true });
  assert.equal(stopped.code, 1);
  assert.match(stopped.text, /the agent exited with 1/);
  assert.equal(existsSync(join(root, 'model/author.json')), true, 'the model that passed is still there');
  rmSync(root, { recursive: true });
});

function fakeAgentAnswer(request) {
  return {
    schema: RESPONSE_SCHEMA,
    files: [{ path: 'src/App.lex.tex', text: `% uses ${RECORD}\n` }],
    sentences: request.spec.intent.map(({ theorem }) => ({ theorem, status: 'proved' })),
    types: request.spec.types.map(({ path }) => ({ source: path, record: RECORD })),
  };
}

test('author.iterate: the token budget stops the loop', async () => {
  const root = project();
  const agent = fakeAgent((request) => ({ ...fakeAgentAnswer(request), usage: { total_tokens: 1000 } }));
  const result = await build(root, agent, fakeToolchain(root, { fail: { check: {} } }), { maxTokens: 1500 });
  assert.equal(result.code, 1);
  assert.equal(agent.requests.length, 2);
  assert.match(result.text, /token budget of 1500 is spent/);
  rmSync(root, { recursive: true });
});

test('author.iterate: no Docker stops at once with 69, since no model can fix it', async () => {
  const root = project();
  const agent = fakeAgent();
  const result = await build(root, agent, fakeToolchain(root, { fail: { check: { code: 69, output: 'docker is not installed' } } }));
  assert.equal(result.code, 69);
  assert.equal(agent.requests.length, 1);
  rmSync(root, { recursive: true });
});

test('author.rollback: a failed rebuild restores the model and the lock bytes, and a killed build recovers', async () => {
  const root = project();
  await build(root, fakeAgent(), fakeToolchain(root));
  const model = modelDigest(readModel(root));
  const lock = readFileSync(join(root, 'prismpm.lock'), 'utf8');
  rmSync(join(root, '.prism/verified'), { recursive: true });
  const result = await build(root, fakeAgent(), fakeToolchain(root, { fail: { verify: {} } }), { noCache: true, maxIterations: 1 });
  assert.equal(result.code, 1);
  assert.equal(modelDigest(readModel(root)), model);
  assert.equal(readFileSync(join(root, 'prismpm.lock'), 'utf8'), lock);

  // A build killed between attempts: the marker, the held model and an attempt in model/.
  cpSync(join(root, 'model'), join(root, '.prism-author/previous'), { recursive: true });
  writeFileSync(join(root, '.prism-author/attempted'), '');
  put(root, 'model/src/App.lex.tex', 'an attempt nothing checked');
  const agent = fakeAgent();
  const recovered = await build(root, agent, fakeToolchain(root));
  assert.equal(recovered.code, 0, recovered.text);
  assert.equal(agent.requests.length, 0);
  assert.equal(modelDigest(readModel(root)), model);
  rmSync(root, { recursive: true });
});

test('author.examples.replay: a holo that disagrees with an example names the example file', async () => {
  const root = project();
  writeFileSync(join(root, 'examples/01-1-add-20-22.json'), '{"input":"1\\tadd\\t20\\t22","output":"1\\tok\\t43"}');
  const result = await build(root, fakeAgent(), fakeToolchain(root), { maxIterations: 1 });
  assert.equal(result.code, 1);
  assert.match(result.text, /Example failed: examples\/01-1-add-20-22\.json/);
  rmSync(root, { recursive: true });
});

test('author.replay.hook: the replay gate on examples/ is named and deferred to #284', () => {
  assert.deepEqual(replayExamplesGate(), { status: 'deferred', issue: 284 });
});

test('author.holo.closed: the agent writes model sources only, so no weights, Lean or escapes reach the build', () => {
  for (const path of ['src/App.lex.tex', 'lexlean.toml', 'lexlean.lock', 'lakefile.toml', 'lean-toolchain', 'lake-manifest.json', 'language/x/lexicon.toml']) assert.equal(modelPathAllowed(path), true, path);
  for (const path of ['', '../prism.toml', 'src/../../x.lex.tex', '/etc/passwd', 'C:/x.lex.tex', 'a\\b.lex.tex', 'src/App.lean', 'weights.safetensors', 'model.gguf', '.git/config', 'src/.hidden.lex.tex', 'author.json', 'prismpm.toml']) assert.equal(modelPathAllowed(path), false, path);
});

test('author.holo.closed: a holo that does not load under the grant fails the gate', async () => {
  const root = project();
  const broken = join(root, 'broken.holo');
  const copy = readFileSync(HOLO);
  copy[copy.length - 1] ^= 1;
  writeFileSync(broken, copy);
  const result = await build(root, fakeAgent(), fakeToolchain(root, { holo: broken }), { maxIterations: 1 });
  assert.equal(result.code, 1);
  assert.match(result.text, /holo failed:/);
  rmSync(root, { recursive: true });
});

test('author.response: the sentences are answered one to one, and files are checked before anything is written', () => {
  const spec = loadSpec(join(FIXTURES, 'calculator'));
  const good = () => fakeAgentAnswer({ spec });
  assert.equal(validateResponse(good(), spec).schema, RESPONSE_SCHEMA);
  const refused = (change, message) => {
    const answer = good();
    change(answer);
    assert.throws(() => validateResponse(answer, spec), (error) => error.code === 'author.agent' && message.test(error.message), String(change));
  };
  refused((a) => a.sentences.pop(), /does not answer/);
  refused((a) => a.sentences.push({ theorem: 'intent_invented_00000000', status: 'proved' }), /no sentence of intent\.md/);
  refused((a) => a.sentences.push({ ...a.sentences[0] }), /twice/);
  refused((a) => { a.sentences[0].status = 'maybe'; }, /must be proved or unproved/);
  refused((a) => { a.sentences[0] = { theorem: a.sentences[0].theorem, status: 'unproved' }; }, /no explanation/);
  refused((a) => a.files.push({ path: '../escape.lex.tex', text: '' }), /not a model source/);
  refused((a) => a.files.push({ ...a.files[0] }), /twice/);
  refused((a) => { a.files[0].text = 'x'.repeat(4 * 1024 * 1024 + 1); }, /exceeds/);
  refused((a) => { a.types = []; }, /maps nothing from types\/calculator\.ts/);
});

test('author.types.record: a record the sources never name goes back to the agent', async () => {
  const root = project();
  const agent = fakeAgent((request, turn) => {
    const answer = fakeAgentAnswer(request);
    if (turn === 1) answer.types = [{ source: 'types/calculator.ts#Request', record: 'Foundation.Nowhere::Nothing' }];
    return answer;
  });
  const result = await build(root, agent, fakeToolchain(root));
  assert.equal(result.code, 0, result.text);
  assert.match(agent.requests[1].previous.diagnostics, /Foundation\.Nowhere::Nothing, which no source names/);
  rmSync(root, { recursive: true });
});

test('author.request: the agent is given the spec, its theorem names, the pin and the rules, and no secret', () => {
  const spec = loadSpec(join(FIXTURES, 'calculator'));
  const request = authorRequest({ spec, pin: { adapter: 'a', model: 'm' }, attempt: 1, maxAttempts: 10, scaffold: { paths: [] }, env: { PRISM_LEXLEAN_MCP_URL: 'http://127.0.0.1:1/mcp', ANTHROPIC_API_KEY: 'do-not-send' } });
  assert.equal(request.agent.model, 'm');
  assert.equal(request.spec.intent[0].theorem, spec.intent[0].theorem);
  assert.deepEqual(request.spec.examples[21].input, { hex: 'ff' }, 'bytes that are not UTF-8 stay exact');
  assert.equal(request.context.lexlean_mcp, 'http://127.0.0.1:1/mcp');
  assert.equal(JSON.stringify(request).includes('do-not-send'), false);
  assert.equal(authorRequest({ spec, pin: {}, attempt: 1, maxAttempts: 1, scaffold: {}, env: {} }).context.lexlean_mcp, null);
});

test('author.agent.process: one document in, one out; a failure, a timeout and bad JSON are refused', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'prism-author-'));
  const script = (name, body) => {
    writeFileSync(join(dir, name), body);
    return join(dir, name);
  };
  const echo = processAgent({ command: script('echo.mjs', "let s='';for await (const c of process.stdin) s+=c;process.stdout.write(JSON.stringify({got:JSON.parse(s).n}));") });
  assert.deepEqual(await echo.turn({ n: 7 }), { got: 7 });
  await assert.rejects(processAgent({ command: script('fail.mjs', 'process.exit(3);') }).turn({}), (e) => e.authorCode === 'author.agent' && /exited with 3/.test(e.message));
  await assert.rejects(processAgent({ command: script('bad.mjs', "process.stdout.write('not json');") }).turn({}), (e) => e.authorCode === 'author.agent' && /not JSON/.test(e.message));
  await assert.rejects(processAgent({ command: script('slow.mjs', 'setTimeout(() => {}, 60000);'), timeoutMs: 100 }).turn({}), (e) => e.authorCode === 'author.timeout');
  await assert.rejects(processAgent({ command: join(dir, 'absent-binary') }).turn({}), (e) => e.authorCode === 'author.agent');
  rmSync(dir, { recursive: true });
});

test('author.check: a built holo is proved to come from this spec, and each tampering fails its own check', async () => {
  const root = project();
  await build(root, fakeAgent(), fakeToolchain(root));
  const failing = async () => (await checkAuthoring(HOLO, { root })).checks.filter((c) => !c.passed).map((c) => c.id);
  assert.deepEqual(await failing(), []);
  const outcome = await checkAuthoring(HOLO, { root });
  assert.equal(outcome.passed, true);
  assert.equal(outcome.rows.every((row) => row.verdict === 'checked'), true);
  assert.deepEqual(outcome.checks.map((c) => c.id), ['author.lock.record', 'author.spec.digest', 'author.model.digest', 'author.holo.closed', 'author.holo.model', 'author.examples.vectors', 'author.examples.replay', 'author.intent.verdicts']);

  rmSync(join(root, '.prism/verified'), { recursive: true });
  assert.deepEqual(await failing(), ['author.intent.verdicts']);

  const intent = readFileSync(join(root, 'intent.md'), 'utf8');
  writeFileSync(join(root, 'intent.md'), `${intent}\n# a comment\n`);
  assert.equal((await failing()).includes('author.spec.digest'), true);
  writeFileSync(join(root, 'intent.md'), intent);

  put(root, 'model/src/App.lex.tex', 'edited');
  assert.equal((await failing()).includes('author.model.digest'), true);

  const lock = JSON.parse(readFileSync(join(root, 'prismpm.lock'), 'utf8'));
  writeFileSync(join(root, 'prismpm.lock'), JSON.stringify({ ...lock, inputs: { ...lock.inputs, model: { sha256: 'f'.repeat(64) } } }));
  assert.equal((await failing()).includes('author.holo.model'), true);
  rmSync(root, { recursive: true });
});

test('author.compare: a holo against itself is identical in Core-Wasm and acceptance; a difference is explained', async () => {
  const bytesOf = new Uint8Array(readFileSync(HOLO));
  const { vectors } = await fromFile(HOLO);
  const same = await compareHolos(bytesOf, bytesOf, vectors, vectors);
  assert.deepEqual(same.map((row) => row.part), ['guest', 'acceptance vectors', 'view', 'capability-set', 'model-document']);
  assert.equal(same.every((row) => row.same), true);
  const fewer = await compareHolos(bytesOf, bytesOf, vectors, vectors.slice(1));
  assert.equal(fewer.find((row) => row.part === 'acceptance vectors').same, false);
  assert.match(fewer.find((row) => row.part === 'acceptance vectors').why, /examples\/ differs/);
  assert.equal((await inspect(bytesOf)).sections.some((section) => section.role === 'guest'), true, 'inspect names the roles compared');
});

test('author.compare: the Calculator spec project reports against the reference holo through the command line', async () => {
  const root = project();
  await build(root, fakeAgent(), fakeToolchain(root));
  const policy = join(root, 'policy.json');
  writeFileSync(policy, JSON.stringify({ name: 'any', minimum: 'P0' }));
  const run = spawnSync(process.execPath, [BIN, 'check', HOLO, '--unstable-author', '--project', root, '--reference', HOLO, '--policy', policy], { encoding: 'utf8' });
  assert.equal(run.status, 0, run.stdout + run.stderr);
  assert.match(run.stdout, /Authoring:/);
  assert.match(run.stdout, /ok {4} {2}author\.spec\.digest/);
  assert.match(run.stdout, /identical {2}guest/);
  assert.match(run.stdout, /waits for #284/);
  const json = spawnSync(process.execPath, [BIN, 'check', HOLO, '--unstable-author', '--project', root, '--policy', policy, '--json'], { encoding: 'utf8' });
  const parsed = JSON.parse(json.stdout);
  assert.equal(parsed.verdict.schema, 'prism/verdict/1');
  assert.equal(parsed.authoring.passed, true);
  rmSync(root, { recursive: true });
});

test('author.cli: the flags are checked, and the flag is needed for its options', () => {
  const run = (...args) => spawnSync(process.execPath, [BIN, ...args], { encoding: 'utf8' });
  assert.equal(run('build', '--unstable-author', '--nonsense').status, 64);
  assert.equal(run('build', '--unstable-author', '--max-iterations', '0').status, 64);
  assert.equal(run('build', '--unstable-author', 'positional').status, 64);
  assert.equal(run('check', HOLO, '--reference', HOLO).status, 64);
  const empty = mkdtempSync(join(tmpdir(), 'prism-author-'));
  const none = run('build', '--unstable-author', '--project', empty);
  assert.equal(none.status, 2);
  assert.match(none.stderr, /author\.spec/);
  rmSync(empty, { recursive: true });
});

test('author.modelstate: absent, stale and current are told apart', async () => {
  const root = project();
  const spec = loadSpec(root);
  assert.equal(modelState(spec, null, root).state, 'absent');
  await build(root, fakeAgent(), fakeToolchain(root));
  const lock = JSON.parse(readFileSync(join(root, 'prismpm.lock'), 'utf8'));
  assert.equal(modelState(spec, lock, root).state, 'current');
  assert.equal(modelState(spec, lock, root, true).state, 'stale');
  assert.equal(modelState({ ...spec, digest: 'other' }, lock, root).state, 'stale');
  rmSync(root, { recursive: true });
});

test('author.anthropic: the request pins the model the lock names and asks for the response shape', () => {
  const body = messageRequest({ agent: { model: 'claude-opus-5-5' }, n: 1 });
  assert.equal(body.model, 'claude-opus-5-5');
  assert.equal(body.fallbacks, 'default');
  assert.deepEqual(body.betas, ['server-side-fallback-2026-07-01']);
  assert.equal(body.output_config.format.type, 'json_schema');
  assert.equal(body.thinking, undefined);
  assert.equal(JSON.parse(body.messages[0].content).n, 1);
});

test('author.anthropic: a finished answer becomes a response; a refusal or an early stop is an error', () => {
  const answer = { files: [], sentences: [], types: [] };
  const done = responseFromMessage({
    stop_reason: 'end_turn',
    content: [{ type: 'thinking', thinking: '' }, { type: 'text', text: JSON.stringify(answer) }],
    usage: { input_tokens: 5, output_tokens: 7, cache_read_input_tokens: 3 },
  });
  assert.equal(done.schema, RESPONSE_SCHEMA);
  assert.equal(done.usage.total_tokens, 15);
  assert.throws(() => responseFromMessage({ stop_reason: 'refusal', stop_details: { category: 'cyber', explanation: 'no' }, content: [] }), /declined.*cyber/);
  assert.throws(() => responseFromMessage({ stop_reason: 'max_tokens', content: [] }), /stopped early: max_tokens/);
});
