// Closed SDK acquisition profile, not a general OCI filesystem extractor.
// Authority: repository-pinned OCI image-spec 1.1.1 layer/config/manifest.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {gunzipSync, inflateRawSync} from 'node:zlib';

export const profile = 'prismpm/sdk-metadata/1';
export const label = 'org.prismpm.sdk.metadata';
export const limits = Object.freeze({document:1024 * 1024, compressed:32 * 1024 * 1024,
  expanded:32 * 1024 * 1024, inventory:8 * 1024 * 1024, standards:16 * 1024 * 1024, layers:256});
export const paths = Object.freeze({inventory:'opt/prismpm/share/inventory.json', standards:'opt/prismpm/share/standards.lock'});
export const sha = bytes => 'sha256:' + createHash('sha256').update(bytes).digest('hex');
const digest = /^sha256:[0-9a-f]{64}$/;
const media = 'application/vnd.oci.image.';
const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);

// JSON.parse alone discards duplicate keys. OCI consumers must not observe a
// different effective configuration through duplicate objects or case aliases.
export function parseJson(bytes) {
  assert(Buffer.isBuffer(bytes) && bytes.length <= limits.document, 'bounded OCI JSON required');
  const text = new TextDecoder('utf-8', {fatal:true}).decode(bytes);
  let at = 0;
  const white = () => {while (' \n\r\t'.includes(text[at]) && at < text.length) at++;};
  const string = () => {
    assert.equal(text[at++], '"', 'JSON string required'); const start = at - 1;
    while (at < text.length) {
      const char = text[at++];
      if (char === '"') return JSON.parse(text.slice(start, at));
      if (char === '\\') at++;
    }
    assert.fail('unterminated JSON string');
  };
  const value = depth => {
    assert(depth <= 64, 'OCI JSON nesting exceeds bound'); white();
    const char = text[at];
    if (char === '"') {string(); return;}
    if (char === '{' || char === '[') {
      at++; white(); const end = char === '{' ? '}' : ']', seen = new Set();
      if (text[at] === end) {at++; return;}
      for (;;) {
        if (char === '{') {
          white(); const key = string(); assert(!seen.has(key), 'duplicate OCI JSON key'); seen.add(key);
          white(); assert.equal(text[at++], ':');
        }
        value(depth + 1); white();
        if (text[at] === end) {at++; return;}
        assert.equal(text[at++], ',', 'JSON delimiter required');
      }
    }
    const token = /^(?:true|false|null|-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?)/.exec(text.slice(at));
    assert(token, 'JSON value required'); at += token[0].length;
  };
  value(0); white(); assert.equal(at, text.length, 'trailing OCI JSON data');
  return JSON.parse(text);
}

function caseKeys(value, names) {
  assert(object(value), 'OCI JSON object required');
  const fold = key => key.toUpperCase().toLowerCase();
  const expected = new Map(names.map(name => [fold(name), name]));
  for (const key of Object.keys(value)) if (expected.has(fold(key)))
    assert.equal(key, expected.get(fold(key)), 'noncanonical OCI field casing');
}

export function descriptor(value, maximum, type) {
  assert(object(value), 'OCI descriptor required');
  caseKeys(value, ['mediaType', 'digest', 'size', 'urls', 'data', 'annotations', 'platform', 'artifactType']);
  assert.equal(value.mediaType, type, 'OCI descriptor media type differs');
  assert.match(value.digest, digest);
  assert(Number.isSafeInteger(value.size) && value.size > 0 && value.size <= maximum, 'OCI descriptor size exceeds bound');
  assert(value.urls === undefined && value.data === undefined, 'alternate OCI blob locations refused');
  return value;
}

export function verifyBlob(bytes, expected, maximum) {
  assert(Buffer.isBuffer(bytes) && bytes.length <= maximum, 'OCI blob exceeds byte bound');
  assert.equal(bytes.length, expected.size, 'OCI blob size differs');
  assert.equal(sha(bytes), expected.digest, 'OCI blob digest differs');
  return bytes;
}

export function parseManifest(bytes, expected) {
  descriptor(expected, limits.document, media + 'manifest.v1+json');
  const value = parseJson(verifyBlob(bytes, expected, limits.document));
  caseKeys(value, ['schemaVersion', 'mediaType', 'config', 'layers', 'subject', 'annotations', 'artifactType']);
  assert.equal(value.schemaVersion, 2);
  assert.equal(value.mediaType, media + 'manifest.v1+json');
  descriptor(value.config, limits.document, media + 'config.v1+json');
  assert(Array.isArray(value.layers) && value.layers.length > 0 && value.layers.length <= limits.layers,
    'bounded nonempty OCI layer sequence required');
  for (const layer of value.layers) {
    assert(object(layer) && typeof layer.mediaType === 'string' &&
      [media + 'layer.v1.tar', media + 'layer.v1.tar+gzip', media + 'layer.v1.tar+zstd'].includes(layer.mediaType),
    'unsupported OCI filesystem layer type');
    descriptor(layer, Number.MAX_SAFE_INTEGER, layer.mediaType);
    assert.match(layer.digest, digest);
    assert(Number.isSafeInteger(layer.size) && layer.size > 0, 'invalid OCI layer size');
    assert(layer.urls === undefined && layer.data === undefined, 'alternate OCI layer locations refused');
  }
  const terminal = value.layers.at(-1);
  assert([media + 'layer.v1.tar', media + 'layer.v1.tar+gzip'].includes(terminal.mediaType),
    'terminal SDK metadata compression unsupported');
  descriptor(terminal, limits.compressed, terminal.mediaType);
  return value;
}

export function parseConfig(bytes, manifest, platform) {
  const config = parseJson(verifyBlob(bytes, manifest.config, limits.document));
  caseKeys(config, ['created', 'author', 'architecture', 'os', 'os.version', 'os.features', 'variant', 'config', 'rootfs', 'history']);
  caseKeys(platform, ['os', 'architecture', 'variant', 'os.version', 'os.features']);
  assert.equal(platform.os, 'linux');
  assert.equal(config.os, 'linux');
  assert(['amd64', 'arm64'].includes(platform.architecture), 'unsupported SDK platform');
  assert.equal(config.architecture, platform.architecture);
  for (const selected of [config.variant, platform.variant]) assert(selected === undefined ||
    (platform.architecture === 'arm64' && selected === 'v8'), 'SDK CPU variant differs');
  assert(config['os.version'] === undefined && platform['os.version'] === undefined &&
    config['os.features'] === undefined && platform['os.features'] === undefined, 'SDK platform requirements unsupported');
  assert.equal(config.rootfs?.type, 'layers');
  caseKeys(config.rootfs, ['type', 'diff_ids']);
  assert(Array.isArray(config.rootfs.diff_ids));
  assert.equal(config.rootfs.diff_ids.length, manifest.layers.length, 'OCI layer/DiffID count differs');
  for (const id of config.rootfs.diff_ids) assert.match(id, digest);
  assert.equal(config.config?.Labels?.[label], profile, 'SDK has no bounded metadata profile; no full-image fallback');
  caseKeys(config.config, ['User', 'ExposedPorts', 'Env', 'Entrypoint', 'Cmd', 'Volumes', 'WorkingDir', 'Labels',
    'StopSignal', 'ArgsEscaped', 'Memory', 'MemorySwap', 'CpuShares', 'Healthcheck', 'OnBuild', 'Shell']);
  if (config.config.Volumes !== undefined) {
    assert(object(config.config.Volumes), 'OCI volumes must be an object');
    for (const volume of Object.keys(config.config.Volumes)) {
      assert(volume.startsWith('/') && !volume.slice(1).split('/').some(part => !part || part === '.' || part === '..'),
        'noncanonical OCI volume path');
      for (const name of Object.values(paths)) {
        const target = '/' + name;
        assert(volume !== target && !target.startsWith(volume + '/') && !volume.startsWith(target + '/'),
          'OCI volume intersects SDK metadata');
      }
    }
  }
  return config;
}

function field(header, begin, length) {
  const bytes = header.subarray(begin, begin + length), end = bytes.indexOf(0);
  if (end >= 0) assert(bytes.subarray(end).every(byte => byte === 0), 'nonzero data after tar field terminator');
  const value = bytes.subarray(0, end < 0 ? bytes.length : end);
  assert(value.every(byte => byte >= 32 && byte <= 126), 'non-ASCII SDK tar field');
  return value.toString('ascii');
}

function octal(header, begin, length) {
  // POSIX octal fields only; GNU base-256 and sparse extensions are refused.
  const value = header.subarray(begin, begin + length).toString('latin1');
  assert(/^ *[0-7]+(?:\0[\0 ]*| *)$/.test(value), 'invalid SDK tar numeric field');
  const result = Number.parseInt(value.replace(/[\0 ]+$/, '').trim(), 8);
  assert(Number.isSafeInteger(result) && result >= 0, 'invalid SDK tar integer');
  return result;
}

export function parseMetadataTar(bytes) {
  assert(Buffer.isBuffer(bytes) && bytes.length <= limits.expanded && bytes.length % 512 === 0,
    'SDK metadata tar length exceeds bound or alignment');
  const directories = new Set(['opt', 'opt/prismpm', 'opt/prismpm/share']);
  const seen = new Set(), files = new Map();
  let offset = 0, ended = false;
  while (offset + 512 <= bytes.length) {
    const header = bytes.subarray(offset, offset + 512);
    if (header.every(byte => byte === 0)) {
      const rest = bytes.subarray(offset);
      assert(rest.length >= 1024 && rest.length <= 10240 && rest.every(byte => byte === 0), 'invalid SDK tar end markers/trailing data');
      ended = true; break;
    }
    assert(seen.size < 6, 'too many SDK metadata entries');
    let checksum = 0;
    for (let index = 0; index < 512; index++) checksum += index >= 148 && index < 156 ? 32 : header[index];
    assert.equal(octal(header, 148, 8), checksum, 'SDK tar checksum differs');
    assert.equal(header.subarray(257, 263).toString('latin1'), 'ustar\0', 'POSIX ustar metadata required');
    assert.equal(header.subarray(263, 265).toString('ascii'), '00');
    assert.equal(field(header, 345, 155), '', 'SDK tar path prefix refused');
    assert.equal(field(header, 157, 100), '', 'SDK tar links refused');
    assert(header.subarray(500).every(byte => byte === 0), 'SDK tar extension refused');
    const rawName = field(header, 0, 100), type = header[156];
    assert(type === 48 || type === 0 || type === 53, 'SDK tar nonregular entry refused');
    const name = type === 53 && rawName.endsWith('/') ? rawName.slice(0, -1) : rawName;
    assert(!seen.has(name), 'duplicate SDK metadata path');
    assert(name === '.' || directories.has(name) || Object.values(paths).includes(name), 'unexpected SDK metadata path');
    assert(name === '.' || !name.split('/').some(part => !part || part === '.' || part === '..'), 'invalid SDK metadata path');
    const size = octal(header, 124, 12), mode = octal(header, 100, 8);
    assert.equal(octal(header, 108, 8), 0, 'SDK metadata owner must be root');
    assert.equal(octal(header, 116, 8), 0, 'SDK metadata group must be root');
    octal(header, 136, 12);
    const isDirectory = name === '.' || directories.has(name);
    if (name.includes('/')) assert(seen.has(name.slice(0, name.lastIndexOf('/'))), 'SDK parent directory must precede child');
    assert.equal(type === 53, isDirectory, 'SDK metadata path type differs');
    assert.equal(mode, isDirectory ? 0o755 : 0o444, 'SDK metadata mode differs');
    if (isDirectory) assert.equal(size, 0, 'SDK directory has a payload');
    else {
      assert(size > 0 && size <= (name === paths.inventory ? limits.inventory : limits.standards), 'SDK metadata file exceeds bound');
      for (const ancestor of directories) assert(seen.has(ancestor), 'SDK metadata ancestors must precede files');
    }
    const end = offset + 512 + size, aligned = Math.ceil(end / 512) * 512;
    assert(aligned <= bytes.length, 'truncated SDK metadata tar');
    assert(bytes.subarray(end, aligned).every(byte => byte === 0), 'nonzero SDK tar payload padding');
    if (!isDirectory) files.set(name, Buffer.from(bytes.subarray(offset + 512, end)));
    seen.add(name); offset = aligned;
  }
  assert(ended, 'SDK tar end markers missing');
  for (const name of [...directories, ...Object.values(paths)]) assert(seen.has(name), 'required SDK metadata path absent');
  return {inventory:files.get(paths.inventory), standards:files.get(paths.standards)};
}

export function decodeMetadataLayer(bytes, manifest, config) {
  const terminal = manifest.layers.at(-1);
  verifyBlob(bytes, terminal, limits.compressed);
  let tar = bytes;
  if (terminal.mediaType.endsWith('+gzip')) {
    // Native zlib checks CRC/ISIZE; exact raw DEFLATE consumption also refuses
    // concatenated members and garbage that gunzip alone may tolerate.
    assert(bytes.length >= 18 && bytes[0] === 31 && bytes[1] === 139 && bytes[2] === 8 && bytes[3] === 0,
      'closed SDK gzip header required');
    const raw = inflateRawSync(bytes.subarray(10, bytes.length - 8), {info:true, maxOutputLength:limits.expanded});
    assert.equal(raw.engine.bytesWritten, bytes.length - 18, 'SDK gzip has trailing data or multiple members');
    tar = gunzipSync(bytes, {maxOutputLength:limits.expanded});
    assert.deepEqual(tar, raw.buffer, 'SDK gzip decoders disagree');
  }
  assert.equal(sha(tar), config.rootfs.diff_ids.at(-1), 'SDK metadata DiffID differs');
  return parseMetadataTar(tar);
}
