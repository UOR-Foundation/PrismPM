// Infrastructure evidence only; these checks never grant release acceptance.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const sha = bytes => `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
const json = path => JSON.parse(readFileSync(path));
const checks = ['non-root', 'inventory', 'model-check', 'shadowed-tool-rejected'];
export function validateConfig(config, architecture, revision) {
  assert.match(revision, /^[0-9a-f]{40}$/);
  assert.ok(['amd64', 'arm64'].includes(architecture));
  assert.equal(config.architecture, architecture);
  assert.equal(config.os, 'linux');
  const labels = config.config.Labels;
  assert.equal(labels['org.opencontainers.image.revision'], revision);
  assert.equal(labels['org.opencontainers.image.source'], 'https://github.com/UOR-Foundation/PrismPM');
  assert.equal(labels['org.opencontainers.image.version'], '0.3.0');
  assert.equal(labels['org.opencontainers.image.created'], '1970-01-01T00:00:00Z');
}

export function validateEnvironment(environment, branches) {
  assert.equal(environment.name, 'sdk-candidate');
  assert.equal(environment.deployment_branch_policy?.custom_branch_policies, true);
  assert.equal(environment.deployment_branch_policy?.protected_branches, false);
  // GitHub enforces any configured reviewers when the publish job enters this
  // environment; this bootstrap does not invent an additional reviewer policy.
  assert.equal(branches.total_count, 1);
  assert.deepEqual(branches.branch_policies.map(row => [row.name, row.type]), [['main', 'branch']]);
}

export function validateEvidence(evidence, architecture, revision, digest, inventoryBytes) {
  assert.deepEqual(evidence, {
    source_revision: revision, platform: `linux/${architecture}`, manifest_digest: digest,
    inventory_digest: sha(inventoryBytes), development_only: true, production_accepted: false, checks,
  });
}

function inspect(archive, architecture, revision, directory) {
  mkdirSync(directory, {recursive: true});
  const index = JSON.parse(execFileSync('tar', ['-xOf', archive, 'index.json']));
  assert.equal(index.schemaVersion, 2);
  assert.equal(index.manifests.length, 1);
  const descriptor = index.manifests[0];
  assert.equal(descriptor.mediaType, 'application/vnd.oci.image.manifest.v1+json');
  assert.match(descriptor.digest, /^sha256:[0-9a-f]{64}$/);
  const reference = `${archive}@${descriptor.digest}`;
  execFileSync('oras', ['manifest', 'fetch', '--oci-layout', reference, '--output', `${directory}/manifest.json`]);
  const bytes = readFileSync(`${directory}/manifest.json`);
  assert.equal(sha(bytes), descriptor.digest);
  assert.equal(bytes.length, descriptor.size);
  const manifest = JSON.parse(bytes);
  assert.equal(manifest.schemaVersion, 2);
  assert.equal(manifest.mediaType, descriptor.mediaType);
  execFileSync('oras', ['manifest', 'fetch-config', '--oci-layout', reference, '--output', `${directory}/config.json`]);
  const configBytes = readFileSync(`${directory}/config.json`);
  assert.equal(sha(configBytes), manifest.config.digest);
  assert.equal(configBytes.length, manifest.config.size);
  validateConfig(JSON.parse(configBytes), architecture, revision);
  writeFileSync(`${directory}/digest.txt`, `${descriptor.digest}\n`);
}

function main([command, ...args]) {
  switch (command) {
    case 'inspect': assert.equal(args.length, 4); inspect(...args); break;
    case 'environment': {
      assert.equal(process.env.GITHUB_REPOSITORY, 'UOR-Foundation/PrismPM');
      assert.equal(process.env.GITHUB_REF, 'refs/heads/main');
      const endpoint = 'repos/UOR-Foundation/PrismPM/environments/sdk-candidate';
      validateEnvironment(JSON.parse(execFileSync('gh', ['api', endpoint])),
        JSON.parse(execFileSync('gh', ['api', `${endpoint}/deployment-branch-policies`])));
      break;
    }
    case 'record': {
      const [directory, architecture, revision, digest] = args;
      assert.equal(args.length, 4);
      assert.equal(json(`${directory}/cli.json`).schema, 'prismpm/completion-result/1');
      const model = json(`${directory}/model-check.json`);
      assert.equal(model.schema, 'prismpm/check-result/1');
      for (const field of ['model_id', 'semantic_id', 'snapshot_id']) assert.match(model[field], /^[0-9a-f]{64}$/);
      const inventory = json(`${directory}/inventory.json`);
      assert.equal(inventory.schema, 'prismpm/sdk-inventory/1');
      assert.ok(inventory.commands.length > 0 && inventory.artifacts.length > 0);
      const evidence = {source_revision: revision, platform: `linux/${architecture}`, manifest_digest: digest,
        inventory_digest: sha(readFileSync(`${directory}/inventory.json`)),
        development_only: true, production_accepted: false, checks};
      writeFileSync(`${directory}/candidate.json`, `${JSON.stringify(evidence)}\n`);
      break;
    }
    case 'verify': {
      const [directory, architecture, revision, digest] = args;
      assert.equal(args.length, 4);
      validateEvidence(json(`${directory}/candidate.json`), architecture, revision, digest,
        readFileSync(`${directory}/inventory.json`));
      break;
    }
    case 'index': {
      const [path, amd64, arm64] = args;
      assert.equal(args.length, 3);
      const index = json(path);
      assert.equal(index.schemaVersion, 2);
      assert.equal(index.mediaType, 'application/vnd.oci.image.index.v1+json');
      assert.deepEqual(index.manifests.map(row => [row.platform.os, row.platform.architecture, row.digest]),
        [['linux', 'amd64', amd64], ['linux', 'arm64', arm64]]);
      break;
    }
    default: throw new Error('unsupported candidate evidence operation');
  }
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main(process.argv.slice(2));
