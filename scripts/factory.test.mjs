// The factory route: κ, receipts and the release workflow checked against
// factory/src/Factory/Release.lex.tex.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join} from 'node:path';
import test from 'node:test';
import {buildArtifacts, buildIdentity, buildJobs, buildPlumbing, findBuildReceipt, findCandidate, findReceipt, imageJobs, isPlumbing, kappa,
  plumbing, plumbingTests, routeRecord, workflowJobs} from './factory.mjs';
import {parts as vvParts} from './vv-assemble.mjs';

const require = createRequire('/opt/prismpm/oracles/package.json'), {load} = require('js-yaml');
const root = new URL('..', import.meta.url).pathname;
const read = path => readFileSync(join(root, path), 'utf8');
const strings = body => [...body.matchAll(/"((?:[^"\\]|\\.)*)"/g)].map(match => match[1]);
const definition = (source, name) => {
  const start = source.indexOf(`  name: ${name},\n`);
  assert(start >= 0, `definition ${name}`);
  return source.slice(start, source.indexOf('\n}\n', start));
};

function repository(t) {
  const path = mkdtempSync(join(tmpdir(), 'prismpm-factory-test-'));
  t.after(() => rmSync(path, {recursive: true, force: true}));
  const git = (...args) => execFileSync('git', args, {cwd: path, encoding: 'utf8',
    env: {...process.env, GIT_AUTHOR_NAME: 'unit', GIT_AUTHOR_EMAIL: 'unit@example.invalid', GIT_COMMITTER_NAME: 'unit',
      GIT_COMMITTER_EMAIL: 'unit@example.invalid', GIT_CONFIG_NOSYSTEM: '1', HOME: path}}).trim();
  git('init', '-q', '-b', 'release/v0.4');
  const commit = (files, message) => {
    for (const [name, text] of Object.entries(files)) {
      mkdirSync(join(path, dirname(name)), {recursive: true}); writeFileSync(join(path, name), text);
    }
    git('add', '-A'); git('commit', '-q', '-m', message); return git('rev-parse', 'HEAD');
  };
  return {path, git, commit};
}

test('the plumbing is exactly the model\'s, and every listed path is tracked', () => {
  const model = read('factory/src/Factory/Release.lex.tex');
  assert.deepEqual(strings(definition(model, 'plumbing')), [...plumbing]);
  assert.deepEqual([...plumbing].sort(), [...plumbing], 'sorted, so a review sees one list');
  const tracked = execFileSync('git', ['ls-files'], {cwd: root, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024}).split('\n');
  for (const entry of plumbing) assert(tracked.some(path => entry.endsWith('/') ? path.startsWith(entry) : path === entry), entry);
  assert(isPlumbing('factory/src/Factory/Release.lex.tex')); assert(isPlumbing('.github/workflows/release.yml'));
  assert(!isPlumbing('.github/workflows/vv.yml')); assert(!isPlumbing('scripts/release-phases.mjs.bak'));
  assert(!isPlumbing('factory')); assert(!isPlumbing('crates/prismpm/src/lib.rs'));
});

test('every plumbing test runs in the plumbing check, and every test that reads the release workflow is plumbing', () => {
  const tests = plumbingTests(root);
  assert(tests.includes('scripts/factory.test.mjs') && tests.includes('scripts/release-phases.test.mjs'));
  for (const path of tests) assert(isPlumbing(path));
  const scripts = readdirSync(join(root, 'scripts')).filter(name => name.endsWith('.test.mjs')).map(name => `scripts/${name}`);
  for (const path of scripts) {
    const text = read(path);
    if (/release\.yml|release-gate-evidence|release-phases|sdk-release-evidence|release-native-contract|scripts\/factory\.mjs/.test(text))
      assert(isPlumbing(path), `${path} pins the release plumbing, so κ must leave it out and the plumbing check must run it`);
  }
});

test('κ ignores the plumbing and follows every other byte, mode and path', t => {
  const {path, commit, git} = repository(t);
  const first = commit({'src/a.txt': 'a\n', '.github/workflows/release.yml': 'one\n', 'factory/x.lex.tex': 'x\n'}, 'first');
  const plumbingOnly = commit({'.github/workflows/release.yml': 'two\n', 'factory/y.lex.tex': 'y\n', 'scripts/release-phases.mjs': '//\n'}, 'plumbing');
  assert.equal(kappa(path, plumbingOnly), kappa(path, first));
  const source = commit({'src/a.txt': 'b\n'}, 'source');
  assert.notEqual(kappa(path, source), kappa(path, first));
  const added = commit({'src/b.txt': 'b\n'}, 'added');
  assert.notEqual(kappa(path, added), kappa(path, source));
  git('update-index', '--chmod=+x', 'src/b.txt'); git('commit', '-q', '-m', 'mode');
  assert.notEqual(kappa(path, git('rev-parse', 'HEAD')), kappa(path, added));
  git('mv', 'src/b.txt', 'src/c.txt'); git('commit', '-q', '-m', 'rename');
  const renamed = git('rev-parse', 'HEAD');
  assert.notEqual(kappa(path, renamed), kappa(path, added));
  assert.match(kappa(path, renamed), /^[0-9a-f]{64}$/);
});

const established = () => [{name: 'gate', conclusion: 'success'}, ...vvParts.map(part => ({name: `source-vv (${part})`, conclusion: 'success'}))];
function client(runs, jobs = {}, artifacts = {}) {
  return {api(path) {
    if (path.includes('/workflows/release.yml/runs?')) return {workflow_runs: runs};
    let match = path.match(/\/runs\/(\d+)\/jobs\?filter=latest&per_page=100$/);
    if (match) return {jobs: jobs[match[1]] ?? established()};
    match = path.match(/\/runs\/(\d+)\/artifacts\?name=source-vv$/);
    if (match) return {artifacts: artifacts[match[1]] ?? [{name: 'source-vv', expired: false}]};
    throw new Error(`unexpected ${path}`);
  }};
}
const run = (id, sha, extra = {}) => ({id, head_sha: sha, run_attempt: 2, event: 'push', head_branch: 'release/v0.4',
  path: '.github/workflows/release.yml', repository: {full_name: 'UOR-Foundation/PrismPM'},
  head_repository: {full_name: 'UOR-Foundation/PrismPM'}, ...extra});

test('a receipt is an ancestor\'s passed gate for the same κ, and nothing else', t => {
  const {path, commit, git} = repository(t);
  const proved = commit({'src/a.txt': 'a\n', '.github/workflows/release.yml': 'one\n'}, 'proved');
  const changedSource = commit({'src/a.txt': 'changed\n'}, 'source');
  const restored = commit({'src/a.txt': 'a\n'}, 'restored');
  const current = commit({'.github/workflows/release.yml': 'fixed\n'}, 'plumbing fix');
  git('checkout', '-q', '-b', 'side', proved); const foreign = commit({'.github/workflows/release.yml': 'side\n'}, 'side');
  git('checkout', '-q', 'release/v0.4');
  assert.equal(kappa(path, current), kappa(path, proved));
  const found = findReceipt(path, current, client([run(7, proved)]));
  assert.deepEqual(found, {route: 'proved', kappa: kappa(path, current), source_revision: proved, run_id: '7', run_attempt: '2'});
  // The newest matching ancestor wins: the restored commit has the same κ again.
  assert.equal(findReceipt(path, current, client([run(9, restored), run(7, proved)])).source_revision, restored);
  const none = {route: 'establish', kappa: kappa(path, current), source_revision: current, run_id: '', run_attempt: ''};
  for (const runs of [
    [], [run(7, changedSource)], [run(7, foreign)],
    [run(7, proved, {event: 'pull_request'})], [run(7, proved, {head_branch: 'main'})],
    [run(7, proved, {repository: {full_name: 'fork/PrismPM'}})], [run(7, proved, {head_repository: {full_name: 'fork/PrismPM'}})],
    [run(7, proved, {path: '.github/workflows/vv.yml'})], [run(7, 'x'.repeat(40))],
  ]) assert.deepEqual(findReceipt(path, current, client(runs)), none, JSON.stringify(runs[0] ?? {}));
  // Another run of this very commit (the rehearsal, for the tag run) is a
  // receipt; this run itself never is.
  assert.deepEqual(findReceipt(path, current, client([run(7, current)]), '8'),
    {route: 'proved', kappa: kappa(path, current), source_revision: current, run_id: '7', run_attempt: '2'});
  assert.deepEqual(findReceipt(path, current, client([run(7, current)]), '7'), none);
  const without = name => established().filter(job => job.name !== name);
  for (const jobs of [[], [{name: 'gate', conclusion: 'failure'}, ...without('gate')], [...established(), {name: 'gate', conclusion: 'success'}],
    // A run proved by a receipt skipped source-vv; it is never itself a receipt.
    [{name: 'gate', conclusion: 'success'}, ...vvParts.map(part => ({name: `source-vv (${part})`, conclusion: 'skipped'}))],
    without('source-vv (gates)')])
    assert.deepEqual(findReceipt(path, current, client([run(7, proved)], {7: jobs})), none);
  for (const artifacts of [[], [{name: 'source-vv', expired: true}], [{name: 'source-vv-part-gates', expired: false}]])
    assert.deepEqual(findReceipt(path, current, client([run(7, proved)], {}, {7: artifacts})), none);
});

const built = () => [...Array.from({length: imageJobs}, (_, index) => ({name: `images (image-${index})`, conclusion: 'success'})),
  {name: 'image-platforms (ubuntu-24.04, amd64)', conclusion: 'success'}, {name: 'image-platforms (ubuntu-24.04-arm, arm64)', conclusion: 'success'},
  {name: 'installed-sdk (ubuntu-24.04, amd64)', conclusion: 'failure'}];
function candidates(runs, jobs = {}, artifacts = {}) {
  return {api(path) {
    if (path.includes('/workflows/release.yml/runs?')) return {workflow_runs: runs};
    let match = path.match(/\/runs\/(\d+)\/jobs\?filter=latest&per_page=100$/);
    if (match) return {jobs: jobs[match[1]] ?? built()};
    match = path.match(/\/runs\/(\d+)\/artifacts\?name=sdk-image$/);
    if (match) return {artifacts: artifacts[match[1]] ?? [{name: 'sdk-image', expired: false}]};
    throw new Error(`unexpected ${path}`);
  }};
}

test('a qualification candidate is the newest release-line ancestor whose images were all built and signed', t => {
  const {path, commit, git} = repository(t);
  const older = commit({'src/a.txt': 'a\n'}, 'older');
  const newer = commit({'src/a.txt': 'b\n'}, 'newer');
  const proposal = commit({'scripts/product-sdk-check.sh': 'fixed\n'}, 'check fix');
  git('checkout', '-q', '-b', 'side', older); const foreign = commit({'src/a.txt': 'side\n'}, 'side');
  git('checkout', '-q', 'release/v0.4');
  // A later failure elsewhere in the run does not disqualify its images.
  assert.deepEqual(findCandidate(path, proposal, candidates([run(9, newer), run(7, older)])), {run_id: '9', revision: newer});
  assert.deepEqual(findCandidate(path, proposal, candidates([run(9, newer), run(7, older)], {9: built().slice(1)})), {run_id: '7', revision: older});
  const failed = built(); failed[0] = {...failed[0], conclusion: 'failure'};
  const platform = built(); platform[imageJobs] = {...platform[imageJobs], conclusion: 'cancelled'};
  for (const [runs, jobs, artifacts] of [
    [[run(9, foreign)], {}, {}], [[run(9, 'x'.repeat(40))], {}, {}], [[run(9, newer, {event: 'pull_request'})], {}, {}],
    [[run(9, newer, {head_branch: 'main'})], {}, {}], [[run(9, newer, {head_repository: {full_name: 'fork/PrismPM'}})], {}, {}],
    [[run(9, newer, {path: '.github/workflows/vv.yml'})], {}, {}], [[run(9, newer)], {9: failed}, {}], [[run(9, newer)], {9: platform}, {}],
    [[run(9, newer)], {}, {9: [{name: 'sdk-image', expired: true}]}], [[run(9, newer)], {}, {9: []}], [[], {}, {}],
  ]) assert.throws(() => findCandidate(path, proposal, candidates(runs, jobs, artifacts)), /no release-line run/);
});

// A release workflow and build plumbing, as written, for build identities.
const releaseWorkflow = (extra = {}) => ({'.github/workflows/release.yml': 'name: release\non:\n  push:\njobs:\n'
  + ['policy', 'factory', 'gate', ...Object.keys(buildJobs), 'oci-native'].map(name => `  ${name}:\n    runs-on: ${extra[name] ?? 'x'}\n`).join(''),
  ...Object.fromEntries(buildPlumbing.map(path => [path, `// ${path}\n`]))});

test('the route record binds this commit, its κ, its build identity and the runs that established the evidence', t => {
  const {path, commit} = repository(t);
  const proved = commit({'src/a.txt': 'a\n', ...releaseWorkflow()}, 'proved');
  const current = commit(releaseWorkflow({gate: 'fixed'}), 'plumbing fix');
  const evidence = Buffer.from('{"schema":"prismpm/vv-evidence/1"}\n');
  const build = {route: 'proved', revision: proved, run_id: '7', run_attempt: '2'};
  const record = JSON.parse(routeRecord(path, current, 'proved', {source_revision: proved, run_id: '7', run_attempt: '2'}, evidence, build));
  assert.equal(record.schema, 'prismpm/factory-route/2'); assert.equal(record.revision, current);
  assert.deepEqual(record.build, {route: 'proved', identity: buildIdentity(path, current), revision: proved, run_id: '7', run_attempt: '2',
    jobs: Object.keys(buildJobs), plumbing: [...buildPlumbing]});
  const own = {route: 'established', revision: current, run_id: '9', run_attempt: '1'};
  assert.equal(JSON.parse(routeRecord(path, current, 'proved', {source_revision: proved, run_id: '7', run_attempt: '2'}, evidence, own)).build.route, 'established');
  for (const wrong of [undefined, {...build, route: 'assumed'}, {...own, revision: proved}, {...build, run_id: ''}])
    assert.throws(() => routeRecord(path, current, 'proved', {source_revision: proved, run_id: '7', run_attempt: '2'}, evidence, wrong));
  const rebuilt = commit(releaseWorkflow({gate: 'fixed', images: 'changed'}), 'build plumbing change');
  assert.throws(() => routeRecord(path, rebuilt, 'proved', {source_revision: proved, run_id: '7', run_attempt: '2'}, evidence, build), /different build identity/);
  assert.equal(record.kappa, kappa(path, current)); assert.equal(record.route, 'proved');
  assert.deepEqual(record.plumbing, [...plumbing]);
  assert.equal(record.receipt.source_revision, proved); assert.equal(record.receipt.source_vv_sha256.length, 64);
  const established = JSON.parse(routeRecord(path, current, 'established', {source_revision: current, run_id: '9', run_attempt: '1'}, evidence, own));
  assert.equal(established.receipt.source_revision, current);
  assert.throws(() => routeRecord(path, current, 'established', {source_revision: proved, run_id: '9', run_attempt: '1'}, evidence, own));
  assert.equal(JSON.parse(routeRecord(path, current, 'proved', {source_revision: current, run_id: '9', run_attempt: '1'}, evidence, own)).route, 'proved');
  assert.throws(() => routeRecord(path, current, 'proved', {source_revision: proved, run_id: '', run_attempt: '1'}, evidence, own));
  assert.throws(() => routeRecord(path, current, 'assumed', {source_revision: proved, run_id: '7', run_attempt: '2'}, evidence, own));
  const changed = commit({'src/a.txt': 'b\n'}, 'source change');
  assert.throws(() => routeRecord(path, changed, 'proved', {source_revision: proved, run_id: '7', run_attempt: '2'}, evidence, own), /different κ/);
});

const buildRun = () => Object.entries(buildJobs).flatMap(([name, count]) =>
  Array.from({length: count}, (_, index) => ({name: `${name} (${index})`, conclusion: 'success'})));
const buildOutputs = () => ['sdk', 'runtime', 'adapter-compose', 'adapter-kubernetes', 'adapter-github-pages', 'oracles'].flatMap(image =>
  [`${image}-image`, `${image}-supply-chain`, `reproducibility-${image}-amd64`, `reproducibility-${image}-arm64`])
  .concat(['native-x86_64-unknown-linux-gnu', 'native-aarch64-unknown-linux-gnu', 'source-vv', 'reproducibility-diagnostics-sdk-amd64'],
    ...['amd64', 'arm64'].map(arch => [`product-cli-sdk-${arch}`, `browser-sdk-sdk-${arch}`, `library-sdk-sdk-${arch}`, `sdk-equivalence-${arch}`]))
  .map(name => ({name, expired: false}));
function builds(runs, jobs = {}, artifacts = {}) {
  return {api(path) {
    if (path.includes('/workflows/release.yml/runs?')) return {workflow_runs: runs};
    let match = path.match(/\/runs\/(\d+)\/jobs\?filter=latest&per_page=100$/);
    if (match) return {jobs: jobs[match[1]] ?? buildRun()};
    match = path.match(/\/runs\/(\d+)\/artifacts\?per_page=100$/);
    if (match) return {artifacts: artifacts[match[1]] ?? buildOutputs()};
    throw new Error(`unexpected ${path}`);
  }};
}

test('a build receipt is another run\'s passed build for the same build identity, with every output retained', t => {
  const {path, commit, git} = repository(t);
  const proved = commit({'src/a.txt': 'a\n', ...releaseWorkflow()}, 'proved');
  const gateFix = commit(releaseWorkflow({gate: 'fixed', 'oci-native': 'fixed'}), 'publication plumbing fix');
  const imageFix = commit(releaseWorkflow({gate: 'fixed', 'oci-native': 'fixed', 'image-platforms': 'changed'}), 'build plumbing fix');
  const rebuildFix = commit({'scripts/release-rebuild.sh': 'changed\n'}, 'rebuild plumbing fix');
  const source = commit({'src/a.txt': 'b\n'}, 'source');
  git('checkout', '-q', '-b', 'side', proved); const foreign = commit({'src/z.txt': 'z\n'}, 'side');
  // A release line older than the build route: no build jobs, no build plumbing.
  git('checkout', '-q', '--orphan', 'legacy'); git('rm', '-rqf', '.');
  const legacy = commit({'src/a.txt': 'a\n', '.github/workflows/release.yml': 'name: legacy\njobs:\n  publish:\n    runs-on: x\n'}, 'legacy');
  git('checkout', '-q', '-f', 'release/v0.4');
  // The workflow is split into its header and each job exactly as written.
  assert.deepEqual(Object.keys(workflowJobs(readFileSync(join(path, '.github/workflows/release.yml'), 'utf8'))),
    ['header', 'policy', 'factory', 'gate', ...Object.keys(buildJobs), 'oci-native']);
  assert.throws(() => workflowJobs('name: x\njobs:\n  a:\n  a:\n'), /duplicate job a/);
  // Publication plumbing does not change the build; build plumbing and source do.
  assert.equal(buildIdentity(path, gateFix), buildIdentity(path, proved));
  for (const revision of [imageFix, rebuildFix, source]) assert.notEqual(buildIdentity(path, revision), buildIdentity(path, proved));
  const expected = {build_route: 'proved', build_identity: buildIdentity(path, gateFix), build_revision: proved, build_run_id: '7', build_run_attempt: '2'};
  assert.deepEqual(findBuildReceipt(path, gateFix, builds([run(7, proved)]), '8'), expected);
  // The tag run of a rehearsed commit is proved by the rehearsal, never by itself.
  assert.equal(findBuildReceipt(path, gateFix, builds([run(7, gateFix)]), '8').build_route, 'proved');
  const none = revision => ({build_route: 'establish', build_identity: buildIdentity(path, revision), build_revision: revision, build_run_id: '', build_run_attempt: ''});
  assert.deepEqual(findBuildReceipt(path, gateFix, builds([run(7, gateFix)]), '7'), none(gateFix));
  for (const revision of [imageFix, rebuildFix, source]) assert.deepEqual(findBuildReceipt(path, revision, builds([run(7, proved)]), '8'), none(revision));
  // Such a run is a different identity, never an error that stops the search.
  assert.notEqual(buildIdentity(path, legacy), buildIdentity(path, proved));
  assert.deepEqual(findBuildReceipt(path, gateFix, builds([run(6, legacy), run(7, proved)]), '8'), expected);
  for (const runs of [[run(7, legacy)], [run(7, foreign)], [run(7, proved, {event: 'pull_request'})], [run(7, proved, {head_branch: 'main'})],
    [run(7, proved, {head_repository: {full_name: 'fork/PrismPM'}})], [run(7, proved, {path: '.github/workflows/vv.yml'})]])
    assert.deepEqual(findBuildReceipt(path, gateFix, builds(runs), '8'), none(gateFix));
  for (const name of Object.keys(buildJobs)) {
    const failed = buildRun().map(job => job.name.startsWith(`${name} (`) ? {...job, conclusion: 'failure'} : job);
    const skipped = buildRun().map(job => job.name.startsWith(`${name} (`) ? {...job, conclusion: 'skipped'} : job);
    const missing = buildRun().filter(job => job.name !== `${name} (0)`);
    for (const jobs of [failed, skipped, missing]) assert.deepEqual(findBuildReceipt(path, gateFix, builds([run(7, proved)], {7: jobs}), '8'), none(gateFix), name);
  }
  for (const [pattern] of buildArtifacts) {
    const outputs = buildOutputs();
    const expired = outputs.map(artifact => pattern.test(artifact.name) ? {...artifact, expired: true} : artifact);
    const dropped = outputs.filter((artifact, index) => !(pattern.test(artifact.name) && index === outputs.findIndex(row => pattern.test(row.name))));
    for (const artifacts of [expired, dropped])
      assert.deepEqual(findBuildReceipt(path, gateFix, builds([run(7, proved)], {}, {7: artifacts}), '8'), none(gateFix), String(pattern));
  }
});

test('the release workflow is the model\'s route', () => {
  const model = read('factory/src/Factory/Release.lex.tex');
  const route = definition(model, 'route');
  assert(!route.includes('Stage.installedVv'), 'the route has no installed re-run of source V&V');
  const workflow = load(read('.github/workflows/release.yml'));
  const stageJobs = {sourceVv: ['source-vv'], images: ['image-platforms', 'images'], reproducibility: ['reproducibility'],
    native: ['native'], installedSdk: ['installed-sdk'], publication: ['gate', 'oci-native', 'crates', 'release']};
  const routed = [...route.matchAll(/Stage\.([a-zA-Z]+)/g)].map(match => match[1]);
  assert.deepEqual(routed.sort(), Object.keys(stageJobs).sort());
  const support = ['policy', 'factory', 'devcontainer-image'];
  assert.deepEqual(Object.keys(workflow.jobs).sort(), [...support, ...Object.values(stageJobs).flat()].sort());
  // onlyPublicationWaitsForTheSource: no projection stage needs source V&V or the gate.
  const needs = job => [workflow.jobs[job].needs ?? []].flat();
  const upstream = job => {const seen = new Set(); const visit = name => {for (const need of needs(name)) if (!seen.has(need)) {seen.add(need); visit(need);}}; visit(job); return seen;};
  for (const job of [...stageJobs.images, ...stageJobs.reproducibility, ...stageJobs.native, ...stageJobs.installedSdk])
    assert(!upstream(job).has('source-vv') && !upstream(job).has('gate'), `${job} starts beside source V&V`);
  for (const job of ['oci-native', 'crates', 'release']) assert(upstream(job).has('gate'), `${job} waits for the gate`);
  assert(upstream('gate').has('source-vv') && upstream('gate').has('factory'));
});

test('the factory catalog names only first-party products', () => {
  const files = execFileSync('git', ['ls-files', 'factory'], {cwd: root, encoding: 'utf8'}).split('\n').filter(Boolean);
  assert(files.includes('factory/src/Factory/Catalog.lex.tex'));
  for (const file of files) {
    const text = read(file).toLowerCase();
    for (const name of ['afflom', 'casey-allard', 'rule30', 'prism-legal', 'lexlean-r4', 'compression'])
      assert(!text.includes(name), `${file} names ${name}; downstream work consumes Prism, it is not in the factory`);
  }
  const catalog = read('factory/src/Factory/Catalog.lex.tex');
  assert.deepEqual([...definition(catalog, 'products').matchAll(/Model \{name := "([^"]+)"/g)].map(match => match[1]), ['PrismPM', 'LexLean']);
});
