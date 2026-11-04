import { test } from 'node:test';
import assert from 'node:assert/strict';
import { BUILD_VARIANT_OMITTED, omitBuildVariantArtifacts } from './inventory-metadata.mjs';

const row = id => ({ id, kind: 'oracle', version: '1', digest: `sha256:${'0'.repeat(64)}` });
const kept = ['conformance-corpus', 'lean', 'lean4-prod', 'lexlean', 'prismpm'].map(row);
const measured = [...kept, ...BUILD_VARIANT_OMITTED.map(row)];

test('build variant keeps every row except the omitted release-acceptance runners', () => {
  const result = omitBuildVariantArtifacts(measured, new Set(['lean', 'lexlean', 'prismpm']));
  assert.deepEqual(result.map(r => r.id), kept.map(r => r.id));
});

test('build variant refuses to omit a row that was never measured', () => {
  const missing = measured.filter(r => r.id !== 'oci-runtime-validate');
  assert.throws(() => omitBuildVariantArtifacts(missing, new Set()), /unmeasured artifact: oci-runtime-validate/);
});

test('build variant refuses a duplicated measured row', () => {
  assert.throws(() => omitBuildVariantArtifacts([...measured, row('openid-conformance')], new Set()), /unmeasured artifact: openid-conformance/);
});

test('build variant refuses when an omitted runner is still on PATH', () => {
  assert.throws(() => omitBuildVariantArtifacts(measured, new Set(['prismpm', 'prismpm-conformance'])),
    /omitted command on PATH: prismpm-conformance/);
});

test('the omitted list never includes what application builds need', () => {
  for (const id of ['lean', 'lean4-prod', 'lean4-prod-exporter', 'lean4-prod-exporter-seed', 'lexlean', 'prismpm',
    'conformance-corpus', 'playwright-headless-shell', 'spdx-validator']) {
    assert(!BUILD_VARIANT_OMITTED.includes(id), id);
  }
});
