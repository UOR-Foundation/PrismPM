// Internal image construction inputs, not SDK or application acceptance.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { constants, closeSync, fstatSync, lstatSync, mkdirSync, mkdtempSync, openSync,
  readSync, readdirSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildClosure, materializeClosure, verifyClosure } from './sdk-vv-inputs.mjs';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const canonical = value => JSON.stringify(value, (_, item) => item && typeof item === 'object' && !Array.isArray(item)
  ? Object.fromEntries(Object.keys(item).sort().map(key => [key, item[key]])) : item) + '\n';
const keys = (value, expected) => assert.deepEqual(Object.keys(value).sort(), expected.slice().sort());
const oid = value => assert.match(value, /^[0-9a-f]{40}$/);
const SOURCE_INPUTS = ['sdk/Dockerfile', 'sdk/vv-inputs.lock.json', 'tools.lock', 'scripts/sdk-vv-inputs.mjs', 'scripts/sdk-image-inputs.mjs'];

function removeOwned(path, identity) {
  const current = lstatSync(path);
  assert(current.isDirectory() && !current.isSymbolicLink() && current.dev === identity.dev && current.ino === identity.ino,
    'refusing cleanup of replaced image-input scratch');
  rmSync(path, { recursive: true });
}

function read(path, limit = 1024 * 1024) {
  assert.equal(realpathSync(path), resolve(path), 'input path must not be aliased');
  assert(lstatSync(path).isFile(), 'bounded regular input required');
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    const before = fstatSync(fd, { bigint: true });
    assert(before.isFile() && before.nlink === 1n && before.size <= BigInt(limit), 'bounded regular input required');
    const bytes = Buffer.alloc(Number(before.size));
    for (let offset = 0; offset < bytes.length;) {
      const count = readSync(fd, bytes, offset, bytes.length - offset, null);
      assert(count > 0, 'input shortened'); offset += count;
    }
    assert.equal(readSync(fd, Buffer.alloc(1), 0, 1, null), 0, 'input grew');
    const after = fstatSync(fd, { bigint: true });
    for (const key of ['dev', 'ino', 'size', 'mode', 'nlink', 'mtimeNs', 'ctimeNs']) assert.equal(after[key], before[key], 'input changed');
    const current = lstatSync(path, {bigint: true});
    for (const key of ['dev', 'ino', 'size', 'mode', 'nlink', 'mtimeNs', 'ctimeNs']) assert.equal(current[key], before[key], 'input path changed');
    assert.equal(realpathSync(path), resolve(path), 'input path changed');
    return bytes;
  } finally { closeSync(fd); }
}

export function inputPolicy(source, revision) {
  oid(revision);
  const bytes = read(join(source, 'sdk/vv-inputs.lock.json'));
  const authority = JSON.parse(bytes);
  assert.equal(canonical(authority), bytes.toString(), 'canonical authority lock required');
  keys(authority, ['schema', 'advisory']);
  assert.equal(authority.schema, 'prismpm/sdk-image-input-authorities/1');
  keys(authority.advisory, ['url', 'revision', 'tree']);
  assert.equal(authority.advisory.url, 'https://github.com/RustSec/advisory-db');
  oid(authority.advisory.revision); oid(authority.advisory.tree);
  const sourceLock = read(join(source, 'tools.lock')).toString();
  const section = sourceLock.match(/^\[bootstrap-sdk\]\r?\n([\s\S]*?)(?=^\[|$(?![\s\S]))/m)?.[1];
  assert(section, 'bootstrap authority missing');
  const lines = section.split('\n').map(line => line.trim()).filter(line => line && !line.startsWith('#'));
  const fields = lines.map(line => {
    const field = /^([a-z][a-z0-9_]*) = "([^"\r\n]+)"$/.exec(line);
    assert(field, 'closed bootstrap authority syntax'); return field;
  });
  assert.equal(new Set(fields.map(row => row[1])).size, fields.length, 'duplicate bootstrap field');
  const bootstrap = Object.fromEntries(fields.map(row => [row[1], row[2]]));
  keys(bootstrap, ['version', 'source_commit', 'tag_object', 'url', 'sha256']);
  assert.equal(bootstrap.version, '0.2.0');
  assert.equal(bootstrap.url, 'https://github.com/UOR-Foundation/PrismPM/releases/download/v0.2.0/prismpm-0.2.0-x86_64-unknown-linux-gnu.tar.gz');
  oid(bootstrap.source_commit); oid(bootstrap.tag_object); assert.match(bootstrap.sha256, /^[0-9a-f]{64}$/);
  return { schema: 'prismpm/sdk-vv-input-policy/1', source_revision: revision,
    historical_revision: bootstrap.source_commit, historical_tag: bootstrap.tag_object,
    bootstrap_sha256: bootstrap.sha256, advisory_revision: authority.advisory.revision,
    advisory_tree: authority.advisory.tree };
}

function boundInputs(source, manifest) {
  for (const path of SOURCE_INPUTS) {
    const row = manifest.source.files.find(row => row.path === path);
    assert(row && row.mode === '100644', `committed image-input helper/policy required: ${path}`);
    const bytes = read(join(source, path));
    assert.equal(bytes.length, row.byte_length); assert.equal(sha(bytes), row.sha256, `image-input source differs: ${path}`);
  }
}

// Explicit paths are lower-level data inputs, never authority overrides. The
// production CLI acquires the independently pinned official sources below.
export function captureImageInputs({ source, revision, advisory, bootstrap, destination }) {
  const policy = inputPolicy(source, revision);
  const manifest = buildClosure({ source, advisory, bootstrap, policy, destination });
  boundInputs(source, manifest);
  verifyClosure(destination, policy);
  return policy;
}

function acquisition(command, args, cwd) {
  const result = spawnSync(command, args, { cwd, encoding: 'utf8', timeout: 300000, maxBuffer: 1024 * 1024,
    env: { PATH: process.env.PATH, LANG: 'C', LC_ALL: 'C', GIT_CONFIG_NOSYSTEM: '1',
      GIT_CONFIG_GLOBAL: '/dev/null', GIT_NO_REPLACE_OBJECTS: '1', GIT_TERMINAL_PROMPT: '0', GIT_ALLOW_PROTOCOL: 'https' } });
  assert.equal(result.error, undefined, 'input acquisition failed');
  assert.equal(result.signal, null, 'input acquisition interrupted');
  assert.equal(result.status, 0, `official input acquisition failed: ${command}`);
}

export function prepareImageInputs(source, revision, destination) {
  const policy = inputPolicy(source, revision);
  const work = mkdtempSync(join(tmpdir(), 'prismpm-image-acquire-'));
  const identity = lstatSync(work);
  try {
    const advisory = join(work, 'advisory'); mkdirSync(advisory);
    acquisition('git', ['init', '--quiet', '--template='], advisory);
    acquisition('git', ['fetch', '--quiet', '--depth=1', 'https://github.com/RustSec/advisory-db', policy.advisory_revision], advisory);
    acquisition('git', ['checkout', '--quiet', '--detach', policy.advisory_revision], advisory);
    const bootstrap = join(work, 'bootstrap.tar.gz');
    acquisition('curl', ['--disable', '--proto', '=https', '--proto-redir', '=https', '--tlsv1.2', '--fail', '--location',
      '--silent', '--show-error', '--max-time', '240', '--max-filesize', '67108864',
      'https://github.com/UOR-Foundation/PrismPM/releases/download/v0.2.0/prismpm-0.2.0-x86_64-unknown-linux-gnu.tar.gz', '--output', bootstrap], work);
    return captureImageInputs({ source, revision, advisory, bootstrap, destination });
  } finally { removeOwned(work, identity); }
}

export function stageImageInputs(closure, policyRoot, revision, destination) {
  const policy = inputPolicy(policyRoot, revision);
  const manifest = verifyClosure(closure, policy);
  boundInputs(policyRoot, manifest);
  const materialized = materializeClosure(closure, policy, destination);
  assert.deepEqual(materialized, manifest);
  // Only this newly created Git store is removed. Raw tracked source remains
  // byte-exact; history is retained independently in the sealed source pack.
  const git = join(destination, 'source/.git');
  assert(lstatSync(git).isDirectory() && !lstatSync(git).isSymbolicLink());
  rmSync(git, { recursive: true });
  rmSync(join(destination, 'advisory'), { recursive: true });
  rmSync(join(destination, 'bootstrap.tar.gz'));
  writeFileSync(join(destination, 'policy.json'), canonical(policy), { flag: 'wx', mode: 0o444 });
  return manifest;
}

const TARGETS = ['runtime', 'adapter-compose', 'adapter-kubernetes', 'adapter-github-pages', 'oracles', 'cli-archive'];
const builderName = value => assert.match(value, /^[A-Za-z0-9][A-Za-z0-9_.-]{0,127}$/, 'bounded Buildx builder name required');
export function dockerArguments(source, revision, target, closure, args) {
  oid(revision); assert(TARGETS.includes(target), 'closed SDK build target');
  assert(Array.isArray(args));
  const values = new Set(['--builder', '--platform', '--output', '--tag', '--metadata-file', '--label']);
  let builderSeen = false;
  for (let i = 0; i < args.length; i++) {
    assert.equal(typeof args[i], 'string'); assert(!args[i].includes('\0'));
    if (values.has(args[i])) {
      assert(typeof args[++i] === 'string' && args[i].length > 0 && !args[i].startsWith('-') && !args[i].includes('\0'));
      if (args[i - 1] === '--builder') {
        assert(!builderSeen, 'one Buildx builder required'); builderSeen = true; builderName(args[i]);
      }
      continue;
    }
    if (args[i] === '--build-arg') {
      assert.match(args[++i] ?? '', /^(?:SOURCE_DATE_EPOCH=0|SDK_VERSION=[0-9]+\.[0-9]+\.[0-9]+)$/); continue;
    }
    assert(['--no-cache', '--load', '--provenance=false', '--sbom=false'].includes(args[i]), `unsupported SDK build argument: ${args[i]}`);
  }
  return ['buildx', 'build', '--build-context', `vv-inputs=${resolve(closure)}`,
    '--build-arg', `SDK_SOURCE_REVISION=${revision}`, '--file', join(resolve(source), 'sdk/Dockerfile'),
    '--target', target, ...args, resolve(source)];
}

function verifyOutputPaths(paths, work) {
  for (const output of paths) {
    if (work) assert(output !== work && !output.startsWith(work + '/') && !work.startsWith(output + '/'),
      'build-set outputs must remain outside private image inputs');
    // Check every existing component, including dangling links. A missing
    // suffix is allowed; Docker owns creation of the explicitly named output.
    for (let path = output;; path = dirname(path)) {
      let current;
      try { current = lstatSync(path); }
      catch (error) { if (error.code !== 'ENOENT') throw error; }
      if (current) {
        assert(!current.isSymbolicLink() && realpathSync(path) === path, 'build-set output path must not be aliased');
        assert(current.isDirectory() || (path === output && current.isFile() && current.nlink === 1),
          'build-set output requires unaliased directories or a single-link regular file');
      }
      if (dirname(path) === path) break;
    }
  }
}

// A build set is an exact output plan for one owning producer. It cannot
// provide input paths, source authorities, Dockerfile/context overrides, or a
// per-build builder/platform. Copy the validated rows before acquisition so a
// caller cannot expand the plan while the authenticated inputs are prepared.
function buildSetPlan(plan) {
  assert(plan && typeof plan === 'object' && !Array.isArray(plan), 'closed SDK build-set plan');
  keys(plan, ['builder', 'platform', 'builds']);
  builderName(plan.builder);
  assert(['linux/amd64', 'linux/arm64'].includes(plan.platform), 'one supported SDK build platform required');
  assert(Array.isArray(plan.builds) && plan.builds.length > 0 && plan.builds.length <= TARGETS.length, 'bounded SDK build set required');
  const targets = new Set(), outputs = new Set(), paths = [];
  const ownPath = value => {
    assert(value && value !== '-' && !/[\x00-\x1f]/.test(value), 'bounded output path required');
    assert(!value.split('/').includes('..'), 'build-set output path must not traverse parents');
    const path = resolve(value);
    assert.notEqual(path, dirname(path), 'bounded output destination required');
    verifyOutputPaths([path]);
    for (const previous of paths) assert(path !== previous && !path.startsWith(previous + '/') && !previous.startsWith(path + '/'),
      'build-set output paths must be distinct and disjoint');
    paths.push(path);
  };
  const builds = plan.builds.map(row => {
    assert(row && typeof row === 'object' && !Array.isArray(row), 'closed SDK build-set row');
    keys(row, ['target', 'args']);
    assert(TARGETS.includes(row.target) && !targets.has(row.target), 'unique closed SDK build-set target required');
    targets.add(row.target);
    assert(Array.isArray(row.args) && row.args.length <= 64, 'bounded SDK build-set arguments required');
    assert(row.args.every(value => typeof value === 'string' && value.length > 0 && value.length <= 4096 && !/[\x00-\x1f]/.test(value)),
      'bounded SDK build-set argument required');
    const seen = new Set(); let output;
    for (let i = 0; i < row.args.length; i++) {
      const option = row.args[i];
      assert(!['--builder', '--platform', '--load', '--tag'].includes(option), 'build-set ownership and outputs are explicit');
      let key = option;
      if (['--output', '--metadata-file', '--label', '--build-arg'].includes(option)) {
        const value = row.args[++i];
        assert(typeof value === 'string', 'SDK build option value required');
        if (['--label', '--build-arg'].includes(option)) {
          assert.match(value, /^[A-Za-z0-9][A-Za-z0-9_.-]*=.+$/, 'closed keyed SDK build option required');
          key += ':' + value.split('=')[0];
        }
        if (option === '--output') output = value;
        if (option === '--metadata-file') ownPath(value);
      }
      assert(!seen.has(key), `duplicate SDK build-set option: ${key}`); seen.add(key);
    }
    assert(output, 'one explicit SDK build-set output required');
    const fields = output.split(',').map(field => {
      const match = /^([a-z][a-z-]*)=([^=,]+)$/.exec(field);
      assert(match, 'closed SDK output syntax required'); return [match[1], match[2]];
    });
    assert.equal(new Set(fields.map(([key]) => key)).size, fields.length, 'duplicate SDK output field');
    const exporter = Object.fromEntries(fields);
    const allowed = exporter.type === 'registry' ? ['type', 'name', 'push-by-digest', 'name-canonical', 'rewrite-timestamp', 'oci-mediatypes']
      : exporter.type === 'oci' ? ['type', 'dest', 'rewrite-timestamp', 'oci-mediatypes', 'tar'] : ['type', 'dest'];
    assert(Object.keys(exporter).every(key => allowed.includes(key)), 'unsupported SDK output field');
    assert(row.target === 'cli-archive' ? exporter.type === 'local' : ['registry', 'oci'].includes(exporter.type), 'SDK target output type differs');
    for (const key of ['push-by-digest', 'name-canonical', 'rewrite-timestamp', 'oci-mediatypes'])
      if (key in exporter) assert.equal(exporter[key], 'true', 'closed SDK output flag required');
    if ('tar' in exporter) assert.equal(exporter.tar, 'false', 'closed SDK OCI tar flag required');
    if (exporter.type === 'registry') {
      assert.match(exporter.name ?? '', /^[a-z0-9]+(?:[._:/-][a-z0-9]+)*$/, 'closed SDK registry output name required');
      assert(seen.has('--metadata-file'), 'registry output metadata required');
      assert(!outputs.has(exporter.name), 'distinct SDK registry outputs required'); outputs.add(exporter.name);
    } else ownPath(exporter.dest);
    const args = ['--builder', plan.builder, '--platform', plan.platform, ...row.args];
    dockerArguments('/unused', '0'.repeat(40), row.target, '/unused-inputs', args);
    return {target: row.target, args};
  });
  return {builds, outputPaths: paths};
}

// Dependency injection is confined to unit transport tests, which execute a
// recording Docker executable. The CLI always performs real acquisition.
function constructImages(source, revision, builds, prepare, outputPaths = []) {
  // Validate every invocation before allocating or acquiring any input.
  for (const {target, args} of builds) dockerArguments(source, revision, target, '/unused-inputs', args);
  const work = mkdtempSync(join(tmpdir(), 'prismpm-sdk-image-'));
  const identity = lstatSync(work);
  try {
    verifyOutputPaths(outputPaths, work);
    const closure = join(work, 'inputs');
    const policy = prepare(source, revision, closure);
    assert.deepEqual(policy, inputPolicy(source, revision), 'independent source policy differs');
    const materialized = join(work, 'materialized');
    const manifest = materializeClosure(closure, policy, materialized);
    boundInputs(source, manifest);
    const context = join(work, 'recipe'); mkdirSync(context, {mode: 0o700});
    for (const path of SOURCE_INPUTS) {
      const destination = join(context, path);
      mkdirSync(dirname(destination), {recursive: true, mode: 0o700});
      writeFileSync(destination, read(join(materialized, 'source', path)), {flag: 'wx', mode: 0o400});
    }
    boundInputs(context, manifest);
    // The verified full checkout is not a Docker context. Only the five
    // independently bound bootstrap files cross the raw recipe boundary.
    removeOwned(materialized, lstatSync(materialized));
    const nodes = [context, join(context, 'sdk'), join(context, 'scripts'), ...SOURCE_INPUTS.map(path => join(context, path)),
      closure, ...['advisory.pack', 'bootstrap.tar.gz', 'manifest.json', 'source.pack'].map(path => join(closure, path))]
      .map(path => ({path, identity: lstatSync(path, {bigint: true})}));
    const verify = () => {
      verifyOutputPaths(outputPaths, work);
      for (const {path, identity} of nodes) {
        assert.equal(realpathSync(path), path, 'image input custody path changed');
        const current = lstatSync(path, {bigint: true});
        for (const key of ['dev', 'ino', 'size', 'mode', 'nlink', 'mtimeNs', 'ctimeNs'])
          assert.equal(current[key], identity[key], path === closure || path.startsWith(closure + '/')
            ? 'image input closure custody changed' : 'recipe context custody changed');
      }
      assert.deepEqual(readdirSync(context).sort(), ['scripts', 'sdk', 'tools.lock']);
      assert.deepEqual(readdirSync(join(context, 'sdk')).sort(), ['Dockerfile', 'vv-inputs.lock.json']);
      assert.deepEqual(readdirSync(join(context, 'scripts')).sort(), ['sdk-image-inputs.mjs', 'sdk-vv-inputs.mjs']);
      boundInputs(context, manifest);
      assert.deepEqual(verifyClosure(closure, policy), manifest, 'image input closure changed');
    };
    let status = 0;
    for (const {target, args} of builds) {
      verify();
      const argv = dockerArguments(context, revision, target, closure, args);
      let result;
      try { result = spawnSync('docker', argv, { stdio: 'inherit' }); }
      finally { verify(); }
      if (result.error) throw result.error;
      assert.equal(result.signal, null, 'SDK image construction interrupted');
      status = result.status;
      if (status !== 0) break;
    }
    return {status,
      input_policy: Buffer.from(canonical(policy)),
      input_manifest: Buffer.from(canonical(manifest))};
  } finally { removeOwned(work, identity); }
}

export function buildImage(source, revision, target, args, prepare = prepareImageInputs) {
  return constructImages(source, revision, [{target, args}], prepare).status;
}

export function buildImageSet(source, revision, plan, prepare = prepareImageInputs) {
  const {builds, outputPaths} = buildSetPlan(plan);
  return constructImages(source, revision, builds, prepare, outputPaths).status;
}

// A separate source-CI constructor exposes only the actual complete verified
// inputs of this construction. No caller-provided receipt can replace them.
// It does not qualify the built SDK or any consuming gate.
export function buildImageWithInputs(source, revision, args, prepare = prepareImageInputs) {
  const result = constructImages(source, revision, [{target: 'runtime', args}], prepare);
  assert.equal(result.status, 0, 'source-CI SDK input construction failed');
  return result;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [command, ...args] = process.argv.slice(2);
  if (command === 'build' && args.length >= 3) {
    const [source, revision, target, ...buildArgs] = args;
    process.exitCode = buildImage(source, revision, target, buildArgs);
  } else if (command === 'build-set' && args.length === 3) {
    const [source, revision, encoded] = args;
    assert(Buffer.byteLength(encoded) <= 65536, 'bounded SDK build-set JSON required');
    process.exitCode = buildImageSet(source, revision, JSON.parse(encoded));
  } else if (command === 'stage' && args.length === 4) {
    const [closure, policyRoot, revision, destination] = args;
    stageImageInputs(closure, policyRoot, revision, destination);
  } else if (command === 'digest' && args.length === 1) {
    const value = JSON.parse(read(args[0]));
    assert.match(value['containerimage.digest'], /^sha256:[0-9a-f]{64}$/);
    process.stdout.write(value['containerimage.digest'] + '\n');
  } else throw Error('usage: sdk-image-inputs.mjs build SOURCE REVISION TARGET [DOCKER_OPTIONS...] | build-set SOURCE REVISION PLAN_JSON | stage CLOSURE POLICY_ROOT REVISION DEST | digest METADATA');
}
