// The journey's proof on the hosted verifier: no Docker, no image. A stand-in server answers; nothing leaves the machine.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { hostedClient } from '../src/hosted.mjs';
import { compile } from '../src/journey/session.mjs';
import { names } from '../src/journey/render.mjs';
import { spawnSync } from 'node:child_process';
import { appDir, readSession, rememberModel, writeApp } from '../src/journey/project.mjs';
import { build, buildRemote, buildWhere, fromHosted, isWarning, prove, proveWhere } from '../src/journey/engine.mjs';
import { fetchPinned, readyForGitHub } from '../src/journey/github.mjs';

const ANSWERS = {
  purpose: 'Payment approval: decide whether a payment request is approved.',
  fields: 'role: clerk|manager, amount: number at least 1, limit: number',
  outcomes: 'approved, refused',
  rules: 'when role is manager then approved\nwhen amount is at most limit then approved\notherwise refused',
  properties: 'never when role is clerk and amount > limit then approved\nalways when role is manager then approved',
  examples: 'clerk 250 1000 -> approved\nclerk 1001 1000 -> refused',
};
const PROFILE = { id: 'p1', name: 'prism-app', roots: ['src'], packages: [{ package: 'lexlean.core' }, { package: 'lexlean.std.nat' }, { package: 'prism.arch' }] };
const MODULES = ['Foundation.Bytes', 'Foundation.View.Text.V1.Model', 'PaymentApproval', 'PaymentApprovalProofs'];

function app() {
  const dir = appDir(mkdtempSync(join(tmpdir(), 'prism-journey-hosted-')));
  return { dir, session: { ...readSession(dir), spec: compile({ ...ANSWERS, review: 'yes' }).spec, axioms: {}, status: {} } };
}

/** A stand-in server whose verify answers come from `answers`, one per call. */
function server(answers) {
  const verifies = [];
  let id = 0;
  const fetch = async (_url, { body }) => {
    const request = JSON.parse(body);
    const tool = request.params.name;
    let result;
    if (tool === 'lexlean_profiles') result = { structuredContent: { profiles: [PROFILE] } };
    else if (tool === 'lexlean_draft_write') result = { structuredContent: { draft: `prism-app/${++id}` } };
    else if (tool === 'lexlean_verify') {
      verifies.push(request.params.arguments);
      result = answers.shift();
    }
    return { status: 200, text: async () => JSON.stringify({ jsonrpc: '2.0', id: request.id, result }) };
  };
  return { verifies, client: hostedClient({ url: 'https://lexlean.example.test/', fetch, sleep: async () => {}, retryMs: 0 }) };
}

const verified = (modules = MODULES) => ({ taskId: 't', status: 'completed', result: { content: [{ type: 'text', text: JSON.stringify({ attestation_id: 'c'.repeat(64), modules }) }], structuredContent: { state: 'verified' } } });
const failed = (diagnostics) => ({ taskId: 't', status: 'completed', result: { content: [{ type: 'text', text: JSON.stringify({ command: 'verify', diagnostics }) }], structuredContent: { state: 'failed' } } });
const WARNING = { code: 'LLV7002', severity: 'error', message: 'Lean rejected `PrismPaymentApproval.PaymentApproval` (warning): This simp argument is unused:\n  Nat.beq_eq', primary: { path: 'src/PaymentApproval.lex.tex', line_start: 3 } };

test('where the proof runs: PRISM_CODE_TIER chooses, and a diagnostic of the hosted verifier reads like a local one', () => {
  const { dir, session } = app();
  assert.equal(proveWhere(dir, session, { env: { PRISM_CODE_TIER: 'hosted' } }), 'hosted');
  assert.equal(proveWhere(dir, session, { env: { PRISM_CODE_TIER: 'local' } }), 'local');
  assert.deepEqual(fromHosted({ code: 'LLV7002', severity: 'error', message: 'm', primary: { path: 'src/A.lex.tex', line_start: 77, column_start: 1 } }), { severity: 'error', code: 'LLV7002', message: 'm', span: { file: 'src/A.lex.tex', line: 77, column: 1 } });
  assert.equal(fromHosted({ code: 'X', message: 'm' }).span, null);
  assert.equal(isWarning(fromHosted(WARNING)), true, 'Lean warnings arrive as LLV7002 errors and are not failures');
  assert.equal(isWarning({ code: 'LLV7002', severity: 'error', message: 'Lean rejected `A` (error): unsolved goals' }), false);
});

test('a hosted proof names every module, the proofs module included, and records where it was proved', async () => {
  const { dir, session } = app();
  const { verifies, client } = server([verified()]);
  const lines = [];
  const r = await prove(dir, session, { where: 'hosted', client, log: (l) => lines.push(l) });
  assert.equal(r.ok, true, r.message);
  assert.equal(r.where, 'hosted');
  assert.equal(r.attestation, 'c'.repeat(64));
  assert.match(r.message, /^Lean proved all 2 properties for every request on the hosted verifier, in \d+ s\.$/);
  assert.deepEqual(verifies[0].files.sort(), ['src/Foundation/Bytes.lex.tex', 'src/Foundation/View/Text/V1/Model.lex.tex', 'src/PaymentApproval.lex.tex', 'src/PaymentApprovalProofs.lex.tex']);
  assert.match(lines[0], /hosted verifier https:\/\/lexlean\.example\.test\/, so the SDK image is not needed yet\. The files under prism\/src\/ are sent and stored unlisted: not searchable, but readable by anyone with the draft's ID\./);
  assert.equal(session.status.proved.where, 'hosted');
  assert.equal(session.status.proved.record, `.lexlean/hosted/${'c'.repeat(64)}.json`);
  assert.ok(existsSync(join(dir, session.status.proved.record)));
  assert.deepEqual(Object.values(session.status.properties), ['proved', 'proved']);
});

test('planted: a verified answer without the proofs module is no proof', async () => {
  const { dir, session } = app();
  const { client } = server([verified(['Foundation.Bytes', 'PaymentApproval'])]);
  const r = await prove(dir, session, { where: 'hosted', client });
  assert.equal(r.ok, false);
  assert.match(r.message, /not the entrypoint modules Foundation\.View\.Text\.V1\.Model, PaymentApprovalProofs; nothing about this project was proved/);
  assert.equal(session.status.proved, undefined);
});

test('a wrong axiom set is corrected and proved again; a Lean warning beside it does not stop that', async () => {
  const { dir, session } = app();
  const axiom = { code: 'LLV7005', severity: 'error', message: '`PrismPaymentApproval.PaymentApproval.decide` violates its exact axiom policy: observed [propext]', primary: { path: 'src/PaymentApproval.lex.tex', line_start: 10 } };
  const { verifies, client } = server([failed([axiom, WARNING]), verified()]);
  const r = await prove(dir, session, { where: 'hosted', client });
  assert.equal(r.ok, true, r.message);
  assert.equal(verifies.length, 2);
  assert.equal(session.axioms.decide, '[propext]');
});

test('planted: Lean refusing a property marks that property failing, found by the line the verifier names', async () => {
  const { dir, session } = app();
  writeApp(dir, session);
  const n = names(session.spec);
  const id = session.spec.properties[0].id;
  const proofs = readFileSync(join(dir, 'src', `${n.proofs}.lex.tex`), 'utf8').split('\n');
  const line = proofs.findIndex((l, i) => /^theorem\{/.test(l) && (proofs[i + 1] ?? '').includes(n.properties[id].theorem)) + 1;
  assert.ok(line > 0, 'the proofs module holds the theorem');
  const refused = { code: 'LLV7002', severity: 'error', message: `Lean rejected \`${n.prefix}.${n.proofs}\` (error): unsolved goals`, primary: { path: `src/${n.proofs}.lex.tex`, line_start: line } };
  const { client } = server([failed([refused, WARNING])]);
  const r = await prove(dir, session, { where: 'hosted', client, overwrite: true });
  assert.equal(r.ok, false);
  assert.equal(session.status.properties[id], 'failing');
  assert.match(r.message, new RegExp(`${id}`));
  assert.match(readFileSync(join(dir, '.prism-last.log'), 'utf8'), new RegExp(`error\\[LLV7002\\]: .*\\n {2}--> src/${n.proofs}\\.lex\\.tex:${line}:`));
});

/** A session whose model is on disk and counts as proved, so `build` goes straight to building. */
function provedApp() {
  const { dir, session } = app();
  writeApp(dir, session);
  rememberModel(dir, session);
  session.status.proved = { modelSha: session.status.modelSha, where: 'hosted' };
  session.status.lastSpec = JSON.stringify(session.spec);
  return { dir, session };
}

test('the build goes where it can: the local image when it is here, else GitHub when signed in, else local; remote and PRISM_CODE_TIER choose', () => {
  const { dir, session } = provedApp();
  const where = (opts) => buildWhere(dir, session, { env: {}, docker: () => ({}), image: () => false, github: () => false, ...opts });
  assert.equal(where({ image: () => true }), 'local', 'Docker and the image are here');
  assert.equal(where({ github: () => true }), 'github', 'no image, signed in to GitHub');
  assert.equal(where({ docker: () => ({ error: 'no docker' }), github: () => true }), 'github', 'no Docker, signed in');
  assert.equal(where({}), 'local', 'neither: local, which downloads the image');
  assert.equal(where({ remote: 'local', github: () => true }), 'local');
  assert.equal(where({ remote: 'github', image: () => true }), 'github');
  assert.equal(where({ env: { PRISM_CODE_TIER: 'local' }, github: () => true }), 'local');
});

/** Stand-ins for every step of a GitHub build that would touch GitHub; `calls` records them. */
function gitHubSteps(overrides = {}) {
  const calls = [];
  return {
    calls,
    steps: {
      signedInLogin: () => 'octo',
      readyForGitHub: async ({ root }) => {
        calls.push('ready');
        mkdirSync(join(root, '.github', 'workflows'), { recursive: true });
        writeFileSync(join(root, '.github', 'workflows', 'build.yml'), 'name: build\n');
        writeFileSync(join(root, 'prismpm.lock'), '{}\n');
        return ['prismpm.lock', '.github/workflows/build.yml'];
      },
      publishRepository: ({ repository }) => {
        calls.push(`publish ${repository}`);
        return { repository, created: true };
      },
      remoteBuild: async ({ allowDirty }) => {
        calls.push(`build allowDirty=${allowDirty}`);
        return { exitCode: 1, run: { url: 'https://github.com/octo/prism-payment-approval/actions/runs/1' }, verdict: { level: 'P0' }, vectors: { passed: 0, total: 3 }, staging: 'staged' };
      },
      ...overrides,
    },
  };
}

test('a GitHub build with no repository asks for one, suggests a name, and commits and creates nothing', async () => {
  const { dir, session } = provedApp();
  const { calls, steps } = gitHubSteps();
  const r = await buildRemote(dir, session, { root: join(dir, '..'), steps });
  assert.equal(r.ok, false);
  assert.equal(r.needs, 'repository');
  assert.equal(r.suggested, 'octo/prism-payment-approval');
  assert.match(r.message, /Ask the user to approve creating the public repository octo\/prism-payment-approval/);
  assert.deepEqual(calls, []);
  assert.equal(existsSync(join(dir, '..', '.git')), false, 'no repository is made before the user says yes');
  assert.equal(session.status.built, undefined);
});

const identity = { GIT_AUTHOR_NAME: 't', GIT_AUTHOR_EMAIL: 't@example.invalid', GIT_COMMITTER_NAME: 't', GIT_COMMITTER_EMAIL: 't@example.invalid' };
async function withIdentity(body) {
  const saved = Object.fromEntries(Object.keys(identity).map((k) => [k, process.env[k]]));
  Object.assign(process.env, identity);
  try {
    return await body();
  } finally {
    for (const [k, v] of Object.entries(saved)) if (v === undefined) delete process.env[k]; else process.env[k] = v;
  }
}

test('a GitHub build in an existing repository commits only Prism\'s paths and builds that commit', async () => {
  const { dir, session } = provedApp();
  const root = join(dir, '..');
  writeFileSync(join(root, 'notes.txt'), 'the user\'s own file\n');
  assert.equal(spawnSync('git', ['-C', root, 'init', '-q', '-b', 'main'], { encoding: 'utf8' }).status, 0);
  await withIdentity(async () => {
    const { calls, steps } = gitHubSteps({ gitRoot: () => root, originRepository: () => ({ owner: 'acme', repo: 'app', full: 'acme/app' }) });
    const r = await buildRemote(dir, session, { root, steps });
    assert.deepEqual(calls, ['ready', 'build allowDirty=true'], 'an existing repository is never created');
    assert.equal(r.ok, false, 'the stand-in build does not pass');
    assert.match(r.message, /did not pass here: verdict P0, 0 of 3 vectors/);
    const files = spawnSync('git', ['-C', root, 'ls-files'], { encoding: 'utf8' }).stdout.trim().split('\n');
    assert.ok(files.includes('.github/workflows/build.yml') && files.includes('prismpm.lock'));
    assert.ok(files.some((f) => f.startsWith('prism/src/')), 'the app folder is committed');
    assert.ok(!files.includes('notes.txt'), 'a file Prism did not write stays out');
  });
});

/** A small stand-in for a template clone: the files writeTemplateApp reads and replaces. */
function templateSkeleton(into) {
  const w = (path, text) => {
    mkdirSync(dirname(join(into, path)), { recursive: true });
    writeFileSync(join(into, path), text);
  };
  const zero = '0'.repeat(64);
  w('app/lexlean.lock', `spec = "lexlean/lock/2"\nproject_config_sha256 = "${zero}"\n\n[[workspace_file]]\npath = "lake-manifest.json"\nsha256 = "${zero}"\n\n[[workspace_file]]\npath = "lakefile.toml"\nsha256 = "${zero}"\n\n[[workspace_file]]\npath = "lean-toolchain"\nsha256 = "${zero}"\n`);
  w('app/lean-toolchain', 'leanprover/lean4:v4.32.1\n');
  w('app/src/Contract.lex.tex', 'the example\n');
  w('crates/conformance/tests/contract.rs', '// the example\n');
  w('features/suites/contract.feature', 'Feature: contract\n');
  w('model/ids.toml', '# the register\n\nspec = "template/1"\n\n[[id]]\nid = "TC-01"\nlevel = "build"\nsuite = "contract"\nstatement = "the example"\n');
  w('CONFORMANCE.md', '# CONFORMANCE\n\nThe levels.\n\n## contract\n\n| ID | Level | Statement |\n| --- | --- | --- |\n| `TC-01` | `build` | the example |\n\n## Cited authorities\n\nNever re-derived.\n');
  w('site/app.json', '{"name": "Template contract", "trusted": [{"name": "SDK image"}]}\n');
}

test('a new app becomes a template instance: created from the template, written into app/, pushed, and live once its gate and page pass', async () => {
  const { dir, session } = provedApp();
  session.status.proved.attestation = 'a'.repeat(64);
  const root = join(dir, '..');
  const calls = [];
  const holo = fileURLToPath(new URL('../fixtures/Calculator.holo', import.meta.url));
  const steps = {
    createFromTemplate: async ({ repository, into }) => {
      calls.push(`create ${repository}`);
      templateSkeleton(into);
      return { repository, clone: into };
    },
    pushClone: ({ clone }) => {
      calls.push(`push ${basename(clone)}`);
      return 'c'.repeat(40);
    },
    waitForWorkflows: async ({ workflows }) => {
      calls.push(`wait ${workflows.join(' ')}`);
      return Object.fromEntries(workflows.map((w) => [w, { conclusion: 'success', html_url: `https://github.com/octo/prism-payment-approval/actions/${w}` }]));
    },
    fetchLive: async ({ live, into }) => {
      calls.push(`fetch ${live}`);
      mkdirSync(dirname(into), { recursive: true });
      writeFileSync(into, readFileSync(holo));
    },
    client: {},
  };
  const r = await buildRemote(dir, session, { root, repository: 'octo/prism-payment-approval', steps });
  assert.deepEqual(calls, ['create octo/prism-payment-approval', 'push prism-payment-approval', 'wait pages.yml bootstrap.yml', 'fetch https://octo.github.io/prism-payment-approval/']);
  assert.equal(r.ok, true, r.message);
  assert.equal(r.live, 'https://octo.github.io/prism-payment-approval/');
  assert.match(r.message, /^Live at https:\/\/octo\.github\.io\/prism-payment-approval\/\. Built and verified/);
  const clone = join(root, 'prism-payment-approval');
  assert.deepEqual(session.status.repository, { full: 'octo/prism-payment-approval', created: true, template: true, clone, live: r.live });
  assert.equal(existsSync(join(clone, 'app/src/Contract.lex.tex')), false, 'the template\'s example is replaced');
  assert.ok(existsSync(join(clone, 'app/src/PaymentApproval.lex.tex')) && existsSync(join(clone, 'app/src/PaymentApprovalProofs.lex.tex')));
  assert.match(readFileSync(join(clone, 'app/lexlean.toml'), 'utf8'), /^entrypoints = \["src\/PaymentApproval\.lex\.tex", "src\/PaymentApprovalProofs\.lex\.tex"\]$/m, 'the proofs are an entrypoint, so the gate checks the theorems');
  assert.doesNotMatch(readFileSync(join(clone, 'app/lexlean.lock'), 'utf8'), /0{64}/, 'every digest the lock binds is recomputed');
  assert.match(readFileSync(join(clone, 'model/ids.toml'), 'utf8'), /^# the register\n\nspec = "template\/1"\n\n\[\[id\]\]\nid = "PA-01"/);
  assert.ok(existsSync(join(clone, 'features/suites/payment_approval.feature')) && existsSync(join(clone, 'crates/conformance/tests/payment_approval.rs')));
  assert.equal(existsSync(join(clone, 'crates/conformance/tests/contract.rs')), false);
  assert.match(readFileSync(join(clone, 'openspec/specs/payment-approval/spec.md'), 'utf8'), /^# payment-approval Specification\n[\s\S]*\n## Purpose\n[\s\S]*\n## Requirements\n/, 'the spec is where OpenSpec looks for it');
  const app = JSON.parse(readFileSync(join(clone, 'site/app.json'), 'utf8'));
  assert.equal(app.repository, 'https://github.com/octo/prism-payment-approval');
  assert.equal(app.attestations.proof.id, 'a'.repeat(64));
  assert.deepEqual(app.trusted, [{ name: 'SDK image' }], 'what the template says it trusts is kept');

  calls.length = 0;
  const again = await buildRemote(dir, session, { root, steps });
  assert.deepEqual(calls, ['push prism-payment-approval', 'wait pages.yml bootstrap.yml', 'fetch https://octo.github.io/prism-payment-approval/'], 'a rebuild uses the clone it made, and creates nothing');
  assert.equal(again.ok, true);
});

test('a template instance whose gate fails says which run failed, and nothing is live', async () => {
  const { dir, session } = provedApp();
  const root = join(dir, '..');
  const steps = {
    createFromTemplate: async ({ repository, into }) => (templateSkeleton(into), { repository, clone: into }),
    pushClone: () => 'd'.repeat(40),
    waitForWorkflows: async () => ({ 'pages.yml': { conclusion: 'success', html_url: 'p' }, 'bootstrap.yml': { conclusion: 'failure', html_url: 'https://github.com/octo/x/actions/runs/9' } }),
    fetchLive: async () => assert.fail('nothing is fetched from a failed commit'),
    client: {},
  };
  const r = await buildRemote(dir, session, { root, repository: 'octo/prism-payment-approval', steps });
  assert.equal(r.ok, false);
  assert.match(r.message, /bootstrap\.yml ended failure: https:\/\/github\.com\/octo\/x\/actions\/runs\/9/);
});

test('a GitHub build refuses a repository that is not the origin, and a name that is not owner/name', async () => {
  const { dir, session } = provedApp();
  const root = join(dir, '..');
  const { calls, steps } = gitHubSteps({ gitRoot: () => root, originRepository: () => ({ owner: 'acme', repo: 'app', full: 'acme/app' }) });
  const other = await buildRemote(dir, session, { root, repository: 'octo/prism-payment-approval', steps });
  assert.match(other.message, /origin is acme\/app, not octo\/prism-payment-approval/);
  const bad = await buildRemote(dir, session, { root, repository: 'not a name', steps });
  assert.match(bad.message, /repository must be owner\/name/);
  assert.deepEqual(calls, []);
});

test('a pinned template file is refused when its bytes do not have the pinned digest', async () => {
  const fetch = async () => new Response('tampered');
  await assert.rejects(fetchPinned('prismpm.lock', { fetch }), /has SHA-256 [0-9a-f]{64}, not the pinned 3773d329/);
  await assert.rejects(fetchPinned('README.md', { fetch }), /not a pinned template file/);
  const { dir, session } = provedApp();
  await assert.rejects(readyForGitHub({ root: join(dir, '..'), dir, session, fetch }), /not the pinned/);
});
