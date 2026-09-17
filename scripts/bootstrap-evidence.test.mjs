import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, truncateSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { canonical, captureProjection, compare, prepare, publish, sha, sourceManifest, validateCapture } from './bootstrap-evidence.mjs';

const OLD = '4'.repeat(64);
const NEW = '5'.repeat(64);
const lock = compiler => `spec = "lexlean/lock/1"\nlanguage = "1.1"\ncompiler_semantics = "${compiler}"\n`;
const manifest = { files: [{ kind: 'file', path: 'source.lex.tex', sha256: '6'.repeat(64), size: 10 }], schema: 'prismpm/source-manifest/1' };

function reseal(capture) {
  capture.check.snapshot_id = sha(`${canonical(capture.snapshot)}\n`);
  capture.check.semantic_id = capture.snapshot.semantic_id;
  capture.build.source_id = capture.snapshot.source_id;
  capture.build.semantic_id = capture.snapshot.semantic_id;
  capture.model.provenance = {
    ...capture.model.provenance,
    compiler_semantics_id: capture.snapshot.compiler_semantics_id,
    semantic_id: capture.snapshot.semantic_id,
    snapshot_id: capture.check.snapshot_id,
    source_id: capture.snapshot.source_id
  };
  capture.check.model_id = sha(canonical(capture.model));
  capture.manifest.inputs = {
    emitter_semantics_id: capture.model.provenance.emitter_semantics_id,
    lexlean_semantic_id: capture.snapshot.semantic_id,
    lexlean_source_id: capture.snapshot.source_id,
    model_id: capture.check.model_id,
    schema: 'prismpm/build-inputs/1'
  };
  capture.build.build_id = sha(canonical(capture.manifest.inputs));
  const base = `.prism/build/${capture.build.build_id}`;
  capture.build.manifest_path = `${base}/manifest.json`;
  capture.build.model_path = `${base}/model.prism.json`;
  capture.manifest.files = [['lexlean/snapshot.json', capture.snapshot], ['model.prism.json', capture.model]]
    .map(([path, value]) => {
      const bytes = canonical(value) + (path === 'lexlean/snapshot.json' ? '\n' : '');
      return { byte_length: Buffer.byteLength(bytes), kind: 'artifact', path, sha256: sha(bytes) };
    });
  return capture;
}

function fixture(compiler = OLD) {
  const definition = (name, value) => ({ body: { kind: 'string', value }, kind: 'definition', name, parameters: [], result: { kind: 'string' } });
  return reseal({
    build: { build_id: '', manifest_path: '', model_path: '', schema: 'prismpm/build-result/1', semantic_id: '', source_id: '' },
    check: { entity_count: 1, model_id: '', schema: 'prismpm/check-result/1', semantic_id: '', snapshot_id: '' },
    lock: lock(compiler),
    manifest: { files: [], inputs: {}, schema: 'prismpm/build-manifest/1' },
    model: { architecture: { components: ['one'] }, provenance: { emitter_semantics_id: '7'.repeat(64), facet_packages: [{ package: 'fixture', content_id: '8'.repeat(64) }] }, schema: 'prismpm/model-document/1' },
    snapshot: {
      compiler_semantics_id: compiler, language: '1.1', lexicon_closure: { packages: ['one'] },
      modules: [{ name: 'Foundation.Core', imports: [], linked_ir: { term: 1, proof: ['refl'] },
        semantic: { declarations: [definition('sourceManifestDigest', `sha256:${sha(canonical(manifest))}`), definition('sourceManifestFileCount', '1')] },
        source: { path: 'source.lex.tex', sha256: sha('source') } }],
      semantic_id: compiler === OLD ? 'a'.repeat(64) : 'b'.repeat(64),
      source_id: compiler === OLD ? 'c'.repeat(64) : 'd'.repeat(64), spec: 'lexlean/semantic-snapshot/1'
    }
  });
}

test('different reviewed compiler identities retain complete semantic and model equality', () => {
  const before = fixture();
  const after = fixture(NEW);
  const evidence = compare(before, after, manifest, lock(OLD), lock(NEW));
  assert.notEqual(evidence.prior.semantic_id, evidence.current.semantic_id);
  assert.notEqual(evidence.prior.snapshot_id, evidence.current.snapshot_id);
  assert.equal(evidence.shared_content_digest.length, 71);
  assert.equal(evidence.shared_model_digest.length, 71);
});

test('owning comparison rejects resealed semantic, proof, closure, source, import and nested-field mutations', () => {
  for (const mutate of [
    capture => { capture.snapshot.modules[0].linked_ir.term = 2; },
    capture => { capture.snapshot.modules[0].linked_ir.proof = ['admit']; },
    capture => { capture.snapshot.lexicon_closure.packages.push('extra'); },
    capture => { capture.snapshot.modules[0].source.sha256 = '0'.repeat(64); },
    capture => { capture.snapshot.modules[0].imports.push('Hidden'); },
    capture => { capture.snapshot.modules[0].unrecognized = true; },
    capture => { capture.snapshot.modules.reverse(); capture.snapshot.modules.push(structuredClone(capture.snapshot.modules[0])); }
  ]) {
    const changed = fixture(NEW);
    mutate(changed);
    reseal(changed);
    assert.throws(() => compare(fixture(), changed, manifest, lock(OLD), lock(NEW)), /complete bootstrap semantic content differs/);
  }
});

test('owning comparison rejects projected domain and facet drift after all hashes are resealed', () => {
  for (const mutate of [
    capture => { capture.model.architecture.components = ['different']; },
    capture => { capture.model.provenance.facet_packages[0].content_id = '9'.repeat(64); },
    capture => { capture.model.extra = true; }
  ]) {
    const changed = fixture(NEW);
    mutate(changed);
    reseal(changed);
    assert.throws(() => compare(fixture(), changed, manifest, lock(OLD), lock(NEW)), /complete projected model content differs/);
  }
});

test('owning capture validator rejects stale and substituted provenance or unrecognized envelopes', () => {
  for (const mutate of [
    capture => { capture.check.snapshot_id = '0'.repeat(64); },
    capture => { capture.check.model_id = '0'.repeat(64); },
    capture => { capture.build.source_id = '0'.repeat(64); },
    capture => { capture.manifest.inputs.emitter_semantics_id = '0'.repeat(64); capture.build.build_id = sha(canonical(capture.manifest.inputs)); },
    capture => { capture.manifest.files[0].sha256 = '0'.repeat(64); },
    capture => { capture.manifest.files.push(capture.manifest.files[0]); },
    capture => { capture.check.extra = true; },
    capture => { capture.manifest.extra = true; },
    capture => { capture.check.semantic_id = [capture.check.semantic_id]; },
    capture => { capture.snapshot.extra = true; },
    capture => { capture.snapshot.compiler_semantics_id = '0'.repeat(64); },
    capture => { capture.model.schema = 'unknown'; reseal(capture); },
    capture => { capture.manifest.inputs.schema = 'unknown'; capture.build.build_id = sha(canonical(capture.manifest.inputs)); }
  ]) {
    const changed = fixture();
    mutate(changed);
    assert.throws(() => validateCapture(changed, OLD));
  }
});

test('exact accepted compiler and projection lock changes cannot be caller substitutions', () => {
  assert.throws(() => compare(fixture(), fixture(NEW), manifest, lock('0'.repeat(64)), lock(NEW)), /unexpected compiler/);
  assert.throws(() => compare(fixture(), fixture(NEW), manifest, lock(OLD), lock('0'.repeat(64))), /unexpected compiler/);
  const changed = fixture(NEW);
  changed.lock += 'extra = "changed"\n';
  assert.throws(() => compare(fixture(), changed, manifest, lock(OLD), lock(NEW)), /lock changed beyond compiler/);
  changed.lock = lock(NEW) + `compiler_semantics = "${NEW}"\n`;
  assert.throws(() => compare(fixture(), changed, manifest, lock(OLD), lock(NEW)), /ambiguous/);
});

test('modeled manifest digest and count must match the supplied exact source closure', () => {
  const changed = structuredClone(manifest);
  changed.files[0].sha256 = '9'.repeat(64);
  assert.throws(() => compare(fixture(), fixture(NEW), changed, lock(OLD), lock(NEW)), /sourceManifestDigest/);
  const before = fixture();
  const after = fixture(NEW);
  for (const capture of [before, after]) {
    capture.snapshot.modules[0].semantic.declarations[1].body.value = '2';
    reseal(capture);
  }
  assert.throws(() => compare(before, after, manifest, lock(OLD), lock(NEW)), /sourceManifestFileCount/);
});

test('capture reads real artifact closure and rejects file tampering, extra files, and symlinks', () => {
  const root = mkdtempSync(join(tmpdir(), 'prismpm-bootstrap-capture-test.'));
  try {
    const capture = fixture();
    const base = join(root, `.prism/build/${capture.build.build_id}`);
    mkdirSync(join(base, 'lexlean'), { recursive: true });
    writeFileSync(join(root, 'source.lex.tex'), 'source');
    writeFileSync(join(root, 'lexlean.lock'), capture.lock);
    writeFileSync(join(root, 'check.json'), `${canonical(capture.check)}\n`);
    writeFileSync(join(root, 'build.json'), `${canonical(capture.build)}\n`);
    for (const [path, value] of [['manifest.json', capture.manifest], ['model.prism.json', capture.model], ['lexlean/snapshot.json', capture.snapshot]]) {
      writeFileSync(join(base, path), canonical(value) + (path === 'lexlean/snapshot.json' ? '\n' : ''));
    }
    const run = () => captureProjection(root, join(root, 'check.json'), join(root, 'build.json'), join(root, 'capture.json'));
    run();
    assert.equal(readFileSync(join(root, 'capture.json'), 'utf8'), canonical(capture));
    writeFileSync(join(root, 'check.json'), canonical(capture.check));
    assert.throws(run, /noncanonical evidence/);
    writeFileSync(join(root, 'check.json'), `${canonical(capture.check).slice(0, -1)},"entity_count":1}\n`);
    assert.throws(run, /noncanonical evidence/);
    writeFileSync(join(root, 'check.json'), `${canonical(capture.check)}\n`);
    writeFileSync(join(root, 'source.lex.tex'), 'tampered');
    assert.throws(run, /snapshot source digest/);
    writeFileSync(join(root, 'source.lex.tex'), 'source');
    writeFileSync(join(base, 'extra'), 'extra');
    assert.throws(run, /artifact closure/);
    rmSync(join(base, 'extra'));
    writeFileSync(join(base, 'model.prism.json'), '{}');
    assert.throws(run, /artifact bytes/);
    rmSync(join(base, 'model.prism.json'));
    assert.throws(run, /ENOENT/);
    symlinkSync(join(root, 'source.lex.tex'), join(base, 'model.prism.json'));
    assert.throws(run, /symlink/);
    rmSync(join(base, 'model.prism.json'));
    execFileSync('mkfifo', [join(base, 'model.prism.json')]);
    assert.throws(run, /not a bounded regular file/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('preparation rejects untracked helpers and publication rejects changed tracked source without replacing evidence', () => {
  const root = mkdtempSync(join(tmpdir(), 'prismpm-bootstrap-publication-test.'));
  try {
    execFileSync('git', ['init', '--quiet', root]);
    writeFileSync(join(root, 'source'), 'one');
    execFileSync('git', ['-C', root, 'add', 'source']);
    const work = join(root, 'work');
    mkdirSync(join(work, 'envelope'), { recursive: true });
    assert.throws(() => prepare(root, join(work, 'envelope')), /helper is not tracked/);
    writeFileSync(join(work, 'envelope/source-manifest.json'), canonical(sourceManifest(root)));
    mkdirSync(join(root, 'target'));
    writeFileSync(join(root, 'target/bootstrap-evidence.json'), 'old-evidence');
    writeFileSync(join(root, 'source'), 'changed');
    assert.throws(() => publish(root, work), /tracked source changed before evidence publication/);
    assert.equal(readFileSync(join(root, 'target/bootstrap-evidence.json'), 'utf8'), 'old-evidence');
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('tracked source manifest changes for changed bytes, added tracked paths and symlink targets', () => {
  const root = mkdtempSync(join(tmpdir(), 'prismpm-bootstrap-source-test.'));
  try {
    execFileSync('git', ['init', '--quiet', root]);
    writeFileSync(join(root, 'source'), 'one');
    execFileSync('git', ['-C', root, 'add', 'source']);
    const first = canonical(sourceManifest(root));
    writeFileSync(join(root, 'source'), 'two');
    assert.notEqual(canonical(sourceManifest(root)), first);
    symlinkSync('source', join(root, 'link'));
    execFileSync('git', ['-C', root, 'add', 'link']);
    const second = canonical(sourceManifest(root));
    rmSync(join(root, 'link'));
    symlinkSync('elsewhere', join(root, 'link'));
    assert.notEqual(canonical(sourceManifest(root)), second);
    truncateSync(join(root, 'source'), 0);
    assert.equal(sourceManifest(root).files.find(file => file.path === 'source').size, 0);
    truncateSync(join(root, 'source'), 64 * 1024 * 1024 + 1);
    assert.throws(() => sourceManifest(root), /not a bounded regular file/);
    assert.throws(() => canonical(JSON.parse('{"n":9007199254740993}')), /unsafe JSON integer/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});
