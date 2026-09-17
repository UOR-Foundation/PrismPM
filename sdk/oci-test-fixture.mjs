// Test-image transport only. OCI 1.1.1 media-types.md documents these mappings.
// Config and layer blobs are retained verbatim; this is not an SDK converter.
import assert from 'node:assert/strict';

const docker = {
  manifest: 'application/vnd.docker.distribution.manifest.v2+json',
  config: 'application/vnd.docker.container.image.v1+json',
  layer: 'application/vnd.docker.image.rootfs.diff.tar.gzip',
};
const oci = {
  manifest: 'application/vnd.oci.image.manifest.v1+json',
  config: 'application/vnd.oci.image.config.v1+json',
  layer: 'application/vnd.oci.image.layer.v1.tar+gzip',
};

export function ociFixtureManifest(bytes) {
  assert.ok(bytes.length <= 1024 * 1024, 'fixture manifest exceeds its byte bound');
  const manifest = JSON.parse(bytes);
  assert.equal(manifest.schemaVersion, 2);
  assert.ok([docker.manifest, oci.manifest].includes(manifest.mediaType), 'unsupported fixture manifest');
  const types = manifest.mediaType === docker.manifest ? docker : oci;
  const descriptor = (value, mediaType) => {
    assert.equal(value.mediaType, mediaType, 'unsupported fixture descriptor');
    assert.match(value.digest, /^sha256:[0-9a-f]{64}$/);
    assert.ok(Number.isSafeInteger(value.size) && value.size > 0);
  };
  descriptor(manifest.config, types.config);
  assert.ok(Array.isArray(manifest.layers) && manifest.layers.length > 0);
  for (const layer of manifest.layers) descriptor(layer, types.layer);
  if (types === oci) return bytes;
  manifest.mediaType = oci.manifest;
  manifest.config.mediaType = oci.config;
  for (const layer of manifest.layers) layer.mediaType = oci.layer;
  return Buffer.from(JSON.stringify(manifest));
}
