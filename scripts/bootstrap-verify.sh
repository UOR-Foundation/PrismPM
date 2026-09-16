#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
source_commit=f378fd3a8dc5711cb4b22cec9ee2f874353628c3
if test "$#" -gt 1 || { test "$#" -eq 1 && test "$1" != --check-source; }; then
  printf 'usage: bootstrap-verify.sh [--check-source]\n' >&2
  exit 2
fi
if ! git -C "$root" cat-file -e "$source_commit^{tree}" 2>/dev/null; then
  printf 'Pinned bootstrap source %s is unavailable; use a full-history checkout (actions/checkout fetch-depth: 0) before verification.\n' \
    "$source_commit" >&2
  exit 1
fi
if test "$#" -eq 1; then
  printf 'Pinned bootstrap source %s is available (source preflight only).\n' "$source_commit"
  exit 0
fi

archive="$root/.prism/cache/bootstrap/prismpm-0.2.0-x86_64-unknown-linux-gnu.tar.gz"
archive_sha=f3dd999f5618db154fa06222a06f9de95d86e1dbf683954426ea91c974cbe24c
work=$(mktemp -d /tmp/prismpm-bootstrap-verify.XXXXXX)
cleanup() {
  rm -rf "$work"
}
trap cleanup EXIT

test -f "$archive"
printf '%s  %s\n' "$archive_sha" "$archive" | sha256sum --check --strict >/dev/null
tar --extract --gzip --file "$archive" --directory "$work" \
  --no-same-owner --no-same-permissions
prior="$work/prismpm-0.2.0-x86_64-unknown-linux-gnu/prismpm"
test -x "$prior"
prior_binary_sha=$(sha256sum "$prior" | cut -d' ' -f1)

envelope="$work/envelope"
mkdir -p "$envelope"
git -C "$root" archive "$source_commit" | tar --extract --directory "$envelope"
node - "$root" "$envelope" <<'NODE'
const [root, envelope] = process.argv.slice(2);
const { createHash } = require('node:crypto');
const { execFileSync } = require('node:child_process');
const { lstatSync, mkdirSync, readFileSync, readlinkSync, writeFileSync } = require('node:fs');
const { join } = require('node:path');

const paths = execFileSync('git', ['-C', root, 'ls-files', '-z'], { maxBuffer: 64 * 1024 * 1024 })
  .toString('utf8').split('\0').filter(Boolean);
const files = paths.map(path => {
  const absolute = join(root, path);
  const metadata = lstatSync(absolute);
  const kind = metadata.isFile() ? 'file' : metadata.isSymbolicLink() ? 'symlink' : null;
  if (kind === null) throw new Error(`tracked source has unsupported kind: ${path}`);
  const bytes = kind === 'file' ? readFileSync(absolute) : Buffer.from(readlinkSync(absolute));
  return {
    kind,
    path,
    sha256: createHash('sha256').update(bytes).digest('hex'),
    size: bytes.length
  };
});
const manifest = Buffer.from(JSON.stringify({ files, schema: 'prismpm/source-manifest/1' }));
const manifestDigest = createHash('sha256').update(manifest).digest('hex');
mkdirSync(envelope, { recursive: true });
writeFileSync(join(envelope, 'source-manifest.json'), manifest);
writeFileSync(join(envelope, 'manifest-metadata.json'), JSON.stringify({
  digest: manifestDigest,
  file_count: files.length
}));
const corePath = join(envelope, 'stdlib', 'src', 'Foundation', 'Core.lex.tex');
const lines = readFileSync(corePath, 'utf8').split('\n');
const index = lines.findIndex(line => line.startsWith('\\semanticdata{'));
if (index === -1 || !lines[index].endsWith('}')) {
  throw new Error('accepted bootstrap model has no canonical semantic data line');
}
const payload = JSON.parse(lines[index].slice('\\semanticdata{'.length, -1));
payload.declarations.push(
  {
    body: { kind: 'string', value: `sha256:${manifestDigest}` },
    kind: 'definition',
    name: 'sourceManifestDigest',
    parameters: [],
    result: { kind: 'string' }
  },
  {
    body: { kind: 'string', value: String(files.length) },
    kind: 'definition',
    name: 'sourceManifestFileCount',
    parameters: [],
    result: { kind: 'string' }
  }
);
lines[index] = `\\semanticdata{${JSON.stringify(payload)}}`;
writeFileSync(corePath, lines.join('\n'));
NODE

# The 0.3 system language intentionally uses generic LexLean 0.3 forms that a
# 0.2 binary cannot interpret.  The prior SDK therefore validates a complete,
# byte-exact manifest of the new tracked source closure through a deliberately
# 0.2-compatible semantic projection.  The current SDK validates that same
# projection and then validates the full production model below.  The evidence
# keeps those scopes separate; it never represents compatibility validation as
# full 0.3 semantic conformance.
if ! "$prior" --project "$envelope" check --json >"$work/prior-projection.json"; then
  cat "$work/prior-projection.json" >&2
  exit 1
fi
if ! cargo run --locked --offline --quiet --manifest-path \
  "$root/vendor/lexlean/Cargo.toml" -- --project "$envelope/lexlean.toml" lock; then
  exit 1
fi
if ! cargo run --locked --offline --quiet --package prismpm -- \
  --project "$envelope" check --json >"$work/current-projection.json"; then
  cat "$work/current-projection.json" >&2
  exit 1
fi
if ! cargo run --locked --offline --quiet --package prismpm -- \
  --project "$root" check --json >"$work/production.json"; then
  cat "$work/production.json" >&2
  exit 1
fi

node - "$work/prior-projection.json" "$work/current-projection.json" \
  "$work/production.json" "$envelope/manifest-metadata.json" \
  "$root/target/bootstrap-evidence.json" "$archive_sha" \
  "$prior_binary_sha" "$source_commit" <<'NODE'
const [priorPath, currentPath, productionPath, manifestMetadataPath,
  evidencePath, archiveSha, binarySha, sourceCommit] = process.argv.slice(2);
const { readFileSync, mkdirSync, writeFileSync } = require('node:fs');
const { dirname } = require('node:path');
const { createHash } = require('node:crypto');
const priorBytes = readFileSync(priorPath);
const currentBytes = readFileSync(currentPath);
const productionBytes = readFileSync(productionPath);
const prior = JSON.parse(priorBytes);
const current = JSON.parse(currentBytes);
const production = JSON.parse(productionBytes);
const manifest = JSON.parse(readFileSync(manifestMetadataPath));
for (const field of ['entity_count', 'semantic_id', 'snapshot_id']) {
  if (prior[field] !== current[field]) {
    throw new Error(`bootstrap/current projection ${field} differs`);
  }
}
if (prior.schema !== 'prismpm/check-result/1' || current.schema !== prior.schema) {
  throw new Error('bootstrap/current projection result schema differs');
}
if (production.schema !== 'prismpm/check-result/1') {
  throw new Error('current production result schema differs');
}
const sha = bytes => `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
const evidence = {
  bootstrap: {
    archive_digest: `sha256:${archiveSha}`,
    binary_digest: `sha256:${binarySha}`,
    source_commit: sourceCommit,
    version: '0.2.0'
  },
  compatibility_projection: {
    current_result_digest: sha(currentBytes),
    prior_result_digest: sha(priorBytes),
    shared_identity: {
      entity_count: current.entity_count,
      semantic_id: current.semantic_id,
      snapshot_id: current.snapshot_id
    }
  },
  production_model: {
    entity_count: production.entity_count,
    result_digest: sha(productionBytes),
    semantic_id: production.semantic_id,
    snapshot_id: production.snapshot_id
  },
  schema: 'prismpm/bootstrap-evidence/1',
  source_manifest: {
    digest: `sha256:${manifest.digest}`,
    file_count: manifest.file_count
  },
  status: 'passed'
};
mkdirSync(dirname(evidencePath), { recursive: true });
writeFileSync(evidencePath, JSON.stringify(evidence));
NODE

# Validate the emitted bytes through PrismPM's registered contract rather than
# treating JSON parsing in this wrapper as conformance.
cargo run --locked --offline --quiet --package xtask -- \
  validate-contract prismpm/bootstrap-evidence/1 \
  "$root/target/bootstrap-evidence.json"

printf 'bootstrap SDK 0.2.0 verified current semantic identity %s\n' \
  "$(node -p "require('$work/production.json').semantic_id")"
