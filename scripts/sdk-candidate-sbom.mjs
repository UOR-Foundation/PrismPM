// Complete SPDX transported as a signed OCI 1.1 artifact, not an inline
// SPDX attestation and never evidence of production acceptance.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

export const repository = 'ghcr.io/uor-foundation/prismpm-sdk-candidate';
export const source = 'https://github.com/UOR-Foundation/PrismPM';
export const workflow = `${source}/.github/workflows/sdk-candidate.yml@refs/heads/main`;
export const issuer = 'https://token.actions.githubusercontent.com';
export const manifestType = 'application/vnd.oci.image.manifest.v1+json';
export const spdxType = 'application/spdx+json';
export const emptyConfig = Buffer.from('{}');
export const trustRootDigest = 'sha256:844a1c6de3986c9f02070266b25e0d1a2fa99ceccc89f6b9ad90aae47b62a16e';
export const digest = bytes => `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
const encode = value => Buffer.from(`${JSON.stringify(value)}\n`);
const read = (directory, name) => readFileSync(join(directory, name));
const run = (tool, args) => execFileSync(tool, args, {stdio: ['ignore', 'pipe', 'inherit'], maxBuffer: 8 * 1024 * 1024});

export function describeSpdx(bytes) {
  const document = JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(bytes));
  assert.equal(document.spdxVersion, 'SPDX-2.3');
  assert.equal(document.SPDXID, 'SPDXRef-DOCUMENT');
  assert.equal(document.dataLicense, 'CC0-1.0');
  assert.equal(typeof document.documentNamespace, 'string');
  assert.ok(document.documentNamespace.length > 0);
  // These counts detect accidental content loss; they do not assert semantic
  // completeness or independently certify Syft's inventory accuracy.
  const counts = {};
  for (const key of ['packages', 'files', 'relationships', 'hasExtractedLicensingInfos']) {
    assert.ok(Array.isArray(document[key]), `complete image SPDX must include ${key}`);
    counts[key] = document[key].length;
  }
  assert.ok(counts.packages > 0 && counts.files > 0 && counts.relationships > 0);
  return {media_type: spdxType, digest: digest(bytes), size: bytes.length, counts};
}

export function sbomRecord(candidateBytes, childBytes, sbomBytes, architecture, revision, childDigest) {
  assert.ok(['amd64', 'arm64'].includes(architecture));
  assert.match(revision, /^[0-9a-f]{40}$/);
  assert.match(childDigest, /^sha256:[0-9a-f]{64}$/);
  const candidate = JSON.parse(candidateBytes);
  assert.equal(candidate.source_revision, revision);
  assert.equal(candidate.platform, `linux/${architecture}`);
  assert.equal(candidate.manifest_digest, childDigest);
  assert.equal(candidate.development_only, true);
  assert.equal(candidate.production_accepted, false);
  assert.equal(digest(childBytes), childDigest);
  const child = JSON.parse(childBytes);
  assert.equal(child.schemaVersion, 2);
  assert.equal(child.mediaType, manifestType);
  return {
    source_revision: revision, platform: candidate.platform,
    subject: {mediaType: manifestType, digest: childDigest, size: childBytes.length},
    candidate_evidence_digest: digest(candidateBytes),
    inventory_digest: candidate.inventory_digest, standards_lock_digest: candidate.standards_lock_digest,
    spdx: describeSpdx(sbomBytes), development_only: true, production_accepted: false,
  };
}

export function artifactManifest(record) {
  return {
    schemaVersion: 2, mediaType: manifestType, artifactType: spdxType,
    config: {mediaType: 'application/vnd.oci.empty.v1+json', digest: digest(emptyConfig), size: emptyConfig.length},
    layers: [{mediaType: spdxType, digest: record.spdx.digest, size: record.spdx.size,
      annotations: {'org.opencontainers.image.title': 'sbom.spdx.json'}}],
    subject: record.subject,
    annotations: {
      'org.opencontainers.image.source': source,
      'org.opencontainers.image.revision': record.source_revision,
      'com.prismpm.sdk.workflow': workflow,
      'com.prismpm.sdk.platform': record.platform,
      'com.prismpm.sdk.candidate-evidence-digest': record.candidate_evidence_digest,
      'com.prismpm.sdk.inventory-digest': record.inventory_digest,
      'com.prismpm.sdk.standards-lock-digest': record.standards_lock_digest,
      'com.prismpm.sdk.development-only': 'true',
      'com.prismpm.sdk.production-accepted': 'false',
    },
  };
}

export function validateArtifact(bytes, expectedDigest, record, sbomBytes) {
  assert.equal(digest(bytes), expectedDigest);
  assert.deepEqual(describeSpdx(sbomBytes), record.spdx);
  assert.deepEqual(JSON.parse(bytes), artifactManifest(record));
}

export function verificationArguments(record, artifactDigest, trustedRoot) {
  assert.match(record.source_revision, /^[0-9a-f]{40}$/);
  assert.match(artifactDigest, /^sha256:[0-9a-f]{64}$/);
  return ['verify', '--trusted-root', trustedRoot,
    '--certificate-identity', workflow, '--certificate-oidc-issuer', issuer,
    '--certificate-github-workflow-repository', 'UOR-Foundation/PrismPM',
    '--certificate-github-workflow-ref', 'refs/heads/main',
    '--certificate-github-workflow-sha', record.source_revision,
    '--certificate-github-workflow-trigger', 'workflow_dispatch',
    '--certificate-github-workflow-name', 'PrismPM development SDK candidate',
    `${repository}@${artifactDigest}`];
}

export function signingArguments(artifactDigest, trustedRoot, bundlePath) {
  assert.match(artifactDigest, /^sha256:[0-9a-f]{64}$/);
  // Cosign 3.1.3 defaults to Sigstore bundles stored as OCI referrers. Its
  // separate legacy-signature referrers flag requires experimental mode and
  // is neither necessary nor used by the default bundle producer/verifier.
  return ['sign', '--yes', '--oidc-provider', 'github-actions', '--trusted-root', trustedRoot,
    '--bundle', bundlePath, `${repository}@${artifactDigest}`];
}

// Caller must first validate the original smoke evidence against the exact
// inspected image, raw inventory and successful locked standards resolution.
export function handleSbom(command, directory, architecture, revision, childDigest, trustedRoot) {
  const record = sbomRecord(read(directory, 'candidate.json'), read(directory, 'manifest.json'),
    read(directory, 'sbom.spdx.json'), architecture, revision, childDigest);
  if (command === 'sbom-record') {
    writeFileSync(join(directory, 'sbom-record.json'), encode(record));
    return;
  }
  assert.deepEqual(JSON.parse(read(directory, 'sbom-record.json')), record);
  const manifestBytes = encode(artifactManifest(record));
  const artifactDigest = digest(manifestBytes);
  const reference = `${repository}@${artifactDigest}`;
  if (command === 'sbom-publish') {
    assert.equal(process.env.GITHUB_REPOSITORY, 'UOR-Foundation/PrismPM');
    assert.equal(process.env.GITHUB_REF, 'refs/heads/main');
    assert.equal(process.env.GITHUB_SHA, revision);
    assert.equal(process.env.GITHUB_EVENT_NAME, 'workflow_dispatch');
    assert.equal(digest(readFileSync(trustedRoot)), trustRootDigest);
    writeFileSync(join(directory, 'sbom-manifest.json'), manifestBytes);
    writeFileSync(join(directory, 'sbom-config.json'), emptyConfig);
    // Registry digest parameters make the registry independently reject any
    // changed/truncated upload. No SPDX serialization, filtering or sharding.
    for (const [name, hash] of [['sbom-config.json', digest(emptyConfig)], ['sbom.spdx.json', record.spdx.digest]]) {
      run('oras', ['blob', 'push', `${repository}@${hash}`, join(directory, name)]);
    }
    run('oras', ['manifest', 'push', reference, join(directory, 'sbom-manifest.json')]);
    run('cosign', signingArguments(artifactDigest, trustedRoot, join(directory, 'sbom.sigstore.json')));
  } else assert.equal(command, 'sbom-verify');
  // Verification is cryptographic, with normal certificate/transparency
  // checks. Reading unsigned predicates or accepting a saved success flag is
  // never sufficient. Re-run this command to independently verify publication.
  assert.equal(digest(readFileSync(trustedRoot)), trustRootDigest);
  const verified = run('cosign', verificationArguments(record, artifactDigest, trustedRoot));
  run('oras', ['manifest', 'fetch', reference, '--output', join(directory, 'sbom-manifest.received.json')]);
  run('oras', ['blob', 'fetch', `${repository}@${record.spdx.digest}`,
    '--output', join(directory, 'sbom.received.spdx.json')]);
  validateArtifact(read(directory, 'sbom-manifest.received.json'), artifactDigest, record,
    read(directory, 'sbom.received.spdx.json'));
  writeFileSync(join(directory, 'sbom-signature-verification.json'), verified);
  writeFileSync(join(directory, 'sbom-artifact.json'), encode({
    kind: 'signed-oci-sbom-artifact', reference, manifest_digest: artifactDigest,
    ...record, certificate_identity: workflow, certificate_oidc_issuer: issuer,
    signature_verification_digest: digest(verified),
  }));
}
