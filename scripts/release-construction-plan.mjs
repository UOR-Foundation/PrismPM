// Construction only. Every consuming acceptance owner still runs independently.
import assert from 'node:assert/strict';
import {resolve, join} from 'node:path';
import {pathToFileURL} from 'node:url';

export function releaseConstructionPlan(revision, architecture, builder, output) {
  assert.match(revision, /^[0-9a-f]{40}$/);
  assert(['amd64', 'arm64'].includes(architecture));
  assert.match(builder, /^[a-zA-Z0-9][a-zA-Z0-9_.-]{0,127}$/);
  output = resolve(output);
  const labels = ['--provenance=false', '--sbom=false', '--build-arg', 'SOURCE_DATE_EPOCH=0',
    '--label', 'org.opencontainers.image.created=1970-01-01T00:00:00Z',
    '--label', `org.opencontainers.image.revision=${revision}`,
    '--label', 'org.opencontainers.image.source=https://github.com/UOR-Foundation/PrismPM',
    '--label', 'org.opencontainers.image.version=0.4.0'];
  const builds = [['sdk', 'runtime'], ['adapter-compose', 'adapter-compose'],
    ['adapter-kubernetes', 'adapter-kubernetes'], ['adapter-github-pages', 'adapter-github-pages'], ['oracles', 'oracles']]
    .map(([name, target]) => ({target, args: [...labels,
      '--output', `type=registry,name=ghcr.io/uor-foundation/prismpm-${name},push-by-digest=true,name-canonical=true,rewrite-timestamp=true,oci-mediatypes=true`,
      '--metadata-file', join(output, `${name}-build.json`)]}));
  builds.push({target: 'cli-archive', args: ['--build-arg', 'SDK_VERSION=0.4.0',
    '--output', `type=local,dest=${join(output, 'native-release')}`]});
  return {builder, platform: `linux/${architecture}`, builds};
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  assert.equal(process.argv.length, 6);
  console.log(JSON.stringify(releaseConstructionPlan(...process.argv.slice(2))));
}
